use std::process::ExitCode;

use crate::compat::upstream::normalize_requested_view;
use crate::config::{OutputFormat, ResolvedConfig, View};
use crate::report::{build_daily_rows, build_monthly_rows};
use crate::runtime::orchestrator::load_report_state;
use crate::ui::{session, summary, table};

pub fn run_table_mode(resolved: &ResolvedConfig) -> anyhow::Result<ExitCode> {
    let report = load_report_state(resolved, None)?;
    let view = normalize_requested_view(resolved.cli.view);
    let theme = super::theme::resolve_theme(resolved.cli.theme);

    if report.blocks.is_empty() {
        println!("No Claude data directory found");
        return Ok(ExitCode::SUCCESS);
    }

    if resolved.cli.output == OutputFormat::Json {
        let json = serde_json::to_string_pretty(&report)?;
        println!("{}", json);
        return Ok(ExitCode::SUCCESS);
    }

    if view == View::Session {
        println!("{}", session::render_session_table(&report, &theme));
        return Ok(ExitCode::SUCCESS);
    }

    let title = match view {
        View::Monthly => format!("monthly usage ({})", resolved.cli.timezone),
        _ => format!("daily usage ({})", resolved.cli.timezone),
    };

    println!("{}", summary::render_summary(&report));
    match view {
        // Row builders receive the resolved timezone so labels and the table
        // title describe the same calendar boundary.
        View::Monthly => println!(
            "{}",
            table::render_table(
                &title,
                &build_monthly_rows(&report, &resolved.cli.timezone),
                &theme
            )
        ),
        _ => println!(
            "{}",
            table::render_table(
                &title,
                &build_daily_rows(&report, &resolved.cli.timezone),
                &theme
            )
        ),
    }
    Ok(ExitCode::SUCCESS)
}
