use openapiv3::OpenAPI;
use scraper::Html;
use serde_json::to_string_pretty;
#[macro_use]
extern crate lazy_static;
use std::io::{self, Read};

mod components;
mod info;
mod paths;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut html = String::new();
    io::stdin().read_to_string(&mut html)?;
    let document = Html::parse_document(&html);
    let spec = OpenAPI {
        openapi: "3.0.2".to_string(),
        info: info::parse(&document)?,
        components: Some(openapiv3::Components {
            schemas: components::schemas::parse_schemas(&document),
            ..Default::default()
        }),
        external_docs: Some(openapiv3::ExternalDocumentation {
            description: Some("Schema source code".to_string()),
            url: "https://github.com/keycloak/keycloak/tree/6.0.1/core/src/main/java/org/keycloak/representations".to_string()
        }),
        paths: paths::paths(&document),
        ..Default::default()
    };

    println!("{}", to_string_pretty(&spec)?);
    Ok(())
}
