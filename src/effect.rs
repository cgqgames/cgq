use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Generic value type for effect operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Value {
    Int(i32),
    Float(f32),
    Bool(bool),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Null,
}

impl Value {
    pub fn as_int(&self) -> Option<i32> {
        match self {
            Value::Int(v) => Some(*v),
            Value::Float(v) => Some(*v as i32),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f32> {
        match self {
            Value::Float(v) => Some(*v),
            Value::Int(v) => Some(*v as f32),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(v) => Some(v),
            _ => None,
        }
    }
}

/// Predicate for filtering and conditional logic
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Predicate {
    /// Field equals value
    Equals {
        field: String,
        value: Value,
    },
    /// Field not equals value
    NotEquals {
        field: String,
        value: Value,
    },
    /// Field greater than value
    GreaterThan {
        field: String,
        value: Value,
    },
    /// Field less than value
    LessThan {
        field: String,
        value: Value,
    },
    /// Field greater than or equal to value
    GreaterOrEqual {
        field: String,
        value: Value,
    },
    /// Field less than or equal to value
    LessOrEqual {
        field: String,
        value: Value,
    },
    /// Check if item has tag
    HasTag {
        tag: String,
    },
    /// Check if item is of type
    IsType {
        card_type: String,
    },
    /// Check if array/string contains value
    Contains {
        field: String,
        value: Value,
    },
    /// Boolean AND of multiple predicates
    And {
        predicates: Vec<Predicate>,
    },
    /// Boolean OR of multiple predicates
    Or {
        predicates: Vec<Predicate>,
    },
    /// Boolean NOT
    Not {
        predicate: Box<Predicate>,
    },
    /// Custom expression (for advanced users)
    Expression {
        expr: String,
    },
}

/// Primitive effect operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EffectOperation {
    /// Add value to target
    Add {
        target: String,
        amount: i32,
    },
    /// Subtract value from target
    Subtract {
        target: String,
        amount: i32,
    },
    /// Multiply target by factor
    Multiply {
        target: String,
        factor: f32,
    },
    /// Set target to specific value
    Set {
        target: String,
        value: Value,
    },
    /// Filter collection by predicate
    Filter {
        target: String,
        predicate: Predicate,
    },
    /// Remove items from collection
    Remove {
        target: String,
        count: usize,
        filter: Option<Predicate>,
        random: Option<bool>,
    },
    /// Append item to collection
    Append {
        target: String,
        item: Value,
    },
    /// Insert item at index
    Insert {
        target: String,
        index: usize,
        item: Value,
    },
    /// Set boolean flag
    SetFlag {
        flag: String,
        value: bool,
    },
    /// Toggle boolean flag
    ToggleFlag {
        flag: String,
    },
    /// Conditional execution
    IfCondition {
        condition: Predicate,
        then: Vec<EffectOperation>,
        #[serde(rename = "else", skip_serializing_if = "Option::is_none")]
        else_: Option<Vec<EffectOperation>>,
    },
    /// Execute operations for each item in collection
    ForEach {
        collection: String,
        operations: Vec<EffectOperation>,
    },
    /// Execute while condition is true
    While {
        condition: Predicate,
        operations: Vec<EffectOperation>,
        max_iterations: Option<usize>,
    },
    /// Register event listener
    OnEvent {
        event: String,
        operations: Vec<EffectOperation>,
    },
    /// Emit custom event
    EmitEvent {
        event: String,
        data: Option<HashMap<String, Value>>,
    },
    /// Schedule operation for future execution
    ScheduleOperation {
        delay_seconds: f32,
        operations: Vec<EffectOperation>,
    },
    /// Store value in temporary variable
    SetVariable {
        name: String,
        value: Value,
    },
    /// Get value from variable/path
    GetVariable {
        name: String,
    },
}

/// Timing for when effect should execute
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EffectTiming {
    Before,
    After,
    OnDeploy,
    OnRemoval,
    OnDiscard,
}

/// Card effect definition
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct CardEffect {
    /// Unique identifier for this effect
    pub id: String,
    /// Human-readable name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Description of what this effect does
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Operations to execute
    pub operations: Vec<EffectOperation>,
    /// When this effect should execute
    #[serde(default = "default_timing")]
    pub timing: EffectTiming,
    /// Priority for execution order (higher = earlier)
    #[serde(default = "default_priority")]
    pub priority: i32,
    /// Optional component interception points
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intercepts: Option<Vec<InterceptPoint>>,
}

fn default_timing() -> EffectTiming {
    EffectTiming::After
}

fn default_priority() -> i32 {
    100
}

/// Defines where effect intercepts component operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterceptPoint {
    pub component: String,
    pub operation: String,
}

/// Active effect instance on a card
#[derive(Debug, Clone, Component)]
pub struct ActiveEffect {
    pub card_id: String,
    pub effect_id: String,
    pub event_listeners: Vec<String>,
    pub variables: HashMap<String, Value>,
}

/// Effect execution context
#[derive(Debug, Clone)]
pub struct EffectContext {
    pub card_id: String,
    pub effect_id: String,
    pub variables: HashMap<String, Value>,
    pub event_data: Option<HashMap<String, Value>>,
}

impl EffectContext {
    pub fn new(card_id: String, effect_id: String) -> Self {
        Self {
            card_id,
            effect_id,
            variables: HashMap::new(),
            event_data: None,
        }
    }

    pub fn with_event_data(mut self, data: HashMap<String, Value>) -> Self {
        self.event_data = Some(data);
        self
    }

    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_conversions() {
        let int_val = Value::Int(42);
        assert_eq!(int_val.as_int(), Some(42));
        assert_eq!(int_val.as_float(), Some(42.0));

        let float_val = Value::Float(3.14);
        assert_eq!(float_val.as_int(), Some(3));
        assert_eq!(float_val.as_float(), Some(3.14));

        let bool_val = Value::Bool(true);
        assert_eq!(bool_val.as_bool(), Some(true));
    }

    #[test]
    fn test_effect_serialization() {
        let effect = CardEffect {
            id: "test_effect".to_string(),
            name: Some("Test Effect".to_string()),
            description: Some("A test effect".to_string()),
            operations: vec![
                EffectOperation::Add {
                    target: "timer.remaining".to_string(),
                    amount: 60,
                },
                EffectOperation::Set {
                    target: "question.points".to_string(),
                    value: Value::Int(5),
                },
            ],
            timing: EffectTiming::After,
            priority: 100,
            intercepts: None,
        };

        let json = serde_json::to_string_pretty(&effect).unwrap();
        let deserialized: CardEffect = serde_json::from_str(&json).unwrap();

        assert_eq!(effect.id, deserialized.id);
        assert_eq!(effect.operations.len(), deserialized.operations.len());
    }

    #[test]
    fn test_predicate_serialization() {
        let predicate = Predicate::And {
            predicates: vec![
                Predicate::GreaterThan {
                    field: "timer.remaining".to_string(),
                    value: Value::Int(300),
                },
                Predicate::HasTag {
                    tag: "idf".to_string(),
                },
            ],
        };

        let json = serde_json::to_string_pretty(&predicate).unwrap();
        let deserialized: Predicate = serde_json::from_str(&json).unwrap();

        match deserialized {
            Predicate::And { predicates } => assert_eq!(predicates.len(), 2),
            _ => panic!("Wrong predicate type"),
        }
    }
}
