use serde::Deserialize;

#[derive(Deserialize, Debug, Hash, PartialEq, Eq, Clone)]

pub struct Package {
    pub name: String,
    pub version: String,
}
