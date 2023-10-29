use super::parameters::{parse_body_param, parse_query_params};
use super::response;
use scraper::Selector;

lazy_static! {
    static ref SUMMARY_SELECTOR: Selector =
        Selector::parse("h4:first-child + div.paragraph > p").unwrap();
    static ref STATUS_CODE_SELECTOR: Selector =
        Selector::parse(".sect4 > .stretch .valign-top:nth-child(1) .tableblock").unwrap();
}

fn parse_status_code(status_code_str: &str) -> Option<openapiv3::StatusCode> {
    let status_code_u16 = status_code_str.parse().ok()?;

    if !(100..=599).contains(&status_code_u16) {
        return None;
    }

    Some(openapiv3::StatusCode::Code(status_code_u16))
}

fn apply_selector(
    section: &scraper::element_ref::ElementRef<'_>,
    selector: &Selector,
) -> Option<String> {
    section
        .select(selector)
        .next()
        .map(|s| s.text().collect::<String>().trim().into())
}

fn status_code(section: &scraper::element_ref::ElementRef<'_>) -> openapiv3::StatusCode {
    match apply_selector(section, &STATUS_CODE_SELECTOR) {
        Some(code) => parse_status_code(&code).unwrap_or(openapiv3::StatusCode::Range(2)),
        None => openapiv3::StatusCode::Range(2),
    }
}

pub fn parse(section: &scraper::element_ref::ElementRef<'_>) -> openapiv3::Operation {
    openapiv3::Operation {
        description: apply_selector(section, &SUMMARY_SELECTOR)
            .or(Some("Missing description".into())),
        responses: openapiv3::Responses {
            default: None,
            responses: [(
                status_code(section),
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
