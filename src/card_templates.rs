//! Expansion of human-friendly YAML card-effect shorthand into primitive
//! `EffectOperation` trees consumed by the executor.

use bevy::log::warn;
use serde::{Deserialize, Serialize};

use crate::effect::{
    CardEffect, EffectOperation, EffectTiming, InterceptPoint, Predicate, Value,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct YamlCardEffect {
    pub id: String,
    #[serde(rename = "type")]
    pub shorthand: String,
    #[serde(flatten)]
    pub parameters: serde_json::Value,
    #[serde(default)]
    pub intercepts: Vec<InterceptPoint>,
    #[serde(default = "default_timing")]
    pub timing: EffectTiming,
    #[serde(default = "default_priority")]
    pub priority: i32,
}

fn default_timing() -> EffectTiming {
    EffectTiming::After
}

fn default_priority() -> i32 {
    100
}

pub fn expand(yaml: YamlCardEffect) -> CardEffect {
    let operations = expand_operations(&yaml.shorthand, &yaml.parameters);
    let intercepts = if yaml.intercepts.is_empty() {
        None
    } else {
        Some(yaml.intercepts)
    };

    CardEffect {
        id: yaml.id,
        name: None,
        description: None,
        operations,
        timing: yaml.timing,
        priority: yaml.priority,
        intercepts,
    }
}

fn expand_operations(shorthand: &str, params: &serde_json::Value) -> Vec<EffectOperation> {
    match shorthand {
        "eliminate_wrong_answer" => {
            let count = params
                .get("count")
                .and_then(|v| v.as_u64())
                .unwrap_or(1) as usize;
            vec![EffectOperation::Remove {
                target: "question.options".to_string(),
                count,
                filter: Some(Predicate::Equals {
                    field: "correct".to_string(),
                    value: Value::Bool(false),
                }),
                random: Some(true),
            }]
        }
        "add_time" => {
            let seconds = params
                .get("seconds")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;
            vec![EffectOperation::Add {
                target: "timer.remaining".to_string(),
                amount: seconds,
            }]
        }
        "add_points" => {
            let points = params
                .get("points")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;
            vec![EffectOperation::Add {
                target: "question.points".to_string(),
                amount: points,
            }]
        }
        "multiply_points" => {
            let multiplier = params
                .get("multiplier")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0) as f32;
            vec![EffectOperation::Multiply {
                target: "question.points".to_string(),
                factor: multiplier,
            }]
        }
        _ => {
            warn!("Unknown card effect shorthand: {}", shorthand);
            Vec::new()
        }
    }
}
