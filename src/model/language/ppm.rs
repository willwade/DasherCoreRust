use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

/// PPM node for trie structure
#[derive(Debug)]
pub struct PPMNode {
    /// Symbol stored in this node
    symbol: Option<char>,
    /// Count of times this sequence has been seen
    count: usize,
    /// Child nodes
    children: HashMap<char, Rc<RefCell<PPMNode>>>,
    /// Parent node
    parent: Option<Rc<RefCell<PPMNode>>>,
    /// Total count of all children
    total_count: usize,
}

impl PPMNode {
    /// Create a new PPM node
    pub fn new(symbol: Option<char>, parent: Option<Rc<RefCell<PPMNode>>>) -> Self {
        Self {
            symbol,
            count: 0,
            children: HashMap::new(),
            parent,
            total_count: 0,
        }
    }

    /// Get child node for symbol, creating if it doesn't exist
    fn get_or_create_child(&mut self, symbol: char) -> Rc<RefCell<PPMNode>> {
        if let Some(child) = self.children.get(&symbol) {
            child.clone()
        } else {
            let child = Rc::new(RefCell::new(PPMNode::new(
                Some(symbol),
                Some(Rc::new(RefCell::new(self.clone()))),
            )));
            self.children.insert(symbol, child.clone());
            child
        }
    }

    /// Update counts for this node
    fn update_counts(&mut self, increment: bool) {
        if increment {
            self.count += 1;
            self.total_count += 1;
        } else if self.count > 0 {
            self.count -= 1;
            self.total_count -= 1;
        }
    }
}

/// PPM model order
#[derive(Debug, Clone, Copy)]
pub enum PPMOrder {
    /// No context (order -1)
    None,
    /// Order 0 (single symbol)
    Zero,
    /// Order 1 (pairs)
    One,
    /// Order 2 (triples)
    Two,
    /// Order 3 (quadruples)
    Three,
    /// Order 4 (quintuples)
    Four,
    /// Order 5 (sextuples)
    Five,
}

impl PPMOrder {
    /// Get numeric value of order
    pub fn value(self) -> i32 {
        match self {
            PPMOrder::None => -1,
            PPMOrder::Zero => 0,
            PPMOrder::One => 1,
            PPMOrder::Two => 2,
            PPMOrder::Three => 3,
            PPMOrder::Four => 4,
            PPMOrder::Five => 5,
        }
    }
}

/// PPM language model
#[derive(Debug)]
pub struct PPMLanguageModel {
    /// Root node of trie
    root: Rc<RefCell<PPMNode>>,
    /// Maximum order of model
    max_order: PPMOrder,
    /// Exclusion flag
    exclusion: bool,
    /// Update exclusion flag
    update_exclusion: bool,
}

impl PPMLanguageModel {
    /// Create a new PPM language model
    pub fn new(max_order: PPMOrder) -> Self {
        Self {
            root: Rc::new(RefCell::new(PPMNode::new(None, None))),
            max_order,
            exclusion: true,
            update_exclusion: true,
        }
    }

    /// Set exclusion flags
    pub fn set_exclusion(&mut self, exclusion: bool, update_exclusion: bool) {
        self.exclusion = exclusion;
        self.update_exclusion = update_exclusion;
    }

    /// Enter symbol into model
    pub fn enter_symbol(&mut self, symbol: char) {
        let mut node = self.root.clone();
        let mut context_length = 0;

        while context_length <= self.max_order.value() {
            node.borrow_mut().update_counts(true);
            let child = node.borrow_mut().get_or_create_child(symbol);
            node = child;
            context_length += 1;
        }
    }

    /// Get probability distribution for next symbol
    pub fn get_probs(&self, context: &str) -> HashMap<char, f64> {
        let mut probs = HashMap::new();
        let mut seen = HashMap::new();
        let mut alpha = 0.0;
        let mut norm = 0.0;

        // Start with maximum order and back off
        for order in (PPMOrder::None.value()..=self.max_order.value()).rev() {
            let context_slice = if order < 0 {
                ""
            } else {
                &context[context.len().saturating_sub(order as usize)..]
            };

            let mut node = self.root.clone();
            for c in context_slice.chars() {
                let next_node = if let Some(child) = node.borrow().children.get(&c) {
                    Some(child.clone())
                } else {
                    None
                };
                if let Some(child) = next_node {
                    node = child;
                } else {
                    break;
                }
            }

            // Calculate probabilities for this order
            let node_ref = node.borrow();
            let total = node_ref.total_count as f64;

            if total > 0.0 {
                for (symbol, child) in &node_ref.children {
                    if !seen.contains_key(symbol) {
                        let child_ref = child.borrow();
                        let count = child_ref.count as f64;
                        let prob = (count / total) * alpha;
                        probs.insert(*symbol, prob);
                        seen.insert(*symbol, true);
                        norm += prob;
                    }
                }
            }

            // Update alpha for next order
            alpha = if order == PPMOrder::None.value() {
                1.0
            } else {
                alpha * (1.0 - norm)
            };
        }

        // Normalize probabilities
        if norm > 0.0 {
            for prob in probs.values_mut() {
                *prob /= norm;
            }
        }

        probs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ppm_basic() {
        let mut model = PPMLanguageModel::new(PPMOrder::Two);

        // Train on simple sequence
        for c in "hello".chars() {
            model.enter_symbol(c);
        }

        // Test predictions
        let probs = model.get_probs("hel");
        assert!(probs.contains_key(&'l'));
        assert!(probs.get(&'l').unwrap() > &0.0);

        let probs = model.get_probs("he");
        assert!(probs.contains_key(&'l'));
        assert!(probs.get(&'l').unwrap() > &0.0);
    }

    #[test]
    fn test_ppm_orders() {
        let mut model = PPMLanguageModel::new(PPMOrder::One);

        // Train on repeated sequence
        for _ in 0..3 {
            for c in "ab".chars() {
                model.enter_symbol(c);
            }
        }

        // Test order-1 predictions
        let probs = model.get_probs("a");
        assert!(probs.contains_key(&'b'));
        assert!(probs.get(&'b').unwrap() > &0.5);

        // Test order-0 predictions
        let probs = model.get_probs("");
        assert!(probs.contains_key(&'a'));
        assert!(probs.contains_key(&'b'));
        assert!((probs.get(&'a').unwrap() - probs.get(&'b').unwrap()).abs() < 0.1);
    }
}
