use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]

pub struct Package {
    pub name: String,
    pub version: String,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct SitePackagesDir {
    pub path: String,
    pub is_venv: bool,
    pub venv_name: Option<String>,
}
