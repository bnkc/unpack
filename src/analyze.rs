use std::collections::HashSet;

use anyhow::{Context, Result};
use serde::Serialize;

use crate::config::Config;
use crate::exit_codes::ExitCode;
use crate::output::Outcome;
use crate::project_assets::get_imports;
use crate::project_assets::get_packages;
use crate::project_assets::{get_dependencies, Dependency};
use crate::project_assets::{get_site_packages, Package, PackageState};

#[derive(Serialize, Debug, PartialEq, Eq, Clone)]
pub struct AnalysisElement<'a> {
    pub package: &'a Package,
    pub dependency: Option<&'a Dependency>,
}

struct ProjectAnalysis {
    packages: HashSet<Package>,
    dependencies: HashSet<Dependency>,
    imports: HashSet<String>,
}

impl ProjectAnalysis {
    fn new(
        packages: HashSet<Package>,
        dependencies: HashSet<Dependency>,
        imports: HashSet<String>,
    ) -> Self {
        Self {
            packages,
            dependencies,
            imports,
        }
    }

    fn get_used(&self) -> Vec<AnalysisElement<'_>> {
        self.dependencies
            .iter()
            .filter_map(|dep| {
                self.packages
                    .iter()
                    .find(|pkg| pkg.id() == dep.id() && !pkg.aliases().is_disjoint(&self.imports))
                    .map(|pkg| AnalysisElement {
                        package: pkg,
                        dependency: Some(dep),
                    })
            })
            .collect()
    }

    fn get_unused(&self) -> Vec<AnalysisElement<'_>> {
        let used_packages = self.get_used();

        let used_requirements: HashSet<String> = used_packages
            .iter()
            .flat_map(|e| e.package.requirements().iter().cloned())
            .collect();

        self.dependencies
            .iter()
            .filter_map(|dep| {
                self.packages
                    .iter()
                    .find(|pkg| pkg.id() == dep.id() && pkg.aliases().is_disjoint(&self.imports))
                    .filter(|pkg| !used_requirements.contains(pkg.id()))
                    .map(|pkg| AnalysisElement {
                        package: pkg,
                        dependency: Some(dep),
                    })
            })
            .collect()
    }

    fn get_untracked(&self) -> Vec<AnalysisElement<'_>> {
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
                        package: pkg,
                        dependency: None,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn scan(&self, config: &Config) -> Vec<AnalysisElement> {
        match config.package_state {
            PackageState::Unused => self.get_unused(),
            PackageState::Untracked => self.get_untracked(),
            PackageState::Used => self.get_used(),
        }
    }
}

pub fn scan(config: Config) -> Result<ExitCode> {
    let mut outcome = Outcome::default();
    let imports = get_imports(&config).context("Failed to get imports from the project.")?;

    let dependencies = get_dependencies(&config.dep_spec_file)
        .context("Failed to get dependencies from the dependency specification file.")?;

    let site_packages = get_site_packages().context("Failed to get site packages.")?;
    let packages = get_packages(site_packages).context("Failed to get packages.")?;

    let analysis = ProjectAnalysis::new(packages, dependencies, imports);
    let elements = analysis.scan(&config);

    outcome.elements = elements;
    outcome.success = outcome.elements.is_empty();

    outcome.print_report(&config, std::io::stdout())
}
#[cfg(test)]
mod tests {

    use super::*;
    use crate::project_assets::{DependencyBuilder, PackageBuilder};

    /// Helper function to create a Package instance.
    fn create_package(id: &str, aliases: &[&str], requirements: HashSet<String>) -> Package {
        let aliases = aliases.iter().map(|s| s.to_string()).collect();
        PackageBuilder::new(id.to_string(), aliases, 0, requirements).build()
    }

    // Helper function to create a Dependency instance.
    fn create_dependency(id: &str) -> Dependency {
        DependencyBuilder::new(id.to_string())
            .version("1.0.0".to_string())
            .build()
    }

