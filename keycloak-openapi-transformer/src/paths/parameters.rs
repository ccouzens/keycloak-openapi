use openapiv3::{Parameter, ReferenceOr};
use scraper::Selector;

pub fn parse_path(section: &scraper::element_ref::ElementRef<'_>) -> Vec<ReferenceOr<Parameter>> {
    let titles_selector = Selector::parse("thead > tr > th").unwrap();
    let titles = section
        .select(&titles_selector)
        .map(|th| th.text().collect::<String>())
        .zip(0..)
        .collect::<std::collections::HashMap<_, _>>();
    let type_index = titles["Type"];
    let name_index = titles["Name"];
    let description_index = titles.get("Description").cloned();
    let schema_index = titles["Schema"];
    let rows_selector = Selector::parse("tbody > tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();
    let name_selector = Selector::parse("strong").unwrap();
    let path_rows = section.select(&rows_selector).filter(|row| {
        row.select(&cell_selector)
            .nth(type_index)
            .unwrap()
            .text()
            .collect::<String>()
            == "Path"
    });
    path_rows
        .map(|row| {
            ReferenceOr::Item(Parameter::Path {
                parameter_data: openapiv3::ParameterData {
                    name: row
                        .select(&cell_selector)
                        .nth(name_index)
                        .unwrap()
                        .select(&name_selector)
                        .next()
                        .unwrap()
                        .text()
                        .collect(),
                    description: description_index
                        .map(|i| row.select(&cell_selector).nth(i).unwrap().text().collect()),
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
                },
                style: Default::default(),
            })
        })
        .collect()
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
        let openapi: OpenAPI = serde_json::from_str(JSON).expect("Could not deserialize example");
        let path = if let ReferenceOr::Item(path) = openapi.paths.get(path).unwrap() {
            path
        } else {
            panic!("Couldn't extract path")
        };
        assert_eq!(
            path.parameters,
            parse_path(
                &Html::parse_document(HTML)
                    .select(&Selector::parse(html_selector).unwrap())
                    .next()
                    .unwrap()
            )
        );
    }

    #[test]
    fn correctly_parses_realm() {
        parse_parameters_correctly(
                    "#_paths + .sectionbody > .sect2 > #_attack_detection_resource + .sect3 [id^=_parameters] + table",
                    "/{realm}/attack-detection/brute-force/users"
                );
    }

    #[test]
    fn correctly_parses_when_description_is_missing() {
        parse_parameters_correctly(
          "#_paths + .sectionbody > .sect2 > #_user_storage_provider_resource + .sect3 [id^=_parameters] + table",
                    "/{id}/name"
                );
    }
}
