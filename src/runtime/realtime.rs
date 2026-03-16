use std::io::{self, Write};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use crate::config::{Cli, ResolvedConfig};
use crate::report::ReportState;
use crate::runtime::orchestrator::load_report_state;
use crate::runtime::terminal::TerminalGuard;
use crate::ui::realtime;

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

/// Keeps the terminal guard alive for the full refresh loop so alternate-screen
/// output persists until an explicit exit path fires. (ref: DL-004)
pub fn run_realtime_mode(resolved: &ResolvedConfig) -> anyhow::Result<ExitCode> {
    install_interrupt_handler()?;
    run_realtime_loop(
        &resolved.cli,
        &mut io::stdout(),
        || load_report_state(resolved),
        thread::sleep,
        LoopControl::from_env(),
    )
}

/// Owns repeated renders, reload cadence, and bounded test exits in one loop so
/// realtime behavior stays visible and measurable. (ref: DL-004)
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

    let _guard = TerminalGuard::enter(cli)?;
    let display_interval = Duration::from_secs_f64(1.0 / cli.refresh_per_second);
    let data_interval = Duration::from_secs(cli.refresh_rate);
    let mut next_reload_at = Instant::now() + data_interval;

    loop {
        render_frame(out, &report)?;
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
/// guard drop rather than at print boundaries. (ref: DL-004)
fn render_frame<W>(out: &mut W, report: &ReportState) -> anyhow::Result<()>
where
    W: Write,
{
    writeln!(out, "\x1b[H\x1b[2J{}", realtime::render_realtime(report))?;
    out.flush()?;
    Ok(())
}

/// Encodes the explicit frame bound used by automated tests without changing
/// the interactive refresh contract. (ref: DL-004)
struct LoopControl {
    max_frames: Option<usize>,
    rendered_frames: usize,
}

impl LoopControl {
    /// Reads the frame bound from environment so regression tests can stop the
    /// loop deterministically at a chosen render count. (ref: DL-004)
    fn from_env() -> Self {
        Self {
            max_frames: std::env::var("CMONITOR_TEST_MAX_FRAMES")
                .ok()
                .and_then(|value| value.parse::<usize>().ok()),
            rendered_frames: 0,
        }
    }

    /// Counts visible frames so automated checks assert the rendered contract
    /// that users observe. (ref: DL-004)
    fn should_exit_after_frame(&mut self) -> bool {
        self.rendered_frames += 1;
        self.max_frames
            .is_some_and(|max_frames| self.rendered_frames >= max_frames)
    }
}

/// Checks the shared interrupt flag so ctrl-c exits through the same terminal
/// cleanup path as bounded test runs. (ref: DL-004)
fn interrupted() -> bool {
    INTERRUPTED.load(Ordering::Relaxed)
}

/// Uses ctrlc crate to install a safe cross-platform interrupt handler that
/// sets the shared flag so loop exit passes through terminal restoration.
/// (ref: DL-004, DL-007)
fn install_interrupt_handler() -> anyhow::Result<()> {
    INTERRUPTED.store(false, Ordering::Relaxed);
    ctrlc::set_handler(move || {
        INTERRUPTED.store(true, Ordering::Relaxed);
    })?;
    Ok(())
}
