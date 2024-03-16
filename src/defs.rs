use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct SitePackages {
    pub paths: Vec<String>,
    pub venv_name: Option<String>,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct UnusedDepsOutcome {
    success: bool,
    note: Option<String>,
    unused_deps: Vec<Package>,
}
