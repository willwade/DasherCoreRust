//! # Model Module
//!
//! This module contains the core Dasher model implementation, including
//! the arithmetic coding algorithm and node tree management.

pub mod node;
mod language;
pub mod word_generator;
pub mod word_prediction;
pub use word_generator::{BaseWordGenerator, PredictiveWordGenerator};
pub use language::{PPMLanguageModel, PPMOrder, PPMNode, LanguageModel, CombinedLanguageModel};
pub use word_prediction::{WordPredictionManager, create_default_manager};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::{Rc, Weak};
use std::path::Path;

use node::{DasherNode, NodeFlags};
use crate::view::{DasherScreen, Color};
use crate::alphabet::Alphabet;
use crate::Result;

/// Event type for node creation
pub type NodeCreationEvent = Box<dyn Fn(&Rc<RefCell<DasherNode>>)>;

/// The main Dasher model that implements arithmetic coding for Dasher.
///
/// It contains a tree of DasherNodes and the current viewpoint, and evolves
/// the tree by expanding leaves and deleting ancestors/parents.
pub struct DasherModel {
    // ... existing fields ...

    /// Actions manager
    pub action_manager: crate::action::ActionManager,

    /// Root node of the tree
    /// Word prediction manager
    word_prediction: Option<WordPredictionManager>,

    /// Root node of the tree
    root: Option<Rc<RefCell<DasherNode>>>,

    /// Old roots that have been replaced
    old_roots: VecDeque<Rc<RefCell<DasherNode>>>,

    /// Minimum coordinate of the root node
    root_min: i64,

    /// Maximum coordinate of the root node
    root_max: i64,

    /// Minimum allowed value for root_min
    root_min_min: i64,

    /// Maximum allowed value for root_max
    root_max_max: i64,

    /// Display offset
    display_offset: i64,

    /// Last node that was output
    last_output: Option<Weak<RefCell<DasherNode>>>,

    /// Queue of steps to execute
    goto_queue: VecDeque<(i64, i64)>,

    /// Whether characters entered by alphabet manager are expected to require conversion
    require_conversion: bool,

    /// Total information entered so far in this model (in nats)
    total_nats: f64,

    /// Event handlers for node creation
    node_creation_handlers: Vec<NodeCreationEvent>,

    /// The alphabet used by this model
    alphabet: Option<Alphabet>,

    /// The language model used by this model
    language_model: Option<Box<dyn LanguageModel>>,

    /// The current output text
    output_text: String,
}

impl DasherModel {
    /// Stub for input coordinate application
    pub fn apply_input_coordinates(&mut self, _coords: (i64, i64)) {
        // Stub: No-op for now
    }
    /// Y origin constant for coordinate calculations
    pub const ORIGIN_Y: i64 = 0;
    /// Placeholder for word predictions
    pub fn get_word_predictions(&mut self) -> Vec<String> {
        if let Some(manager) = &mut self.word_prediction {
            manager.get_predictions(&self.output_text)
        } else {
            Vec::new()
        }
    }
    /// Maximum Y coordinate for the model (placeholder value)
    pub const MAX_Y: i64 = 1 << 20;
    /// Normalization constant for probability calculations
    pub const NORMALIZATION: u32 = 1 << 16;
    /// Create a new Dasher model with default settings
    pub fn new() -> Self {
        Self::with_language_model(Box::new(CombinedLanguageModel::new(PPMOrder::Three)))
    }

    /// Create a new Dasher model with custom language model
    pub fn with_language_model(language_model: Box<dyn LanguageModel>) -> Self {
    
        use crate::model::word_prediction::WordPredictionManager;
        use crate::action::{ActionManager, BackspaceAction, SpaceAction, AcceptAction};
        let mut word_prediction = WordPredictionManager::new(10, 32);
    
        let mut action_manager = ActionManager::new();
        action_manager.register_action(Box::new(BackspaceAction));
        action_manager.register_action(Box::new(SpaceAction));
        action_manager.register_action(Box::new(AcceptAction));
        Self {
            action_manager,
            root: None,
            language_model: Some(language_model),
            word_prediction: Some(word_prediction),
            old_roots: VecDeque::new(),
            root_min: 0,
            root_max: 0,
            root_min_min: i64::MIN / (Self::NORMALIZATION as i64) / 2,
            root_max_max: i64::MAX / (Self::NORMALIZATION as i64) / 2,
            display_offset: 0,
            last_output: None,
            goto_queue: VecDeque::new(),
            require_conversion: false,
            total_nats: 0.0,
            node_creation_handlers: Vec::new(),
            alphabet: Some(Alphabet::english()),
            output_text: String::new(),
        }
    }

