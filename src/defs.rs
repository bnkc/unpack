#![allow(dead_code)]
use anyhow::Result;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
#[derive(Deserialize, Debug, PartialEq)]

pub struct Package {
    pub name: String,
    pub version: String,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct SitePackages {
    pub paths: Vec<String>,
    pub venv_name: Option<String>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct PackageInfo {
    pub name: String,
    pub import_names: HashSet<String>,
}

impl PackageInfo {
    pub fn from_info_dir(info_dir: PathBuf) -> Result<Self> {
        let pkg_name = info_dir
            .file_stem()
            .and_then(|stem| stem.to_str())
            .and_then(|s| s.split('-').next())
            .ok_or_else(|| anyhow::anyhow!("Invalid package name format"))
            .map(ToString::to_string)?;

        let top_level_path = info_dir.join("top_level.txt");
        let import_names = if top_level_path.exists() {
            fs::read_to_string(&top_level_path)?
                .lines()
                .map(str::trim)
                .map(ToString::to_string)
                .collect()
        } else {
            let mut set = HashSet::new();
            set.insert(pkg_name.clone());
            set
        };

        Ok(PackageInfo {
            name: pkg_name,
            import_names,
        })
    }
}
