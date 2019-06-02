use openapiv3::{Parameter, ReferenceOr};
use scraper::Selector;

use regex::Regex;

lazy_static! {
    static ref PATH_PARAM_REGEX: Regex = Regex::new(r"\{([^}]+)}").unwrap();
    static ref PARAMS_TABLE_SELECTOR: Selector =
        Selector::parse("h5[id^=_parameters] + table").unwrap();
    static ref TITLES_SELECTOR: Selector = Selector::parse("thead > tr > th").unwrap();
    static ref ROWS_SELECTOR: Selector = Selector::parse("tbody > tr").unwrap();
    static ref CELL_SELECTOR: Selector = Selector::parse("td").unwrap();
    static ref NAME_SELECTOR: Selector = Selector::parse("strong").unwrap();
}

pub fn parse_parameters(
    section: &scraper::element_ref::ElementRef<'_>,
    param_type: &str,
) -> Vec<ReferenceOr<Parameter>> {
    if let Some(section) = section.select(&PARAMS_TABLE_SELECTOR).next() {
        let titles = section
            .select(&TITLES_SELECTOR)
            .map(|th| th.text().collect::<String>())
            .zip(0..)
            .collect::<std::collections::HashMap<_, _>>();
        let type_index = titles["Type"];
        let name_index = titles["Name"];
        let description_index = titles.get("Description").cloned();
        let schema_index = titles["Schema"];
        let path_rows = section.select(&ROWS_SELECTOR).filter(|row| {
            row.select(&CELL_SELECTOR)
                .nth(type_index)
                .unwrap()
                .text()
                .collect::<String>()
                == param_type
        });
        path_rows
            .map(|row| {
                let parameter_data = openapiv3::ParameterData {
                    name: row
                        .select(&CELL_SELECTOR)
                        .nth(name_index)
                        .unwrap()
                        .select(&NAME_SELECTOR)
                        .next()
                        .unwrap()
                        .text()
                        .collect(),
                    description: description_index
                        .map(|i| row.select(&CELL_SELECTOR).nth(i).unwrap().text().collect())
                        .and_then(|des: String| if des.is_empty() { None } else { Some(des) }),
                    required: true,
                    deprecated: None,
                    format: openapiv3::ParameterSchemaOrContent::Schema(
                        openapiv3::ReferenceOr::Item(openapiv3::Schema {
                            schema_data: Default::default(),
                            schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(
                                Default::default(),
                            )),
                        }),
                    ),
                    example: None,
                    examples: Default::default(),
                };
                let parameter = match param_type {
                    "Path" => Parameter::Path {
                        parameter_data,
                        style: Default::default(),
                    },
                    "Query" => Parameter::Query {
                        parameter_data,
                        allow_reserved: false,
                        style: Default::default(),
                        allow_empty_value: None,
                    },
                    _ => panic!(format!("Don't know how to parse {}", param_type)),
                };
                ReferenceOr::Item(parameter)
            })
            .collect()
    } else {
        Vec::new()
    }
}

pub fn parse_path(
    section: &scraper::element_ref::ElementRef<'_>,
    path: &str,
) -> Vec<ReferenceOr<Parameter>> {
    let mut params = parse_parameters(section, "Path");

    for (index, cap) in PATH_PARAM_REGEX
        .captures_iter(path)
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

#[cfg(test)]
mod tests {
    use super::parse_path;
    use openapiv3::{OpenAPI, ReferenceOr};
    use scraper::Html;
    use scraper::Selector;

    const HTML: &str = include_str!("../../../keycloak/6.0.html");
    const JSON: &str = include_str!("../../../keycloak/6.0.json");

    fn parse_parameters_correctly(html_selector: &str, path: &str) {
        let openapi: Result<OpenAPI, _> = serde_json::from_str(JSON);
        if let Ok(Some(ReferenceOr::Item(openapiv3::PathItem { parameters, .. }))) =
            openapi.as_ref().map(|o| o.paths.get(path))
        {
            assert_eq!(
                parameters,
                &parse_path(
                    &Html::parse_document(HTML)
                        .select(&Selector::parse(html_selector).unwrap())
                        .next()
                        .unwrap(),
                    path
                )
            );
        } else {
            panic!("Couldn't extract path")
        };
    }

    #[test]
    fn correctly_parses_realm() {
        parse_parameters_correctly(
            "#_paths + .sectionbody > .sect2 > #_attack_detection_resource + .sect3",
            "/{realm}/attack-detection/brute-force/users",
        );
    }

    #[test]
    fn correctly_parses_when_description_is_missing() {
        parse_parameters_correctly(
            "#_paths + .sectionbody > .sect2 > #_user_storage_provider_resource + .sect3",
            "/{id}/name",
        );
    }

    #[test]
    fn correctly_parses_when_description_is_blank() {
        parse_parameters_correctly(
            "#_paths + .sectionbody > .sect2 > #_attack_detection_resource + .sect3 + .sect3",
            "/{realm}/attack-detection/brute-force/users/{userId}",
        );
    }
}
