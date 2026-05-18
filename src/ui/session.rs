use crate::report::ReportState;
use crate::runtime::theme::ThemePalette;
use crate::ui::realtime::format_number;
use crate::ui::timezone::format_datetime_seconds;

pub fn render_session_table(report: &ReportState, theme: &ThemePalette, timezone: &str) -> String {
    let t = theme;
    let w = 80;
    let mut out = String::new();

    // Title + divider
    out.push_str(&format!(
        "\n {bold}{header}Session View ({timezone}){reset}\n",
        bold = t.bold,
        header = t.header,
        timezone = timezone,
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
        "   {dim}{:<4} {:<19} {:<19} {:>8} {:>10} {:>9} {:>5}  {:<}{reset}\n",
        "#",
        "Started",
        "Ends",
        "Window",
        "Tokens",
        "Cost",
        "Msgs",
        "Models",
        dim = t.dim,
        reset = t.reset,
    ));
    out.push_str(&format!(
        "   {accent}{:<4} {:<19} {:<19} {:>8} {:>10} {:>9} {:>5}  {:<}{reset}\n",
        "\u{2500}\u{2500}\u{2500}\u{2500}",
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        accent = t.accent,
        reset = t.reset,
    ));

    for (i, block) in sessions.iter().enumerate() {
        let idx = i + 1;
        let started = format_block_time(block.start_time, timezone);
        let ends = format_block_time(block.end_time, timezone);
        let duration = format_duration(block.end_time - block.start_time);
        let total_tokens = format_number(block.tokens.total_tokens());
        let cost = format!("${:.4}", block.cost_usd);
        let models: Vec<&str> = block.models.iter().map(|m| short_model(m)).collect();
        let active_marker = if block.is_active {
            format!(
                " {bar_low}\u{25C9}{reset}",
                bar_low = t.bar_low,
                reset = t.reset
            )
        } else {
            String::new()
        };

        out.push_str(&format!(
            "   {dim}{:<4}{reset} {value}{:<19}{reset} {value}{:<19}{reset} {:>8} {:>10} {:>9} {:>5}  {dim}{}{reset}{active_marker}\n",
            idx,
            started,
            ends,
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

pub fn format_block_time(ts: time::OffsetDateTime, timezone: &str) -> String {
    format_datetime_seconds(ts, timezone)
}

fn format_duration(duration: time::Duration) -> String {
    let total_secs = duration.whole_seconds().max(0);
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    if hours > 0 {
        format!("{}h {:02}m", hours, minutes)
    } else if seconds > 0 {
        format!("{:02}m {:02}s", minutes, seconds)
    } else {
        format!("{:02}m", minutes)
    }
}

fn short_model(model: &str) -> &str {
    if model.contains("mythos") {
        "mythos"
    } else if model.contains("opus") {
        "opus"
    } else if model.contains("sonnet") {
        "sonnet"
    } else if model.contains("haiku") {
        "haiku"
    } else {
        model.split('-').next().unwrap_or(model)
    }
}
