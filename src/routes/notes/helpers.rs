use serde::{de::{self, MapAccess, Visitor}, Deserialize};

use crate::proto::notes::{filters, sort, Filters, Pagination, Sort};

// helper struct that the url query deserializes into
#[derive(Debug, Default)]
pub struct NoteQuery {
    pub pagination: Pagination,
    pub sort: Sort,
    pub filters: Filters,
}

fn handle_field<'de, V>(
    field: &Option<&str>,
    field_name: &str,
    map: &mut V,
) -> Result<Option<&'de str>, V::Error>
where
    V: MapAccess<'de>,
{
    if field.is_some() {
        return Err(de::Error::custom(format!("invalid {field_name}")));
    }

    Ok(Some(map.next_value()?))
}

fn get_field_err<'de, V>(field_name: &str) -> Result<NoteQuery, V::Error>
where
    V: MapAccess<'de>,
{
    Err(de::Error::custom(format!("invalid {field_name}")))
}

// custom deserialize implementation
impl<'de> Deserialize<'de> for NoteQuery {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        enum Field { Unknown, Page, PerPage, SortBy, SortType, Tags, Date, DateModif, Title }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("NoteQuery fields")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "page" => Ok(Field::Page),
                            "per_page" => Ok(Field::PerPage),
                            "sort_by" => Ok(Field::SortBy),
                            "sort_type" => Ok(Field::SortType),
                            "tags" => Ok(Field::Tags),
                            "date" => Ok(Field::Date),
                            "date_modif" => Ok(Field::DateModif),
                            "title" => Ok(Field::Title),
                            _ => Ok(Field::Unknown),
                            // _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct NoteQueryVisitor;

        impl<'de> Visitor<'de> for NoteQueryVisitor {
            type Value = NoteQuery;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct NoteQuery")
            }

            fn visit_map<V>(self, mut map: V) -> Result<NoteQuery, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut rpage = None;
                let mut rper_page = None;
                let mut rsort_by = None;
                let mut rsort_type = None;
                let mut rtags = None;
                let mut rdate = None;
                let mut rdate_modif = None;
                let mut rtitle = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Page => rpage = handle_field(&rpage, "page", &mut map)?,
                        Field::PerPage => rper_page = handle_field(&rper_page, "per_page", &mut map)?,
                        Field::SortBy => rsort_by = handle_field(&rsort_by, "sort_by", &mut map)?,
                        Field::SortType => rsort_type = handle_field(&rsort_type, "sort_type", &mut map)?,
                        Field::Tags => rtags = handle_field(&rtags, "tags", &mut map)?,
                        Field::Date => rdate = handle_field(&rdate, "date", &mut map)?,
                        Field::DateModif => rdate_modif = handle_field(&rdate_modif, "date_modif", &mut map)?,
                        Field::Title => rtitle = handle_field(&rtitle, "title", &mut map)?,
                        Field::Unknown => (),
                    }
                }

                let page = match rpage {
                    None => 1,
                    Some(p) => match p.parse() {
                        Ok(p) if p > 0 => p,
                        _ => return get_field_err::<V>("page"),
                    },
                };

                let per_page = match rper_page {
                    None => 20,
                    Some(pp) => match pp.parse() {
                        Ok(pp) if pp > 0 => pp,
                        _ => return get_field_err::<V>("per_page"),
                    },
                };

                let sort_field = match rsort_by {
                    None => sort::Field::Date,
                    Some(sb) => match sb {
                        "date" => sort::Field::Date,
                        "date_modif" => sort::Field::DateModif,
                        "title" => sort::Field::Title,
                        _ => return get_field_err::<V>("sort_by"),
                    },
                };

                let sort_type = match rsort_type {
                    None => sort::Type::Desc,
                    Some(st) => match st {
                        "asc" => sort::Type::Asc,
                        "desc" => sort::Type::Desc,
                        _ => return get_field_err::<V>("sort_type"),
                    },
                };

                let filter_tags = match rtags {
                    None => None,
                    Some(t) => {
                        let tags = t.split(',').map(|v| v.parse());
                        match tags.clone().all(|v| v.is_ok()) {
                            true => Some(filters::Tags { tag_ids: tags.map(|v| v.unwrap()).collect() }),
                            false => return get_field_err::<V>("tags"),
                        }
                    },
                };

                let filter_date = match rdate {
                    None => None,
                    Some(d) => {
                        let dates: Vec<_> = d.split('-').map(|v| v.parse().unwrap_or(-1)).collect();
                        match (dates.iter().all(|v| *v >= 0), dates.len()) {
                            (true, 2) => Some(filters::Date { start: dates[0], end: dates[1] }),
                            _ => return get_field_err::<V>("date"),
                        }
                    },
                };

                let filter_date_modif = match rdate_modif {
                    None => None,
                    Some(d) => {
                        let dates: Vec<_> = d.split('-').map(|v| v.parse().unwrap_or(-1)).collect();
                        match (dates.iter().all(|v| *v >= 0), dates.len()) {
                            (true, 2) => Some(filters::DateModif { start: dates[0], end: dates[1] }),
                            _ => return get_field_err::<V>("date_modif"),
                        }
                    },
                };

                let filter_search = rtitle.map(|v| filters::Search { query: v.into() });

                Ok(NoteQuery {
                    pagination: Pagination { page, per_page },
                    sort: Sort { sort_type: sort_type.into(), sort_field: sort_field.into() },
                    filters: Filters { filter_tags, filter_date, filter_date_modif, filter_search },
                })
            }
        }

        const FIELDS: &[&str] = &[
            "page",
            "per_page",
            "sort_by",
            "sort_type",
            "tags",
            "date",
            "date_modif",
            "title",
        ];
        deserializer.deserialize_struct("NoteQuery", FIELDS, NoteQueryVisitor)
    }
}
