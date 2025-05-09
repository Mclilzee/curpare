use serde::Deserialize;

#[derive(Deserialize)]
pub struct MetaData {
    pub left: Data,
    pub right: Data,
}

#[derive(Deserialize)]
pub struct Data {
    pub name: String,
    pub url: String,
    pub ignore: Option<Vec<String>>,
    pub format: Option<bool>,
    pub cached: Option<bool>,
}
