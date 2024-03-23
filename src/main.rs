use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::env;

use pip_udeps::{
    cli::{Config, Opts},
    exit_codes::ExitCode,
    get_unused_dependencies,
};

fn main() {
    let result = run();
    match result {
        Ok(exit_code) => {
            exit_code.exit();
        }
        Err(err) => {
            eprintln!("[pip-udeps error]: {:#}", err);
            ExitCode::GeneralError.exit();
        }
    }
}

fn run() -> Result<ExitCode> {
    let opts = Opts::parse();
    let config = Config::build(opts)?;
    set_working_dir(&config)?;
    let outcome = get_unused_dependencies(&config, std::io::stdout())?;
    outcome.print_human(std::io::stdout())
}

fn set_working_dir(config: &Config) -> Result<()> {
    if !config.base_directory.exists() {
        return Err(anyhow!("The provided path does not exist."));
    } else if !config.base_directory.is_dir() {
        return Err(anyhow!("The provided path is not a directory."));
    }
    env::set_current_dir(&config.base_directory).with_context(|| {
        format!(
            "Could not set '{}' as the current working directory. Please check the path provided.",
            env::current_dir().unwrap().to_string_lossy()
        )
    })?;

    Ok(())
}
