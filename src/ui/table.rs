use crate::report::AggregateRow;
use crate::runtime::theme::ThemePalette;
use crate::ui::realtime::format_number;

pub fn render_table(title: &str, rows: &[AggregateRow], theme: &ThemePalette) -> String {
    let t = theme;
    let w = 57;
    let mut out = String::with_capacity(512);

    // Title + divider
    out.push_str(&format!(
        "\n {bold}{header}{title}{reset}\n",
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

    // Column headers
    out.push_str(&format!(
        "   {dim}{:<14} {:>9} {:>9} {:>10}    {:<}{reset}\n",
        "Date",
        "Input",
        "Output",
        "Cost",
        "Models",
        dim = t.dim,
        reset = t.reset,
    ));
    out.push_str(&format!(
        "   {accent}{:<14} {:>9} {:>9} {:>10}    {:<}{reset}\n",
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        accent = t.accent,
        reset = t.reset,
    ));

    for row in rows {
        let models_short: Vec<&str> = row.models.iter().map(|m| short_model_display(m)).collect();
        let input = format_number(row.per_model.iter().map(|m| m.input_tokens).sum::<u64>());
        let output = format_number(row.per_model.iter().map(|m| m.output_tokens).sum::<u64>());
        // Fallback: if per_model is empty, show total_tokens as input
        let (input, output) = if row.per_model.is_empty() {
            (format_number(row.total_tokens), "0".to_owned())
        } else {
            (input, output)
        };
        out.push_str(&format!(
            "   {value}{:<14}{reset} {value}{:>9}{reset} {value}{:>9}{reset} {value}{:>10}{reset}    {dim}{}{reset}\n",
            row.label,
            input,
            output,
            format!("${:.4}", row.total_cost_usd),
            models_short.join(", "),
            value = t.value,
            dim = t.dim,
            reset = t.reset,
        ));
    }
    out
}

fn short_model_display(model: &str) -> &str {
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
