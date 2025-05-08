use serde::Deserialize;

#[derive(Deserialize)]
pub struct MetaData {
    name: String,
    url: String,
    ignore: Option<Vec<String>>,
    format: Option<bool>,
    cached: Option<bool>,
}
