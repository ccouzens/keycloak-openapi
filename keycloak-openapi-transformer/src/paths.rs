use scraper::Selector;

mod operation;
pub mod parameters;
mod response;

lazy_static! {
    static ref PATH_SECTION_SELECTOR: Selector =
        Selector::parse("#_paths + .sectionbody > .sect2 > .sect3").unwrap();
    static ref PARAMS_TABLE_SELECTOR: Selector =
        Selector::parse("h5[id^=_parameters] + table").unwrap();
    static ref SUMMARY_SELECTOR: Selector = Selector::parse("h4:first-child").unwrap();
    static ref PRE_PATH_SELECTOR: Selector = Selector::parse("pre").unwrap();
}

pub fn paths(document: &scraper::html::Html) -> openapiv3::Paths {
    let mut paths = openapiv3::Paths::default();

    let sections = document.select(&PATH_SECTION_SELECTOR).collect::<Vec<_>>();
    for section in sections.iter().rev() {
        let (verb, path) = verb_path_split(&section);
        if path == "/{any}" {
            continue;
        };
        if let openapiv3::ReferenceOr::Item(path_item) =
            paths.entry(path.clone()).or_insert_with(|| {
                let params_section = section.select(&PARAMS_TABLE_SELECTOR).next();
                openapiv3::ReferenceOr::Item(openapiv3::PathItem {
                    parameters: if let Some(s) = params_section {
                        parameters::parse_path(&s, &path)
                    } else {
                        Default::default()
                    },
                    ..Default::default()
                })
            })
        {
            let operation = Some(operation::parse(&section));
            (match verb.as_ref() {
                "DELETE" => {
                    path_item.delete = operation;
                }
                "GET" => {
                    path_item.get = operation;
                }
                "POST" => {
                    path_item.post = operation;
                }
                "PUT" => {
                    path_item.put = operation;
                }
                "OPTIONS" => {
                    path_item.options = operation;
                }
                _ => panic!(format!("Unexpected HTTP verb: {:?}", verb)),
            });
        }
    }

    paths
}

fn verb_path_split(section: &scraper::element_ref::ElementRef<'_>) -> (String, String) {
    let verb_path = section
        .select(&PRE_PATH_SELECTOR)
        .next()
        .or_else(|| section.select(&SUMMARY_SELECTOR).next())
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
            let path = if let Some(ReferenceOr::Item(path)) = paths.get("/") {
                path
            } else {
                panic!("Couldn't extract path")
            };
            assert_eq!(path.parameters, vec![]);
        }

        #[test]
        fn correctly_parses_when_there_are_three_parameters() {
            let paths = paths(&Html::parse_document(HTML));
            let path = if let Some(ReferenceOr::Item(path)) =
                paths.get("/{realm}/client-scopes/{id}/protocol-mappers/protocol/{protocol}")
            {
                path
            } else {
                panic!("Couldn't extract path")
            };
            let names: Vec<_> = path
                .parameters
                .iter()
                .filter_map(|p| match p {
                    openapiv3::ReferenceOr::Item(openapiv3::Parameter::Path {
                        parameter_data: openapiv3::ParameterData { name, .. },
                        ..
                    }) => Some(name),
                    _ => None,
                })
                .collect();
            assert_eq!(names, vec!["realm", "id", "protocol"]);
        }

        #[test]
        fn adds_descriptions_when_not_always_present() {
            let paths = paths(&Html::parse_document(HTML));
            let path_item = if let Some(ReferenceOr::Item(path)) =
                paths.get("/{realm}/roles-by-id/{role-id}/composites")
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
            if let Some(ReferenceOr::Item(path)) = paths.get(path) {
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

        // This path is problematic as it doesn't have it's parameter defined.
        // Additionally, it couldn't be defined as sub paths can't be substituted in
        #[test]
        fn does_not_parse_the_any_path() {
            let paths = paths(&Html::parse_document(HTML));
            assert!(!paths.contains_key("/{any}"));
        }

        #[test]
        fn correctly_parses_the_post_case() {
            let path_item = get_path("/{realm}/testLDAPConnection");
            assert_eq!(
                path_item.post.as_ref().and_then(|op| op.summary.as_ref()),
                Some(&"Test LDAP connection".to_string())
            );
        }

        #[test]
        fn correctly_parses_the_put_case() {
            let path_item = get_path("/{realm}/users/{id}");
            assert_eq!(
                path_item.put.as_ref().and_then(|op| op.summary.as_ref()),
                Some(&"Update the user".to_string())
            );
        }
    }
}
