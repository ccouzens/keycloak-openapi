use openapiv3::Info;
use scraper::Selector;

lazy_static! {
    static ref TITLE_SELECTOR: Selector = Selector::parse("h1").unwrap();
    static ref DESCRIPTION_SELECTOR: Selector =
        Selector::parse("#_overview + .sectionbody > .paragraph").unwrap();
    static ref VERSION_SELECTOR: Selector =
        Selector::parse("#_version_information + .paragraph").unwrap();
}

#[derive(Debug, PartialEq)]
pub enum TransformError {
    NoFindErr(String),
}

impl std::fmt::Display for TransformError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TransformError::NoFindErr(selector) => {
                write!(f, "Could not find element by {}", selector)
            }
        }
    }
}

impl std::error::Error for TransformError {}

fn extract_string(
    document: &scraper::html::Html,
    selector: &Selector,
) -> Result<String, TransformError> {
    Ok(document
        .select(&selector)
        .next()
        .ok_or_else(|| TransformError::NoFindErr(format!("{:?}", selector)))?
        .text()
        .collect::<String>()
        .trim()
        .to_string())
}

pub fn parse(document: &scraper::html::Html) -> Result<Info, TransformError> {
    Ok(Info {
        title: extract_string(document, &TITLE_SELECTOR)?,
        description: Some(extract_string(document, &DESCRIPTION_SELECTOR)?),
        version: extract_string(document, &VERSION_SELECTOR)?
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
