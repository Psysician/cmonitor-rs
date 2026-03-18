use time::OffsetDateTime;

use crate::report::ReportState;
use crate::runtime::theme::ThemePalette;
use crate::ui::realtime::format_number;

pub fn render_session_table(report: &ReportState, theme: &ThemePalette) -> String {
    let t = theme;
    let w = 57;
    let mut out = String::new();

    // Title + divider
    out.push_str(&format!(
        "\n {bold}{header}Session View{reset}\n",
        bold = t.bold,
        header = t.header,
        reset = t.reset,
    ));
    out.push_str(&format!(
        " {accent}{line}{reset}\n",
        accent = t.accent,
        line = t.box_h.to_string().repeat(w),
        reset = t.reset,
    ));

    let sessions: Vec<_> = report.blocks.iter().filter(|b| !b.is_gap).collect();

    if sessions.is_empty() {
        out.push_str(&format!(
            "   {dim}No sessions found{reset}\n",
            dim = t.dim,
            reset = t.reset,
        ));
        return out;
    }

    // Column headers
    out.push_str(&format!(
        "   {dim}{:<4} {:<20} {:>10} {:>10} {:>10} {:>5}  {:<}{reset}\n",
        "#", "Started", "Duration", "Tokens", "Cost", "Msgs", "Models",
        dim = t.dim,
        reset = t.reset,
    ));
    out.push_str(&format!(
        "   {accent}{:<4} {:<20} {:>10} {:>10} {:>10} {:>5}  {:<}{reset}\n",
        "\u{2500}\u{2500}\u{2500}\u{2500}",
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        accent = t.accent,
        reset = t.reset,
    ));

    for (i, block) in sessions.iter().enumerate() {
        let idx = i + 1;
        let started = format_block_time(block.start_time);
        let duration = match block.actual_end_time {
            Some(end) => {
                let dur = end - block.start_time;
                let total_secs = dur.whole_seconds().max(0);
                let hours = total_secs / 3600;
                let minutes = (total_secs % 3600) / 60;
                let seconds = total_secs % 60;
                if hours > 0 {
                    format!("{}h {:02}m {:02}s", hours, minutes, seconds)
                } else {
                    format!("{:02}m {:02}s", minutes, seconds)
                }
            }
            None => "-".to_string(),
        };
        let total_tokens = format_number(block.tokens.total_tokens());
        let cost = format!("${:.4}", block.cost_usd);
        let models: Vec<&str> = block.models.iter().map(|m| short_model(m)).collect();
        let active_marker = if block.is_active {
            format!(" {bar_low}\u{25C9}{reset}", bar_low = t.bar_low, reset = t.reset)
        } else {
            String::new()
        };

        out.push_str(&format!(
            "   {dim}{:<4}{reset} {value}{:<20}{reset} {:>10} {:>10} {:>10} {:>5}  {dim}{}{reset}{active_marker}\n",
            idx,
            started,
            duration,
            total_tokens,
            cost,
            block.message_count,
            models.join(", "),
            dim = t.dim,
            value = t.value,
            reset = t.reset,
        ));
    }

    out
}

pub fn format_block_time(ts: OffsetDateTime) -> String {
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        ts.year(),
        ts.month() as u8,
        ts.day(),
        ts.hour(),
        ts.minute(),
        ts.second(),
    )
}

fn short_model(model: &str) -> &str {
    if model.contains("opus") {
        "opus"
    } else if model.contains("sonnet") {
        "sonnet"
    } else if model.contains("haiku") {
        "haiku"
    } else {
        model.split('-').next().unwrap_or(model)
    }
}
