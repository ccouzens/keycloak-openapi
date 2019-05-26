use scraper::Selector;

mod operation;
pub mod parameters;

pub fn paths(document: &scraper::html::Html) -> openapiv3::Paths {
    let path_section_selector =
        Selector::parse("#_paths + .sectionbody > .sect2 > .sect3").unwrap();
    let params_table_selector = Selector::parse("h5[id^=_parameters] + table").unwrap();

    let mut paths = openapiv3::Paths::default();

    let sections = document.select(&path_section_selector).collect::<Vec<_>>();
    for section in sections.iter().rev() {
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
            match verb.as_ref() {
                "DELETE" => {
                    path_item.delete = Some(operation::parse(&section));
                }
                "GET" => {
                    path_item.get = Some(operation::parse(&section));
                }
                "OPTIONS" => {
                    path_item.options = Some(operation::parse(&section));
                }
                _ => {}
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

        #[test]
        fn adds_descriptions_when_not_always_present() {
            let paths = paths(&Html::parse_document(HTML));
            let path_item = if let ReferenceOr::Item(path) = paths
                .get("/{realm}/roles-by-id/{role-id}/composites")
                .unwrap()
            {
                path
            } else {
                panic!("Couldn't extract path")
            };
            for reference in path_item.parameters.iter() {
                if let ReferenceOr::Item(openapiv3::Parameter::Path { parameter_data, .. }) =
                    reference
                {
                    assert_ne!(None, parameter_data.description);
                    assert_ne!(Some("".to_string()), parameter_data.description);
                }
            }
        }
    }

    mod operatitions {
        use super::super::paths;
        use super::HTML;
        use openapiv3::ReferenceOr;
        use scraper::Html;

        fn get_path(path: &str) -> openapiv3::PathItem {
            let paths = paths(&Html::parse_document(HTML));
            if let ReferenceOr::Item(path) = paths.get(path).unwrap() {
                path.clone()
            } else {
                panic!("Couldn't extract path")
            }
        }

        #[test]
        fn correctly_parses_simple_delete_case() {
            let path_item = get_path("/{realm}");
            assert_eq!(
                path_item.delete.as_ref().and_then(|op| op.summary.as_ref()),
                Some(&"Delete the realm".to_string())
            );
        }

        #[test]
        fn correctly_parses_the_get_case() {
            let path_item = get_path("/{realm}/groups/{id}/role-mappings");
            assert_eq!(
                path_item.get.as_ref().and_then(|op| op.summary.as_ref()),
                Some(&"Get role mappings".to_string())
            );
        }

        #[test]
        fn correctly_parses_the_options_case() {
            let path_item = get_path("/{any}");
            assert_eq!(
                path_item
                    .options
                    .as_ref()
                    .and_then(|op| op.summary.as_ref()),
                Some(&"CORS preflight".to_string())
            );
        }

    }
}
