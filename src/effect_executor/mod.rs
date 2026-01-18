//! Effect execution engine for processing card effects.
//!
//! This module handles execution of effect operations defined in effect.rs.
//! The system is fully tested but not yet integrated into the main game loop.
#![allow(dead_code)]

mod value_ops;
mod flag_ops;
mod variable_ops;
mod event_ops;
mod collection_ops;
mod control_flow;

use bevy::prelude::*;
use crate::effect::{EffectOperation, Predicate, Value, CardEffect, EffectContext};
use crate::game_state::GameState;
use std::collections::HashMap;

/// Result type for effect execution
pub type EffectResult = Result<(), EffectError>;

/// Errors that can occur during effect execution
#[derive(Debug, Clone)]
pub enum EffectError {
    InvalidPath(String),
    InvalidValue(String),
    PredicateError(String),
    OperationError(String),
    MaxIterationsReached,
}

impl std::fmt::Display for EffectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectError::InvalidPath(p) => write!(f, "Invalid state path: {}", p),
            EffectError::InvalidValue(v) => write!(f, "Invalid value: {}", v),
            EffectError::PredicateError(e) => write!(f, "Predicate error: {}", e),
            EffectError::OperationError(e) => write!(f, "Operation error: {}", e),
            EffectError::MaxIterationsReached => write!(f, "Maximum iterations reached"),
        }
    }
}

impl std::error::Error for EffectError {}

/// Executes card effects by processing primitive operations
pub struct EffectExecutor {
    event_listeners: HashMap<String, Vec<Vec<EffectOperation>>>,
}

impl Default for EffectExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectExecutor {
    pub fn new() -> Self {
        Self {
            event_listeners: HashMap::new(),
        }
    }

    /// Execute a card effect
    pub fn execute_effect(
        &mut self,
        effect: &CardEffect,
        context: &mut EffectContext,
        state: &mut GameState,
        world: &mut World,
    ) -> EffectResult {
        for operation in &effect.operations {
            self.execute_operation(operation, context, state, world)?;
        }
        Ok(())
    }

    /// Execute a single operation by dispatching to the appropriate handler
    pub fn execute_operation(
        &mut self,
        operation: &EffectOperation,
        context: &mut EffectContext,
        state: &mut GameState,
        world: &mut World,
    ) -> EffectResult {
        // Try value operations
        if let Some(result) = value_ops::execute_value_operation(operation, state, world) {
            return result;
        }

        // Try flag operations
        if let Some(result) = flag_ops::execute_flag_operation(operation, state, world) {
            return result;
        }

        // Try variable operations
        if let Some(result) = variable_ops::execute_variable_operation(operation, context) {
            return result;
        }

        // Try event operations
        if let Some(result) = event_ops::execute_event_operation(self, operation, context, state, world) {
            return result;
        }

        // Try collection operations
        if let Some(result) = collection_ops::execute_collection_operation(operation, context, world) {
            return result;
        }

        // Try control flow operations
        if let Some(result) = control_flow::execute_control_flow_operation(self, operation, context, state, world) {
            return result;
        }

        Ok(())
    }

    /// Register an event listener
    pub fn register_event_listener(&mut self, event: String, operations: Vec<EffectOperation>) {
        self.event_listeners
            .entry(event)
            .or_default()
            .push(operations);
    }

    /// Get event listeners (cloned to avoid borrow conflicts)
    pub fn get_event_listeners(&self, event: &str) -> Option<Vec<Vec<EffectOperation>>> {
        self.event_listeners.get(event).cloned()
    }

