use std::collections::HashSet;
use std::str;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::dependencies::{get_dependencies, Dependency};
use crate::exit_codes::*;
use crate::imports::get_imports;
use crate::packages::get_packages;
use crate::packages::{get_site_packages, Package, PackageState};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct AnalysisElement {
    package: Package,
    dependency: Option<Dependency>,
}
struct ProjectAnalysis {
    config: Config,
    packages: HashSet<Package>,
    dependencies: HashSet<Dependency>,
    imports: HashSet<String>,
}

impl ProjectAnalysis {
    pub fn new(
        config: Config,
        packages: HashSet<Package>,
        dependencies: HashSet<Dependency>,
        imports: HashSet<String>,
    ) -> Self {
        Self {
            config,
            packages,
            dependencies,
            imports,
        }
    }

    fn get_used(&self) -> Vec<AnalysisElement> {
        self.dependencies
            .iter()
            .filter_map(|dep| {
                self.packages
                    .iter()
                    .find(|pkg| pkg.id() == dep.id() && !pkg.aliases().is_disjoint(&self.imports))
                    .map(|pkg| AnalysisElement {
                        package: pkg.clone(),
                        dependency: Some(dep.clone()),
                    })
            })
            .collect()
    }

    fn get_unused(&self) -> Vec<AnalysisElement> {
        self.dependencies
            .iter()
            .filter_map(|dep| {
                self.packages
                    .iter()
                    .find(|pkg| pkg.id() == dep.id() && pkg.aliases().is_disjoint(&self.imports))
                    .map(|pkg| AnalysisElement {
                        package: pkg.clone(),
                        dependency: Some(dep.clone()),
                    })
            })
            .collect()
    }

