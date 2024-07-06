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

fn get_field_err(field_name: &str) -> Result<ReadNotesReq, ResError> {
    Err(ResError::InvalidFields(format!("invalid query field: {field_name}")))
}

/// makes sure that the query fields are ok, and then returns a valid `ReadNotesReq`
pub fn parse_note_query(user_id: i32, q: NoteQuery) -> Result<ReadNotesReq, ResError> {
    let page = match q.page {
        None => 1,
        Some(p) => match p.parse() {
            Ok(p) if p > 0 => p,
            _ => return get_field_err("page"),
        },
    };

    let per_page = match q.per_page {
        None => 20,
        Some(pp) => match pp.parse() {
            Ok(pp) if pp > 0 && pp <= 100 => pp,
            _ => return get_field_err("per_page"),
        },
    };

    let sort_field = match q.sort_by {
        None => sort::Field::Date,
        Some(sb) => match sb.as_str() {
            "date" => sort::Field::Date,
            "date_modif" => sort::Field::DateModif,
            "title" => sort::Field::Title,
            _ => return get_field_err("sort_by"),
        },
    };

    let sort_type = match q.sort_type {
        None => sort::Type::Desc,
        Some(st) => match st.as_str() {
            "asc" => sort::Type::Asc,
            "desc" => sort::Type::Desc,
            _ => return get_field_err("sort_type"),
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
                _ => return get_field_err("tags"),
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
                _ => return get_field_err("date"),
            }
        },
    };

    let filter_date_modif = match q.date_modif {
        None => None,
        Some(d) => {
            let dates: Vec<_> = d.split('-')
                .map(|v| v.parse().unwrap_or(-1))
                .collect();

            match (dates.iter().all(|v| *v >= 0), dates.len()) {
                (true, 2) => Some(filters::DateModif {
                    start: dates[0],
                    end: if dates[1] != 0 { dates[1] } else { i64::MAX - 1 },
                }),
                _ => return get_field_err("date_modif"),
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
