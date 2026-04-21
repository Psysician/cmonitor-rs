//! Unicode block-character sparkline renderer. Converts a slice of `u64`
//! token counts into a fixed-width string of Unicode block chars (`▁`-`█`).
//! (ref: DL-009)

static BLOCKS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// Converts `values` into a sparkline string of exactly `width` characters.
///
/// When `values` is longer than `width`, adjacent samples are bucketed by
/// taking the max of each bucket so short-duration spikes remain visible.
/// Scales each bucket to the eight Unicode block characters proportionally
/// to the global maximum value. Returns an empty string when `values` is
/// empty or `width` is zero. (ref: DL-009)
pub fn render_sparkline(values: &[u64], width: usize) -> String {
    if values.is_empty() || width == 0 {
        return String::new();
    }

    let max = *values.iter().max().unwrap_or(&1);
    let max = max.max(1);

    let display: Vec<u64> = if values.len() > width {
        let step = values.len() as f64 / width as f64;
        (0..width)
            .map(|i| {
                let start = (i as f64 * step) as usize;
                let end = ((i + 1) as f64 * step) as usize;
                let end = end.min(values.len());
                let slice = &values[start..end];
                if slice.is_empty() {
                    0
                } else {
                    *slice.iter().max().unwrap_or(&0)
                }
            })
            .collect()
    } else {
        values.to_vec()
    };

    let pad_count = width.saturating_sub(display.len());
    let sparkline: String = display
        .iter()
        .map(|&v| {
            let idx = ((v as f64 / max as f64) * (BLOCKS.len() - 1) as f64).round() as usize;
            BLOCKS[idx.min(BLOCKS.len() - 1)]
        })
        .collect();
    format!("{}{}", " ".repeat(pad_count), sparkline)
}
