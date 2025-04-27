use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

/// PPM node for trie structure
#[derive(Debug, Clone)]
pub struct PPMNode {
    /// Symbol stored in this node
    symbol: Option<char>,
    /// Count of times this sequence has been seen
    count: usize,
    /// Child nodes
    pub children: HashMap<char, Rc<RefCell<PPMNode>>>, 
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
    #[allow(dead_code)]
    fn get_or_create_child(&mut self, symbol: char) -> Rc<RefCell<PPMNode>> {
        if let Some(child) = self.children.get(&symbol) {
            child.clone()
        } else {
            // To avoid infinite recursion, parent should be a Weak ref or None here.
            let child = Rc::new(RefCell::new(PPMNode::new(
                Some(symbol),
                None,
            )));
            self.children.insert(symbol, child.clone());
            child
        }
    }

    /// Update counts for this node
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    Zero,
    /// Order 1 (pairs)
    #[allow(dead_code)]
    One,
    /// Order 2 (triples)
    #[allow(dead_code)]
    Two,
    /// Order 3 (quadruples)
    Three,
    /// Order 4 (quintuples)
    #[allow(dead_code)]
    Four,
    /// Order 5 (sextuples)
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    exclusion: bool,
    /// Update exclusion flag
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn set_exclusion(&mut self, exclusion: bool, update_exclusion: bool) {
        self.exclusion = exclusion;
        self.update_exclusion = update_exclusion;
    }

    /// Enter symbol into model
    pub fn enter_symbol(&mut self, context: &str, symbol: char) {
        let context_len = context.len();
        for order in (PPMOrder::None.value()..=self.max_order.value()).rev() {
            let order_usize = order as usize;
            let ctx_slice = if order == 0 || context_len < order_usize {
                ""
            } else {
                &context[context_len - order_usize..]
            };
            // println!("[PPM][train] order: {}, ctx_slice: '{}', symbol: '{}'", order, ctx_slice, symbol);
            let mut node = self.root.clone();
            if !ctx_slice.is_empty() {
                for c in ctx_slice.chars() {
                    let next = {
                        let mut node_mut = node.borrow_mut();
                        node_mut.children.entry(c).or_insert_with(|| Rc::new(RefCell::new(PPMNode::new(Some(c), None)))).clone()
                    };
                    node = next;
                }
            }
            // Insert symbol at this context
            node.borrow_mut().children.entry(symbol).or_insert_with(|| Rc::new(RefCell::new(PPMNode::new(Some(symbol), None))));
        }
    }

    /// Get probability distribution for next symbol
    pub fn get_probs(&self, context: &str) -> HashMap<char, f64> {
        let mut probs = HashMap::new();
        let mut seen = HashMap::new();
        let alpha = 1.0; // Start with all probability mass
        let mut context_orders: Vec<&str> = Vec::new();
        // Build context slices for each order (from max down to 0), using the LAST N symbols (matching enter_symbol)
        for order in (PPMOrder::None.value()..=self.max_order.value()).rev() {
            let context_slice = if order < 0 {
                ""
            } else {
                let len = context.len();
                if order as usize > len {
                    &context[..]
                } else {
                    &context[len - order as usize..]
                }
            };
            context_orders.push(context_slice);
        }
        context_orders.reverse();

        let mut found_any_context = false;
        for (_order, context_slice) in context_orders.iter().enumerate().rev() {
            let mut node = self.root.clone();
            let mut found = true;
            // println!("[PPM][predict] order: {}, context_slice: '{}'", order, context_slice);
            if !context_slice.is_empty() {
                for c in context_slice.chars() {
                    let next = {
                        let node_ref = node.borrow();
                        node_ref.children.get(&c).cloned()
                    };
                    if let Some(child) = next {
                        node = child;
                    } else {
                        found = false;
                        break;
                    }
                }
            }
            // println!("[PPM][predict]  -> found: {}", found);
            if !found {
                continue;
            }
            found_any_context = true;
            let node_ref = node.borrow();
            let total = node_ref.children.len() as f64;
            if total == 0.0 {
                // Context exists but has no children: classic PPM-C says assign nothing, do not fallback
                break;
            }
            let mass = alpha * 1.0;
            let mut assigned = 0;
            for (symbol, _child) in &node_ref.children {
                if !seen.contains_key(symbol) {
                    probs.insert(*symbol, mass / total);
                    // println!("[PPM][predict]   assigned symbol '{}' prob {}", symbol, mass / total);
                    seen.insert(*symbol, true);
                    assigned += 1;
                }
            }
            if assigned > 0 {
                // Classic PPM-C: stop as soon as any symbol is assigned at any order
                break;
            }
        }

        // Only fallback if no context node was found at any order
        if !found_any_context {
            let root_ref = self.root.borrow();
            let unseen: Vec<char> = root_ref.children.keys()
                .filter(|k| !seen.contains_key(k))
                .cloned()
                .collect();
            let n = unseen.len();
            if alpha > 0.0 && n > 0 {
                // println!("[PPM][fallback] assigning fallback for unseen: {:?}", unseen);
                let add = alpha / n as f64;
                for symbol in unseen {
                    probs.insert(symbol, add);
                }
                // Normalize only if fallback was applied
                let sum: f64 = probs.values().sum();
                if sum > 0.0 {
                    for prob in probs.values_mut() {
                        *prob /= sum;
                    }
                }
            }
        }
        probs
    }
}

