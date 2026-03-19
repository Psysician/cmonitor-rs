use crate::domain::{
    COMMON_COST_LIMITS, COMMON_TOKEN_LIMITS, DEFAULT_CUSTOM_COST_MINIMUM, DEFAULT_CUSTOM_MINIMUM,
    LIMIT_THRESHOLD, SessionBlock,
};

pub fn calculate_custom_limit(blocks: &[SessionBlock]) -> Option<u64> {
    if blocks.is_empty() {
        return None;
    }

    let completed = completed_totals(blocks);
    if completed.is_empty() {
        return Some(DEFAULT_CUSTOM_MINIMUM);
    }

    let mut hit_limits = completed
        .iter()
        .copied()
        .filter(|total| {
            COMMON_TOKEN_LIMITS
                .iter()
                .any(|limit| (*total as f64) >= (*limit as f64 * LIMIT_THRESHOLD))
        })
        .collect::<Vec<_>>();

    if hit_limits.is_empty() {
        hit_limits = completed;
    }

    hit_limits.sort_unstable();
    let index = ((hit_limits.len() - 1) as f64 * 0.9).round() as usize;
    Some(hit_limits[index].max(DEFAULT_CUSTOM_MINIMUM))
}

pub fn calculate_custom_cost_limit(blocks: &[SessionBlock]) -> Option<f64> {
    if blocks.is_empty() {
        return None;
    }

    let completed = completed_costs(blocks);
    if completed.is_empty() {
        return Some(DEFAULT_CUSTOM_COST_MINIMUM);
    }

    let mut hit_limits: Vec<f64> = completed
        .iter()
        .copied()
        .filter(|cost| {
            COMMON_COST_LIMITS
                .iter()
                .any(|limit| *cost >= *limit * LIMIT_THRESHOLD)
        })
        .collect();

    if hit_limits.is_empty() {
        hit_limits = completed;
    }

    hit_limits.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let index = ((hit_limits.len() - 1) as f64 * 0.9).round() as usize;
    Some(hit_limits[index].max(DEFAULT_CUSTOM_COST_MINIMUM))
}

fn completed_totals(blocks: &[SessionBlock]) -> Vec<u64> {
    blocks
        .iter()
        .filter(|block| !block.is_gap && !block.is_active)
        .map(|block| block.tokens.input_tokens + block.tokens.output_tokens)
        .filter(|total| *total > 0)
        .collect()
}

fn completed_costs(blocks: &[SessionBlock]) -> Vec<f64> {
    blocks
        .iter()
        .filter(|block| !block.is_gap && !block.is_active)
        .map(|block| block.cost_usd)
        .filter(|cost| *cost > 0.0)
        .collect()
}
