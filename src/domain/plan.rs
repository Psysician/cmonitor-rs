use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PlanType {
    Pro,
    Max5,
    Max20,
    Custom,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlanDefinition {
    pub name: String,
    pub token_limit: Option<u64>,
    pub cost_limit: Option<f64>,
    pub message_limit: Option<u32>,
    pub default_custom_minimum: u64,
}

pub const DEFAULT_CUSTOM_MINIMUM: u64 = 44_000;
pub const LIMIT_THRESHOLD: f64 = 0.90;
pub const COMMON_TOKEN_LIMITS: [u64; 3] = [44_000, 220_000, 880_000];

pub fn plan_definition(plan: PlanType, custom_limit: Option<u64>) -> PlanDefinition {
    match plan {
        PlanType::Pro => PlanDefinition {
            name: "pro".to_owned(),
            token_limit: Some(COMMON_TOKEN_LIMITS[0]),
            cost_limit: Some(18.0),
            message_limit: Some(45),
            default_custom_minimum: DEFAULT_CUSTOM_MINIMUM,
        },
        PlanType::Max5 => PlanDefinition {
            name: "max5".to_owned(),
            token_limit: Some(COMMON_TOKEN_LIMITS[1]),
            cost_limit: Some(35.0),
            message_limit: Some(225),
            default_custom_minimum: DEFAULT_CUSTOM_MINIMUM,
        },
        PlanType::Max20 => PlanDefinition {
            name: "max20".to_owned(),
            token_limit: Some(COMMON_TOKEN_LIMITS[2]),
            cost_limit: Some(140.0),
            message_limit: Some(900),
            default_custom_minimum: DEFAULT_CUSTOM_MINIMUM,
        },
        PlanType::Custom => PlanDefinition {
            name: "custom".to_owned(),
            token_limit: custom_limit,
            cost_limit: Some(50.0),
            message_limit: None,
            default_custom_minimum: DEFAULT_CUSTOM_MINIMUM,
        },
    }
}
