use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Package {
    pub name: String,
    pub version: String,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct SitePackages {
    pub paths: Vec<String>,
    pub venv_name: Option<String>,
}
