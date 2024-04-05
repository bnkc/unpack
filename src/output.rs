use std::io::Write;

use anyhow::Result;
use serde::Serialize;

use crate::analyze::AnalysisElement;
use crate::cli::OutputKind;
use crate::config::Config;
use crate::exit_codes::ExitCode;

#[derive(Default, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct Outcome<'a> {
    pub success: bool,
    pub elements: Vec<AnalysisElement<'a>>,
    pub note: Option<String>,
}

impl<'a> Outcome<'a> {
    pub fn print_report(&self, config: &Config, mut stdout: impl Write) -> Result<ExitCode> {
        match config.output {
            OutputKind::Human => self.pretty_print(&mut stdout, config),
            OutputKind::Json => self.json_print(&mut stdout),
        }
    }

    fn json_print(&self, stdout: &mut impl Write) -> Result<ExitCode> {
        let json = serde_json::to_string(&self).expect("Failed to serialize to JSON.");
        writeln!(stdout, "{}", json)?;
        stdout.flush()?;
        Ok(ExitCode::Success)
    }

    fn pretty_print(&self, stdout: &mut impl Write, config: &Config) -> Result<ExitCode> {
        writeln!(stdout, "Analysis Outcome:\n")?;
        if self.elements.is_empty() {
            writeln!(
                stdout,
                "No relevant packages found for the selected criteria."
            )?;
        } else {
            writeln!(stdout, "{:?} Packages:", config.package_state)?;
            for element in &self.elements {
                let dep_note = if let Some(dep) = element.dependency {
                    format!("(dependency: {}, version: {:?})", dep.id(), dep.version())
                } else {
                    "Untracked".to_string()
                };

                writeln!(stdout, "- {} {}", element.package.id(), dep_note)?;
            }
        }

        if let Some(note) = &self.note {
            writeln!(stdout, "\nNote: {}", note)?;
        }

        stdout.flush()?;
        Ok(ExitCode::Success)
    }
}
