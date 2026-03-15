use crate::report::AggregateRow;

pub fn render_table(title: &str, rows: &[AggregateRow]) -> String {
    let mut lines = vec![
        title.to_owned(),
        "label | tokens | cost | models".to_owned(),
    ];
    for row in rows {
        lines.push(format!(
            "{} | {} | {:.4} | {}",
            row.label,
            row.total_tokens,
            row.total_cost_usd,
            row.models.join(", ")
        ));
    }
    lines.join("\n")
}
