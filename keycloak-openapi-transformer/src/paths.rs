use scraper::Selector;

pub mod parameters;

pub fn paths(document: &scraper::html::Html) -> openapiv3::Paths {
    let path_section_selector =
        Selector::parse("#_paths + .sectionbody > .sect2 > .sect3").unwrap();
    let primary_path_selector = Selector::parse("pre").unwrap();
    let secondary_path_selector = Selector::parse("h4").unwrap();
    let params_table_selector = Selector::parse("h5[id^=_parameters] + table").unwrap();

    document
        .select(&path_section_selector)
        .map(|s| {
            let params_section = s.select(&params_table_selector).next();
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
                    parameters: if let Some(s) = params_section {
                        parameters::parse_path(&s)
                    } else {
                        Default::default()
                    },
                    ..Default::default()
                }),
            )
        })
        .collect()
}
