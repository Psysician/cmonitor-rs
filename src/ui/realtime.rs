use time::OffsetDateTime;

use crate::report::ReportState;
use crate::runtime::theme::ThemePalette;

pub struct RealtimeContext {
    pub plan_name: String,
    pub token_limit: Option<u64>,
    pub message_limit: Option<u32>,
    pub timezone: String,
    pub theme: ThemePalette,
    pub now: OffsetDateTime,
}

pub fn render_realtime(report: &ReportState, ctx: &RealtimeContext) -> String {
    let mut out = String::with_capacity(1024);
    let t = &ctx.theme;
    let sep = "============================================================";

    // Header
    out.push_str(&format!(
        " {bold}{header}CLAUDE CODE USAGE MONITOR{reset}\n",
        bold = t.bold,
        header = t.header,
        reset = t.reset,
    ));
    out.push_str(&format!("{dim}{sep}{reset}\n", dim = t.dim, reset = t.reset));
    out.push_str(&format!(
        " {label}{plan}{reset} | {dim}{tz}{reset}\n",
        label = t.label,
        plan = ctx.plan_name,
        dim = t.dim,
        tz = ctx.timezone,
        reset = t.reset,
    ));

    let Some(active) = &report.active_session else {
        out.push('\n');
        out.push_str(&format!(
            " {label}No active session{reset}\n",
            label = t.label,
            reset = t.reset,
        ));
        out.push('\n');
        out.push_str(&format!(
            " {dim}Waiting for Claude Code activity...{reset}\n",
            dim = t.dim,
            reset = t.reset,
        ));
        out.push_str(&format!(
            " {dim}Ctrl+C to exit{reset}\n",
            dim = t.dim,
            reset = t.reset,
        ));
        return out;
    };

    out.push('\n');

    // Token usage bar
    if let Some(limit) = ctx.token_limit {
        let pct = usage_pct(active.totals.total_tokens, limit);
        let bar = render_progress_bar(pct, 30, t);
        out.push_str(&format!(
            " {label}Token Usage{reset}     {bar} {value}{pct:>5.1}%{reset}    {value}{used}{reset} / {dim}{limit_fmt}{reset}\n",
            label = t.label,
            value = t.value,
            dim = t.dim,
            reset = t.reset,
            pct = pct,
            used = format_number(active.totals.total_tokens),
            limit_fmt = format_number(limit),
        ));
    } else {
        out.push_str(&format!(
            " {label}Tokens{reset}          {value}{used}{reset}\n",
            label = t.label,
            value = t.value,
            reset = t.reset,
            used = format_number(active.totals.total_tokens),
        ));
    }

    // Cost bar
    let cost = active.totals.total_cost_usd;
    out.push_str(&format!(
        " {label}Cost{reset}            {value}${cost:.2}{reset}\n",
        label = t.label,
        value = t.value,
        reset = t.reset,
    ));

    // Messages bar
    if let Some(msg_limit) = ctx.message_limit {
        let pct = usage_pct(active.totals.total_messages as u64, msg_limit as u64);
        let bar = render_progress_bar(pct, 30, t);
        out.push_str(&format!(
            " {label}Messages{reset}        {bar} {value}{pct:>5.1}%{reset}    {value}{used}{reset} / {dim}{limit_fmt}{reset}\n",
            label = t.label,
            value = t.value,
            dim = t.dim,
            reset = t.reset,
            pct = pct,
            used = active.totals.total_messages,
            limit_fmt = msg_limit,
        ));
    } else {
        out.push_str(&format!(
            " {label}Messages{reset}        {value}{used}{reset}\n",
            label = t.label,
            value = t.value,
            reset = t.reset,
            used = active.totals.total_messages,
        ));
    }

    out.push_str(&format!("{dim}{sep}{reset}\n", dim = t.dim, reset = t.reset));

    // Time remaining
    let (hours, minutes) = time_remaining(active.ends_at, ctx.now);
    let total_secs = (active.ends_at - active.started_at).whole_seconds().max(1) as f64;
    let elapsed_secs = (ctx.now - active.started_at).whole_seconds().max(0) as f64;
    let time_pct = (elapsed_secs / total_secs * 100.0).min(100.0);
    let time_bar = render_progress_bar(time_pct, 30, t);
    out.push_str(&format!(
        " {label}Time Remaining{reset}  {time_bar}          {value}{h}h {m}m left{reset}\n",
        label = t.label,
        value = t.value,
        reset = t.reset,
        h = hours,
        m = minutes,
    ));

    // Model split
    if !active.per_model.is_empty() {
        let total = active.totals.total_tokens.max(1) as f64;
        let parts: Vec<String> = active
            .per_model
            .iter()
            .map(|m| {
                let pct = m.total_tokens as f64 / total * 100.0;
                format!("{} {:.1}%", short_model_name(&m.model), pct)
            })
            .collect();
        out.push_str(&format!(
            " {label}Model Split{reset}     {dim}{parts}{reset}\n",
            label = t.label,
            dim = t.dim,
            reset = t.reset,
            parts = parts.join(" | "),
        ));
    }

    out.push_str(&format!("{dim}{sep}{reset}\n", dim = t.dim, reset = t.reset));

    // Burn rate
    let burn = calculate_burn_rate(active.totals.total_tokens, active.started_at, ctx.now);
    out.push_str(&format!(
        " {label}Burn Rate{reset}       {value}{burn:.1} tok/min{reset}\n",
        label = t.label,
        value = t.value,
        reset = t.reset,
    ));

    // Cost rate
    let elapsed_min = (ctx.now - active.started_at).whole_seconds().max(1) as f64 / 60.0;
    let cost_rate = cost / elapsed_min;
    out.push_str(&format!(
        " {label}Cost Rate{reset}       {value}${cost_rate:.4}/min{reset}\n",
        label = t.label,
        value = t.value,
        reset = t.reset,
    ));

    // Reset time
    let reset_fmt = format_reset_time(active.ends_at, &ctx.timezone);
    out.push_str(&format!(
        " {label}Resets at{reset}       {value}{reset_fmt}{reset}\n",
        label = t.label,
        value = t.value,
        reset = t.reset,
    ));

    // Warnings
    if !active.warnings.is_empty() {
        out.push('\n');
        for w in &active.warnings {
            out.push_str(&format!(
                " {warn}! {msg}{reset}\n",
                warn = t.warning,
                msg = w.message,
                reset = t.reset,
            ));
        }
    }

    out.push('\n');
    out.push_str(&format!(
        " {dim}Active session | Ctrl+C to exit{reset}\n",
        dim = t.dim,
        reset = t.reset,
    ));

    out
}

