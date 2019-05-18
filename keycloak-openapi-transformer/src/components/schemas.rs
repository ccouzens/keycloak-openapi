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
    let type_selector = Selector::parse("td:first-child + td").unwrap();
    let properties = section.select(&row_selector).map(|row| {
        let schema_type = match row
            .select(&type_selector)
            .next()
            .unwrap()
            .text()
            .collect::<String>()
            .as_str()
        {
            "integer(int32)" => openapiv3::Type::Integer(openapiv3::IntegerType {
                format: openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::IntegerFormat::Int32),
                multiple_of: None,
                exclusive_minimum: false,
                exclusive_maximum: false,
                minimum: None,
                maximum: None,
                enumeration: vec![],
            }),
            "number(float)" => openapiv3::Type::Number(openapiv3::NumberType {
                format: openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::NumberFormat::Float),
                multiple_of: None,
                exclusive_minimum: false,
                exclusive_maximum: false,
                minimum: None,
                maximum: None,
                enumeration: vec![],
            }),
            "boolean" => openapiv3::Type::Boolean {},
            _ => openapiv3::Type::String(openapiv3::StringType {
                format: Default::default(),
                pattern: None,
                enumeration: vec![],
            }),
        };

        (
            row.select(&property_name_selector)
                .next()
                .unwrap()
                .text()
                .collect::<String>(),
            openapiv3::ReferenceOr::Item(Box::new(Schema {
                schema_data: Default::default(),
                schema_kind: SchemaKind::Type(schema_type),
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

    fn parse_schema_correctly(schema: &str) {
        let openapi: OpenAPI = serde_json::from_str(JSON).expect("Could not deserialize example");
        let components = openapi.components.expect("Couldn't deserialize components");

        assert_eq!(
            components.schemas.get(schema),
            parse_schemas(&Html::parse_document(HTML)).get(schema)
        );
    }

    #[test]
    fn parses_string_only_schema_as_expected() {
        parse_schema_correctly("AccessToken-CertConf");
    }

    #[test]
    fn parses_int32_only_schema_as_expected() {
        parse_schema_correctly("ClientInitialAccessCreatePresentation");
    }

    #[test]
    fn parses_schema_with_bool_as_expected() {
        parse_schema_correctly("SynchronizationResult");
    }

    #[test]
    fn parses_schema_with_float_as_expected() {
        parse_schema_correctly("MultivaluedHashMap");
    }
}