    fn get_untracked(&self) -> Vec<AnalysisElement> {
        let dep_ids: HashSet<String> = self
            .dependencies
            .iter()
            .map(|dep| dep.id().to_string())
            .collect();

        self.packages
            .iter()
            .filter_map(|pkg| {
                if !pkg.aliases().is_disjoint(&self.imports) && !dep_ids.contains(pkg.id()) {
                    Some(AnalysisElement {
                        package: pkg.clone(),
                        dependency: None,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn scan(&self) -> Vec<AnalysisElement> {
        match self.config.package_state {
            PackageState::Unused => self.get_unused(),
            PackageState::Untracked => self.get_untracked(),
            PackageState::Used => self.get_used(),
        }
    }
}

pub fn scan(config: Config) -> Result<ExitCode> {
    let dependencies = get_dependencies(&config.dep_spec_file)
        .context("Failed to get dependencies from the dependency specification file.")?;

    let imports = get_imports(&config).context("Failed to get imports from the project.")?;
    let site_packages = get_site_packages().context("Failed to get site packages.")?;
    let packages = get_packages(site_packages).context("Failed to get packages.")?;

    let analysis = ProjectAnalysis::new(config, packages, dependencies, imports);
    let scanned_packages = analysis.scan();
    println!("{:#?}", scanned_packages);

    Ok(ExitCode::Success)
}

// pub fn analyze(config: Config) -> Result<ExitCode> {
//     let dependencies = get_dependencies(&config.dep_spec_file)?;
//     let imports = get_imports(&config)?;
//     let site_packages = get_site_packages()?;

//     let packages = get_packages(site_packages)?;
//     let analysis = ProjectAnalysis::new(config, packages, dependencies, imports);
//     let scanned_packages = analysis.scan();
//     println!("{:#?}", scanned_packages);

//     Ok(ExitCode::Success)
// }

// #[derive(Default)]
// pub struct Packages {
//     manifest: Vec<Package>,

// impl Packages {
//     pub fn new() -> Self {
//         Self {
//             manifest: Vec::new(),
//         }
//     }

//     pub fn add_package(&mut self, package: Package) {
//         self.manifest.push(package);
//     }

//     fn get_used(&self, deps: &HashSet<Dependency>, imports: &HashSet<String>) -> Vec<Package> {
//         deps.iter()
//             .filter_map(|dep| {
//                 self.manifest
//                     .iter()
//                     .find(|pkg| pkg.id == dep.id && !pkg.aliases.is_disjoint(imports))
//                     .map(|pkg| {
//                         PackageBuilder::new(pkg.id.clone(), pkg.aliases.clone(), pkg.size)
//                             .dependency(dep.clone())
//                             .build()
//                     })
//             })
//             .collect()
//     }

//     fn get_unused(&self, deps: &HashSet<Dependency>, imports: &HashSet<String>) -> Vec<Package> {
//         deps.iter()
//             .filter_map(|dep| {
//                 self.manifest
//                     .iter()
//                     .find(|pkg| pkg.id == dep.id && pkg.aliases.is_disjoint(imports))
//                     .map(|pkg| {
//                         PackageBuilder::new(pkg.id.clone(), pkg.aliases.clone(), pkg.size)
//                             .dependency(dep.clone())
//                             .build()
//                     })
//             })
//             .collect()
//     }

//     fn get_untracked(&self, deps: &HashSet<Dependency>, imports: &HashSet<String>) -> Vec<Package> {
//         let dep_ids: HashSet<String> = deps.iter().map(|dep| dep.id.clone()).collect();

//         self.manifest
//             .iter()
//             .filter(|pkg| !pkg.aliases.is_disjoint(imports) && !dep_ids.contains(&pkg.id))
//             .cloned()
//             .collect()
//     }

//     pub fn scan(
//         &self,
//         config: &Config,
//         deps: &HashSet<Dependency>,
//         imports: &HashSet<String>,
//     ) -> Vec<Package> {
//         match config.package_state {
//             PackageState::Untracked => self.get_untracked(deps, imports),
//             PackageState::Used => self.get_used(deps, imports),
//             PackageState::Unused => self.get_unused(deps, imports),
//         }
//     }

//     // For `testing` purposes ONLY. Not intended to be public facing API.
//     #[cfg(test)]
//     pub fn _mapping(&self) -> &Vec<Package> {
//         &self.manifest
//     }
// }

// pub fn analyze(config: Config) -> Result<ExitCode> {
//     // let mut outcome = Outcome::default();

//     let dependencies = get_dependencies(&config.dep_spec_file)?;
//     let imports = get_imports(&config)?;
//     let site_packages = get_site_packages()?;

//     let packages = get_packages(site_packages)?;
//     let scanned_packages = packages.scan(&config, &dependencies, &imports);
//     println!("{:?}", scanned_packages);

//     // outcome.packages = scanned_packages;
//     // outcome.success = outcome.packages.is_empty();

//     // if !outcome.success {
//     //     let mut note = "".to_owned();
//     //     note += "Note: There might be false-positives.\n";
//     //     note += "      For example, `pip-udeps` cannot detect usage of packages that are not imported under `[tool.poetry.*]`.\n";
//     //     outcome.note = Some(note);
//     // }

//     // outcome.print_report(&config, std::io::stdout())
//     Ok(ExitCode::Success)
// }

// #[cfg(test)]
// mod tests {

//     use super::*;

//     use std::fs::File;
//     use std::io::Write;
//     use std::io::{self};
//     use tempfile::TempDir;
//     use test::Bencher;

//     // Used to create a temporary directory with the given directories and files
//     fn create_working_directory(
//         dirs: &[&'static str],
//         files: Option<&[&'static str]>,
//     ) -> Result<TempDir, io::Error> {
//         let temp_dir = TempDir::new()?;

//         dirs.iter().for_each(|directory| {
//             let dir_path = temp_dir.path().join(directory);
//             fs::create_dir_all(dir_path).unwrap();
//         });

//         if let Some(files) = files {
//             files.iter().for_each(|file| {
//                 let file_path = temp_dir.path().join(file);
//                 File::create(file_path).unwrap();
//             });
//         }

//         Ok(temp_dir)
//     }

//     struct TestEnv {
//         /// Temporary project directory
//         _temp_dir: TempDir,

//         /// Test Configuration struct
//         config: Config,
//     }

//     impl TestEnv {
//         fn new(dirs: &[&'static str], files: Option<&[&'static str]>) -> Self {
//             let temp_dir = create_working_directory(dirs, files).unwrap();
//             let base_directory = temp_dir.path().join(dirs[0]);
//             let pyproject_path: PathBuf = base_directory.join("pyproject.toml");
//             let mut file = File::create(&pyproject_path).unwrap();

//             file.write_all(
//                 r#"
//                             [tool.poetry.dependencies]
//                             requests = "2.25.1"
//                             python = "^3.8"
//                             pandas = "^1.2.0"
//                             "#
//                 .as_bytes(),
//             )
//             .unwrap();

//             let config = Config {
//                 base_directory,
//                 dep_spec_file: pyproject_path,
//                 ignore_hidden: false,
//                 env: Env::Test,
//                 output: OutputKind::Human,
//                 package_state: PackageState::Unused,
//             };

//             Self {
//                 _temp_dir: temp_dir,
//                 config,
//             }
//         }
//     }

//     #[bench]
//     fn bench_get_used_imports(b: &mut Bencher) {
//         let te = TestEnv::new(&["dir1", "dir2"], Some(&["file1.py"]));
//         b.iter(|| get_imports(&te.config));
//     }

//     #[bench]
//     fn bench_get_dependencies(b: &mut Bencher) {
//         let te = TestEnv::new(&["dir1", "dir2"], Some(&["pyproject.toml"]));
//         b.iter(|| get_dependencies(&te.config.dep_spec_file));
//     }

//     #[test]
//     fn stem_import_correctly_stems() {
//         let first_part = stem_import("os.path");
//         assert_eq!(first_part.as_str(), "os");

//         let first_part = stem_import("os");
//         assert_eq!(first_part.as_str(), "os");

//         let first_part = stem_import("");
//         assert_eq!(first_part.as_str(), "");
//     }

//     #[test]
//     fn get_imports_correctly_collects() {
//         let te = TestEnv::new(
//             &["dir1", "dir2"],
//             Some(&["requirements.txt", "pyproject.toml", "file1.py"]),
//         );

//         let used_imports = get_imports(&te.config);
//         assert!(used_imports.is_ok());

//         let used_imports = used_imports.unwrap();
//         assert_eq!(used_imports.len(), 0);

//         let file_path = te.config.base_directory.join("file1.py");
//         let mut file = File::create(file_path).unwrap();
//         file.write_all(r#"import pandas as pd"#.as_bytes()).unwrap();

//         let used_imports = get_imports(&te.config);
//         assert!(used_imports.is_ok());

//         let used_imports = used_imports.unwrap();
//         assert_eq!(used_imports.len(), 1);
//         assert!(used_imports.contains("pandas"));
//         assert!(!used_imports.contains("sklearn"));
//     }

//     #[test]
//     fn get_site_package_dir_success() {
//         let site_pkgs = get_site_packages().unwrap();

//         assert_eq!(site_pkgs.paths.len(), 1);
//     }

//     #[test]
//     fn get_installed_packages() {
//         // THIS IS BECAUSE YOU ARE A FUCKING IDIOT THAT USES THE METADATA AND THE RECORD
//         // NOW

//         // Create a temporary environment resembling site-packages
//         let temp_dir = tempfile::TempDir::new().unwrap();
//         let site_packages_dir = temp_dir.path().join("site-packages");
//         fs::create_dir(&site_packages_dir).unwrap();

//         // Simulate a couple of installed packages with top_level.txt files
//         let pkg1_dir = site_packages_dir.join("example_pkg1-0.1.0-info");
//         fs::create_dir_all(&pkg1_dir).unwrap();
//         fs::write(pkg1_dir.join("top_level.txt"), "example_pkg1\n").unwrap();

//         let pkg2_dir = site_packages_dir.join("example_pkg2-0.2.0-info");
//         fs::create_dir_all(&pkg2_dir).unwrap();
//         fs::write(pkg2_dir.join("top_level.txt"), "example_pkg2\n").unwrap();

//         // lets do another package like scikit_learn where we know the name will get remapped to sklearn
//         let pkg3_dir = site_packages_dir.join("scikit_learn-0.24.1-info");
//         fs::create_dir_all(&pkg3_dir).unwrap();
//         fs::write(pkg3_dir.join("top_level.txt"), "sklearn\n").unwrap();

//         // let te = TestEnv::new(
//         //     &["dir1", "dir2"],
//         //     Some(&["requirements.txt", "pyproject.toml", "file1.py"]),
//         // );

//         let dirs: HashSet<PathBuf> = vec![site_packages_dir].into_iter().collect();

//         let site_pkgs = SitePackages { paths: dirs };

//         // let installed_pkgs = get_installed_packages(site_pkgs).unwrap();
//         let mut packages = Packages::default();
//         packages.load(site_pkgs).unwrap();

//         assert_eq!(packages.manifest.len(), 3);

//         // let imports = get_impos

//         // let dependencies = get_dependencies(&te.config.dep_spec_file).unwrap();

//         // let installed_packages = packages.scan(&te.config, &dependencies, &imports);

//         // assert_eq!(installed_packages.len(), 3);
//         // assert!(installed_packages.contains(
//         //     &PackageBuilder::new(
//         //         "example_pkg1".to_string(),
//         //         HashSet::from_iter(vec!["example_pkg1".to_string()]),
//         //         0
//         //     )
//         //     .build()
//         // ));

//         // // Assert that the correct number of packages were found

//         // assert_eq!(
//         //     installed_pkgs._mapping().len(),
//         //     3,
//         //     "Should have found two installed packages"
//         // );

//         // // Assert that the package names and import names are correct
//         // assert!(
//         //     installed_pkgs._mapping().get("example-pkg1").is_some(),
//         //     "Should contain example_pkg1"
//         // );

//         // assert!(
//         //     installed_pkgs
//         //         ._mapping()
//         //         .get("example-pkg1")
//         //         .unwrap()
//         //         .contains("example_pkg1"),
//         //     "example-pkg1 should contain example_pkg1"
//         // );
//         // assert!(
//         //     installed_pkgs._mapping().get("example-pkg2").is_some(),
//         //     "Should contain example_pkg2"
//         // );

//         // assert!(
//         //     installed_pkgs
//         //         ._mapping()
//         //         .get("example-pkg2")
//         //         .unwrap()
//         //         .contains("example_pkg2"),
//         //     "example-pkg2 should contain example_pkg2"
//         // );

//         // assert!(
//         //     installed_pkgs._mapping().get("scikit-learn").is_some(),
//         //     "Should contain scikit_learn"
//         // );

//         // assert!(
//         //     installed_pkgs
//         //         ._mapping()
//         //         .get("scikit-learn")
//         //         .unwrap()
//         //         .contains("sklearn"),
//         //     "scikit_learn should contain sklearn"
//         // );
//         // // non-existent package
//         // assert!(
//         //     !installed_pkgs._mapping().get("non-existent").is_some(),
//         //     "Should not contain non-existent"
//         // );
//     }

//     // #[test]
//     // fn get_deps_from_pyproject_toml_success() {
//     //     let temp_dir =
//     //         create_working_directory(&["dir1", "dir2"], Some(&["pyproject.toml"])).unwrap();
//     //     let base_directory = temp_dir.path().join("dir1");
//     //     let file_path = base_directory.join("pyproject.toml");
//     //     let mut file = File::create(&file_path).unwrap();
//     //     file.write_all(
//     //         r#"
//     //         [tool.poetry.dependencies]
//     //         requests = "2.25.1"
//     //         python = "^3.8"
//     //         "#
//     //         .as_bytes(),
//     //     )
//     //     .unwrap();

//     //     let packages = get_dependencies_from_pyproject_toml(&file_path).unwrap();
//     //     assert_eq!(packages.len(), 2);
//     //     assert!(packages.contains(&PyProjectDeps {
//     //         name: "requests".to_string()
//     //     }));
//     //     assert!(packages.contains(&PyProjectDeps {
//     //         name: "python".to_string()
//     //     }));
//     // }
// }
