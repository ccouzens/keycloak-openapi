use super::parameters;
use openapiv3::{ReferenceOr, RequestBody};

pub fn parse(section: &scraper::element_ref::ElementRef<'_>) -> Option<ReferenceOr<RequestBody>> {
    let body_row =
        parameters::parse_parameter_rows(section)?.find(|row| row.parameter_type == "Body")?;
    Some(ReferenceOr::Item(RequestBody {
        description: body_row.description,
        required: body_row.required,
        ..Default::default()
    }))
}
