// extern crate bytesize;

// use crate::builders;
// use crate::exit_codes::ExitCode;
// use crate::Config;

// use anyhow::Result;
// use builders::{Package, PackageState};
// use bytesize::ByteSize;

// use serde::{Deserialize, Serialize};
// use std::collections::HashMap;
// use std::io::Write;

// use std::vec;
// use tabled::settings::Panel;
// use tabled::{settings::Style, Table, Tabled};

// #[derive(clap::ValueEnum, Clone, Copy, Debug)]
// pub enum OutputKind {
//     /// Human-readable output format.
//     Human,
//     /// JSON output format.
//     Json,
// }

// #[derive(Tabled)]
// struct Record {
//     name: String,
//     version: String,
//     size: String,
// }

// #[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
// pub struct Analysis {
//     pub success: bool,
//     pub packages: Vec<Package>,
//     pub note: Option<String>,
// }

// impl Analysis {
//     pub fn print_report(&self, config: &Config, stdout: impl Write) -> Result<ExitCode> {
//         match config.output {
//             OutputKind::Human => self.pretty_print(stdout, config),
//             OutputKind::Json => self.json_print(stdout),
//         }
//     }

//     fn pretty_print(&self, mut stdout: impl Write, config: &Config) -> Result<ExitCode> {
//         if self.success {
//             writeln!(stdout, "All dependencies are correctly managed!")?;
//         } else {
//             writeln!(stdout, "\n{:?} Dependencies", config.package_state)?;

//             match config.package_state {
//                 PackageState::Untracked => self.print_untracked(&mut stdout)?,
//                 _ => self.print(&mut stdout)?,
//             }

//             if let Some(note) = &self.note {
//                 writeln!(stdout, "\nNote: {}", note)?;
//             }
//         }

//         stdout.flush()?;
//         Ok(ExitCode::Success)
//     }

//     fn print_untracked(&self, stdout: &mut impl Write) -> Result<()> {
//         let records: Vec<Record> = self
//             .packages
//             .iter()
//             .map(|package| Record {
//                 name: package.id.clone(),
//                 version: String::from("N/A"),
//                 size: ByteSize::b(package.size).to_string_as(true),
//             })
//             .collect();

//         let table = Table::new(records).to_string();
//         write!(stdout, "{}", table)?;
//         Ok(())
//     }

//     fn print(&self, stdout: &mut impl Write) -> Result<(), std::io::Error> {
//         let mut category_groups: HashMap<String, Vec<Record>> = HashMap::new();
//         for package in &self.packages {
//             if let Some(ref dep) = package.dependency {
//                 category_groups
//                     .entry(dep.category.clone().unwrap_or_else(|| "N/A".to_string()))
//                     .or_default()
//                     .push(Record {
//                         name: dep.id.clone(),
//                         version: dep.version.clone().unwrap_or_else(|| "N/A".to_string()),
//                         size: ByteSize::b(package.size).to_string_as(true),
//                     });
//             }
//         }

//         for (category, records) in category_groups {
//             let mut table = Table::new(&records);

//             table
//                 .with(Panel::header(category))
//                 .with(Style::ascii())
//                 .with(Panel::footer("End of table"));

//             writeln!(stdout, "\n{}", table)?;
//         }

//         Ok(())
//     }

//     // JSON printing remains unchanged
//     fn json_print(&self, mut stdout: impl Write) -> Result<ExitCode> {
//         let json = serde_json::to_string(self).expect("Failed to serialize to JSON.");
//         writeln!(stdout, "{}", json)?;
//         stdout.flush()?;
//         Ok(ExitCode::Success)
//     }
// }
