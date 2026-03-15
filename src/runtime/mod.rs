use std::process::ExitCode;

use crate::config::ResolvedConfig;

pub mod orchestrator;
pub mod realtime;
pub mod table_mode;
pub mod terminal;
pub mod theme;

pub fn run_table_mode(resolved: &ResolvedConfig) -> anyhow::Result<ExitCode> {
    table_mode::run_table_mode(resolved)
}

pub fn run_realtime_mode(resolved: &ResolvedConfig) -> anyhow::Result<ExitCode> {
    realtime::run_realtime_mode(resolved)
}