    /// Load dictionary for language model
    pub fn load_dictionary<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        if let Some(model) = &mut self.language_model {
            if let Some(combined) = model.as_any().downcast_mut::<CombinedLanguageModel>() {
                combined.dictionary_mut().load(path)?;
            }
        }
        Ok(())
    }

    /// Get current probability distribution
    pub fn get_probabilities(&self) -> Option<Vec<(char, f64)>> {
        self.language_model.as_ref().map(|model| {
            let context = self.output_text.as_str();
            model.get_probs(context).into_iter().collect()
        })
    }

    /// Update language model with new symbol
    pub fn update_language_model(&mut self, symbol: char) {
        if let Some(model) = &mut self.language_model {
            model.enter_symbol(symbol);
        }
    }

    /// Set the alphabet for this model
    pub fn set_alphabet(&mut self, alphabet: Alphabet) {
        self.alphabet = Some(alphabet);
    }

    /// Initialize the model
    pub fn initialize(&mut self) -> Result<()> {
        // Create an English alphabet
        let alphabet = crate::alphabet::Alphabet::english();

        // Set the alphabet
        self.set_alphabet(alphabet);

        // Create a root node
        let root = Rc::new(RefCell::new(DasherNode::new(0, Some("Root".to_string()))));

        // Set the root node
        self.root = Some(root.clone());

        // Expand the root node to create its children
        self.expand_node(&root);

        Ok(())
    }

    /// Get a reference to the alphabet
    pub fn alphabet(&self) -> Option<&Alphabet> {
        self.alphabet.as_ref()
    }

    /// Set the language model for this model
    pub fn set_language_model(&mut self, language_model: Box<dyn LanguageModel>) {
        self.language_model = Some(language_model);
    }

    /// Get a reference to the language model
    pub fn language_model(&self) -> Option<&dyn LanguageModel> {
        if let Some(lm) = &self.language_model {
            Some(lm.as_ref())
        } else {
            None
        }
    }

    /// Get the current output text
    pub fn output_text(&self) -> &str {
        &self.output_text
    }

    /// Append a character to the output text
    pub fn append_to_output(&mut self, c: char) {
        self.output_text.push(c);
        self.update_language_model(c);
    }

    /// Set the output text
    pub fn set_output_text(&mut self, text: &str) {
        self.output_text = text.to_string();
    }

    /// Set the root node
    pub fn set_node(&mut self, new_root: Rc<RefCell<DasherNode>>) {
        // Clear any scheduled steps
        self.abort_offset();
        self.clear_root_queue();

        // Set the new root
        self.root = Some(new_root.clone());

        // Create children of the root
        self.expand_node(&new_root);

        // Set the root coordinates
        new_root.borrow_mut().set_flag(NodeFlags::SEEN, true);
        self.last_output = Some(Rc::downgrade(&new_root));

        // Calculate the root size based on the most probable child
        let most_probable = new_root.borrow().most_probable_child() as f64;
        let fraction = 1.0 - (1.0 - most_probable / (Self::NORMALIZATION as f64)) / 2.0;

        let width = (Self::MAX_Y as f64 / (2.0 * fraction)) as i64;

        self.root_min = Self::MAX_Y / 2 - width / 2;
        self.root_max = Self::MAX_Y / 2 + width / 2;
    }

    /// Get the current offset in the text buffer
    pub fn get_offset(&self) -> i32 {
        if let Some(last_output) = &self.last_output {
            if let Some(node) = last_output.upgrade() {
                return node.borrow().offset() + 1;
            }
        }

        if let Some(root) = &self.root {
            return root.borrow().offset() + 1;
        }

        0
    }

    /// Get the node that was under the crosshair in the last frame
    pub fn get_node_under_crosshair(&self) -> Option<Rc<RefCell<DasherNode>>> {
        if let Some(last_output) = &self.last_output {
            last_output.upgrade()
        } else {
            None
        }
    }

    /// Expand a node by creating its children
    pub fn expand_node(&mut self, node: &Rc<RefCell<DasherNode>>) {
        // Insert action nodes as children (e.g., at the end)
        // We'll use a special label and NodeFlags for action nodes
        for action in self.action_manager.all_actions() {
            let mut action_node = DasherNode::new(0, Some(action.label().to_string()));
            action_node.set_flag(NodeFlags::CONTROL, true); // Mark as control/action node
            action_node.set_flag(NodeFlags::ALL_CHILDREN, true); // No further expansion
            action_node.set_parent(Rc::downgrade(node));
            node.borrow_mut().add_child(Rc::new(RefCell::new(action_node)));
        }

        // Get word predictions if this is a word boundary
        let _predictions = if node.borrow().is_word_boundary() {
            self.get_word_predictions()
        } else {
            Vec::new()
        };
        let has_all_children = {
            let node_ref = node.borrow();
            node_ref.get_flag(NodeFlags::ALL_CHILDREN)
        };

        if has_all_children {
            return;
        }

        // Delete existing children
        node.borrow_mut().delete_children();

        // Get the alphabet
        if let Some(alphabet) = &self.alphabet {
            // Get the current offset
            let offset = node.borrow().offset();

            // Get the probabilities for each symbol
            let probs = if let Some(lm) = &mut self.language_model {
                // Use the language model to get probabilities
                let context = lm.create_empty_context();
                let probs = lm.get_probs(&context);
                // lm.release_context(context);
                probs
            } else {
                // Use uniform probabilities
                let num_symbols = alphabet.size();
                let mut map = std::collections::HashMap::new();
                for symbol in alphabet.symbols().iter() {
                    map.insert(symbol.character, 1.0 / num_symbols as f64);
                }
                map
            };

            // Create a child for each symbol in the alphabet
            let mut lower_bound = 0;

            for symbol in alphabet.symbols().iter() {
                let prob = probs.get(&symbol.character).copied().unwrap_or(0.0) as u32;
                if prob > 0 {
                    let upper_bound = lower_bound + prob;

                    // Create a new node for this symbol
                    let child = Rc::new(RefCell::new(DasherNode::new(
                        offset + 1,
                        Some(symbol.display_text.clone()),
                    )));

                    // Set the bounds
                    child.borrow_mut().set_bounds(lower_bound, upper_bound);

                    // Set the symbol
                    child.borrow_mut().set_symbol(symbol.character);

                    // Set the colors
                    child.borrow_mut().set_colors((symbol.foreground_color.r, symbol.foreground_color.g, symbol.foreground_color.b), (symbol.background_color.r, symbol.background_color.g, symbol.background_color.b));

                    // Set the parent
                    child.borrow_mut().set_parent(Rc::downgrade(node));

                    // Add the child to the parent
                    node.borrow_mut().add_child(child);

                    // Update the lower bound for the next symbol
                    lower_bound = upper_bound;
                }
            }
        }

        // Set the ALL_CHILDREN flag
        node.borrow_mut().set_flag(NodeFlags::ALL_CHILDREN, true);

        // Notify event handlers
        for handler in &self.node_creation_handlers {
            handler(node);
        }
    }

    /// Make a child of the root into a new root
    pub fn make_root(&mut self, new_root: &Rc<RefCell<DasherNode>>) {
        // Get the current root
        if let Some(root) = &self.root {
            // Delete nephews of the new root
            root.borrow_mut().delete_nephews(new_root);
            root.borrow_mut().set_flag(NodeFlags::COMMITTED, true);

            // Add the old root to the queue
            self.old_roots.push_back(root.clone());

            // Clean up old roots if necessary
            while self.old_roots.len() > 10 &&
                  (!self.require_conversion ||
                   self.old_roots[0].borrow().get_flag(NodeFlags::CONVERTED)) {
                if let Some(old_root) = self.old_roots.pop_front() {
                    if let Some(next_root) = self.old_roots.front() {
                        old_root.borrow_mut().orphan_child(next_root);
                    }
                }
            }

            // Set the new root
            self.root = Some(new_root.clone());

            // Update the root coordinates
            let range = self.root_max - self.root_min;
            let new_root_ref = new_root.borrow();
            self.root_max = self.root_min + (range * new_root_ref.upper_bound() as i64) / (Self::NORMALIZATION as i64);
            self.root_min = self.root_min + (range * new_root_ref.lower_bound() as i64) / (Self::NORMALIZATION as i64);

            // Update any scheduled steps
            for step in &mut self.goto_queue {
                let r = step.1 - step.0;
                step.1 = step.0 + (r * new_root_ref.upper_bound() as i64) / (Self::NORMALIZATION as i64);
                step.0 = step.0 + (r * new_root_ref.lower_bound() as i64) / (Self::NORMALIZATION as i64);
            }
        }
    }

    /// Reparent the root to its parent
    pub fn reparent_root(&mut self) -> bool {
        // Get the parent of the current root
        let parent_node = {
            if let Some(root) = &self.root {
                if let Some(parent) = root.borrow().parent() {
                    parent.upgrade()
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(parent_node) = parent_node {
            // Calculate the new coordinates
            let (lower, upper, range, root_width) = {
                if let Some(root) = &self.root {
                    let root_ref = root.borrow();
                    let lower = root_ref.lower_bound() as i64;
                    let upper = root_ref.upper_bound() as i64;
                    let range = upper - lower;
                    let root_width = self.root_max - self.root_min;
                    (lower, upper, range, root_width)
                } else {
                    return false;
                }
            };

            // Check if the new root would be too big
            if ((Self::NORMALIZATION as i64 - upper) as f64 / range as f64) >
               ((self.root_max_max - self.root_max) as f64 / root_width as f64) ||
               ((lower as f64) / range as f64) >
               ((self.root_min - self.root_min_min) as f64 / root_width as f64) {
                // Cache the unusable root node
                self.old_roots.push_back(parent_node);
                return false;
            }

            // Update the root
            self.root = Some(parent_node);

            // Update the coordinates
            self.root_max = self.root_max + ((Self::NORMALIZATION as i64 - upper) * root_width) / range;
            self.root_min = self.root_min - (lower * root_width) / range;

            // Update any scheduled steps
            for step in &mut self.goto_queue {
                let step_width = step.1 - step.0;
                step.1 += ((Self::NORMALIZATION as i64 - upper) * step_width) / range;
                step.0 -= (lower * step_width) / range;
            }

            return true;
        }

        false
    }

    /// Process the next scheduled step
    pub fn next_scheduled_step(&mut self) -> bool {
        if self.goto_queue.is_empty() {
            return false;
        }

        if let Some((new_root_min, new_root_max)) = self.goto_queue.pop_front() {
            // Update the total information
            self.total_nats += ((new_root_max - new_root_min) as f64 / (self.root_max - self.root_min) as f64).ln();

            // Update the display offset
            self.display_offset = (self.display_offset * 90) / 100;

            // Check if we need to make a child the new root
            if new_root_min <= Self::ORIGIN_Y && new_root_max > Self::ORIGIN_Y {
                // Find a child that covers the crosshair
                let child_to_make_root = {
                    let mut result = None;

                    if let Some(root) = &self.root {
                        let root_ref = root.borrow();
                        let width = self.root_max - self.root_min;
                        let is_game_path = root_ref.get_flag(NodeFlags::GAME);

                        // Find the child covering the crosshair
                        for child in root_ref.children() {
                            let child_ref = child.borrow();
                            let child_min = self.root_min + ((child_ref.lower_bound() as i64 * width) / (Self::NORMALIZATION as i64));
                            let child_max = self.root_min + ((child_ref.upper_bound() as i64 * width) / (Self::NORMALIZATION as i64));

                            if child_min <= Self::ORIGIN_Y && child_max > Self::ORIGIN_Y {
                                // Check if the child is on the game path
                                if is_game_path && !child_ref.get_flag(NodeFlags::GAME) {
                                    return false;
                                }

                                result = Some(child.clone());
                                break;
                            }
                        }
                    }

                    result
                };

                if let Some(child) = child_to_make_root {
                    // Update the target coordinates
                    self.goto_queue.push_back((new_root_min, new_root_max));

                    // Make the child the new root
                    self.root = Some(child);

                    // Get the updated coordinates
                    if let Some((updated_min, updated_max)) = self.goto_queue.pop_back() {
                        // Continue with the updated coordinates
                        if updated_min <= Self::ORIGIN_Y && updated_max > Self::ORIGIN_Y &&
                           (updated_max - updated_min) > Self::MAX_Y / 4 {
                            self.root_min = updated_min;
                            self.root_max = updated_max;
                            return true;
                        }
                    }

                    return false;
                }
            }

            // Only allow the update if it won't make the root too small
            if (new_root_max - new_root_min) > Self::MAX_Y / 4 {
                self.root_min = new_root_min;
                self.root_max = new_root_max;
                return true;
            }
        }

        false
    }

    /// Schedule a single step
    pub fn schedule_one_step(&mut self, y1: i64, y2: i64, n_steps: i32, lim_x: i32, exact: bool) {
        self.goto_queue.clear();

        // Rename for readability
        let r1 = self.root_min;
        let r2 = self.root_max;

        // Calculate the bounds of the root node when the target range y1-y2
        // fills the viewport
        let target_range = y2 - y1;

        let r1_new = Self::MAX_Y * (r1 - y1) / target_range;
        let r2_new = Self::MAX_Y * (r2 - y1) / target_range;

        let m1 = r1_new - r1;
        let m2 = r2_new - r2;

        // Apply speed limit if necessary
        let (m1_final, m2_final) = if target_range < 2 * lim_x as i64 {
            // Exact dynamics
            if exact {
                let frac = if target_range == Self::MAX_Y {
                    1.0 / n_steps as f64
                } else {
                    let tr = target_range as f64;
                    // Expansion factor for one step
                    let e_fac = (Self::MAX_Y as f64 / tr).powf(1.0 / n_steps as f64);
                    // Fraction of way along linear interpolation
                    (e_fac - 1.0) / (Self::MAX_Y as f64 / tr - 1.0)
                };

                ((m1 as f64 * frac) as i64, (m2 as f64 * frac) as i64)
            } else {
                // Approximate dynamics
                let ap_sq = (target_range as f64).sqrt() as i64;
                let denom = 64 * (n_steps - 1) as i64 + ap_sq;

                ((m1 * ap_sq) / denom, (m2 * ap_sq) / denom)
            }
        } else {
            (m1, m2)
        };

        // Add the step to the queue
        self.goto_queue.push_back((r1 + m1_final, r2 + m2_final));
    }

    /// Schedule a zoom operation
    pub fn schedule_zoom(&mut self, y1: i64, y2: i64, mut n_steps: i32) {
        self.goto_queue.clear();

        // Rename for readability
        let r1 = self.root_min;
        let r2 = self.root_max;

        // Calculate the target coordinates
        let r1_new = Self::MAX_Y * (r1 - y1) / (y2 - y1);
        let r2_new = Self::MAX_Y * (r2 - y1) / (y2 - y1);

        // Calculate the maximum number of steps
        let max = (n_steps * (n_steps + 1)) / 2;

        // Calculate the height multiplier
        let oh = r2 - r1;
        let nh = r2_new - r1_new;
        let log_height_mul = if nh == oh { 0.0 } else { (nh as f64 / oh as f64).ln() };

        // Add steps to the queue
        let mut s = n_steps;
        while n_steps > 1 {
            let d_frac = if nh == oh {
                s as f64 / max as f64
            } else {
                // Interpolate expansion logarithmically
                let h = oh as f64 * ((log_height_mul * s as f64) / max as f64).exp();
                // Treat as a fraction of the way between oh and nh
                (h - oh as f64) / (nh as f64 - oh as f64)
            };

            // Use the fraction to interpolate from R to r
            let new_min = r1 + (d_frac * (r1_new - r1) as f64) as i64;
            let new_max = r2 + (d_frac * (r2_new - r2) as f64) as i64;

            self.goto_queue.push_back((new_min, new_max));

            s += n_steps - 1;
            n_steps -= 1;
        }

        // Add the final point
        self.goto_queue.push_back((r1_new, r2_new));
    }

    /// Clear all scheduled steps
    pub fn clear_scheduled_steps(&mut self) {
        self.goto_queue.clear();
    }

    /// Abort any offset operation
    fn abort_offset(&mut self) {
        // TODO: Implement if needed
    }

    /// Clear the root queue
    fn clear_root_queue(&mut self) {
        // TODO: Implement if needed
    }

    /// Apply an offset to the model
    pub fn offset(&mut self, offset: i64) {
        self.root_min += offset;
        self.root_max += offset;
        self.display_offset -= offset;
    }

    /// Output to a new node
    pub fn output_to(&mut self, new_node: &Rc<RefCell<DasherNode>>) {
        // Check if the node has been seen
        if !new_node.borrow().get_flag(NodeFlags::SEEN) {
            // Recurse to parent first
            if let Some(parent) = new_node.borrow().parent() {
                if let Some(parent_node) = parent.upgrade() {
                    self.output_to(&parent_node);
                }
            }

            // Set the last output
            self.last_output = Some(Rc::downgrade(new_node));

            // Get the symbol from the node
            let symbol = new_node.borrow().symbol();

            // If the node has a symbol, append it to the output text
            if let Some(c) = symbol {
                self.output_text.push(c);
            }

            // Perform the node's action
            new_node.borrow_mut().do_action();

            // Mark the node as seen
            new_node.borrow_mut().set_flag(NodeFlags::SEEN, true);
        }
    }

    /// Register a handler for node creation events
    pub fn on_node_children_created<F>(&mut self, handler: F)
    where
        F: Fn(&Rc<RefCell<DasherNode>>) + 'static,
    {
        self.node_creation_handlers.push(Box::new(handler));
    }

    /// Limit the root node to a maximum width
    pub fn limit_root(&mut self, max_width: i32) {
        let current_width = self.root_max - self.root_min;

        if current_width > max_width as i64 {
            let center = (self.root_min + self.root_max) / 2;
            self.root_min = center - (max_width as i64) / 2;
            self.root_max = center + (max_width as i64) / 2;
        }
    }

    /// Render the model to a view
    pub fn render_to_view<S: DasherScreen + ?Sized>(&mut self, view: &mut S) -> Result<()> {
        // Get the screen dimensions
        let width = view.get_width();
        let height = view.get_height();

        // Debug output
        eprintln!("Rendering to view: width={}, height={}", width, height);

        // Draw a background
        view.draw_rectangle(0, 0, width, height,
                           crate::view::color_palette::WHITE,
                           crate::view::color_palette::BLACK,
                           1);

        // Always render the Dasher interface for now
        eprintln!("Rendering Dasher interface");
        self.render_dasher_interface(view, width, height);

        // Draw a crosshair
        let cx = width / 2;
        let cy = height / 2;
        view.draw_line(cx - 10, cy, cx + 10, cy, crate::view::color_palette::RED, 2);
        view.draw_line(cx, cy - 10, cx, cy + 10, crate::view::color_palette::RED, 2);

        // Draw a circle at the crosshair
        view.draw_circle(cx, cy, 5,
                        crate::view::color_palette::RED,
                        crate::view::color_palette::BLACK,
                        1);

        // Signal that the frame is complete
        view.display();

        Ok(())
    }

    /// Render a node and its children
    fn render_node<S: DasherScreen + ?Sized>(&self, view: &mut S, node: &Rc<RefCell<DasherNode>>, x1: i32, y1: i32, x2: i32, y2: i32) {
        let node_ref = node.borrow();

        // Draw the node
        let bg_color = Color::from_tuple((node_ref.background_color().0, node_ref.background_color().1, node_ref.background_color().2, 200));
        let fg_color = Color::from_tuple((node_ref.foreground_color().0, node_ref.foreground_color().1, node_ref.foreground_color().2, 255));

        // Draw the node background
        view.draw_rectangle(x1, y1, x2, y2, bg_color, crate::view::color_palette::BLACK, 1);

        // Draw the node label
        if let Some(label) = node_ref.label() {
            let label_obj = view.make_label(label, 0);
            let font_size = 24;
            let (text_width, text_height) = view.text_size(&*label_obj, font_size);

            // Center the text in the node
            let text_x = x1 + (x2 - x1 - text_width) / 2;
            let text_y = y1 + (y2 - y1 - text_height) / 2;

            view.draw_string(&*label_obj, text_x, text_y, font_size, fg_color);
        }

        // Draw the children
        let children = node_ref.children();
        if !children.is_empty() {
            // Calculate the total range
            let total_range = node_ref.range() as f32;

            // In the original Dasher, the nodes are arranged vertically
            // We'll divide the vertical space among the children
            let height_per_child = (y2 - y1) / children.len() as i32;

            // Draw each child
            for (i, child) in children.iter().enumerate() {
                let child_ref = child.borrow();

                // Calculate the child's position based on its probability range
                let child_lower = child_ref.lower_bound() as f32 / total_range;
                let child_upper = child_ref.upper_bound() as f32 / total_range;

                // Map the probability range to screen coordinates
                let child_x1 = x1 + ((x2 - x1) as f32 * child_lower) as i32;
                let child_x2 = x1 + ((x2 - x1) as f32 * child_upper) as i32;

                // Calculate the vertical position
                let child_y1 = y1 + (i as i32 * height_per_child);
                let child_y2 = child_y1 + height_per_child;

                // Recursively render the child
                self.render_node(view, child, child_x1, child_y1, child_x2, child_y2);
            }
        }
    }

    /// Render the Dasher interface with a more realistic appearance
    fn render_dasher_interface<S: DasherScreen + ?Sized>(&self, view: &mut S, width: i32, height: i32) {
        eprintln!("IMPORTANT: render_dasher_interface called with width={}, height={}", width, height);

        // Define colors for different node types - using the original Dasher color scheme
        let colors = [
            Color::from_tuple((180, 225, 180, 255)), // Light green
            Color::from_tuple((160, 200, 240, 255)), // Light blue
            Color::from_tuple((250, 200, 160, 255)), // Light orange
            Color::from_tuple((230, 175, 175, 255)), // Light red
            Color::from_tuple((190, 175, 250, 255)), // Light purple
            Color::from_tuple((225, 225, 175, 255)), // Light yellow
        ];

        // In the traditional Dasher, the interface is divided into horizontal slices
        // Each slice represents a character or group of characters
        // The letters are arranged vertically along the right side

        // Number of horizontal slices
        let num_slices = 26; // One for each letter of the alphabet

        // Height of each slice
        let slice_height = height / num_slices;

        eprintln!("Drawing {} horizontal slices, each with height {}", num_slices, slice_height);

        // Draw the horizontal slices
        for i in 0..num_slices {
            let y1 = i * slice_height;
            let y2 = (i + 1) * slice_height;

            // Use a different color for each slice
            let color_index = (i as usize) % colors.len();
            let color = colors[color_index];

            eprintln!("Drawing slice {} at y1={}, y2={} with color {:?}", i, y1, y2, color);

            // Draw the slice
            view.draw_rectangle(0, y1, width, y2, color, crate::view::color_palette::BLACK, 1);

            // Draw a letter in the slice
            let letter = ('a' as u8 + i as u8) as char;
            let label = view.make_label(&letter.to_string(), 0);
            let font_size = 24;
            let (text_width, text_height) = view.text_size(&*label, font_size);

            // Right-align the text in the slice
            let text_x = width - text_width - 10; // 10px padding from right edge
            let text_y = y1 + (slice_height - text_height) / 2;

            eprintln!("Drawing letter '{}' at x={}, y={}", letter, text_x, text_y);

            view.draw_string(&*label, text_x, text_y, font_size, crate::view::color_palette::BLACK);
        }

        // Draw horizontal lines between slices
        for i in 0..=num_slices {
            let y = i * slice_height;
            view.draw_line(0, y, width, y, crate::view::color_palette::BLACK, 1);
        }

        eprintln!("render_dasher_interface completed");
    }

    /// Draw nested boxes to simulate the Dasher zooming effect
    fn draw_nested_boxes<S: DasherScreen + ?Sized>(
        &self,
        view: &mut S,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        depth: i32,
        color: (u8, u8, u8, u8)
    ) {
        if depth <= 0 {
            return;
        }

        // Calculate the size of the inner box
        let inner_width = (x2 - x1) / 3;
        let inner_height = (y2 - y1) / 3;

        // Calculate the position of the inner box
        let inner_x1 = x1 + inner_width;
        let inner_y1 = y1 + inner_height;
        let inner_x2 = x2 - inner_width;
        let inner_y2 = y2 - inner_height;

        // Draw the inner box
        let darker_color = (
            (color.0 as f32 * 0.8) as u8,
            (color.1 as f32 * 0.8) as u8,
            (color.2 as f32 * 0.8) as u8,
            color.3
        );

        view.draw_rectangle(
            inner_x1,
            inner_y1,
            inner_x2,
            inner_y2,
            Color::from_tuple(darker_color),
            crate::view::color_palette::BLACK,
            1
        );

        // Recursively draw nested boxes
        self.draw_nested_boxes(
            view,
            inner_x1,
            inner_y1,
            inner_x2,
            inner_y2,
            depth - 1,
            darker_color
        );
    }
}
