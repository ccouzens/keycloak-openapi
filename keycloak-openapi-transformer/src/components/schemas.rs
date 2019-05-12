use openapiv3::ObjectType;
use openapiv3::Schema;
use openapiv3::SchemaKind;
use scraper::Selector;

pub fn parse_schema(document: &scraper::html::Html, schema_name: &str) -> Schema {
    let selector = Selector::parse(&format!(
        "#_{} + table > tbody > tr",
        schema_name.to_ascii_lowercase()
    ))
    .unwrap();
    let properties = document.select(&selector).map(|row| {
        (
            row.select(&Selector::parse("td:first-child strong").unwrap())
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
    use super::parse_schema;
    use openapiv3::OpenAPI;
    use scraper::Html;

    #[test]
    fn parses_simple_schema_as_expected() {
        const HTML: &str = include_str!("../../../keycloak/6.0.html");
        const JSON: &str = include_str!("../../../keycloak/6.0.json");
        let openapi: OpenAPI = serde_json::from_str(JSON).expect("Could not deserialize example");
        let components = openapi.components.expect("Couldn't deserialize components");
        let address_claim_set = components
            .schemas
            .get("AddressClaimSet")
            .expect("Couldn't find address claim set");
        let address_claim_set_schema = match address_claim_set {
            openapiv3::ReferenceOr::Item(schema) => schema,
            openapiv3::ReferenceOr::Reference { reference: _ } => {
                panic!("Could not extract schema")
            }
        };

        assert_eq!(
            address_claim_set_schema,
            &parse_schema(&Html::parse_document(HTML), "AddressClaimSet")
        );
    }
}
