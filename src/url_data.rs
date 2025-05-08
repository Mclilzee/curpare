use std::str::FromStr;

pub struct UrlData {
    name: String,
    status_code: u32,
    body_content: String,
}

impl FromStr for UrlData {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}
