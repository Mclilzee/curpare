use serde::Deserialize;

#[derive(Deserialize)]
pub struct MetaData {
    pub name: String,
    pub url: String,
    pub ignore: Option<Vec<String>>,
    pub format: Option<bool>,
    pub cached: Option<bool>,
    pub headers: Option<Vec<(String, String)>>,
}
