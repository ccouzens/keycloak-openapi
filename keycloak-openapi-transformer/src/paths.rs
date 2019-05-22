use scraper::Selector;

pub mod parameters;

pub fn paths(document: &scraper::html::Html) -> openapiv3::Paths {
    let path_section_selector =
        Selector::parse("#_paths + .sectionbody > .sect2 > .sect3").unwrap();
    let primary_path_selector = Selector::parse("pre").unwrap();
    let secondary_path_selector = Selector::parse("h4").unwrap();

    document
        .select(&path_section_selector)
        .map(|s| {
            (
                s.select(&primary_path_selector)
                    .next()
                    .or_else(|| s.select(&secondary_path_selector).next())
                    .unwrap()
                    .text()
                    .collect::<String>()
                    .split_whitespace()
                    .nth(1)
                    .unwrap()
                    .to_string(),
                openapiv3::ReferenceOr::Item(openapiv3::PathItem {
                    parameters: parameters::parse_path(&s),
                    ..Default::default()
                }),
            )
        })
        .collect()
}
