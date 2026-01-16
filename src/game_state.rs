use bevy::prelude::*;
use crate::effect::Value;
use crate::resources::{GameTimer, Score, CardManager};
use std::collections::HashMap;

/// Path-based access to game state for effects
/// Allows effects to query/modify state using string paths like "timer.remaining"
#[derive(Resource)]
pub struct GameState {
    // Cached values for quick access
    variables: HashMap<String, Value>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

impl GameState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get value from game state using path notation
    pub fn get(&self, path: &str, world: &World) -> Option<Value> {
        let parts: Vec<&str> = path.split('.').collect();

        match parts.as_slice() {
            // Timer paths
            ["timer", "remaining"] => {
                world.get_resource::<GameTimer>().map(|gt| {
                    Value::Int(gt.timer.remaining().as_secs() as i32)
                })
            }
            ["timer", "elapsed"] => {
                world.get_resource::<GameTimer>().map(|gt| {
                    Value::Int(gt.timer.elapsed().as_secs() as i32)
                })
            }
            ["timer", "percent_remaining"] => {
                world.get_resource::<GameTimer>().map(|gt| {
                    let total = gt.timer.duration().as_secs_f32();
                    let elapsed = gt.timer.elapsed().as_secs_f32();
                    if total > 0.0 {
                        Value::Float(((total - elapsed) / total) * 100.0)
                    } else {
                        Value::Float(0.0)
                    }
                })
            }
            ["timer", "paused"] => {
                world.get_resource::<GameTimer>()
                    .map(|gt| Value::Bool(gt.paused))
            }

            // Score paths
            ["score", "current"] => {
                world.get_resource::<Score>()
                    .map(|score| Value::Int(score.current))
            }
            ["score", "passing_grade"] => {
                world.get_resource::<Score>()
                    .map(|score| Value::Int(score.passing_grade))
            }

            // Question paths
            ["question", "points"] => {
                // Need to query active question
                // For now, return None - will be implemented when we add active question tracking
                None
            }

            // Card paths
            ["cards", "slots", "max"] => {
                // TODO: Add max_slots field to CardManager
                Some(Value::Int(10)) // Default max slots
            }
            ["cards", "slots", "occupied"] => {
                world.get_resource::<CardManager>()
                    .map(|cm| Value::Int(cm.deployed_card_ids.len() as i32))
            }

            // Variables (temporary storage)
            ["var", var_name] => {
                self.variables.get(*var_name).cloned()
            }

            _ => {
                warn!("Unknown state path: {}", path);
                None
            }
        }
    }

    /// Set value in game state using path notation
    pub fn set(&mut self, path: &str, value: Value, world: &mut World) -> bool {
        let parts: Vec<&str> = path.split('.').collect();

        match parts.as_slice() {
            // Timer paths
            ["timer", "remaining"] => {
                if let Some(seconds) = value.as_int() {
                    if let Some(mut gt) = world.get_resource_mut::<GameTimer>() {
                        let elapsed = gt.timer.elapsed();
                        let new_duration = std::time::Duration::from_secs(seconds.max(0) as u64) + elapsed;
                        gt.timer.set_duration(new_duration);
                        return true;
                    }
                }
                false
            }
            ["timer", "paused"] => {
                if let Some(paused) = value.as_bool() {
                    if let Some(mut gt) = world.get_resource_mut::<GameTimer>() {
                        gt.paused = paused;
                        if paused {
                            gt.timer.pause();
                        } else {
                            gt.timer.unpause();
                        }
                        return true;
                    }
                }
                false
            }

            // Score paths
            ["score", "current"] => {
                if let Some(amount) = value.as_int() {
                    if let Some(mut score) = world.get_resource_mut::<Score>() {
                        score.current = amount;
                        return true;
                    }
                }
                false
            }
            ["score", "passing_grade"] => {
                if let Some(amount) = value.as_int() {
                    if let Some(mut score) = world.get_resource_mut::<Score>() {
                        score.passing_grade = amount;
                        return true;
                    }
                }
                false
            }

            // Variables
            ["var", var_name] => {
                self.variables.insert(var_name.to_string(), value);
                true
            }

            _ => {
                warn!("Cannot set unknown state path: {}", path);
                false
            }
        }
    }

    /// Add to numeric value at path
    pub fn add(&mut self, path: &str, amount: i32, world: &mut World) -> bool {
        if let Some(current) = self.get(path, world) {
            if let Some(current_val) = current.as_int() {
                let new_val = Value::Int(current_val + amount);
                return self.set(path, new_val, world);
            }
        }
        false
    }

    /// Subtract from numeric value at path
    pub fn subtract(&mut self, path: &str, amount: i32, world: &mut World) -> bool {
        self.add(path, -amount, world)
    }

