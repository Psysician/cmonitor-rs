use crate::report::ReportState;

pub fn render_realtime(report: &ReportState) -> String {
    if let Some(active) = &report.active_session {
        return format!(
            "active block: {}\nends at: {}\ntokens: {}\ncost: {:.4}\nwarnings: {}",
            active.block_id,
            active.ends_at,
            active.totals.total_tokens,
            active.totals.total_cost_usd,
            active.warnings.len()
        );
    }

    "no active claude session".to_owned()
}
