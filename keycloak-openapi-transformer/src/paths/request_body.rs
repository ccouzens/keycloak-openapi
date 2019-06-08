use super::super::components::schemas::parse_type;
use super::parameters;
use openapiv3::{MediaType, ReferenceOr, RequestBody};
use scraper::Selector;

lazy_static! {
    static ref CONSUMES_SELECTOR: Selector =
        Selector::parse("[id^=_consumes] + .ulist code").unwrap();
}

pub fn parse(section: &scraper::element_ref::ElementRef<'_>) -> Option<ReferenceOr<RequestBody>> {
    let mut body_row =
        parameters::parse_parameter_rows(section)?.find(|row| row.parameter_type == "Body")?;
    Some(ReferenceOr::Item(RequestBody {
        description: body_row.description.take(),
        required: body_row.required,
        content: section
            .select(&CONSUMES_SELECTOR)
            .map(|content_section| {
                (
                    content_section.text().collect(),
                    MediaType {
                        schema: Some(parse_type(&body_row.schema)),
                        ..Default::default()
                    },
                )
            })
            .collect(),
    }))
}
