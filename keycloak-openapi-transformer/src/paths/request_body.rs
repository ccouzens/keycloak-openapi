use super::super::components::schemas::parse_type;
use super::parameters;
use openapiv3::{MediaType, ReferenceOr, RequestBody};
use scraper::Selector;
use std::collections::BTreeMap;

lazy_static! {
    static ref CONSUMES_SELECTOR: Selector =
        Selector::parse("[id^=_consumes] + .ulist code").unwrap();
}

pub fn parse(section: &scraper::element_ref::ElementRef<'_>) -> Option<ReferenceOr<RequestBody>> {
    let body_row =
        parameters::parse_parameter_rows(section)?.find(|row| row.parameter_type == "Body")?;
    let is_text = section
        .select(&CONSUMES_SELECTOR)
        .map(|c| c.text().collect())
        .any(|t: String| t == "text/plain");
    let media_type = if is_text {
        "text/plain"
    } else {
        "application/json"
    };
    let mut content = BTreeMap::new();
    content.insert(
        media_type.to_string(),
        MediaType {
            schema: Some(parse_type(&body_row.schema)),
            ..Default::default()
        },
    );
    Some(ReferenceOr::Item(RequestBody {
        description: body_row.description,
        required: body_row.required,
        content,
    }))
}

#[cfg(test)]
mod tests {
    const HTML: &str = include_str!("../../../keycloak/6.0.html");
    use super::parse;
    use openapiv3::{ReferenceOr, RequestBody};
    use scraper::Html;
    use scraper::Selector;

    #[test]
    fn most_operations_are_json() {
        const CSS_SELECTOR: &str =
            "#_paths + .sectionbody > .sect2 > #_authentication_management_resource + .sect3 + .sect3 + .sect3 + .sect3 + .sect3";
        let document = Html::parse_document(&HTML);
        let section = document
            .select(&Selector::parse(CSS_SELECTOR).unwrap())
            .next()
            .unwrap();
        if let Some(ReferenceOr::Item(RequestBody { content, .. })) = parse(&section) {
            assert!(content.contains_key("application/json"));
        } else {
            panic!("couldn't get content")
        };
    }

    #[test]
    fn operations_have_a_type_even_when_not_defined() {
        const CSS_SELECTOR: &str =
            "#_paths + .sectionbody > .sect2 > #_users_resource + .sect3 + .sect3 + .sect3 + .sect3 + .sect3 + .sect3 + .sect3 + .sect3 + .sect3 + .sect3 + .sect3 + .sect3";
        let document = Html::parse_document(&HTML);
        let section = document
            .select(&Selector::parse(CSS_SELECTOR).unwrap())
            .next()
            .unwrap();
        if let Some(ReferenceOr::Item(RequestBody { content, .. })) = parse(&section) {
            assert!(content.contains_key("application/json"));
        } else {
            panic!("couldn't get content")
        };
    }

    #[test]
    fn string_outputs_are_text_only() {
        const CSS_SELECTOR: &str =
            "#_paths + .sectionbody > .sect2 > #_realms_admin_resource + .sect3 + .sect3 + .sect3 + .sect3 + .sect3 + .sect3 + .sect3 + .sect3 + .sect3 + .sect3";
        let document = Html::parse_document(&HTML);
        let section = document
            .select(&Selector::parse(CSS_SELECTOR).unwrap())
            .next()
            .unwrap();
        if let Some(ReferenceOr::Item(RequestBody { content, .. })) = parse(&section) {
            assert_eq!(content.len(), 1);
            assert!(content.contains_key("text/plain"));
        } else {
            panic!("couldn't get content")
        };
    }
}
