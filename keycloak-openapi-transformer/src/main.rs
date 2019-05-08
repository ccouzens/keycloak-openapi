use cssparser;
use openapiv3::Info;
use scraper::{Html, Selector};
use selectors;
use std::error::Error;

const HTML: &str = include_str!("../../keycloak/6.0.html");

#[derive(Debug)]
enum TransformError<'i> {
    SelectorErr(cssparser::ParseError<'i, selectors::parser::SelectorParseErrorKind<'i>>),
    NoFindErr,
    JsonErr(serde_json::error::Error),
}

impl<'i> std::fmt::Display for TransformError<'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TransformError::NoFindErr => write!(f, "Could not find element"),
            x => std::fmt::Display::fmt(x, f),
        }
    }
}

impl<'i> std::error::Error for TransformError<'i> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TransformError::SelectorErr(_) => None,
            TransformError::NoFindErr => None,
            TransformError::JsonErr(x) => Some(x),
        }
    }
}

fn main() -> Result<(), TransformError<'static>> {
    let document = Html::parse_document(HTML);

    let info = Info {
        title: document
            .select(&Selector::parse("h1").map_err(|e| TransformError::SelectorErr(e))?)
            .next()
            .ok_or(TransformError::NoFindErr)?
            .text()
            .collect(),
        description: Some(
            document
                .select(
                    &Selector::parse("#_overview + .sectionbody > .paragraph")
                        .map_err(|e| TransformError::SelectorErr(e))?,
                )
                .next()
                .ok_or(TransformError::NoFindErr)?
                .text()
                .collect::<String>()
                .trim()
                .to_string(),
        ),
        terms_of_service: None,
        contact: None,
        license: None,
        version: document
            .select(
                &Selector::parse("#_version_information + .paragraph")
                    .map_err(|e| TransformError::SelectorErr(e))?,
            )
            .next()
            .ok_or(TransformError::NoFindErr)?
            .text()
            .collect::<String>()
            .trim()
            .split("Version: ")
            .collect(),
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&info).map_err(|e| TransformError::JsonErr(e))?
    );
    Ok(())

    // let openapi: OpenAPI = OpenAPI { openapi: String::from("3.0.0"), info:  };
}
