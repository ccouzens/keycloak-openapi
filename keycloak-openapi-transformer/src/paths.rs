use scraper::Selector;

pub mod parameters;

pub fn paths(document: &scraper::html::Html) -> openapiv3::Paths {
    let path_section_selector =
        Selector::parse("#_paths + .sectionbody > .sect2 > .sect3").unwrap();
    let params_table_selector = Selector::parse("h5[id^=_parameters] + table").unwrap();
    let summary_selector = Selector::parse("h4:first-child").unwrap();

    let mut paths = openapiv3::Paths::default();

    for section in document.select(&path_section_selector) {
        let (verb, path) = verb_path_split(&section);
        if let openapiv3::ReferenceOr::Item(path_item) = paths.entry(path).or_insert_with(|| {
            let params_section = section.select(&params_table_selector).next();
            openapiv3::ReferenceOr::Item(openapiv3::PathItem {
                parameters: if let Some(s) = params_section {
                    parameters::parse_path(&s)
                } else {
                    Default::default()
                },
                ..Default::default()
            })
        }) {
            if verb == "DELETE" {
                path_item.delete = Some(openapiv3::Operation {
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
                })
            }
        }
    }

    paths
}

fn verb_path_split(section: &scraper::element_ref::ElementRef<'_>) -> (String, String) {
    let verb_path = section
        .select(&Selector::parse("pre").unwrap())
        .next()
        .or_else(|| section.select(&Selector::parse("h4").unwrap()).next())
        .unwrap()
        .text()
        .collect::<String>();
    let mut split = verb_path.split_whitespace();
    (
        split.next().unwrap().to_string(),
        split.next().unwrap().to_string(),
    )
}

#[cfg(test)]
mod tests {
    const HTML: &str = include_str!("../../keycloak/6.0.html");

    mod parameters {
        use super::super::paths;
        use super::HTML;
        use openapiv3::ReferenceOr;
        use scraper::Html;

        #[test]
        fn correctly_parses_when_there_are_no_parameters() {
            let paths = paths(&Html::parse_document(HTML));
            let path = if let ReferenceOr::Item(path) = paths.get("/{any}").unwrap() {
                path
            } else {
                panic!("Couldn't extract path")
            };
            assert_eq!(path.parameters, vec![]);
        }

        #[test]
        fn correctly_parses_when_there_are_three_parameters() {
            let paths = paths(&Html::parse_document(HTML));
            let path = if let ReferenceOr::Item(path) = paths
                .get("/{realm}/client-scopes/{id}/protocol-mappers/protocol/{protocol}")
                .unwrap()
            {
                path
            } else {
                panic!("Couldn't extract path")
            };
            assert_eq!(path.parameters.len(), 3);
        }
    }

    mod delete {
        use super::super::paths;
        use super::HTML;
        use openapiv3::ReferenceOr;
        use scraper::Html;

        #[test]
        fn correctly_parses_simple_case() {
            let paths = paths(&Html::parse_document(HTML));
            let path_item = if let ReferenceOr::Item(path) = paths.get("/{realm}").unwrap() {
                path
            } else {
                panic!("Couldn't extract path")
            };
            assert_eq!(
                path_item.delete.as_ref().and_then(|op| op.summary.as_ref()),
                Some(&"Delete the realm".to_string())
            );
        }
    }
}
