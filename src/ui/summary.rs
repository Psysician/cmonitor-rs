use crate::report::ReportState;

pub fn render_summary(report: &ReportState) -> String {
    format!(
        "total tokens: {}\ntotal cost: {:.4}\ntotal messages: {}",
        report.totals.total_tokens, report.totals.total_cost_usd, report.totals.total_messages
    )
}

pub fn render_empty_state(view: &str) -> String {
    format!("no claude usage data available for {view}")
}
