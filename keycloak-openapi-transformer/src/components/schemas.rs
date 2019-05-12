use openapiv3::ObjectType;
use openapiv3::Schema;
use openapiv3::SchemaKind;
use scraper::Selector;
use std::collections::BTreeMap;

pub fn parse_schemas(
    document: &scraper::html::Html,
) -> BTreeMap<String, openapiv3::ReferenceOr<Schema>> {
    let schemas_selector = Selector::parse("#_definitions + .sectionbody > .sect2").unwrap();
    let title_selector = Selector::parse("h3").unwrap();
    document
        .select(&schemas_selector)
        .map(|section| {
            (
                section
                    .select(&title_selector)
                    .next()
                    .unwrap()
                    .text()
                    .collect(),
                openapiv3::ReferenceOr::Item(parse_schema(section)),
            )
        })
        .collect()
}

fn parse_schema(section: scraper::element_ref::ElementRef<'_>) -> Schema {
    let row_selector = Selector::parse("table > tbody > tr").unwrap();
    let property_name_selector = Selector::parse("td:first-child strong").unwrap();
    let properties = section.select(&row_selector).map(|row| {
        (
            row.select(&property_name_selector)
                .next()
                .unwrap()
                .text()
                .collect::<String>(),
            openapiv3::ReferenceOr::Item(Box::new(Schema {
                schema_data: Default::default(),
                schema_kind: SchemaKind::Type(openapiv3::Type::String(openapiv3::StringType {
                    format: Default::default(),
                    pattern: None,
                    enumeration: vec![],
                })),
            })),
        )
    });
    Schema {
        schema_data: Default::default(),
        schema_kind: SchemaKind::Type(openapiv3::Type::Object(ObjectType {
            properties: properties.collect(),
            required: vec![],
            additional_properties: None,
            min_properties: None,
            max_properties: None,
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_schemas;
    use openapiv3::OpenAPI;
    use scraper::Html;

    const HTML: &str = include_str!("../../../keycloak/6.0.html");
    const JSON: &str = include_str!("../../../keycloak/6.0.json");

    #[test]
    fn parses_simple_schema_as_expected() {
        let openapi: OpenAPI = serde_json::from_str(JSON).expect("Could not deserialize example");
        let components = openapi.components.expect("Couldn't deserialize components");

        assert_eq!(
            components.schemas.get("AccessToken-CertConf"),
            parse_schemas(&Html::parse_document(HTML)).get("AccessToken-CertConf")
        );
    }
}