    /// Multiply numeric value at path
    pub fn multiply(&mut self, path: &str, factor: f32, world: &mut World) -> bool {
        if let Some(current) = self.get(path, world) {
            let new_val = match current {
                Value::Int(v) => Value::Int((v as f32 * factor) as i32),
                Value::Float(v) => Value::Float(v * factor),
                _ => return false,
            };
            return self.set(path, new_val, world);
        }
        false
    }

    /// Set boolean flag
    pub fn set_flag(&mut self, flag: &str, value: bool, world: &mut World) -> bool {
        self.set(flag, Value::Bool(value), world)
    }

    /// Toggle boolean flag
    pub fn toggle_flag(&mut self, flag: &str, world: &mut World) -> bool {
        if let Some(current) = self.get(flag, world) {
            if let Some(current_bool) = current.as_bool() {
                return self.set_flag(flag, !current_bool, world);
            }
        }
        false
    }

    /// Store variable
    pub fn set_variable(&mut self, name: &str, value: Value) {
        self.variables.insert(format!("var.{}", name), value);
    }

    /// Get variable
    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(&format!("var.{}", name))
    }

    /// Clear all variables
    pub fn clear_variables(&mut self) {
        self.variables.clear();
    }
}

/// Helper trait for getting mutable references to collections in game state
pub trait CollectionAccess {
    fn get_collection_mut(&mut self, path: &str) -> Option<&mut Vec<Value>>;
    fn get_collection(&self, path: &str) -> Option<&Vec<Value>>;
}

// Will be implemented when we need collection operations
impl CollectionAccess for GameState {
    fn get_collection_mut(&mut self, _path: &str) -> Option<&mut Vec<Value>> {
        // TODO: Implement collection access for question.options, cards.deployed, etc.
        None
    }

    fn get_collection(&self, _path: &str) -> Option<&Vec<Value>> {
        // TODO: Implement collection access
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_paths() {
        let mut world = World::new();
        let mut timer = Timer::from_seconds(400.0, TimerMode::Once);
        timer.tick(std::time::Duration::from_secs(100));
        world.insert_resource(GameTimer {
            timer,
            paused: false,
        });

        let state = GameState::new();

        // Test get - remaining should be 300 (400 - 100)
        if let Some(Value::Int(remaining)) = state.get("timer.remaining", &world) {
            assert!(remaining >= 299 && remaining <= 301); // Allow small variance
        } else {
            panic!("Expected Int value");
        }

        // Test elapsed
        if let Some(Value::Int(elapsed)) = state.get("timer.elapsed", &world) {
            assert!(elapsed >= 99 && elapsed <= 101);
        } else {
            panic!("Expected Int value");
        }

        assert_eq!(
            state.get("timer.paused", &world),
            Some(Value::Bool(false))
        );

        // Test percent remaining (should be around 75%)
        if let Some(Value::Float(percent)) = state.get("timer.percent_remaining", &world) {
            assert!((percent - 75.0).abs() < 1.0);
        } else {
            panic!("Expected Float value");
        }
    }

    #[test]
    fn test_timer_modification() {
        let mut world = World::new();
        world.insert_resource(GameTimer {
            timer: Timer::from_seconds(300.0, TimerMode::Once),
            paused: false,
        });

        let mut state = GameState::new();

        // Test add
        assert!(state.add("timer.remaining", 60, &mut world));
        if let Some(Value::Int(remaining)) = state.get("timer.remaining", &world) {
            assert!(remaining >= 359 && remaining <= 361);
        }

        // Test subtract
        assert!(state.subtract("timer.remaining", 120, &mut world));
        if let Some(Value::Int(remaining)) = state.get("timer.remaining", &world) {
            assert!(remaining >= 239 && remaining <= 241);
        }

        // Test set flag
        assert!(state.set_flag("timer.paused", true, &mut world));
        assert_eq!(
            state.get("timer.paused", &world),
            Some(Value::Bool(true))
        );

        // Test toggle flag
        assert!(state.toggle_flag("timer.paused", &mut world));
        assert_eq!(
            state.get("timer.paused", &world),
            Some(Value::Bool(false))
        );
    }

    #[test]
    fn test_score_paths() {
        let mut world = World::new();
        world.insert_resource(Score {
            current: 10,
            passing_grade: 20,
            correct_answers: 0,
            total_answered: 0,
        });

        let mut state = GameState::new();

        assert_eq!(
            state.get("score.current", &world),
            Some(Value::Int(10))
        );

        assert!(state.add("score.current", 5, &mut world));
        assert_eq!(
            state.get("score.current", &world),
            Some(Value::Int(15))
        );
    }

    #[test]
    fn test_variables() {
        let mut state = GameState::new();

        state.set_variable("test", Value::Int(42));
        assert_eq!(
            state.get_variable("test"),
            Some(&Value::Int(42))
        );

        state.clear_variables();
        assert_eq!(state.get_variable("test"), None);
    }
}
