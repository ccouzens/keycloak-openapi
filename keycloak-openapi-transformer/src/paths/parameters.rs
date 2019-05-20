use openapiv3::{Parameter, ReferenceOr};
use scraper::Selector;

pub fn parse_path(section: &scraper::element_ref::ElementRef<'_>) -> Vec<ReferenceOr<Parameter>> {
    let cell_selector = Selector::parse("td").unwrap();
    let name_selector = Selector::parse("td:first-child + td strong").unwrap();
    section
        .select(&Selector::parse("tbody > tr").unwrap())
        .filter(|row| {
            row.select(&cell_selector)
                .next()
                .unwrap()
                .text()
                .collect::<String>()
                == "Path"
        })
        .map(|row| {
            ReferenceOr::Item(Parameter::Path {
                parameter_data: openapiv3::ParameterData {
                    name: row.select(&name_selector).next().unwrap().text().collect(),
                    description: Some(row.select(&cell_selector).nth(2).unwrap().text().collect()),
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
            "#_paths + .sectionbody > .sect2 > #_attack_detection_resource + .sect3 [id=_parameters] + table",
            "/{realm}/attack-detection/brute-force/users"
        );
    }

}
