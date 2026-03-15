use crate::config::View;
use crate::discovery::RootDiscovery;
use crate::domain::SessionBlock;

pub fn normalize_requested_view(view: View) -> View {
    match view {
        View::Session => View::Realtime,
        other => other,
    }
}

pub fn session_view_name() -> &'static str {
    "session"
}

pub fn primary_root_index(discovery: &RootDiscovery) -> Option<usize> {
    discovery.selected.as_ref().and_then(|selected| {
        discovery
            .roots
            .iter()
            .position(|root| &root.path == selected)
    })
}

pub fn display_total_tokens(block: &SessionBlock) -> u64 {
    block.tokens.input_tokens + block.tokens.output_tokens + block.tokens.cache_read_tokens
}