    #[test]
    fn test_get_used() {
        // let pkg_requirements = HashSet::from(["requirement1".to_string()]);

        let pkg1 = create_package(
            "pkg1",
            &["alias1"],
            HashSet::from(["requirement1".to_string()]),
        );
        let dep1 = create_dependency("pkg1");
        let imports = HashSet::from(["alias1".to_string()]);

        let analysis = ProjectAnalysis::new(
            // config,
            HashSet::from([pkg1]),
            HashSet::from([dep1]),
            imports,
        );

        let used = analysis.get_used();
        assert_eq!(used.len(), 1);
        assert_eq!(used[0].package.id(), "pkg1");
        assert_eq!(used[0].dependency.map(|d| d.id()), Some("pkg1"));
    }

    #[test]
    fn test_get_used_no_dependencies() {
        let pkg1 = create_package(
            "pkg1",
            &["alias1"],
            HashSet::from(["requirement1".to_string()]),
        );
        let imports = HashSet::from(["alias1".to_string()]);

        let analysis = ProjectAnalysis::new(HashSet::from([pkg1]), HashSet::new(), imports);

        let used = analysis.get_used();
        assert!(
            used.is_empty(),
            "No packages should be considered used as there are no dependencies."
        );
    }

    #[test]
    fn test_get_unused_no_packages() {
        let dep1 = create_dependency("pkg1");
        let imports = HashSet::new();

        let analysis = ProjectAnalysis::new(HashSet::new(), HashSet::from([dep1]), imports);

        let unused = analysis.get_unused();
        assert!(
            unused.is_empty(),
            "No packages should be considered unused as there are no packages."
        );
    }

    #[test]
    fn test_multiple_dependencies_and_packages() {
        let pkg1 = create_package(
            "pkg1",
            &["alias1"],
            HashSet::from(["requirement1".to_string()]),
        );
        let pkg2 = create_package(
            "pkg2",
            &["alias2", "alias3"],
            HashSet::from(["requirement1".to_string()]),
        );
        let dep1 = create_dependency("pkg1");
        let dep2 = create_dependency("pkg2");
        let imports = HashSet::from(["alias1".to_string(), "alias3".to_string()]);

        let analysis = ProjectAnalysis::new(
            HashSet::from([pkg1, pkg2]),
            HashSet::from([dep1, dep2]),
            imports,
        );

        let used = analysis.get_used();
        assert_eq!(used.len(), 2, "Should identify both packages as used");
        let pkg_ids: Vec<&str> = used.iter().map(|el| el.package.id()).collect();
        assert!(pkg_ids.contains(&"pkg1"));
        assert!(pkg_ids.contains(&"pkg2"));
    }

