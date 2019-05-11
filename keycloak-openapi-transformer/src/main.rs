use scraper::Html;
use serde_json::to_string_pretty;

const HTML: &str = include_str!("../../keycloak/6.0.html");

mod info;

fn main() -> Result<(), Box<std::error::Error>> {
    let document = Html::parse_document(HTML);

    println!("{}", to_string_pretty(&info::parse(document)?)?);
    Ok(())
}
