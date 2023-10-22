use super::parameters::{parse_body_param, parse_query_params};
use super::response;
use scraper::Selector;

lazy_static! {
    static ref SUMMARY_SELECTOR: Selector = Selector::parse("h4:first-child + div").unwrap();
}

pub fn parse(section: &scraper::element_ref::ElementRef<'_>) -> openapiv3::Operation {
    openapiv3::Operation {
        summary: section
            .select(&SUMMARY_SELECTOR)
            .next()
            .map(|s| s.text().collect::<String>().trim().into()),
        responses: openapiv3::Responses {
            default: None,
            responses: [(
                openapiv3::StatusCode::Range(2),
                openapiv3::ReferenceOr::Item(response::parse(&section)),
            )]
            .iter()
            .cloned()
            .collect(),
        },
        parameters: parse_query_params(section),
        request_body: parse_body_param(section),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    const HTML: &str = include_str!("../../../keycloak/9.0.html");
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
