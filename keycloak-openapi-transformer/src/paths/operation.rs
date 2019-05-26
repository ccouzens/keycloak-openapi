use scraper::Selector;

pub fn parse(section: &scraper::element_ref::ElementRef<'_>) -> openapiv3::Operation {
    let summary_selector = Selector::parse("h4:first-child").unwrap();

    openapiv3::Operation {
        summary: section
            .select(&summary_selector)
            .next()
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
