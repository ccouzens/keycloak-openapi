use super::parameters::parse_parameters;
use super::request_body;
use super::response;
use scraper::Selector;

lazy_static! {
    static ref SUMMARY_SELECTOR: Selector = Selector::parse("h4:first-child").unwrap();
    static ref PRE_PATH_SELECTOR: Selector = Selector::parse("pre").unwrap();
}

pub fn parse(section: &scraper::element_ref::ElementRef<'_>) -> openapiv3::Operation {
    openapiv3::Operation {
        summary: section
            .select(&PRE_PATH_SELECTOR)
            .next()
            .and_then(|_| section.select(&SUMMARY_SELECTOR).next())
            .map(|s| s.text().collect()),
        responses: openapiv3::Responses {
            default: None,
            responses: [(
                "2XX".to_string(),
                openapiv3::ReferenceOr::Item(response::parse(&section)),
            )]
            .iter()
            .cloned()
            .collect(),
        },
        parameters: parse_parameters(section, "Query"),
        request_body: request_body::parse(section),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    const HTML: &str = include_str!("../../../keycloak/8.0.html");
    use super::parse;
    use scraper::Html;
    use scraper::Selector;

    #[test]
    fn most_operations_have_summaries() {
        const CSS_SELECTOR: &str =
            "#_paths + .sectionbody > .sect2 > #_client_initial_access_resource + .sect3";
        let document = Html::parse_document(&HTML);
        let section = document
            .select(&Selector::parse(CSS_SELECTOR).unwrap())
            .next()
            .unwrap();
        assert_eq!(
            parse(&section).summary,
            Some("Create a new initial access token.".to_string())
        );
    }

    #[test]
    fn some_operations_do_not_have_summaries() {
        const CSS_SELECTOR: &str =
            "#_paths + .sectionbody > .sect2 > #_client_initial_access_resource + .sect3 + .sect3";
        let document = Html::parse_document(&HTML);
        let section = document
            .select(&Selector::parse(CSS_SELECTOR).unwrap())
            .next()
            .unwrap();
        assert_eq!(parse(&section).summary, None);
    }
}