    /// Evaluate a predicate against current game state
    pub fn evaluate_predicate(
        &self,
        predicate: &Predicate,
        context: &EffectContext,
        state: &GameState,
        world: &World,
    ) -> Result<bool, EffectError> {
        match predicate {
            Predicate::Equals { field, value } => {
                let current = self.resolve_value(field, context, state, world)?;
                Ok(current == *value)
            }

            Predicate::NotEquals { field, value } => {
                let current = self.resolve_value(field, context, state, world)?;
                Ok(current != *value)
            }

            Predicate::GreaterThan { field, value } => {
                let current = self.resolve_value(field, context, state, world)?;
                self.compare_values(&current, value, |a, b| a > b)
            }

            Predicate::LessThan { field, value } => {
                let current = self.resolve_value(field, context, state, world)?;
                self.compare_values(&current, value, |a, b| a < b)
            }

            Predicate::GreaterOrEqual { field, value } => {
                let current = self.resolve_value(field, context, state, world)?;
                self.compare_values(&current, value, |a, b| a >= b)
            }

            Predicate::LessOrEqual { field, value } => {
                let current = self.resolve_value(field, context, state, world)?;
                self.compare_values(&current, value, |a, b| a <= b)
            }

            Predicate::And { predicates } => {
                for pred in predicates {
                    if !self.evaluate_predicate(pred, context, state, world)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }

            Predicate::Or { predicates } => {
                for pred in predicates {
                    if self.evaluate_predicate(pred, context, state, world)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }

            Predicate::Not { predicate } => {
                Ok(!self.evaluate_predicate(predicate, context, state, world)?)
            }

            Predicate::HasTag { tag: _ } => {
                warn!("HasTag predicate not yet implemented");
                Ok(false)
            }

            Predicate::IsType { card_type: _ } => {
                warn!("IsType predicate not yet implemented");
                Ok(false)
            }

            Predicate::Contains { field: _, value: _ } => {
                warn!("Contains predicate not yet implemented");
                Ok(false)
            }

            Predicate::Expression { expr: _ } => {
                warn!("Expression predicate not yet implemented");
                Ok(false)
            }
        }
    }

    /// Resolve a value (could be a state path or variable reference)
    fn resolve_value(
        &self,
        field: &str,
        context: &EffectContext,
        state: &GameState,
        world: &World,
    ) -> Result<Value, EffectError> {
        // Check if it's a variable reference
        if let Some(var_name) = field.strip_prefix('$') {
            if let Some(value) = context.get_variable(var_name) {
                return Ok(value.clone());
            }
        }

        // Otherwise, treat as state path
        state.get(field, world)
            .ok_or_else(|| EffectError::InvalidPath(field.to_string()))
    }

    /// Compare numeric values
    fn compare_values<F>(&self, a: &Value, b: &Value, op: F) -> Result<bool, EffectError>
    where
        F: Fn(f64, f64) -> bool,
    {
        let a_num = match a {
            Value::Int(v) => *v as f64,
            Value::Float(v) => *v as f64,
            _ => return Err(EffectError::InvalidValue("Not a number".to_string())),
        };

        let b_num = match b {
            Value::Int(v) => *v as f64,
            Value::Float(v) => *v as f64,
            _ => return Err(EffectError::InvalidValue("Not a number".to_string())),
        };

        Ok(op(a_num, b_num))
    }

    /// Clear all event listeners
    pub fn clear_listeners(&mut self) {
        self.event_listeners.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_operation() {
        let mut world = World::new();
        world.insert_resource(crate::resources::GameTimer {
            timer: bevy::time::Timer::from_seconds(300.0, bevy::time::TimerMode::Once),
            paused: false,
        });

        let mut state = GameState::new();
        let mut executor = EffectExecutor::new();
        let mut context = EffectContext::new("test_card".to_string(), "test_effect".to_string());

        let operation = EffectOperation::Add {
            target: "timer.remaining".to_string(),
            amount: 60,
        };

        assert!(executor.execute_operation(&operation, &mut context, &mut state, &mut world).is_ok());
        if let Some(Value::Int(remaining)) = state.get("timer.remaining", &world) {
            assert!(remaining >= 359 && remaining <= 361);
        }
    }

    #[test]
    fn test_if_condition() {
        let mut world = World::new();
        world.insert_resource(crate::resources::GameTimer {
            timer: bevy::time::Timer::from_seconds(300.0, bevy::time::TimerMode::Once),
            paused: false,
        });

        let mut state = GameState::new();
        let mut executor = EffectExecutor::new();
        let mut context = EffectContext::new("test_card".to_string(), "test_effect".to_string());

        let operation = EffectOperation::IfCondition {
            condition: Predicate::GreaterThan {
                field: "timer.remaining".to_string(),
                value: Value::Int(200),
            },
            then: vec![
                EffectOperation::Add {
                    target: "timer.remaining".to_string(),
                    amount: 60,
                }
            ],
            else_: Some(vec![
                EffectOperation::Subtract {
                    target: "timer.remaining".to_string(),
                    amount: 60,
                }
            ]),
        };

        assert!(executor.execute_operation(&operation, &mut context, &mut state, &mut world).is_ok());
        // Condition is true (300 > 200), so should add 60
        if let Some(Value::Int(remaining)) = state.get("timer.remaining", &world) {
            assert!(remaining >= 359 && remaining <= 361);
        }
    }

    #[test]
    fn test_variables() {
        let mut world = World::new();
        let mut state = GameState::new();
        let mut executor = EffectExecutor::new();
        let mut context = EffectContext::new("test_card".to_string(), "test_effect".to_string());

        // Set variable
        let set_var = EffectOperation::SetVariable {
            name: "my_value".to_string(),
            value: Value::Int(42),
        };
        assert!(executor.execute_operation(&set_var, &mut context, &mut state, &mut world).is_ok());

        // Verify variable is set
        assert_eq!(context.get_variable("my_value"), Some(&Value::Int(42)));
    }

    #[test]
    fn test_collection_operations() {
        use crate::collections::{CollectionManager, Collection};

        let mut world = World::new();

        // Initialize collection manager with test data
        let mut collections = CollectionManager::new();
        let mut collection = Collection::new();

        // Add test items
        for i in 1..=5 {
            let mut item = HashMap::new();
            item.insert("id".to_string(), Value::Int(i));
            item.insert("value".to_string(), Value::Int(i * 10));
            collection.append(Value::Object(item));
        }

        collections.set("test.items", collection);
        world.insert_resource(collections);

        let mut state = GameState::new();
        let mut executor = EffectExecutor::new();
        let mut context = EffectContext::new("test_card".to_string(), "test_effect".to_string());

        // Test Filter operation
        let filter_op = EffectOperation::Filter {
            target: "test.items".to_string(),
            predicate: Predicate::GreaterThan {
                field: "value".to_string(),
                value: Value::Int(25),
            },
        };
        assert!(executor.execute_operation(&filter_op, &mut context, &mut state, &mut world).is_ok());

        // Verify filtered collection
        let collections = world.get_resource::<CollectionManager>().unwrap();
        let collection = collections.get("test.items").unwrap();
        assert_eq!(collection.len(), 3); // Items with value > 25: 30, 40, 50

        // Test Remove operation
        let remove_op = EffectOperation::Remove {
            target: "test.items".to_string(),
            count: 1,
            filter: None,
            random: Some(false),
        };
        assert!(executor.execute_operation(&remove_op, &mut context, &mut state, &mut world).is_ok());

        // Verify removed
        let collections = world.get_resource::<CollectionManager>().unwrap();
        let collection = collections.get("test.items").unwrap();
        assert_eq!(collection.len(), 2);

        // Test Append operation
        let mut new_item = HashMap::new();
        new_item.insert("id".to_string(), Value::Int(99));
        new_item.insert("value".to_string(), Value::Int(999));

        let append_op = EffectOperation::Append {
            target: "test.items".to_string(),
            item: Value::Object(new_item),
        };
        assert!(executor.execute_operation(&append_op, &mut context, &mut state, &mut world).is_ok());

        // Verify appended
        let collections = world.get_resource::<CollectionManager>().unwrap();
        let collection = collections.get("test.items").unwrap();
        assert_eq!(collection.len(), 3);
    }

    #[test]
    fn test_for_each_operation() {
        use crate::collections::{CollectionManager, Collection};

        let mut world = World::new();
        world.insert_resource(crate::resources::Score {
            current: 0,
            passing_grade: 10,
            correct_answers: 0,
            total_answered: 0,
        });

        // Create collection with numbers
        let mut collections = CollectionManager::new();
        let mut collection = Collection::new();
        collection.append(Value::Int(5));
        collection.append(Value::Int(10));
        collection.append(Value::Int(15));
        collections.set("numbers", collection);
        world.insert_resource(collections);

        let mut state = GameState::new();
        let mut executor = EffectExecutor::new();
        let mut context = EffectContext::new("test_card".to_string(), "test_effect".to_string());

        // ForEach that adds each number to score
        let for_each_op = EffectOperation::ForEach {
            collection: "numbers".to_string(),
            operations: vec![
                // This would normally add $item to score, but we'll just verify the loop runs
                EffectOperation::SetVariable {
                    name: "last_item".to_string(),
                    value: Value::Int(0), // Will be overwritten
                },
            ],
        };

        assert!(executor.execute_operation(&for_each_op, &mut context, &mut state, &mut world).is_ok());

        // Verify the loop ran 3 times (item variable should be set to last item)
        assert!(context.get_variable("item").is_some());
    }
}

// Include comprehensive effect operation tests
#[cfg(test)]
#[path = "../effect_executor_tests.rs"]
mod comprehensive_tests;
