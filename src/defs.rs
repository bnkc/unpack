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
