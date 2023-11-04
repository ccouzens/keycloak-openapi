use std::collections::{HashMap, HashSet};

use heck::ToLowerCamelCase;
use scraper::Selector;

mod operation;
mod parameters;
mod response;
mod verb_path;

use verb_path::VerbPath;

lazy_static! {
    static ref TAG_SECTION_SELECTOR: Selector =
        Selector::parse("#_resources + .sectionbody > .sect2").unwrap();
    static ref TAG_TITLE_SELECTOR: Selector = Selector::parse("h3").unwrap();
    static ref PATH_SECTION_SELECTOR: Selector = Selector::parse(".sect3").unwrap();
    static ref SUMMARY_SELECTOR: Selector = Selector::parse("h4:first-child").unwrap();
    static ref PRE_PATH_SELECTOR: Selector = Selector::parse("pre").unwrap();
}

pub fn paths(document: &scraper::html::Html) -> (openapiv3::Paths, HashSet<String>) {
    let mut paths = openapiv3::Paths::default();
    let mut tag_set = HashSet::new();
    let mut id_state_map: HashMap<String, usize> = HashMap::new();

    // First pass to find operation id collisions
    for tag_section in document.select(&TAG_SECTION_SELECTOR) {
        let sections = tag_section
            .select(&PATH_SECTION_SELECTOR)
            .collect::<Vec<_>>();

        for section in sections.iter().rev() {
            let verb_path = verb_path_split(section);

            if verb_path.unrepresentable() {
                continue;
            };

            for id in generate_operation_ids(&verb_path.verb, &verb_path.path()) {
                let seen: usize = *id_state_map.get(&id).unwrap_or(&0);

                id_state_map.insert(id.clone(), seen + 1);
            }
        }
    }

    for tag_section in document.select(&TAG_SECTION_SELECTOR) {
        let tag = tag_section
            .select(&TAG_TITLE_SELECTOR)
            .next()
            .unwrap()
            .text()
            .collect::<String>();

        let sections = tag_section
            .select(&PATH_SECTION_SELECTOR)
            .collect::<Vec<_>>();
        for section in sections.iter().rev() {
            let verb_path = verb_path_split(section);

            if verb_path.unrepresentable() {
                continue;
            };
            if let openapiv3::ReferenceOr::Item(path_item) =
                paths.paths.entry(verb_path.path()).or_insert_with(|| {
                    openapiv3::ReferenceOr::Item(openapiv3::PathItem {
                        parameters: parameters::parse_path(section, &verb_path),
                        ..Default::default()
                    })
                })
            {
                let mut operation = operation::parse(section);

                operation.operation_id = generate_operation_ids(&verb_path.verb, &verb_path.path())
                    .into_iter()
                    .find(|id| match id_state_map.get(id) {
                        Some(seen) => *seen < 2,
                        None => true,
                    });

                tag_set.insert(tag.clone());

                operation.tags = vec![tag.clone()];
                let operation = Some(operation);
                match verb_path.verb.as_ref() {
                    "DELETE" => {
                        path_item.delete = operation;
                    }
                    "GET" => {
                        path_item.get = operation;
                    }
                    "PATCH" => {
                        path_item.patch = operation;
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
                    _ => panic!("Unexpected HTTP verb: {:?}", verb_path.verb),
                };
            }
        }
    }
    paths.paths.sort_keys();

    (paths, tag_set)
}

fn verb_path_split(section: &scraper::element_ref::ElementRef<'_>) -> VerbPath {
    section
        .select(&PRE_PATH_SELECTOR)
        .next()
        .or_else(|| section.select(&SUMMARY_SELECTOR).next())
        .unwrap()
        .text()
        .collect::<String>()
        .parse()
        .unwrap()
}

/// Generate a list of possible operation ids starting with shortest
///
/// A possible id of `GET /{realm}/clients/{id}/roles/{role-name}/management/permissions`
/// is `getClientRoleManagementPermissions`.
///
/// Assumes that the each parameter is a selector
/// for the previous path segment. Assumes previous part is named as a list.
/// strip suffix of 's' to indicate singular
///
/// getClient(s)Role(s)ManagementPermissions
///
/// Last id is a fallback with all params appended at the end
///
/// `getClientRoleManagementPermissionsByRealmByRoleName`
fn generate_operation_ids(verb: &str, path: &str) -> Vec<String> {
    let path = path.trim_matches('/');

    let mut ids = vec![];

    let mut operation_id = String::new();

    let parts: Vec<&str> = path.split('/').collect();

    let mut parts_removed_params = vec![];

    let mut params = vec![];

    for (index, part) in parts.iter().enumerate() {
        let peek = parts.get(index + 1);

        if part.starts_with('{') && part.ends_with('}') {
            params.push(part.strip_prefix('{').unwrap().strip_suffix('}').unwrap());
            continue;
        }

        if let Some(peek) = peek {
            if peek.starts_with('{') && peek.ends_with('}') {
                let part = part.strip_suffix('s').unwrap_or(part);

                parts_removed_params.push(part);

                continue;
            }
        }

        parts_removed_params.push(part);
    }

    for part in parts_removed_params.into_iter().rev() {
        operation_id = format!("{}_{}", part, operation_id);
        ids.push(format!("{}_{}", verb, operation_id).to_lower_camel_case());
    }

    ids.push(format!("{}_{}_by_{}", verb, operation_id, params.join("_by_")).to_lower_camel_case());

    ids
}

#[cfg(test)]
mod tests {
    const HTML: &str = include_str!("../../keycloak/22.0.0.html");

    mod parameters {
        use super::super::paths;
        use super::HTML;
        use openapiv3::ReferenceOr;
        use scraper::Html;

        #[test]
        fn correctly_parses_when_there_are_no_parameters() {
            let paths = paths(&Html::parse_document(HTML)).0.paths;
            let path = if let Some(ReferenceOr::Item(path)) = paths.get("/") {
                path
            } else {
                panic!("Couldn't extract path")
            };
            assert_eq!(path.parameters, vec![]);
        }

        #[test]
        fn correctly_parses_when_there_are_three_parameters() {
            let paths = paths(&Html::parse_document(HTML)).0.paths;
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
        fn correctly_parse_when_there_are_repeating_ids_parameters() {
            let paths = paths(&Html::parse_document(HTML)).0.paths;
            let path = if let Some(ReferenceOr::Item(path)) =
                paths.get("/{realm}/clients/{id1}/protocol-mappers/models/{id2}")
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
            assert_eq!(names, vec!["realm", "id1", "id2"]);
        }

        #[test]
        fn adds_descriptions_when_not_always_present() {
            let paths = paths(&Html::parse_document(HTML)).0.paths;
            let path_item = if let Some(ReferenceOr::Item(path)) =
                paths.get("/{realm}/authentication/client-authenticator-providers")
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

    mod operations {
        use super::super::paths;
        use super::HTML;
        use openapiv3::ReferenceOr;
        use scraper::Html;

        fn get_path(path: &str) -> openapiv3::PathItem {
            let paths = paths(&Html::parse_document(HTML)).0.paths;
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
                path_item
                    .delete
                    .as_ref()
                    .and_then(|op| op.description.as_ref()),
                Some(&"Delete the realm".to_string())
            );
        }

        #[test]
        fn correctly_parses_the_get_case() {
            let path_item = get_path("/{realm}/groups/{id}/role-mappings");
            assert_eq!(
                path_item
                    .get
                    .as_ref()
                    .and_then(|op| op.description.as_ref()),
                Some(&"Get role mappings".to_string())
            );
        }

        // This path is problematic as it doesn't have it's parameter defined.
        // Additionally, it couldn't be defined as sub paths can't be substituted in
        #[test]
        fn does_not_parse_the_any_path() {
            let paths = paths(&Html::parse_document(HTML)).0.paths;
            assert!(!paths.contains_key("/{any}"));
        }

        #[test]
        fn correctly_parses_the_post_case() {
            let path_item = get_path("/{realm}/groups/{id}/children");
            assert_eq!(
                path_item
                    .post
                    .as_ref()
                    .and_then(|op| op.description.as_ref()),
                Some(&"Set or create child.".to_string())
            );
        }

        #[test]
        fn correctly_parses_the_put_case() {
            let path_item = get_path("/{realm}/users/{id}");
            assert_eq!(
                path_item
                    .put
                    .as_ref()
                    .and_then(|op| op.description.as_ref()),
                Some(&"Update the user".to_string())
            );
        }
    }
}
