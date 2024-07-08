use serde::Deserialize;

use crate::{error::ResError, proto::notes::{filters, sort, Filters, Pagination, ReadNotesReq, Sort}};

/// helper struct that the `notes_get` url query deserializes into
#[derive(Debug, Default, Deserialize)]
pub struct NoteQuery {
    page: Option<String>,
    per_page: Option<String>,
    sort_by: Option<String>,
    sort_type: Option<String>,
    tags: Option<String>,
    date: Option<String>,
    date_modif: Option<String>,
    title: Option<String>,
}

fn field_err<T: std::fmt::Display>(object: T) -> Result<ReadNotesReq, ResError> {
    Err(ResError::InvalidFields(format!("Received invalid query field: {object}")))
}

/// makes sure that the query fields are ok, and then returns a valid `ReadNotesReq`
pub fn parse_note_query(user_id: i32, q: NoteQuery) -> Result<ReadNotesReq, ResError> {
    let page = match q.page {
        None => 1,
        Some(p) => match p.parse() {
            Ok(p) if p > 0 => p,
            Ok(f) => return field_err(f),
            Err(e) => return field_err(e),
        },
    };

    let per_page = match q.per_page {
        None => 20,
        Some(pp) => match pp.parse() {
            Ok(pp) if pp > 0 && pp <= 100 => pp,
            Ok(f) => return field_err(f),
            Err(e) => return field_err(e),
        },
    };

    let sort_field = match q.sort_by {
        None => sort::Field::Date,
        Some(sb) => match sb.as_str() {
            "date" => sort::Field::Date,
            "date_modif" => sort::Field::DateModif,
            "title" => sort::Field::Title,
            f => return field_err(f),
        },
    };

    let sort_type = match q.sort_type {
        None => sort::Type::Desc,
        Some(st) => match st.as_str() {
            "asc" => sort::Type::Asc,
            "desc" => sort::Type::Desc,
            f => return field_err(f),
        },
    };

    let filter_tags = match q.tags {
        None => None,
        Some(t) => {
            let tags: Vec<_> = t.split(',')
                .filter_map(|v| {
                    if v.is_empty() { None }
                    else { Some(v.parse().unwrap_or(-1)) }
                }).collect();

            match (tags.iter().all(|v| *v >= 1), tags.len()) {
                (true, l) if l <= 100 => Some(filters::Tags {
                    tag_ids: tags,
                }),
                _ => return field_err(t),
            }
        },
    };

    let filter_date = match q.date {
        None => None,
        Some(d) => {
            let dates: Vec<_> = d.split('-')
                .map(|v| v.parse().unwrap_or(-1))
                .collect();

            match (dates.iter().all(|v| *v >= 0), dates.len()) {
                (true, 2) => Some(filters::Date {
                    start: dates[0],
                    end: if dates[1] != 0 { dates[1] } else { i64::MAX - 1 },
                }),
                _ => return field_err(d),
            }
        },
    };

    let filter_date_modif = match q.date_modif {
        None => None,
        Some(dm) => {
            let dates: Vec<_> = dm.split('-')
                .map(|v| v.parse().unwrap_or(-1))
                .collect();

            match (dates.iter().all(|v| *v >= 0), dates.len()) {
                (true, 2) => Some(filters::DateModif {
                    start: dates[0],
                    end: if dates[1] != 0 { dates[1] } else { i64::MAX - 1 },
                }),
                _ => return field_err(dm),
            }
        },
    };

    let filter_search = q.title.map(|v| filters::Search { query: v });

    Ok(ReadNotesReq {
        user_id,
        pagination: Some(Pagination { page, per_page }),
        sort: Some(Sort { sort_type: sort_type.into(), sort_field: sort_field.into() }),
        filters: Some(Filters { filter_tags, filter_date, filter_date_modif, filter_search }),
    })
}
