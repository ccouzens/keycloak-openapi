use super::parameters::{parse_body_param, parse_query_params};
use super::response;
use scraper::Selector;

lazy_static! {
    static ref SUMMARY_SELECTOR: Selector =
        Selector::parse("h4:first-child + div.paragraph > p").unwrap();
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
                openapiv3::ReferenceOr::Item(response::parse(section)),
            )]
            .iter()
            .cloned()
            .collect(),
            extensions: Default::default(),
        },
        parameters: parse_query_params(section),
        request_body: parse_body_param(section),
        ..Default::default()
    }
}
