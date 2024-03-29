use openapiv3::{OpenAPI, ReferenceOr, SecurityRequirement, SecurityScheme};
use scraper::Html;
use serde_json::to_string_pretty;
#[macro_use]
extern crate lazy_static;
use indexmap::IndexMap;
use std::io::{self, Read};

mod components;
mod info;
mod paths;
mod table;

const ACCESS_TOKEN: &str = "access_token";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut html = String::new();
    io::stdin().read_to_string(&mut html)?;
    let document = Html::parse_document(&html);

    let mut security_schemes = IndexMap::new();
    security_schemes.insert(
        ACCESS_TOKEN.to_string(),
        ReferenceOr::Item(SecurityScheme::HTTP {
            scheme: "bearer".to_string(),
            bearer_format: None,
            description: None,
        }),
    );

    let mut security_requirement: SecurityRequirement = IndexMap::new();
    security_requirement.insert(ACCESS_TOKEN.to_string(), Vec::new());

    let (paths, tags) = paths::paths(&document);

    let mut tags: Vec<String> = tags.into_iter().collect();

    tags.sort();

    let tags = tags
        .into_iter()
        .map(|tag| openapiv3::Tag {
            name: tag,
            ..Default::default()
        })
        .collect();

    let spec = OpenAPI {
        openapi: "3.0.2".to_string(),
        info: info::parse(&document)?,
        components: Some(openapiv3::Components {
            schemas: components::schemas::parse_schemas(&document),
            security_schemes,
            ..Default::default()
        }),
        paths,
        security: Some(vec![security_requirement]),
        tags,
        servers: vec![openapiv3::Server {
            url: "https://keycloak.example.com/admin/realms".into(),
            ..Default::default()
        }],
        ..Default::default()
    };

    println!("{}", to_string_pretty(&spec)?);
    Ok(())
}
