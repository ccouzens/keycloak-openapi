use scraper::Selector;

pub fn parse(section: &scraper::element_ref::ElementRef<'_>) -> openapiv3::Operation {
    let summary_selector = Selector::parse("h4:first-child").unwrap();
    let pre_path_selector = Selector::parse("pre").unwrap();

    openapiv3::Operation {
        summary: section
            .select(&pre_path_selector)
            .next()
            .and_then(|_| section.select(&summary_selector).next())
            .map(|s| s.text().collect()),
        responses: openapiv3::Responses {
            default: None,
            responses: [(
                "2XX".to_string(),
                openapiv3::ReferenceOr::Item(openapiv3::Response {
                    description: "success".to_string(),
                    ..Default::default()
                }),
            )]
            .iter()
            .cloned()
            .collect(),
        },
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    const HTML: &str = include_str!("../../../keycloak/6.0.html");
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
