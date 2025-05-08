use serde::Deserialize;

#[derive(Deserialize)]
pub struct MetaData {
    left: Data,
    right: Data,
}

#[derive(Deserialize)]
pub struct Data {
    name: String,
    url: String,
    ignore: Option<Vec<String>>,
    format: Option<bool>,
    cached: Option<bool>,
}
