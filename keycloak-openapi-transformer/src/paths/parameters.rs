use openapiv3::{Parameter, ReferenceOr};
fn parse_path(section: &scraper::element_ref::ElementRef<'_>) -> Vec<ReferenceOr<Parameter>> {
    vec![]
}

#[cfg(test)]
mod tests {
    use super::parse_path;
    use openapiv3::{OpenAPI, ReferenceOr};
    use scraper::Html;
    use scraper::Selector;

    const HTML: &str = include_str!("../../../keycloak/6.0.html");
    const JSON: &str = include_str!("../../../keycloak/6.0.json");

    fn parse_parameters_correctly(html_selector: &str, path: &str) {
        let openapi: OpenAPI = serde_json::from_str(JSON).expect("Could not deserialize example");
        let path = if let ReferenceOr::Item(path) = openapi.paths.get(path).unwrap() {
            path
        } else {
            panic!("Couldn't extract path")
        };
        assert_eq!(
            path.parameters,
            parse_path(
                &Html::parse_document(HTML)
                    .select(&Selector::parse(html_selector).unwrap())
                    .next()
                    .unwrap()
            )
        );
    }

    #[test]
    fn correctly_parses_realm() {
        parse_parameters_correctly(
            "#_paths + .sectionbody > .sect2 > #_attack_detection_resource + .sect3 [id=_parameters] + table",
            "/{realm}/attack-detection/brute-force/users"
        );
    }

}
