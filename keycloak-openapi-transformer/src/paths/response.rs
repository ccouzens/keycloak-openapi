use super::super::components::schemas::parse_type;
use openapiv3::MediaType;
use scraper::Selector;

lazy_static! {
    static ref RESPONSES_SELECTOR: Selector =
        Selector::parse("h5[id^=_responses] + table > tbody > tr").unwrap();
    static ref PRODUCES_SELECTOR: Selector =
        Selector::parse("h5[id^=_content_type] + div code").unwrap();
    static ref DESCRIPTION_SELECTOR: Selector = Selector::parse("td:first-child + td").unwrap();
    static ref SCHEMA_SELECTOR: Selector = Selector::parse("td:first-child + td + td").unwrap();
}

pub fn parse(section: &scraper::element_ref::ElementRef<'_>) -> openapiv3::Response {
    let response_table = section.select(&RESPONSES_SELECTOR).next().unwrap();
    let description = response_table
        .select(&DESCRIPTION_SELECTOR)
        .next()
        .unwrap()
        .text()
        .collect();
    let raw_schema: String = response_table
        .select(&SCHEMA_SELECTOR)
        .next()
        .unwrap()
        .text()
        .collect();
    let media_type = section
        .select(&PRODUCES_SELECTOR)
        .next()
        .map(|p| p.text().collect::<String>());

    let content = match (media_type, raw_schema.as_ref()) {
        (None, _) | (_, "<<>>") => Default::default(),
        (Some(produces), _) => [(
            produces,
            MediaType {
                schema: Some(parse_type(&raw_schema)),
                ..Default::default()
            },
        )]
        .iter()
        .cloned()
        .collect(),
    };
    openapiv3::Response {
        description,
        content,
        ..Default::default()
    }
}

#[cfg(test)]
mod test {
    const HTML: &str = include_str!("../../../keycloak/22.0.0.html");
    use super::parse;
    use indexmap::IndexMap;
    use openapiv3::MediaType;
    use scraper::Html;
    use scraper::Selector;

    #[test]
    fn octet_streams() {
        const CSS_SELECTOR: &str = ".adminRealmsRealmClientsIdCertificatesAttrDownloadPost";
        const EXPECTED: &str = r#"
        {
            "application/octet-stream": {
                "schema": {
                    "type": "string",
                    "format": "binary"
                }
            }
        }
        "#;
        let document = Html::parse_document(HTML);
        let section = document
            .select(&Selector::parse(CSS_SELECTOR).unwrap())
            .next()
            .unwrap();
        assert_eq!(
            parse(&section).content,
            serde_json::from_str::<IndexMap<String, MediaType>>(EXPECTED).unwrap()
        );
    }

    #[test]
    fn json() {
        const CSS_SELECTOR: &str = ".adminRealmsRealmAuthenticationConfigIdGet";
        const EXPECTED: &str = r##"
        {
            "application/json": {
                "schema": {
                    "type": "array",
                    "$ref": "#/components/schemas/AuthenticatorConfigRepresentation"
                }
            }
        }
        "##;
        let document = Html::parse_document(HTML);
        let section = document
            .select(&Selector::parse(CSS_SELECTOR).unwrap())
            .next()
            .unwrap();

        assert_eq!(
            parse(&section).content,
            serde_json::from_str::<IndexMap<String, MediaType>>(EXPECTED).unwrap()
        );
    }

    #[test]
    fn no_content() {
        const CSS_SELECTOR: &str = ".adminRealmsRealmAttackDetectionBruteForceUsersDelete";
        let document = Html::parse_document(HTML);
        let section = document
            .select(&Selector::parse(CSS_SELECTOR).unwrap())
            .next()
            .unwrap();
        assert!(parse(&section).content.is_empty());
    }

    #[test]
    fn no_content_response() {
        const CSS_SELECTOR: &str = ".adminRealmsRealmAttackDetectionBruteForceUsersDelete";
        let document = Html::parse_document(HTML);
        let section = document
            .select(&Selector::parse(CSS_SELECTOR).unwrap())
            .next()
            .unwrap();
        assert!(parse(&section).content.is_empty());
    }
}
