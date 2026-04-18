//! Expansion of human-friendly YAML/TOML card-effect shorthand into primitive
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
        // --- Value modifications ---
        "add_time" => vec![add_op("timer.remaining", param_i32(params, "seconds"))],
        "subtract_time" => vec![subtract_op("timer.remaining", param_i32(params, "seconds"))],
        "add_points" => vec![add_op("question.points", param_i32(params, "points"))],
        "subtract_points" => vec![subtract_op("question.points", param_i32(params, "points"))],
        "modify_passing_grade" => vec![add_op("score.passing_grade", param_i32(params, "amount"))],
        "modify_max_slots" => vec![add_op("cards.slots.max", param_i32(params, "amount"))],
        "multiply_points" => vec![EffectOperation::Multiply {
            target: "question.points".to_string(),
            factor: param_f32(params, "multiplier", 1.0),
        }],

        // --- Collection operations ---
        "eliminate_wrong_answer" => vec![EffectOperation::Remove {
            target: "question.options".to_string(),
            count: param_usize(params, "count", 1),
            filter: Some(Predicate::Equals {
                field: "correct".to_string(),
                value: Value::Bool(false),
            }),
            random: Some(true),
        }],

        // --- Card bans ---
        "ban_card_types" => ban_ops("cards.banned_types", params, "types"),
        "ban_cards" => ban_ops("cards.banned_ids", params, "ids"),

        // --- Vote-requirement modifiers (per-type) ---
        "modify_vote_requirement" => {
            let amount = param_i32(params, "amount");
            let card_type = params
                .get("card_type")
                .and_then(|v| v.as_str())
                .unwrap_or("*")
                .to_string();
            vec![add_op(&format!("cards.vote_req.{}", card_type), amount)]
        }

        // --- Conditionals ---
        "if_timer_above" => expand_conditional(params, true),
        "if_timer_below" => expand_conditional(params, false),

        // --- Answer event hooks ---
        // `on_correct_answer { do = [...] }` registers a listener that runs
        // the nested `do` effects whenever an answer resolves as correct.
        "on_correct_answer" => vec![EffectOperation::OnEvent {
            event: "answer_correct".to_string(),
            operations: expand_nested(params, "do"),
        }],
        "on_wrong_answer" => vec![EffectOperation::OnEvent {
            event: "answer_wrong".to_string(),
            operations: expand_nested(params, "do"),
        }],

        _ => {
            warn!("Unknown card effect shorthand: {}", shorthand);
            Vec::new()
        }
    }
}

fn expand_conditional(params: &serde_json::Value, above: bool) -> Vec<EffectOperation> {
    let percent = param_f32(params, "percent", 50.0);
    let then_ops = expand_nested(params, "then");
    let else_ops = expand_nested(params, "else");

    let field = "timer.percent_remaining".to_string();
    let threshold = Value::Float(percent);
    let condition = if above {
        Predicate::GreaterThan { field, value: threshold }
    } else {
        Predicate::LessThan { field, value: threshold }
    };

    vec![EffectOperation::IfCondition {
        condition,
        then: then_ops,
        else_: if else_ops.is_empty() { None } else { Some(else_ops) },
    }]
}

fn expand_nested(params: &serde_json::Value, key: &str) -> Vec<EffectOperation> {
    let Some(raw) = params.get(key) else { return Vec::new() };
    let effects: Vec<YamlCardEffect> = match serde_json::from_value(raw.clone()) {
        Ok(effects) => effects,
        Err(e) => {
            warn!("Failed to parse nested '{}' effects: {}", key, e);
            return Vec::new();
        }
    };
    effects
        .into_iter()
        .flat_map(|eff| expand(eff).operations)
        .collect()
}

fn ban_ops(target: &str, params: &serde_json::Value, key: &str) -> Vec<EffectOperation> {
    param_strings(params, key)
        .into_iter()
        .map(|s| EffectOperation::Append {
            target: target.to_string(),
            item: Value::String(s),
        })
        .collect()
}

fn add_op(target: &str, amount: i32) -> EffectOperation {
    EffectOperation::Add {
        target: target.to_string(),
        amount,
    }
}

fn subtract_op(target: &str, amount: i32) -> EffectOperation {
    EffectOperation::Subtract {
        target: target.to_string(),
        amount,
    }
}

fn param_i32(params: &serde_json::Value, key: &str) -> i32 {
    params
        .get(key)
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .unwrap_or(0)
}

fn param_usize(params: &serde_json::Value, key: &str, default: usize) -> usize {
    params
        .get(key)
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(default)
}

fn param_f32(params: &serde_json::Value, key: &str, default: f32) -> f32 {
    params
        .get(key)
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(default)
}

fn param_strings(params: &serde_json::Value, key: &str) -> Vec<String> {
    params
        .get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}
