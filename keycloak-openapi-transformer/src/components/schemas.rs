use indexmap::IndexMap;
use openapiv3::ObjectType;
use openapiv3::Schema;
use openapiv3::SchemaKind;
use scraper::Selector;

lazy_static! {
    static ref SCHEMAS_SELECTOR: Selector =
        Selector::parse("#models + .sectionbody > .sect2").unwrap();
    static ref TITLE_SELECTOR: Selector = Selector::parse("h3").unwrap();
    static ref ROW_SELECTOR: Selector = Selector::parse("table > tbody > tr").unwrap();
    static ref PROPERTY_NAME_SELECTOR: Selector = Selector::parse("td:first-child strong").unwrap();
    static ref TYPE_SELECTOR: Selector = Selector::parse("td:first-child + td").unwrap();
}

pub fn parse_schemas(
    document: &scraper::html::Html,
) -> IndexMap<String, openapiv3::ReferenceOr<Schema>> {
    document
        .select(&SCHEMAS_SELECTOR)
        .map(|section| {
            (
                section
                    .select(&TITLE_SELECTOR)
                    .next()
                    .unwrap()
                    .text()
                    .collect(),
                openapiv3::ReferenceOr::Item(parse_schema(section)),
            )
        })
        .collect()
}

fn array_type(raw_type: &str) -> Option<openapiv3::Type> {
    const START: &str = "List  of ";
    if raw_type.starts_with(START) {
        let inner_type = raw_type.get(START.len()..)?;
        Some(openapiv3::Type::Array(openapiv3::ArrayType {
            items: parse_type_boxed(inner_type),
            min_items: None,
            max_items: None,
            unique_items: false,
        }))
    } else {
        None
    }
}

fn set_type(raw_type: &str) -> Option<openapiv3::Type> {
    const START: &str = "Set  of ";
    if raw_type.starts_with(START) {
        let inner_type = raw_type.get(START.len()..)?;
        Some(openapiv3::Type::Array(openapiv3::ArrayType {
            items: parse_type_boxed(inner_type),
            min_items: None,
            max_items: None,
            unique_items: true,
        }))
    } else {
        None
    }
}

fn map_type(raw_type: &str) -> Option<openapiv3::Type> {
    if raw_type.starts_with("Map  of") || raw_type.starts_with("Map[") || raw_type == "Map" {
        Some(openapiv3::Type::Object(openapiv3::ObjectType {
            additional_properties: Some(openapiv3::AdditionalProperties::Any(true)),
            ..Default::default()
        }))
    } else {
        None
    }
}

fn wrapper(raw_type: &str) -> Option<openapiv3::Type> {
    const START: &str = "[";
    const END: &str = "]";
    if raw_type.starts_with(START) && raw_type.ends_with(END) {
        let inner_type = raw_type.get(START.len()..raw_type.len() - END.len())?;
        item_type(inner_type)
    } else {
        None
    }
}

pub fn item_type(raw_type: &str) -> Option<openapiv3::Type> {
    wrapper(&raw_type)
        .or_else(|| array_type(&raw_type))
        .or_else(|| set_type(&raw_type))
        .or_else(|| map_type(&raw_type))
        .or_else(|| match raw_type {
            "Integer" => Some(openapiv3::Type::Integer(openapiv3::IntegerType {
                format: openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::IntegerFormat::Int32),
                ..Default::default()
            })),
            "Long" => Some(openapiv3::Type::Integer(openapiv3::IntegerType {
                format: openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::IntegerFormat::Int64),
                ..Default::default()
            })),
            "Boolean" => Some(openapiv3::Type::Boolean {}),
            "Object" | "[Object]" => Some(openapiv3::Type::Object(Default::default())),
            "string" | "String" => Some(openapiv3::Type::String(Default::default())),
            "List" => Some(openapiv3::Type::Array(openapiv3::ArrayType {
                items: parse_type_boxed("Object"),
                min_items: None,
                max_items: None,
                unique_items: false,
            })),
            _ => None,
        })
}

pub fn parse_type(raw_type: &str) -> openapiv3::ReferenceOr<openapiv3::Schema> {
    if let Some(simple_type) = item_type(raw_type) {
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

fn parse_type_boxed(raw_type: &str) -> openapiv3::ReferenceOr<Box<Schema>> {
    if let Some(simple_type) = item_type(raw_type) {
        openapiv3::ReferenceOr::Item(Box::new(Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(simple_type),
        }))
    } else {
        openapiv3::ReferenceOr::Reference {
            reference: format!("#/components/schemas/{}", raw_type),
        }
    }
}

fn parse_schema(section: scraper::element_ref::ElementRef<'_>) -> Schema {
    let properties = section
        .select(&ROW_SELECTOR)
        .map(|row| {
            (
                row.select(&PROPERTY_NAME_SELECTOR)
                    .next()
                    .unwrap()
                    .text()
                    .collect::<String>(),
                parse_type_boxed(
                    &row.select(&TYPE_SELECTOR)
                        .next()
                        .unwrap()
                        .text()
                        .collect::<String>(),
                ),
            )
        })
        .collect();
    Schema {
        schema_data: Default::default(),
        schema_kind: SchemaKind::Type(openapiv3::Type::Object(ObjectType {
            properties,
            ..Default::default()
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_schemas;
    use openapiv3::OpenAPI;
    use scraper::Html;

    const HTML: &str = include_str!("../../../keycloak/9.0.html");
    const JSON: &str = include_str!("../../../keycloak/9.0.json");

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

    #[test]
    fn parses_schema_with_int64_as_expected() {
        parse_schema_correctly("MemoryInfoRepresentation");
    }

    #[test]
    fn parses_schema_only_map_as_expected() {
        parse_schema_correctly("SpiInfoRepresentation");
    }

    #[test]
    fn parses_schema_with_string_array_as_expected() {
        parse_schema_correctly("GlobalRequestResult");
    }

    #[test]
    fn parses_schema_with_enum_as_expected() {
        parse_schema_correctly("PolicyRepresentation");
    }

    #[test]
    fn parses_schema_with_object_as_expected() {
        parse_schema_correctly("ConfigPropertyRepresentation");
    }

    #[test]
    fn parses_schema_with_reference_as_expected() {
        parse_schema_correctly("ComponentExportRepresentation");
    }

    #[test]
    fn parses_schema_only_reference_array_as_expected() {
        parse_schema_correctly("AccessToken-Authorization");
    }
}
