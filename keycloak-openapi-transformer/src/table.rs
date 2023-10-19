use std::collections::HashMap;

use scraper::Selector;

lazy_static! {
    static ref HEADINGS_SELECTOR: Selector = Selector::parse("thead > tr > th").unwrap();
    static ref ROWS_SELECTOR: Selector = Selector::parse("tbody > tr").unwrap();
    static ref CELL_SELECTOR: Selector = Selector::parse("td").unwrap();
}

pub fn parse_table_rows<'a>(
    container: &'a scraper::element_ref::ElementRef<'a>,
    selector: &'a Selector,
) -> Vec<HashMap<String, String>> {
    let table = container.select(selector).next();
    if let Some(table) = table {
        let headings = table
            .select(&HEADINGS_SELECTOR)
            .map(|th| th.text().collect::<String>())
            .collect::<Vec<String>>();
        table
            .select(&ROWS_SELECTOR)
            .map(move |row| {
                row.select(&CELL_SELECTOR)
                    .map(|td| td.text().collect::<String>())
                    .zip(headings.iter())
                    .map(|(value, heading)| (heading.clone(), value))
                    .collect::<HashMap<String, String>>()
            })
            .collect()
    } else {
        Vec::new()
    }
}
