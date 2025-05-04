use std::str::FromStr;

pub struct UrlData {
    url: String,
    data: String,
}

impl FromStr for UrlData {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}
