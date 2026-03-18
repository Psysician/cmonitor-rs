use std::io::{Write, stdout};

use crate::compat::terminal_policy::{TerminalPolicy, default_terminal_policy};
use crate::config::Cli;

pub struct TerminalGuard {
    policy: TerminalPolicy,
}

impl TerminalGuard {
    pub fn enter(cli: &Cli) -> anyhow::Result<Self> {
        let policy = default_terminal_policy(cli);
        if policy.force_alternate_screen {
            print!("\x1b[?1049h");
            // Immediate flush makes alternate-screen entry observable to the
            // realtime loop and test harness.
            stdout().flush()?;
        }
        Ok(Self { policy })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if self.policy.force_alternate_screen {
            print!("\x1b[?1049l");
            // Drop flushes the return escape so bounded exits and SIGINT both
            // restore the main screen predictably.
            let _ = stdout().flush();
        }
    }
}
