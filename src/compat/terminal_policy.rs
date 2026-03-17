use std::io::{IsTerminal, stdin, stdout};

use crate::config::Cli;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TerminalPolicy {
    pub force_alternate_screen: bool,
    pub deferred_non_tty_gate: bool,
}

pub fn default_terminal_policy(_cli: &Cli) -> TerminalPolicy {
    let force_tty = std::env::var("CMONITOR_TEST_FORCE_TTY")
        .ok()
        .is_some_and(|value| value != "0");
    let interactive_terminal = force_tty || (stdin().is_terminal() && stdout().is_terminal());
    let term_is_dumb = std::env::var("TERM")
        .ok()
        .is_some_and(|value| value.eq_ignore_ascii_case("dumb"));
    let force_alternate_screen = interactive_terminal && !term_is_dumb;

    TerminalPolicy {
        force_alternate_screen,
        deferred_non_tty_gate: !force_alternate_screen,
    }
}
