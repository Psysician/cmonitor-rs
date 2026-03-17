use std::ffi::OsString;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser, Clone)]
#[command(
    name = "cmonitor-rs",
    about = "Rust rewrite scaffold for Claude Code Usage Monitor",
    disable_help_subcommand = true,
    disable_version_flag = true
)]
pub struct CliArgs {
    #[arg(long, value_enum)]
    pub plan: Option<Plan>,
    #[arg(long)]
    pub custom_limit_tokens: Option<u32>,
    #[arg(long, value_enum)]
    pub view: Option<View>,
    #[arg(long)]
    pub timezone: Option<String>,
    #[arg(long, value_enum)]
    pub time_format: Option<TimeFormat>,
    #[arg(long, value_enum)]
    pub theme: Option<Theme>,
    #[arg(long, value_parser = clap::value_parser!(u64).range(1..=60))]
    pub refresh_rate: Option<u64>,
    #[arg(long, value_parser = parse_refresh_per_second)]
    pub refresh_per_second: Option<f64>,
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=23))]
    pub reset_hour: Option<u8>,
    #[arg(long)]
    pub log_level: Option<String>,
    #[arg(long)]
    pub log_file: Option<PathBuf>,
    #[arg(long)]
    pub debug: bool,
    #[arg(long)]
    pub clear: bool,
    #[arg(short = 'v', long)]
    pub version: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Plan {
    Pro,
    Max5,
    Max20,
    Custom,
}

