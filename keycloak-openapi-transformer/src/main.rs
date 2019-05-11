use openapiv3::OpenAPI;
use scraper::Html;
use serde_json::to_string_pretty;
use std::collections::BTreeMap;

const HTML: &str = include_str!("../../keycloak/6.0.html");

mod components;
mod info;

fn main() -> Result<(), Box<std::error::Error>> {
    let document = Html::parse_document(HTML);
    let spec = OpenAPI {
        openapi: "3.0.2".to_string(),
        info: info::parse(&document)?,
        servers: vec![],
        paths: BTreeMap::new(),
        components: Some(openapiv3::Components {
            security_schemes: BTreeMap::new(),
            responses: BTreeMap::new(),
            parameters: BTreeMap::new(),
            examples: BTreeMap::new(),
            request_bodies: BTreeMap::new(),
            headers: BTreeMap::new(),
            schemas: [(
                "AddressClaimSet".to_string(),
                openapiv3::ReferenceOr::Item(components::schemas::parse_schema(
                    &document,
                    "AddressClaimSet",
                )),
            )]
            .iter()
            .cloned()
            .collect(),
            links: BTreeMap::new(),
            callbacks: BTreeMap::new(),
        }),
        security: vec![],
        tags: vec![],
        external_docs: None,
    };

    println!("{}", to_string_pretty(&spec)?);
    Ok(())
}
