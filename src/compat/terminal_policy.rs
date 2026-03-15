use crate::config::Cli;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TerminalPolicy {
    pub force_alternate_screen: bool,
    pub deferred_non_tty_gate: bool,
}

pub fn default_terminal_policy(_cli: &Cli) -> TerminalPolicy {
    TerminalPolicy {
        force_alternate_screen: true,
        deferred_non_tty_gate: false,
    }
}
