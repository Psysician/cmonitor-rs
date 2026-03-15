use std::ffi::OsString;
use std::process::ExitCode;

pub mod analysis;
pub mod compat;
pub mod config;
pub mod discovery;
pub mod domain;
pub mod parser;
pub mod report;
pub mod runtime;
pub mod ui;

pub fn run<I, T>(args: I) -> anyhow::Result<ExitCode>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let resolved = config::ResolvedConfig::load(args)?;

    if resolved.cli.version {
        println!("{}", config::version_banner());
        return Ok(ExitCode::SUCCESS);
    }

    if resolved.cli.clear {
        resolved.store.clear()?;
    } else {
        resolved.store.save_last_used(&resolved.cli)?;
    }

    match resolved.cli.view {
        config::View::Daily | config::View::Monthly => runtime::run_table_mode(&resolved),
        config::View::Realtime | config::View::Session => runtime::run_realtime_mode(&resolved),
    }
}
