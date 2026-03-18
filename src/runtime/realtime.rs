use std::collections::VecDeque;
use std::io::{self, Write};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use time::OffsetDateTime;

use crate::compat::terminal_policy::default_terminal_policy;
use crate::config::{Cli, Plan, ResolvedConfig};
use crate::domain::{PlanType, plan_definition};
use crate::report::ReportState;
use crate::runtime::orchestrator::{DeltaCache, load_report_state};
use crate::runtime::terminal::TerminalGuard;
use crate::runtime::theme::resolve_theme;
use crate::ui::realtime::{self, RealtimeContext};
use crate::ui::sparkline::render_sparkline;

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

fn build_realtime_context(cli: &Cli, report: &ReportState) -> RealtimeContext {
    let plan_type = match cli.plan {
        Plan::Pro => PlanType::Pro,
        Plan::Max5 => PlanType::Max5,
        Plan::Max20 => PlanType::Max20,
        Plan::Custom => PlanType::Custom,
    };

    let custom_limit = report.custom_limit.or(cli.custom_limit_tokens.map(|v| v as u64));
    let def = plan_definition(plan_type, custom_limit);
    let theme = resolve_theme(cli.theme);

    RealtimeContext {
        plan_name: def.name,
        token_limit: def.token_limit,
        message_limit: def.message_limit,
        timezone: cli.timezone.clone(),
        theme,
        now: OffsetDateTime::now_utc(),
    }
}

/// Keeps the terminal guard alive for the full refresh loop so alternate-screen
/// output persists until an explicit exit path fires.
pub fn run_realtime_mode(resolved: &ResolvedConfig) -> anyhow::Result<ExitCode> {
    install_interrupt_handler()?;

    if resolved.cli.output == crate::config::OutputFormat::Json {
        let report = load_report_state(resolved, None)?;
        let json = serde_json::to_string_pretty(&report)?;
        println!("{}", json);
        return Ok(ExitCode::SUCCESS);
    }

    let mut delta_cache = DeltaCache::new();
    run_realtime_loop(
        &resolved.cli,
        &mut io::stdout(),
        || load_report_state(resolved, Some(&mut delta_cache)),
        thread::sleep,
        LoopControl::from_env(),
    )
}

/// Owns repeated renders, reload cadence, and bounded test exits in one loop so
/// realtime behavior stays visible and measurable.
fn run_realtime_loop<Load, Sleep, W>(
    cli: &Cli,
    out: &mut W,
    mut load_report: Load,
    mut sleep: Sleep,
    mut control: LoopControl,
) -> anyhow::Result<ExitCode>
where
    Load: FnMut() -> anyhow::Result<ReportState>,
    Sleep: FnMut(Duration),
    W: Write,
{
    let mut report = load_report()?;
    if report.blocks.is_empty() {
        writeln!(out, "No Claude data directory found")?;
        return Ok(ExitCode::SUCCESS);
    }

    let policy = default_terminal_policy(cli);
    if policy.deferred_non_tty_gate {
        let ctx = build_realtime_context(cli, &report);
        writeln!(out, "{}", realtime::render_realtime(&report, &ctx))?;
        out.flush()?;
        return Ok(ExitCode::SUCCESS);
    }

    let _guard = TerminalGuard::enter(cli)?;
    let display_interval = Duration::from_secs_f64(1.0 / cli.refresh_per_second);
    let data_interval = Duration::from_secs(cli.refresh_rate);
    let mut next_reload_at = Instant::now() + data_interval;
    let mut token_history: VecDeque<u64> = VecDeque::with_capacity(60);

    loop {
        if let Some(active) = &report.active_session {
            if token_history.len() >= 60 {
                token_history.pop_front();
            }
            token_history.push_back(active.totals.total_tokens);
        }
        let spark = render_sparkline(&token_history.iter().copied().collect::<Vec<_>>(), 20);
        let ctx = build_realtime_context(cli, &report);
        render_frame(out, &report, &ctx, &spark)?;
        if control.should_exit_after_frame() || interrupted() {
            return Ok(ExitCode::SUCCESS);
        }
        sleep(display_interval);
        if interrupted() {
            return Ok(ExitCode::SUCCESS);
        }
        if Instant::now() >= next_reload_at {
            report = load_report()?;
            next_reload_at = Instant::now() + data_interval;
        }
    }
}

/// Flushes each frame immediately because alternate-screen teardown happens on
/// guard drop rather than at print boundaries.
fn render_frame<W>(out: &mut W, report: &ReportState, ctx: &RealtimeContext, spark: &str) -> anyhow::Result<()>
where
    W: Write,
{
    let rendered = realtime::render_realtime(report, ctx);
    if spark.is_empty() {
        writeln!(out, "\x1b[H\x1b[2J{rendered}")?;
    } else {
        writeln!(out, "\x1b[H\x1b[2J{rendered} {spark}")?;
    }
    out.flush()?;
    Ok(())
}

/// Encodes the explicit frame bound used by automated tests without changing
/// the interactive refresh contract.
struct LoopControl {
    max_frames: Option<usize>,
    rendered_frames: usize,
}

impl LoopControl {
    /// Reads the frame bound from environment so regression tests can stop the
    /// loop deterministically at a chosen render count.
    fn from_env() -> Self {
        Self {
            max_frames: std::env::var("CMONITOR_TEST_MAX_FRAMES")
                .ok()
                .and_then(|value| value.parse::<usize>().ok()),
            rendered_frames: 0,
        }
    }

    /// Counts visible frames so automated checks assert the rendered contract
    /// that users observe.
    fn should_exit_after_frame(&mut self) -> bool {
        self.rendered_frames += 1;
        self.max_frames
            .is_some_and(|max_frames| self.rendered_frames >= max_frames)
    }
}

/// Checks the shared interrupt flag so ctrl-c exits through the same terminal
/// cleanup path as bounded test runs.
fn interrupted() -> bool {
    INTERRUPTED.load(Ordering::Relaxed)
}

/// Uses ctrlc crate to install a safe cross-platform interrupt handler that
/// sets the shared flag so loop exit passes through terminal restoration.
fn install_interrupt_handler() -> anyhow::Result<()> {
    INTERRUPTED.store(false, Ordering::Relaxed);
    ctrlc::set_handler(move || {
        INTERRUPTED.store(true, Ordering::Relaxed);
    })?;
    Ok(())
}