fn render_progress_bar(pct: f64, width: usize, theme: &ThemePalette) -> String {
    let clamped = pct.clamp(0.0, 100.0);
    let filled = ((clamped / 100.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);

    let color = if clamped >= 90.0 {
        theme.bar_high
    } else if clamped >= 50.0 {
        theme.bar_mid
    } else {
        theme.bar_low
    };

    format!(
        "[{color}{filled}{dim}{empty}{reset}]",
        color = color,
        filled = "\u{2588}".repeat(filled),
        dim = theme.dim,
        empty = "\u{2591}".repeat(empty),
        reset = theme.reset,
    )
}

pub fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result.chars().rev().collect()
}

fn calculate_burn_rate(tokens: u64, started_at: OffsetDateTime, now: OffsetDateTime) -> f64 {
    let elapsed_minutes = (now - started_at).whole_seconds().max(1) as f64 / 60.0;
    tokens as f64 / elapsed_minutes
}

fn time_remaining(ends_at: OffsetDateTime, now: OffsetDateTime) -> (u64, u64) {
    let remaining_secs = (ends_at - now).whole_seconds().max(0) as u64;
    let hours = remaining_secs / 3600;
    let minutes = (remaining_secs % 3600) / 60;
    (hours, minutes)
}

fn usage_pct(used: u64, limit: u64) -> f64 {
    if limit == 0 {
        return 0.0;
    }
    (used as f64 / limit as f64) * 100.0
}

fn short_model_name(model: &str) -> &str {
    // models are already lowercased by normalize_model() at ingest time
    if model.contains("opus") {
        "opus"
    } else if model.contains("haiku") {
        "haiku"
    } else if model.contains("sonnet") {
        "sonnet"
    } else {
        model.split('-').next().unwrap_or(model)
    }
}

fn format_reset_time(ends_at: OffsetDateTime, timezone: &str) -> String {
    let (h, m, _) = ends_at.time().as_hms();
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02} {}",
        ends_at.year(),
        ends_at.month() as u8,
        ends_at.day(),
        h,
        m,
        timezone,
    )
}