impl Plan {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pro => "pro",
            Self::Max5 => "max5",
            Self::Max20 => "max20",
            Self::Custom => "custom",
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum View {
    Realtime,
    Daily,
    Monthly,
    Session,
}

impl View {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Realtime => "realtime",
            Self::Daily => "daily",
            Self::Monthly => "monthly",
            Self::Session => "session",
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
pub enum TimeFormat {
    #[value(name = "12h")]
    #[serde(rename = "12h")]
    TwelveHour,
    #[value(name = "24h")]
    #[serde(rename = "24h")]
    TwentyFourHour,
    #[value(name = "auto")]
    #[serde(rename = "auto")]
    Auto,
}

impl TimeFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TwelveHour => "12h",
            Self::TwentyFourHour => "24h",
            Self::Auto => "auto",
        }
    }

    fn from_label(label: &str) -> Option<Self> {
        match label {
            "12h" => Some(Self::TwelveHour),
            "24h" => Some(Self::TwentyFourHour),
            "auto" => Some(Self::Auto),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    Classic,
    Auto,
}

impl Theme {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::Classic => "classic",
            Self::Auto => "auto",
        }
    }

    fn from_label(label: &str) -> Option<Self> {
        match label {
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            "classic" => Some(Self::Classic),
            "auto" => Some(Self::Auto),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Cli {
    pub plan: Plan,
    pub custom_limit_tokens: Option<u32>,
    pub view: View,
    pub timezone: String,
    pub time_format: TimeFormat,
    pub theme: Theme,
    pub refresh_rate: u64,
    pub refresh_per_second: f64,
    pub reset_hour: Option<u8>,
    // Accepted for upstream CLI parity but not wired to a logging backend.
    // The upstream oracle uses these with setup_logging() but parity tests
    // do not assert logging output. Wire tracing-subscriber when debug
    // output becomes a parity requirement.
    pub log_level: String,
    pub log_file: Option<PathBuf>,
    pub debug: bool,
    pub clear: bool,
    pub version: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct LastUsedConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_limit_tokens: Option<u32>,
    pub view: Option<View>,
    pub timezone: Option<String>,
    pub time_format: Option<TimeFormat>,
    pub theme: Option<Theme>,
    pub refresh_rate: Option<u64>,
    pub reset_hour: Option<u8>,
}

#[derive(Clone, Debug)]
pub struct LastUsedStore {
    path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct ResolvedConfig {
    pub cli: Cli,
    pub store: LastUsedStore,
}

impl Cli {
    pub fn defaults() -> Self {
        Self {
            plan: Plan::Custom,
            custom_limit_tokens: None,
            view: View::Realtime,
            timezone: "auto".to_owned(),
            time_format: TimeFormat::Auto,
            theme: Theme::Auto,
            refresh_rate: 10,
            refresh_per_second: 0.75,
            reset_hour: None,
            log_level: "INFO".to_owned(),
            log_file: None,
            debug: false,
            clear: false,
            version: false,
        }
    }
}

impl CliArgs {
    pub fn parse_from_os<I, T>(args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        Self::parse_from(args)
    }

    pub fn merge_with_last_used(self, stored: Option<LastUsedConfig>) -> Cli {
        let cli_theme_provided = self.theme.is_some();
        let cli_timezone_provided = self.timezone.is_some();
        let cli_time_format_provided = self.time_format.is_some();
        let cli_view_provided = self.view.is_some();
        let cli_refresh_rate_provided = self.refresh_rate.is_some();
        let cli_reset_hour_provided = self.reset_hour.is_some();
        let cli_custom_limit_provided = self.custom_limit_tokens.is_some();
        let cli_plan_provided = self.plan.is_some();
        let stored = stored.unwrap_or_default();
        let mut cli = Cli::defaults();

        cli.plan = self.plan.unwrap_or(cli.plan);
        cli.custom_limit_tokens = self
            .custom_limit_tokens
            .or(stored.custom_limit_tokens)
            .filter(|_| matches!(cli.plan, Plan::Custom));
        if cli_plan_provided && matches!(cli.plan, Plan::Custom) && !cli_custom_limit_provided {
            cli.custom_limit_tokens = None;
        }

        cli.view = self.view.or(stored.view).unwrap_or(cli.view);
        cli.timezone = self.timezone.or(stored.timezone).unwrap_or(cli.timezone);
        cli.time_format = self
            .time_format
            .or(stored.time_format)
            .unwrap_or(cli.time_format);
        cli.theme = self.theme.or(stored.theme).unwrap_or(cli.theme);
        cli.refresh_rate = self
            .refresh_rate
            .or(stored.refresh_rate)
            .unwrap_or(cli.refresh_rate);
        cli.refresh_per_second = self.refresh_per_second.unwrap_or(cli.refresh_per_second);
        cli.reset_hour = self.reset_hour.or(stored.reset_hour);
        cli.log_level = self
            .log_level
            .map(|value| value.to_ascii_uppercase())
            .unwrap_or_else(|| cli.log_level.clone());
        cli.log_file = self.log_file;
        cli.debug = self.debug;
        cli.clear = self.clear;
        cli.version = self.version;

        if cli.timezone == "auto" {
            cli.timezone = detect_timezone();
        }
        if cli.time_format == TimeFormat::Auto {
            cli.time_format = detect_time_format();
        }
        if cli.theme == Theme::Auto || (!cli_theme_provided && stored.theme.is_none()) {
            cli.theme = detect_theme();
        }

        if !cli_timezone_provided && cli.timezone.is_empty() {
            cli.timezone = detect_timezone();
        }
        if !cli_time_format_provided && cli.time_format == TimeFormat::Auto {
            cli.time_format = detect_time_format();
        }
        if !cli_view_provided && cli.view == View::Realtime {
            cli.view = stored.view.unwrap_or(View::Realtime);
        }
        if !cli_refresh_rate_provided && cli.refresh_rate == 0 {
            cli.refresh_rate = 10;
        }
        if !cli_reset_hour_provided && cli.reset_hour.is_none() {
            cli.reset_hour = stored.reset_hour;
        }

        if cli.debug {
            cli.log_level = "DEBUG".to_owned();
        }

        cli
    }
}

impl LastUsedStore {
    pub fn new() -> anyhow::Result<Self> {
        let home = home_dir().ok_or_else(|| anyhow!("home directory unavailable"))?;
        Ok(Self {
            path: home.join(".claude-monitor/last_used.json"),
        })
    }

    pub fn load(&self) -> anyhow::Result<Option<LastUsedConfig>> {
        match fs::read_to_string(&self.path) {
            Ok(contents) => Ok(Some(
                serde_json::from_str(&contents)
                    .with_context(|| format!("parse {}", self.path.display()))?,
            )),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn save_last_used(&self, cli: &Cli) -> anyhow::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let payload = LastUsedConfig {
            custom_limit_tokens: cli
                .custom_limit_tokens
                .filter(|_| matches!(cli.plan, Plan::Custom)),
            view: Some(cli.view),
            timezone: Some(cli.timezone.clone()),
            time_format: Some(cli.time_format),
            theme: Some(cli.theme),
            refresh_rate: Some(cli.refresh_rate),
            reset_hour: cli.reset_hour,
        };

        fs::write(&self.path, serde_json::to_string_pretty(&payload)?)?;
        Ok(())
    }

    pub fn clear(&self) -> anyhow::Result<()> {
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl ResolvedConfig {
    pub fn load<I, T>(args: I) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let raw = CliArgs::parse_from_os(args);
        let store = LastUsedStore::new()?;
        let stored = if raw.clear { None } else { store.load()? };
        let cli = raw.merge_with_last_used(stored);
        Ok(Self { cli, store })
    }
}

pub fn version_banner() -> String {
    format!("claude-monitor {}", env!("CARGO_PKG_VERSION"))
}

/// Treats unwritable config homes as non-fatal for ordinary runs so reporting
/// still works in read-only environments. `--clear` remains strict because the
/// write/delete side effect is the point of that command path.
pub fn should_ignore_last_used_save_error(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<std::io::Error>()
            .is_some_and(|io_error| {
                matches!(io_error.kind(), ErrorKind::PermissionDenied)
                    || io_error.raw_os_error() == Some(30)
            })
    })
}

fn parse_refresh_per_second(value: &str) -> Result<f64, String> {
    let parsed = value
        .parse::<f64>()
        .map_err(|error| format!("invalid refresh rate '{value}': {error}"))?;

    if (0.1..=20.0).contains(&parsed) {
        Ok(parsed)
    } else {
        Err("refresh-per-second must be between 0.1 and 20.0".to_owned())
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
}

fn detect_timezone() -> String {
    std::env::var("CMONITOR_TEST_TIMEZONE").unwrap_or_else(|_| "UTC".to_owned())
}

fn detect_time_format() -> TimeFormat {
    std::env::var("CMONITOR_TEST_TIME_FORMAT")
        .ok()
        .and_then(|value| TimeFormat::from_label(value.as_str()))
        .unwrap_or(TimeFormat::TwentyFourHour)
}

fn detect_theme() -> Theme {
    std::env::var("CMONITOR_TEST_THEME")
        .ok()
        .and_then(|value| Theme::from_label(value.as_str()))
        .unwrap_or(Theme::Dark)
}
