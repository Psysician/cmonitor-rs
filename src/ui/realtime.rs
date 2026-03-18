use time::OffsetDateTime;

use crate::report::model::ModelStats;
use crate::report::ReportState;
use crate::runtime::theme::ThemePalette;

const W: usize = 57;

pub struct RealtimeContext {
    pub plan_name: String,
    pub token_limit: Option<u64>,
    pub message_limit: Option<u32>,
    pub timezone: String,
    pub theme: ThemePalette,
    pub now: OffsetDateTime,
}

pub fn render_realtime(report: &ReportState, ctx: &RealtimeContext) -> String {
    let mut out = String::with_capacity(2048);
    let t = &ctx.theme;

    // ╭─── CLAUDE CODE USAGE MONITOR ───────────────────────╮
    out.push_str(&render_box_header("CLAUDE CODE USAGE MONITOR", W, t));
    out.push('\n');

    let Some(active) = &report.active_session else {
        // ╰─────────────────────────────────────────────────────╯
        let subtitle = format!(
            "{label}{plan}{reset} {accent}{dot}{reset} {dim}{tz}{reset}",
            label = t.label, plan = ctx.plan_name,
            accent = t.accent, dot = t.dot,
            dim = t.dim, tz = ctx.timezone,
            reset = t.reset,
        );
        out.push_str(&render_box_line(&subtitle, W, t));
        out.push('\n');
        out.push_str(&render_box_footer(W, t));
        out.push('\n');
        out.push('\n');
        out.push_str(&format!(
            "   {label}No active session{reset}\n",
            label = t.label, reset = t.reset,
        ));
        out.push_str(&format!(
            "   {dim}Waiting for Claude Code activity...{reset}\n",
            dim = t.dim, reset = t.reset,
        ));
        out.push_str(&format!(
            "   {dim}Ctrl+C to exit{reset}\n",
            dim = t.dim, reset = t.reset,
        ));
        return out;
    };

    // │  pro plan · UTC · ◉ active                          │
    let subtitle = format!(
        "{label}{plan}{reset} {accent}{dot}{reset} {dim}{tz}{reset} {accent}{dot}{reset} {bar_low}◉ active{reset}",
        label = t.label, plan = ctx.plan_name,
        accent = t.accent, dot = t.dot,
        dim = t.dim, tz = ctx.timezone,
        bar_low = t.bar_low,
        reset = t.reset,
    );
    out.push_str(&render_box_line(&subtitle, W, t));
    out.push('\n');
    // ╰─────────────────────────────────────────────────────╯
    out.push_str(&render_box_footer(W, t));
    out.push('\n');
    out.push('\n');

    // Tokens bar — uses input+output only (excludes cache) for meaningful %
    let billable_tokens = active.totals.input_tokens + active.totals.output_tokens;
    if let Some(limit) = ctx.token_limit {
        let pct = usage_pct(billable_tokens, limit);
        out.push_str(&format!(
            "   {label}Tokens{reset}      {bar} {value}{pct:>5.1}%{reset}   {value}{used}{reset} / {dim}{lim}{reset}\n",
            label = t.label, value = t.value, dim = t.dim, reset = t.reset,
            bar = render_progress_bar(pct, 26, t),
            pct = pct,
            used = format_number(billable_tokens),
            lim = format_number(limit),
        ));
    } else {
        out.push_str(&format!(
            "   {label}Tokens{reset}      {value}{used}{reset}\n",
            label = t.label, value = t.value, reset = t.reset,
            used = format_number(billable_tokens),
        ));
    }
    out.push_str(&format!(
        "   {dim}  in {input}  out {output}{reset}\n",
        dim = t.dim, reset = t.reset,
        input = format_number(active.totals.input_tokens),
        output = format_number(active.totals.output_tokens),
    ));

    // Cost bar
    let cost = active.totals.total_cost_usd;
    if let Some(limit) = ctx.token_limit {
        let avg_cost_per_token = if billable_tokens > 0 {
            cost / billable_tokens as f64
        } else {
            0.0
        };
        let estimated_max_cost = limit as f64 * avg_cost_per_token;
        let cost_pct = if estimated_max_cost > 0.0 {
            (cost / estimated_max_cost * 100.0).min(100.0)
        } else {
            0.0
        };
        out.push_str(&format!(
            "   {label}Cost{reset}        {bar} {value}{pct:>5.1}%{reset}   {value}${cost:.2}{reset}\n",
            label = t.label, value = t.value, reset = t.reset,
            bar = render_progress_bar(cost_pct, 26, t),
            pct = cost_pct,
        ));
    } else {
        out.push_str(&format!(
            "   {label}Cost{reset}        {value}${cost:.2}{reset}\n",
            label = t.label, value = t.value, reset = t.reset,
        ));
    }

    // Messages bar
    if let Some(msg_limit) = ctx.message_limit {
        let pct = usage_pct(active.totals.total_messages as u64, msg_limit as u64);
        out.push_str(&format!(
            "   {label}Messages{reset}    {bar} {value}{pct:>5.1}%{reset}   {value}{used}{reset} / {dim}{lim}{reset}\n",
            label = t.label, value = t.value, dim = t.dim, reset = t.reset,
            bar = render_progress_bar(pct, 26, t),
            used = active.totals.total_messages,
            lim = msg_limit,
        ));
    } else {
        out.push_str(&format!(
            "   {label}Messages{reset}    {value}{used}{reset}\n",
            label = t.label, value = t.value, reset = t.reset,
            used = active.totals.total_messages,
        ));
    }

    // Cache indicator — hit = read, miss = creation (write)
    let cache_total = active.totals.cache_read_tokens + active.totals.cache_creation_tokens;
    if cache_total > 0 {
        let hit_rate = active.totals.cache_read_tokens as f64 / cache_total as f64 * 100.0;
        out.push_str(&format!(
            "   {label}Cache{reset}       {bar_low}◆{reset} {value}{hit:.0}% hit{reset}  {dim}({read} read / {write} write){reset}\n",
            label = t.label, bar_low = t.bar_low, value = t.value, dim = t.dim, reset = t.reset,
            hit = hit_rate,
            read = format_number(active.totals.cache_read_tokens),
            write = format_number(active.totals.cache_creation_tokens),
        ));
    }

    out.push('\n');
    out.push_str(&render_divider(W, t));
    out.push('\n');
    out.push('\n');

    // Time remaining
    let (hours, minutes) = time_remaining(active.ends_at, ctx.now);
    let total_secs = (active.ends_at - active.started_at).whole_seconds().max(1) as f64;
    let elapsed_secs = (ctx.now - active.started_at).whole_seconds().max(0) as f64;
    let time_pct = (elapsed_secs / total_secs * 100.0).min(100.0);
    out.push_str(&format!(
        "   {label}Time Left{reset}   {bar}           {value}{h}h {m}m{reset}\n",
        label = t.label, value = t.value, reset = t.reset,
        bar = render_progress_bar(time_pct, 26, t),
        h = hours, m = minutes,
    ));

    // Model split
    if !active.per_model.is_empty() {
        let total = active.totals.total_tokens.max(1);
        let bar = render_model_split_bar(&active.per_model, total, 26, t);
        let legend: Vec<String> = active
            .per_model
            .iter()
            .map(|m| {
                let pct = m.total_tokens as f64 / total as f64 * 100.0;
                format!("{} {:.0}%", short_model_name(&m.model), pct)
            })
            .collect();
        out.push_str(&format!(
            "   {label}Models{reset}      {bar}  {dim}{legend}{reset}\n",
            label = t.label, dim = t.dim, reset = t.reset,
            legend = legend.join(" {dot} "),
        ));
    }

    out.push('\n');
    out.push_str(&render_divider(W, t));
    out.push('\n');
    out.push('\n');

    // Burn rate + Cost rate on compact lines
    let burn = calculate_burn_rate(active.totals.total_tokens, active.started_at, ctx.now);
    let elapsed_min = (ctx.now - active.started_at).whole_seconds().max(1) as f64 / 60.0;
    let cost_rate = cost / elapsed_min;
    out.push_str(&format!(
        "   {dim}⚡{reset} {label}Burn{reset}        {value}{burn:.1} tok/min{reset}        {dim}${reset} {label}Rate{reset}    {value}${rate:.4}/min{reset}\n",
        label = t.label, value = t.value, dim = t.dim, reset = t.reset,
        rate = cost_rate,
    ));

    // Reset time
    let resets_at = format_reset_time(active.ends_at, &ctx.timezone);
    out.push_str(&format!(
        "   {dim}⏱{reset} {label}Resets{reset}      {value}{resets_at}{reset}\n",
        label = t.label, value = t.value, dim = t.dim, reset = t.reset,
    ));

    // Cost by Model
    if !active.per_model.is_empty() {
        out.push('\n');
        out.push_str(&format!(
            "   {label}Cost by Model{reset}\n",
            label = t.label, reset = t.reset,
        ));
        for m in &active.per_model {
            out.push_str(&format!(
                "     {dim}{model:<18}{reset} {value}${cost:.4}{reset}\n",
                model = short_model_name(&m.model),
                cost = m.cost_usd,
                dim = t.dim, value = t.value, reset = t.reset,
            ));
        }
    }

    // Warnings
    if !active.warnings.is_empty() {
        out.push('\n');
        for w in &active.warnings {
            out.push_str(&format!(
                "   {warn}⚠ {msg}{reset}\n",
                warn = t.warning, msg = w.message, reset = t.reset,
            ));
        }
    }

    out.push('\n');
    out.push_str(&render_divider(W, t));
    out.push('\n');
    out.push_str(&format!(
        "   {bar_low}◉{reset} {dim}Active session {dot} Ctrl+C to exit{reset}\n",
        bar_low = t.bar_low, dim = t.dim, dot = t.dot, reset = t.reset,
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
        "{color}{filled}{track}{empty}{reset}",
        color = color,
        filled = theme.bar_filled.to_string().repeat(filled),
        track = theme.bar_track,
        empty = theme.bar_empty.to_string().repeat(empty),
        reset = theme.reset,
    )
}

fn render_model_split_bar(
    per_model: &[ModelStats],
    total_tokens: u64,
    width: usize,
    theme: &ThemePalette,
) -> String {
    let total = total_tokens.max(1) as f64;
    let mut segments: Vec<(usize, &str)> = Vec::new();
    let mut chars_assigned = 0usize;

    for (i, m) in per_model.iter().enumerate() {
        let color = match short_model_name(&m.model) {
            "opus" => theme.bar_high,
            "sonnet" => theme.bar_mid,
            "haiku" => theme.bar_low,
            _ => theme.dim,
        };
        let chars = if i == per_model.len() - 1 {
            width.saturating_sub(chars_assigned)
        } else {
            ((m.total_tokens as f64 / total) * width as f64).round() as usize
        };
        let chars = chars.min(width.saturating_sub(chars_assigned));
        chars_assigned += chars;
        segments.push((chars, color));
    }

    let empty = width.saturating_sub(chars_assigned);

    let mut bar = String::with_capacity(width * 8 + 16);
    for (chars, color) in &segments {
        if *chars > 0 {
            bar.push_str(color);
            bar.push_str(&theme.bar_filled.to_string().repeat(*chars));
        }
    }
    if empty > 0 {
        bar.push_str(theme.bar_track);
        bar.push_str(&theme.bar_empty.to_string().repeat(empty));
    }
    bar.push_str(theme.reset);
    bar
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

fn render_box_header(title: &str, width: usize, theme: &ThemePalette) -> String {
    let inner = format!(" {} ", title);
    let inner_vis_len = inner.len();
    let dashes_total = width.saturating_sub(inner_vis_len + 2);
    let left_dashes = 3;
    let right_dashes = dashes_total.saturating_sub(left_dashes);
    format!(
        " {accent}{tl}{left}{reset}{bold}{header}{inner}{reset}{accent}{right}{tr}{reset}",
        accent = theme.accent,
        tl = theme.box_tl,
        left = theme.box_h.to_string().repeat(left_dashes),
        bold = theme.bold,
        header = theme.header,
        inner = inner,
        right = theme.box_h.to_string().repeat(right_dashes),
        tr = theme.box_tr,
        reset = theme.reset,
    )
}

fn render_box_line(content: &str, width: usize, theme: &ThemePalette) -> String {
    let visible_len = strip_ansi_len(content);
    let padding = width.saturating_sub(visible_len + 4);
    format!(
        " {accent}{v}{reset} {content}{pad} {accent}{v}{reset}",
        accent = theme.accent,
        v = theme.box_v,
        content = content,
        pad = " ".repeat(padding),
        reset = theme.reset,
    )
}

fn render_box_footer(width: usize, theme: &ThemePalette) -> String {
    let inner_width = width.saturating_sub(2);
    format!(
        " {accent}{bl}{line}{br}{reset}",
        accent = theme.accent,
        bl = theme.box_bl,
        line = theme.box_h.to_string().repeat(inner_width),
        br = theme.box_br,
        reset = theme.reset,
    )
}

fn render_divider(width: usize, theme: &ThemePalette) -> String {
    format!(
        " {accent}{line}{reset}",
        accent = theme.accent,
        line = theme.box_h.to_string().repeat(width),
        reset = theme.reset,
    )
}

fn strip_ansi_len(s: &str) -> usize {
    let mut len = 0usize;
    let mut in_escape = false;
    for ch in s.chars() {
        if ch == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if ch == 'm' {
                in_escape = false;
            }
        } else {
            len += 1;
        }
    }
    len
}
