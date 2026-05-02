use crate::domain::TokenUsage;

struct ModelRates {
    input: f64,
    output: f64,
    cache_create: f64,
    cache_read: f64,
}

const OPUS_RATES: ModelRates = ModelRates {
    input: 15.0,
    output: 75.0,
    cache_create: 18.75,
    cache_read: 1.50,
};

const SONNET_RATES: ModelRates = ModelRates {
    input: 3.0,
    output: 15.0,
    cache_create: 3.75,
    cache_read: 0.30,
};

const HAIKU_RATES: ModelRates = ModelRates {
    input: 0.25,
    output: 1.25,
    cache_create: 0.3125,
    cache_read: 0.025,
};

const MYTHOS_PREVIEW_RATES: ModelRates = ModelRates {
    input: 25.0,
    output: 125.0,
    cache_create: 31.25,
    cache_read: 2.50,
};

const MYTHOS_RATES: ModelRates = ModelRates {
    input: 25.0,
    output: 125.0,
    cache_create: 31.25,
    cache_read: 2.50,
};

fn rates_for_model(model: &str) -> &'static ModelRates {
    let lower = model.to_lowercase();
    if lower.contains("mythos") {
        if lower.contains("preview") {
            &MYTHOS_PREVIEW_RATES
        } else {
            &MYTHOS_RATES
        }
    } else if lower.contains("opus") {
        &OPUS_RATES
    } else if lower.contains("haiku") {
        &HAIKU_RATES
    } else {
        &SONNET_RATES
    }
}

pub fn calculate_entry_cost(model: &str, tokens: &TokenUsage) -> f64 {
    let rates = rates_for_model(model);
    let per_million = 1_000_000.0;

    (tokens.input_tokens as f64 * rates.input / per_million)
        + (tokens.output_tokens as f64 * rates.output / per_million)
        + (tokens.cache_creation_tokens as f64 * rates.cache_create / per_million)
        + (tokens.cache_read_tokens as f64 * rates.cache_read / per_million)
}
