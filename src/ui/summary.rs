use crate::report::ReportState;
use crate::ui::realtime::format_number;

pub fn render_summary(report: &ReportState) -> String {
    format!(
        "{} input \u{00B7} {} output \u{00B7} ${:.4} \u{00B7} {} messages",
        format_number(report.totals.input_tokens),
        format_number(report.totals.output_tokens),
        report.totals.total_cost_usd,
        report.totals.total_messages,
    )
}

pub fn render_empty_state(view: &str) -> String {
    format!("no claude usage data available for {view}")
}
