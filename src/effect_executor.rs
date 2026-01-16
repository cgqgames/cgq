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

    /// Execute a single operation
    pub fn execute_operation(
        &mut self,
        operation: &EffectOperation,
        context: &mut EffectContext,
        state: &mut GameState,
        world: &mut World,
    ) -> EffectResult {
        match operation {
            EffectOperation::Add { target, amount } => {
                if !state.add(target, *amount, world) {
                    return Err(EffectError::InvalidPath(target.clone()));
                }
            }

            EffectOperation::Subtract { target, amount } => {
                if !state.subtract(target, *amount, world) {
                    return Err(EffectError::InvalidPath(target.clone()));
                }
            }

            EffectOperation::Multiply { target, factor } => {
                if !state.multiply(target, *factor, world) {
                    return Err(EffectError::InvalidPath(target.clone()));
                }
            }

            EffectOperation::Set { target, value } => {
                if !state.set(target, value.clone(), world) {
                    return Err(EffectError::InvalidPath(target.clone()));
                }
            }

            EffectOperation::SetFlag { flag, value } => {
                if !state.set_flag(flag, *value, world) {
                    return Err(EffectError::InvalidPath(flag.clone()));
                }
            }

            EffectOperation::ToggleFlag { flag } => {
                if !state.toggle_flag(flag, world) {
                    return Err(EffectError::InvalidPath(flag.clone()));
                }
            }

            EffectOperation::IfCondition { condition, then, else_ } => {
                let condition_met = self.evaluate_predicate(condition, context, state, world)?;

                if condition_met {
                    for op in then {
                        self.execute_operation(op, context, state, world)?;
                    }
                } else if let Some(else_ops) = else_ {
                    for op in else_ops {
                        self.execute_operation(op, context, state, world)?;
                    }
                }
            }

            EffectOperation::While { condition, operations, max_iterations } => {
                let max = max_iterations.unwrap_or(100);
                let mut iterations = 0;

                while self.evaluate_predicate(condition, context, state, world)? {
                    if iterations >= max {
                        return Err(EffectError::MaxIterationsReached);
                    }

                    for op in operations {
                        self.execute_operation(op, context, state, world)?;
                    }

                    iterations += 1;
                }
            }

            EffectOperation::SetVariable { name, value } => {
                context.set_variable(name.clone(), value.clone());
            }

            EffectOperation::GetVariable { name } => {
                // Variable retrieval happens during value resolution
                // This operation is mainly for explicit documentation
                if context.get_variable(name).is_none() {
                    warn!("Variable '{}' not found in context", name);
                }
            }

            EffectOperation::OnEvent { event, operations } => {
                // Register event listener
                self.event_listeners
                    .entry(event.clone())
                    .or_insert_with(Vec::new)
                    .push(operations.clone());
            }

            EffectOperation::EmitEvent { event, data } => {
                // Get listeners for this event
                if let Some(listeners) = self.event_listeners.get(event) {
                    // Update context with event data
                    if let Some(event_data) = data {
                        for (key, value) in event_data {
                            context.set_variable(format!("event.{}", key), value.clone());
                        }
                    }

                    // Execute all listeners
                    for operations in listeners.clone() {
                        for op in &operations {
                            self.execute_operation(op, context, state, world)?;
                        }
                    }
                }
            }

            EffectOperation::ScheduleOperation { delay_seconds: _, operations: _ } => {
                // TODO: Implement scheduled operations using Bevy's timer system
                warn!("ScheduleOperation not yet implemented");
            }

            EffectOperation::Filter { target: _, predicate: _ } => {
                // TODO: Implement collection filtering
                warn!("Filter operation not yet implemented");
            }

            EffectOperation::Remove { target: _, count: _, filter: _, random: _ } => {
                // TODO: Implement collection removal
                warn!("Remove operation not yet implemented");
            }

            EffectOperation::Append { target: _, item: _ } => {
                // TODO: Implement collection append
                warn!("Append operation not yet implemented");
            }

            EffectOperation::Insert { target: _, index: _, item: _ } => {
                // TODO: Implement collection insert
                warn!("Insert operation not yet implemented");
            }

            EffectOperation::ForEach { collection: _, operations: _ } => {
                // TODO: Implement collection iteration
                warn!("ForEach operation not yet implemented");
            }
        }

        Ok(())
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
                // TODO: Implement tag checking
                warn!("HasTag predicate not yet implemented");
                Ok(false)
            }

            Predicate::IsType { card_type: _ } => {
                // TODO: Implement type checking
                warn!("IsType predicate not yet implemented");
                Ok(false)
            }

            Predicate::Contains { field: _, value: _ } => {
                // TODO: Implement contains checking
                warn!("Contains predicate not yet implemented");
                Ok(false)
            }

            Predicate::Expression { expr: _ } => {
                // TODO: Implement expression evaluation
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
        if field.starts_with('$') {
            let var_name = &field[1..];
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
}
