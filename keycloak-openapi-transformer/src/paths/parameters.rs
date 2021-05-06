use super::super::components::schemas::parse_type;
use crate::paths::verb_path::VerbPath;
use openapiv3::{Parameter, ParameterData, ReferenceOr};
use regex::Regex;
use scraper::Selector;

lazy_static! {
    static ref PATH_PARAM_REGEX: Regex = Regex::new(r"\{([^}]+)}").unwrap();
    static ref PARAMS_TABLE_SELECTOR: Selector =
        Selector::parse("h5[id^=_parameters] + table").unwrap();
    static ref TITLES_SELECTOR: Selector = Selector::parse("thead > tr > th").unwrap();
    static ref ROWS_SELECTOR: Selector = Selector::parse("tbody > tr").unwrap();
    static ref CELL_SELECTOR: Selector = Selector::parse("td").unwrap();
    static ref NAME_SELECTOR: Selector = Selector::parse("strong").unwrap();
    static ref REQUIRED_SELECTOR: Selector = Selector::parse("em").unwrap();
}

pub struct ParameterRow {
    pub parameter_type: String,
    pub name: String,
    pub required: bool,
    pub description: Option<String>,
    pub schema: String,
}

pub fn parse_parameter_rows<'a>(
    section: &scraper::element_ref::ElementRef<'a>,
) -> Option<impl Iterator<Item = ParameterRow> + 'a> {
    let table = section.select(&PARAMS_TABLE_SELECTOR).next()?;
    let titles = table
        .select(&TITLES_SELECTOR)
        .map(|th| th.text().collect::<String>())
        .zip(0..)
        .collect::<std::collections::HashMap<String, usize>>();
    let &type_index = titles.get("Type")?;
    let &name_index = titles.get("Name")?;
    let description_index = titles.get("Description").cloned();
    let &schema_index = titles.get("Schema")?;
    Some(table.select(&ROWS_SELECTOR).filter_map(move |row| {
        let cells = row.select(&CELL_SELECTOR).collect::<Vec<_>>();
        let name_cell = cells.get(name_index)?;

        Some(ParameterRow {
            parameter_type: cells.get(type_index)?.text().collect(),
            name: name_cell.select(&NAME_SELECTOR).next()?.text().collect(),
            required: name_cell
                .select(&REQUIRED_SELECTOR)
                .next()?
                .text()
                .collect::<String>()
                == "required",
            description: description_index
                .and_then(|di| Some(cells.get(di)?.text().collect()))
                .and_then(|des: String| if des.is_empty() { None } else { Some(des) }),
            schema: cells.get(schema_index)?.text().collect(),
        })
    }))
}

pub fn parse_parameters(
    section: &scraper::element_ref::ElementRef<'_>,
    param_type: &str,
) -> Vec<ReferenceOr<Parameter>> {
    if let Some(rows) = parse_parameter_rows(section) {
        rows.filter(|row| row.parameter_type == param_type)
            .map(|row| {
                let parameter_data = openapiv3::ParameterData {
                    name: row.name,
                    description: row.description,
                    required: row.required,
                    deprecated: None,
                    format: openapiv3::ParameterSchemaOrContent::Schema(parse_type(&row.schema)),
                    example: None,
                    examples: Default::default(),
                };
                let parameter = match row.parameter_type.as_ref() {
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
                    _ => panic!("Don't know how to parse {}", param_type),
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
    verb_path: &VerbPath,
) -> Vec<ReferenceOr<Parameter>> {
    let mut params = parse_parameters(section, "Path");

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

#[cfg(test)]
mod tests {
    use super::parse_path;
    use crate::paths::verb_path::VerbPath;
    use openapiv3::{OpenAPI, ReferenceOr};
    use scraper::Html;
    use scraper::Selector;

    const HTML: &str = include_str!("../../../keycloak/9.0.html");
    const JSON: &str = include_str!("../../../keycloak/9.0.json");

    fn parse_parameters_correctly(html_selector: &str, path: &str) {
        let openapi: Result<OpenAPI, _> = serde_json::from_str(JSON);
        let verb_path: VerbPath = format!("GET {}", path).parse().unwrap();
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
                    &verb_path
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
