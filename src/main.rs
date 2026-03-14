use clap::{Parser, ValueEnum};
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Parser, Clone)]
#[command(
    name = "cmonitor-rs",
    version,
    about = "Rust rewrite scaffold for Claude Code Usage Monitor",
    disable_help_subcommand = true
)]
struct Cli {
    #[arg(long, value_enum, default_value_t = Plan::Custom)]
    plan: Plan,
    #[arg(long)]
    custom_limit_tokens: Option<u32>,
    #[arg(long, value_enum, default_value_t = View::Realtime)]
    view: View,
    #[arg(long)]
    timezone: Option<String>,
    #[arg(long, value_enum, default_value_t = TimeFormat::Auto)]
    time_format: TimeFormat,
    #[arg(long, value_enum, default_value_t = Theme::Auto)]
    theme: Theme,
    #[arg(long, value_parser = clap::value_parser!(u64).range(1..=60), default_value_t = 10)]
    refresh_rate: u64,
    #[arg(long, value_parser = parse_refresh_per_second, default_value_t = 0.75)]
    refresh_per_second: f64,
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=23))]
    reset_hour: Option<u8>,
    #[arg(long, default_value = "INFO")]
    log_level: String,
    #[arg(long)]
    log_file: Option<PathBuf>,
    #[arg(long)]
    debug: bool,
    #[arg(long)]
    clear: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum Plan {
    Pro,
    Max5,
    Max20,
    Custom,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum View {
    Realtime,
    Daily,
    Monthly,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum TimeFormat {
    Auto,
    #[value(name = "12h")]
    TwelveHour,
    #[value(name = "24h")]
    TwentyFourHour,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum Theme {
    Auto,
    Light,
    Dark,
    Classic,
}

fn main() -> ExitCode {
    run(std::env::args_os())
}

fn run<I, T>(args: I) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = Cli::parse_from(args);
    println!("{}", render_placeholder(&cli));
    ExitCode::SUCCESS
}

fn render_placeholder(cli: &Cli) -> String {
    let mut lines = vec![
        "cmonitor-rs bootstrap: realtime terminal monitoring is not implemented yet.".to_owned(),
        "Parity target: upstream Maciek-roboblog/Claude-Code-Usage-Monitor behavior.".to_owned(),
        format!("Plan: {}", plan_label(cli.plan)),
        format!("View: {}", view_label(cli.view)),
        format!("Theme: {}", theme_label(cli.theme)),
    ];

    if let Some(limit) = cli.custom_limit_tokens {
        lines.push(format!("Custom token limit: {limit}"));
    }
    if let Some(timezone) = &cli.timezone {
        lines.push(format!("Timezone override: {timezone}"));
    }
    lines.push(format!(
        "Refresh: {}s data / {:.2}Hz display",
        cli.refresh_rate, cli.refresh_per_second
    ));
    if let Some(reset_hour) = cli.reset_hour {
        lines.push(format!("Reset hour: {reset_hour}"));
    }
    if let Some(path) = &cli.log_file {
        lines.push(format!("Log file: {}", path.display()));
    }
    if cli.debug {
        lines.push("Debug logging requested.".to_owned());
    }
    if cli.clear {
        lines.push("Saved configuration clear requested.".to_owned());
    }

    lines.join("\n")
}

fn parse_refresh_per_second(value: &str) -> Result<f64, String> {
    let parsed = value
        .parse::<f64>()
        .map_err(|err| format!("invalid refresh rate '{value}': {err}"))?;

    if (0.1..=20.0).contains(&parsed) {
        Ok(parsed)
    } else {
        Err("refresh-per-second must be between 0.1 and 20.0".to_owned())
    }
}

fn plan_label(plan: Plan) -> &'static str {
    match plan {
        Plan::Pro => "pro",
        Plan::Max5 => "max5",
        Plan::Max20 => "max20",
        Plan::Custom => "custom",
    }
}

fn view_label(view: View) -> &'static str {
    match view {
        View::Realtime => "realtime",
        View::Daily => "daily",
        View::Monthly => "monthly",
    }
}

fn theme_label(theme: Theme) -> &'static str {
    match theme {
        Theme::Auto => "auto",
        Theme::Light => "light",
        Theme::Dark => "dark",
        Theme::Classic => "classic",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_defaults_match_upstream_bootstrap_choices() {
        let cli = Cli::parse_from(["cmonitor-rs"]);

        assert_eq!(cli.plan, Plan::Custom);
        assert_eq!(cli.view, View::Realtime);
        assert_eq!(cli.theme, Theme::Auto);
        assert_eq!(cli.refresh_rate, 10);
    }

    #[test]
    fn placeholder_includes_explicit_overrides() {
        let cli = Cli::parse_from([
            "cmonitor-rs",
            "--plan",
            "max20",
            "--view",
            "daily",
            "--theme",
            "dark",
            "--timezone",
            "UTC",
            "--refresh-rate",
            "5",
            "--refresh-per-second",
            "1.5",
            "--debug",
        ]);

        let output = render_placeholder(&cli);
        assert!(output.contains("Plan: max20"));
        assert!(output.contains("View: daily"));
        assert!(output.contains("Timezone override: UTC"));
        assert!(output.contains("Debug logging requested."));
    }
}
