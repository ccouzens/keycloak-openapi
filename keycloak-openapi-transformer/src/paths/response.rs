use super::super::components::schemas;
use scraper::Selector;

fn parse_type(raw_type: &str) -> openapiv3::ReferenceOr<openapiv3::Schema> {
    if let Some(simple_type) = schemas::item_type(raw_type) {
        openapiv3::ReferenceOr::Item(openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(simple_type),
        })
    } else {
        openapiv3::ReferenceOr::Reference {
            reference: format!("#/components/schemas/{}", raw_type),
        }
    }
}

pub fn parse(section: &scraper::element_ref::ElementRef<'_>) -> openapiv3::Response {
    let responses_selector = Selector::parse("h5[id^=_responses] + table > tbody > tr").unwrap();
    let produces_selector = Selector::parse("h5[id^=_produces] + div code").unwrap();
    let description_selector = Selector::parse("td:first-child + td").unwrap();
    let schema_selector = Selector::parse("td:first-child + td + td").unwrap();

    let response_table = section.select(&responses_selector).next().unwrap();
    let description = response_table
        .select(&description_selector)
        .next()
        .unwrap()
        .text()
        .collect();
    if let Some(produces) = section
        .select(&produces_selector)
        .next()
        .map(|p| p.text().collect::<String>())
    {
        let raw_schema: String = response_table
            .select(&schema_selector)
            .next()
            .unwrap()
            .text()
            .collect();

        openapiv3::Response {
            description,
            content: [(
                produces,
                openapiv3::ReferenceOr::Item(openapiv3::MediaType {
                    schema: Some(parse_type(&raw_schema)),
                    ..Default::default()
                }),
            )]
            .iter()
            .cloned()
            .collect(),
            ..Default::default()
        }
    } else {
        openapiv3::Response {
            description,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod test {
    const HTML: &str = include_str!("../../../keycloak/6.0.html");
    use super::parse;
    use scraper::Html;
    use scraper::Selector;

    #[test]
    fn octet_streams() {
        const CSS_SELECTOR: &str =
            "#_paths + .sectionbody > .sect2 > #_client_attribute_certificate_resource + .sect3 + .sect3";
        const EXPECTED: &str = r#"
        {
            "application/octet-stream": {
                "schema": {
                    "type": "string",
                    "format": "byte"
                }
            }
        }
        "#;
        let document = Html::parse_document(&HTML);
        let section = document
            .select(&Selector::parse(CSS_SELECTOR).unwrap())
            .next()
            .unwrap();
        assert_eq!(
            parse(&section).content,
            serde_json::from_str(EXPECTED).unwrap()
        );
    }

    #[test]
    fn json() {
        const CSS_SELECTOR: &str =
            "#_paths + .sectionbody > .sect2 > #_client_initial_access_resource + .sect3 + .sect3";
        const EXPECTED: &str = r##"
        {
            "application/json": {
                "schema": {
                    "type": "array",
                    "items": {
                        "$ref": "#/components/schemas/ClientInitialAccessPresentation"
                    }
                }
            }
        }
        "##;
        let document = Html::parse_document(&HTML);
        let section = document
            .select(&Selector::parse(CSS_SELECTOR).unwrap())
            .next()
            .unwrap();
        assert_eq!(
            parse(&section).content,
            serde_json::from_str(EXPECTED).unwrap()
        );
    }

    #[test]
    fn no_content() {
        const CSS_SELECTOR: &str =
            "#_paths + .sectionbody > .sect2 > #_attack_detection_resource + .sect3";
        let document = Html::parse_document(&HTML);
        let section = document
            .select(&Selector::parse(CSS_SELECTOR).unwrap())
            .next()
            .unwrap();
        assert_eq!(parse(&section).content, std::collections::BTreeMap::new());
    }
}
