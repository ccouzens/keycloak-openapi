use openapiv3::Info;
use scraper::{Html, Selector};
use std::error::Error;

const HTML: &str = include_str!("../../keycloak/6.0.html");

fn main() -> Result<(), Box<dyn Error>> {
    let document = Html::parse_document(HTML);

    let info = Info {
        title: document
            .select(&Selector::parse("h1").unwrap())
            .next()
            .unwrap()
            .text()
            .collect(),
        description: document
            .select(&Selector::parse("#_overview + .sectionbody > .paragraph").unwrap())
            .next()
            .map(|n| n.text().collect::<String>().trim().to_string()),
        terms_of_service: None,
        contact: None,
        license: None,
        version: document
            .select(&Selector::parse("#_version_information + .paragraph").unwrap())
            .next()
            .unwrap()
            .text()
            .collect::<String>()
            .trim()
            .split("Version: ")
            .collect(),
    };

    println!("{}", serde_json::to_string_pretty(&info)?);
    Ok(())

    // let openapi: OpenAPI = OpenAPI { openapi: String::from("3.0.0"), info:  };
}