impl PPMLanguageModel {
    /// Public getter for max_order
    pub fn max_order(&self) -> PPMOrder {
        self.max_order
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ppm_basic() {
        let mut model = PPMLanguageModel::new(PPMOrder::Two);

        // Train on simple sequence, maintaining a rolling context buffer
        let mut ctx = String::new();
        let max_order = model.max_order().value() as usize;
        for c in "hello".chars() {
            model.enter_symbol(&ctx, c);
            if ctx.len() == max_order {
                ctx.remove(0);
            }
            ctx.push(c);
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
        let mut ctx = String::new();
        let max_order = model.max_order().value() as usize;
        for _ in 0..3 {
            for c in "ab".chars() {
                // println!("[PPM][test] ctx before: '{}', entering: '{}'", ctx, c);
                model.enter_symbol(&ctx, c);
                if ctx.len() == max_order {
                    ctx.remove(0);
                }
                ctx.push(c);
            }
        }

        // After training, print trie structure for inspection
        fn print_trie(node: &Rc<RefCell<PPMNode>>, prefix: String) {
            let node_ref = node.borrow();
            for (k, child) in &node_ref.children {
                // println!("[PPM][trie] {} -> {}", prefix, k);
                print_trie(child, format!("{}{}", prefix, k));
            }
        }
        // println!("[PPM][trie] root children: {:?}", model.root.borrow().children.keys().collect::<Vec<_>>());
        print_trie(&model.root, String::new());

        // Print children of node for context 'a' before prediction
        let mut node = model.root.clone();
        let mut found = true;
        for c in "a".chars() {
            let next = { node.borrow().children.get(&c).cloned() };
            if let Some(child) = next {
                node = child;
            } else {
                found = false;
                break;
            }
        }
        if found {
            let _children: Vec<char> = node.borrow().children.keys().cloned().collect();
            // println!("[PPM][test] children after context 'a': {:?}", children);
        } else {
            // println!("[PPM][test] context 'a' not found in trie");
        }

        // Test order-1 predictions
        let probs = model.get_probs("a");
        // println!("[test_ppm_orders] probs for 'a': {:?}", probs);
        assert!(probs.contains_key(&'b'));
        assert!(probs.get(&'b').unwrap() > &0.5);

        // Test order-0 predictions
        let probs = model.get_probs("");
        assert!(probs.contains_key(&'a'));
        assert!(probs.contains_key(&'b'));
        assert!((probs.get(&'a').unwrap() - probs.get(&'b').unwrap()).abs() < 0.1);
    }
}
