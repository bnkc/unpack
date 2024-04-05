use std::io::Write;

use anyhow::Result;
use bytesize::ByteSize;
use serde::Serialize;

use tabled::{settings::Style, Table, Tabled};

use crate::analyze::AnalysisElement;
use crate::cli::OutputKind;
use crate::config::Config;
use crate::exit_codes::ExitCode;

#[derive(Default, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct Outcome<'a> {
    pub success: bool,
    pub elements: Vec<AnalysisElement<'a>>,
}

#[derive(Tabled)]
struct Record<'r> {
    package: &'r str,
    version: &'r str,
    size: String,
}

impl<'a> Outcome<'a> {
    pub fn print_report(&self, config: &Config, mut stdout: impl Write) -> Result<ExitCode> {
        match config.output {
            OutputKind::Human => self.pretty_print(&mut stdout, &config),
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
        if self.success {
            writeln!(
                stdout,
                "\n 📭 No {:?} packages found.",
                config.package_state
            )?;
            stdout.flush()?;
            return Ok(ExitCode::Success);
        }

        writeln!(stdout, "\n 📦 {:?} Packages", config.package_state)?;

        // sort elements by size
        let mut elements = self.elements.clone();
        elements.sort_by_key(|el| el.package.size());

        let records: Vec<Record> = self
            .elements
            .iter()
            .map(|e| Record {
                package: e.package.id(),
                version: e.dependency.as_ref().map_or("N/A", |dep| dep.version()),
                size: ByteSize::b(e.package.size()).to_string_as(true),
            })
            .collect();

        let mut table = Table::new(&records);
        table.with(Style::psql());

        writeln!(stdout, "\n{}", table)?;

        let total_size: u64 = self.elements.iter().map(|el| el.package.size()).sum();
        let total_size = ByteSize::b(total_size).to_string_as(true);

        let mut note = "".to_owned();
        note += " 💽 Total disk space: ";
        note += &total_size;
        note += "\n\n Note: There might be false-positives.\n";
        note += "      For example, `pip-udeps` cannot detect usage of packages that are not imported under `[tool.poetry.*]`.\n";

        writeln!(stdout, "\n{}", note)?;

        stdout.flush()?;
        Ok(ExitCode::Success)
    }
}
