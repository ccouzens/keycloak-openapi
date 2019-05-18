use cssparser;
use openapiv3::Info;
use scraper::Selector;
use selectors;

#[derive(Debug, PartialEq)]
pub enum TransformError {
    SelectorErr(cssparser::ParseError<'static, selectors::parser::SelectorParseErrorKind<'static>>),
    NoFindErr(&'static str),
}

impl std::fmt::Display for TransformError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TransformError::NoFindErr(selector) => {
                write!(f, "Could not find element by {}", selector)
            }
            x => std::fmt::Display::fmt(x, f),
        }
    }
}

impl std::error::Error for TransformError {}

fn extract_string(
    document: &scraper::html::Html,
    selector: &'static str,
) -> Result<String, TransformError> {
    Ok(document
        .select(&Selector::parse(selector).map_err(TransformError::SelectorErr)?)
        .next()
        .ok_or_else(|| TransformError::NoFindErr(selector))?
        .text()
        .collect::<String>()
        .trim()
        .to_string())
}

pub fn parse(document: &scraper::html::Html) -> Result<Info, TransformError> {
    Ok(Info {
        title: extract_string(document, "h1")?,
        description: Some(extract_string(
            document,
            "#_overview + .sectionbody > .paragraph",
        )?),
        version: extract_string(document, "#_version_information + .paragraph")?
            .split("Version: ")
            .collect(),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::parse;
    use openapiv3::OpenAPI;
    use scraper::Html;

    #[test]
    fn parses_as_expected() {
        const HTML: &str = include_str!("../../keycloak/6.0.html");
        const JSON: &str = include_str!("../../keycloak/6.0.json");
        let openapi: OpenAPI = serde_json::from_str(JSON).expect("Could not deserialize example");

        assert_eq!(Ok(openapi.info), parse(&Html::parse_document(HTML)));
    }
}
