use openapiv3::OpenAPI;
use scraper::Html;
use scraper::Selector;
use serde_json::to_string_pretty;

const HTML: &str = include_str!("../../keycloak/6.0.html");

mod components;
mod info;
mod paths;

fn main() -> Result<(), Box<std::error::Error>> {
    let document = Html::parse_document(HTML);
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
        paths: document.select(
            &Selector::parse("#_paths + .sectionbody > .sect2 > .sect3")
            .unwrap()
        ).map(|s| (
            s.select(&Selector::parse("pre").unwrap()).next()
            .or_else(|| s.select(&Selector::parse("h4").unwrap()).next())
            .unwrap().text().collect::<String>().split_whitespace().nth(1).unwrap().to_string(),
            openapiv3::ReferenceOr::Item(openapiv3::PathItem{
                parameters: paths::parameters::parse_path(&s),
                ..Default::default()
            }))).collect(),
        ..Default::default()
    };

    println!("{}", to_string_pretty(&spec)?);
    Ok(())
}
