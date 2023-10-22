use super::super::components::schemas::parse_type;
use crate::{paths::verb_path::VerbPath, table::parse_table_rows};
use indexmap::IndexMap;
use openapiv3::{MediaType, Parameter, ParameterData, ReferenceOr, RequestBody};
use regex::Regex;
use scraper::Selector;

lazy_static! {
    static ref PATH_PARAM_REGEX: Regex = Regex::new(r"\{([^}]+)}").unwrap();
    static ref PATH_PARAMS_TABLE_SELECTOR: Selector =
        Selector::parse("h6[id^=_path_parameters] + table").unwrap();
    static ref QUERY_PARAMS_TABLE_SELECTOR: Selector =
        Selector::parse("h6[id^=_query_parameters] + table").unwrap();
    static ref BODY_PARAMS_TABLE_SELECTOR: Selector =
        Selector::parse("h6[id^=_body_parameter] + table").unwrap();
}

pub fn parse_body_param(
    section: &scraper::element_ref::ElementRef<'_>,
) -> Option<ReferenceOr<RequestBody>> {
    match parse_table_rows(section, &BODY_PARAMS_TABLE_SELECTOR).first() {
        None => None,
        Some(row) => {
            let name = &row["Name"];
            let mut content = IndexMap::new();
            content.insert(
                "application/json".to_string(),
                MediaType {
                    schema: Some(parse_type(name.split_ascii_whitespace().next().unwrap())),
                    ..Default::default()
                },
            );
            Some(ReferenceOr::Item(RequestBody {
                description: Some(row["Description"].clone()),
                required: false,
                content,
            }))
        }
    }
}

pub fn parse_query_params(
    section: &scraper::element_ref::ElementRef<'_>,
) -> Vec<ReferenceOr<Parameter>> {
    let mut out = Vec::new();

    for row in parse_table_rows(section, &QUERY_PARAMS_TABLE_SELECTOR) {
        out.push(ReferenceOr::Item(Parameter::Query {
            parameter_data: openapiv3::ParameterData {
                name: row["Name"].split('\n').next().unwrap().to_string(),
                description: if row["Description"].is_empty() {
                    None
                } else {
                    Some(row["Description"].clone())
                },
                required: false,
                deprecated: None,
                format: openapiv3::ParameterSchemaOrContent::Schema(openapiv3::ReferenceOr::Item(
                    openapiv3::Schema {
                        schema_data: Default::default(),
                        schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(
                            openapiv3::StringType::default(),
                        )),
                    },
                )),
                example: None,
                examples: Default::default(),
            },
            allow_reserved: false,
            style: Default::default(),
            allow_empty_value: None,
        }));
    }

    out
}
pub fn parse_path_params(
    section: &scraper::element_ref::ElementRef<'_>,
) -> Vec<ReferenceOr<Parameter>> {
    let mut out = Vec::new();

    for row in parse_table_rows(section, &PATH_PARAMS_TABLE_SELECTOR) {
        out.push(ReferenceOr::Item(Parameter::Path {
            parameter_data: openapiv3::ParameterData {
                name: row["Name"].split('\n').next().unwrap().to_string(),
                description: if row["Description"].is_empty() {
                    None
                } else {
                    Some(row["Description"].clone())
                },
                required: true,
                deprecated: None,
                format: openapiv3::ParameterSchemaOrContent::Schema(openapiv3::ReferenceOr::Item(
                    openapiv3::Schema {
                        schema_data: Default::default(),
                        schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(
                            openapiv3::StringType::default(),
                        )),
                    },
                )),
                example: None,
                examples: Default::default(),
            },
            style: Default::default(),
        }));
    }

    out
}

pub fn parse_path(
    section: &scraper::element_ref::ElementRef<'_>,
    verb_path: &VerbPath,
) -> Vec<ReferenceOr<Parameter>> {
    let mut params = parse_path_params(section);

    if let Some(repeats) = verb_path.repeating_ids() {
        params = params
            .into_iter()
            .filter(|p| {
                if let ReferenceOr::Item(Parameter::Path {
                    parameter_data: ParameterData { name, .. },
                    ..
                }) = p
                {
                    name != "id"
                } else {
                    true
                }
            })
            .collect();
        params.extend((1..=repeats).map(|i| {
            ReferenceOr::Item(Parameter::Path {
                parameter_data: ParameterData {
                    name: format!("id{}", i),
                    required: true,
                    format: openapiv3::ParameterSchemaOrContent::Schema(parse_type("string")),
                    description: None,
                    deprecated: None,
                    example: None,
                    examples: Default::default(),
                },
                style: Default::default(),
            })
        }))
    }

    for (index, cap) in PATH_PARAM_REGEX
        .captures_iter(&verb_path.path())
        .enumerate()
        .take(params.len())
    {
        let path_var = &cap[1];
        let position = params
            .iter()
            .enumerate()
            .find(|(_, param)| {
                if let ReferenceOr::Item(Parameter::Path {
                    parameter_data: openapiv3::ParameterData { name, .. },
                    ..
                }) = param
                {
                    name == path_var
                } else {
                    false
                }
            })
            .map(|(i, _)| i);
        if let Some(position) = position {
            params.swap(index, position);
        }
    }

    params
}
