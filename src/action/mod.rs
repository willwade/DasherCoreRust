//! Dasher Actions System - Rust scaffold

use std::collections::HashMap;

/// Trait for actions that can be triggered in Dasher (e.g., backspace, space, accept)
pub trait Action {
    /// The unique name of the action
    fn name(&self) -> &str;
    /// The display label (e.g., "␣" for space, "⌫" for backspace)
    fn label(&self) -> &str;
    /// Execute the action, given mutable access to the model
    fn execute(&self, model: &mut crate::model::DasherModel);
}

/// Manages available actions and their registration
pub struct ActionManager {
    actions: HashMap<String, Box<dyn Action>>,
}

impl ActionManager {
    pub fn new() -> Self {
        Self { actions: HashMap::new() }
    }
    pub fn register_action(&mut self, action: Box<dyn Action>) {
        self.actions.insert(action.name().to_string(), action);
    }
    pub fn get_action(&self, name: &str) -> Option<&Box<dyn Action>> {
        self.actions.get(name)
    }
    pub fn all_actions(&self) -> Vec<&Box<dyn Action>> {
        self.actions.values().collect()
    }
}

// Example standard actions
pub struct BackspaceAction;
impl Action for BackspaceAction {
    fn name(&self) -> &str { "backspace" }
    fn label(&self) -> &str { "⌫" }
    fn execute(&self, model: &mut crate::model::DasherModel) {
        let text = model.output_text();
        if !text.is_empty() {
            let mut new_text = text.to_string();
            if let Some(idx) = new_text.char_indices().rev().next().map(|(i, _)| i) {
                new_text.truncate(idx);
            } else {
                new_text.clear();
            }
            model.set_output_text(&new_text);
        }
    }
}

pub struct SpaceAction;
impl Action for SpaceAction {
    fn name(&self) -> &str { "space" }
    fn label(&self) -> &str { "␣" }
    fn execute(&self, model: &mut crate::model::DasherModel) {
        model.append_to_output(' ');
    }
}

pub struct AcceptAction;
impl Action for AcceptAction {
    fn name(&self) -> &str { "accept" }
    fn label(&self) -> &str { "✔" }
    fn execute(&self, _model: &mut crate::model::DasherModel) {
        // Placeholder: mark input as accepted, trigger callback, etc.
    }
}

pub struct UndoAction;
impl Action for UndoAction {
    fn name(&self) -> &str { "undo" }
    fn label(&self) -> &str { "↶" }
    fn execute(&self, model: &mut crate::model::DasherModel) {
        let text = model.output_text();
        if !text.is_empty() {
            let mut new_text = text.to_string();
            if let Some(idx) = new_text.char_indices().rev().next().map(|(i, _)| i) {
                new_text.truncate(idx);
            } else {
                new_text.clear();
            }
            model.set_output_text(&new_text);
        }
    }
}

impl ActionManager {
    pub fn unregister_action(&mut self, name: &str) {
        self.actions.remove(name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::DasherModel;

    #[test]
    fn test_backspace_action() {
        let mut model = DasherModel::new();
        model.set_output_text("abc");
        let action = BackspaceAction;
        action.execute(&mut model);
        assert_eq!(model.output_text(), "ab");
    }
}