    #[test]
    fn test_get_unused() {
        let pkg1 = create_package(
            "pkg1",
            &["alias1"],
            HashSet::from(["requirement1".to_string()]),
        );
        let dep1 = create_dependency("pkg1");
        let imports = HashSet::new(); // No imports, so pkg1 should be unused.

        let analysis = ProjectAnalysis::new(HashSet::from([pkg1]), HashSet::from([dep1]), imports);

        let unused = analysis.get_unused();
        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0].package.id(), "pkg1");
        assert_eq!(unused[0].dependency.map(|d| d.id()), Some("pkg1"));
    }

    #[test]
    fn test_get_unused_with_overlapping_package_dependencies() {
        // Here, pkg1 has a dependency on pkg2, but pkg2 is not imported.
        let pkg1 = create_package("pkg1", &["alias1"], HashSet::from(["pkg2".to_string()]));
        let pkg2 = create_package(
            "pkg2",
            &["alias2"],
            HashSet::from(["requirement2".to_string()]),
        );
        let dep1 = create_dependency("pkg1");
        let dep2 = create_dependency("pkg2");
        let imports = HashSet::from(["alias1".to_string()]);
        let analysis = ProjectAnalysis::new(
            HashSet::from([pkg1, pkg2.clone()]),
            HashSet::from([dep1.clone(), dep2.clone()]),
            imports.clone(),
        );

        let unused = analysis.get_unused();

        // There should be 0 unused packages as pkg2 is a dependency of pkg1. so even though it is not imported, it is still used indirectly.
        assert!(
            unused.is_empty(),
            "No packages should be considered unused as pkg2 is a dependency of pkg1."
        );

        // Now, if we remove the dependency on pkg2 from pkg1, pkg2 should be considered unused.
        let pkg1 = create_package("pkg1", &["alias1"], HashSet::new());
        let analysis = ProjectAnalysis::new(
            HashSet::from([pkg1, pkg2]),
            HashSet::from([dep1, dep2]),
            imports,
        );

        let unused = analysis.get_unused();
        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0].package.id(), "pkg2");
    }

    #[test]
    fn test_get_untracked() {
        let pkg1 = create_package(
            "pkg1",
            &["alias1"],
            HashSet::from(["requirement1".to_string()]),
        );
        let imports = HashSet::from(["alias1".to_string()]);

        let analysis = ProjectAnalysis::new(
            HashSet::from([pkg1]),
            HashSet::new(), // No dependencies, so pkg1 should be untracked.
            imports,
        );

        let untracked = analysis.get_untracked();
        assert_eq!(untracked.len(), 1);
        assert_eq!(untracked[0].package.id(), "pkg1");
        assert!(untracked[0].dependency.is_none());
    }

    #[test]
    fn test_get_untracked_no_aliases_imported() {
        let pkg1 = create_package(
            "pkg1",
            &["alias1"],
            HashSet::from(["requirement1".to_string()]),
        );
        let imports = HashSet::from(["unrelated_alias".to_string()]);

        let analysis = ProjectAnalysis::new(HashSet::from([pkg1]), HashSet::new(), imports);

        let untracked = analysis.get_untracked();
        assert!(
            untracked.is_empty(),
            "No packages should be untracked as their aliases are not imported."
        );
    }

    #[test]
    fn test_packages_with_no_corresponding_dependency() {
        let pkg1 = create_package(
            "pkg1",
            &["alias1"],
            HashSet::from(["requirement1".to_string()]),
        );
        let pkg2 = create_package(
            "pkg2",
            &["alias2"],
            HashSet::from(["requirement1".to_string()]),
        ); // This package does not have a corresponding dependency.
        let dep1 = create_dependency("pkg1");
        let imports = HashSet::from(["alias2".to_string()]);

        let analysis =
            ProjectAnalysis::new(HashSet::from([pkg1, pkg2]), HashSet::from([dep1]), imports);

        let untracked = analysis.get_untracked();
        assert_eq!(
            untracked.len(),
            1,
            "Only pkg2 should be identified as untracked"
        );
        assert_eq!(untracked[0].package.id(), "pkg2");
    }
    #[test]
    fn test_case_sensitivity() {
        let pkg1 = create_package(
            "PKG1",
            &["Alias1"],
            HashSet::from(["requirement1".to_string()]),
        );
        let dep1 = create_dependency("pkg1"); // Different case from the package ID.
        let imports = HashSet::from(["alias1".to_string()]); // Different case from the alias.

        let analysis = ProjectAnalysis::new(HashSet::from([pkg1]), HashSet::from([dep1]), imports);

        let used = analysis.get_used();
        assert!(used.is_empty(), "Case differences should prevent matching");
    }

    #[test]
    fn test_overlapping_dependencies_and_imports() {
        let pkg1 = create_package(
            "pkg1",
            &["alias1", "alias2"],
            HashSet::from(["requirement1".to_string()]),
        );
        let pkg2 = create_package(
            "pkg2",
            &["alias2"],
            HashSet::from(["requirement1".to_string()]),
        );
        let dep1 = create_dependency("pkg1");
        let dep2 = create_dependency("pkg2");
        let imports = HashSet::from(["alias2".to_string()]);

        let analysis = ProjectAnalysis::new(
            HashSet::from([pkg1, pkg2]),
            HashSet::from([dep1, dep2]),
            imports,
        );

        let used = analysis.get_used();
        assert_eq!(
            used.len(),
            2,
            "Both pkg1 and pkg2 should be considered used as alias2 is imported by both."
        );
    }
}
