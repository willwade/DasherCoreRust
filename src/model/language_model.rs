//! # Language Model Module
//!
//! This module contains the implementation of the language model used by Dasher.

use std::collections::HashMap;

/// Symbol type for language model
pub type Symbol = usize;

/// Context type for language model
pub type Context = usize;

/// Interface for language models
pub trait LanguageModel {
    /// Create an empty context
    fn create_empty_context(&self) -> Context;

    /// Clone a context
    fn clone_context(&mut self, context: Context) -> Context;

    /// Release a context
    fn release_context(&mut self, context: Context);

    /// Enter a symbol into the context
    fn enter_symbol(&mut self, context: Context, symbol: Symbol) -> Context;

    /// Learn a symbol in the context
    fn learn_symbol(&mut self, context: Context, symbol: Symbol);

    /// Get probabilities for symbols in the context
    fn get_probs(&self, context: Context, norm: usize, uniform: usize) -> Vec<usize>;

    /// Get the number of symbols in the model
    fn num_symbols(&self) -> usize;
}

/// Node in the PPM trie
struct PPMNode {
    /// Symbol counts for this context
    counts: HashMap<Symbol, usize>,

    /// Total count of all symbols in this context
    total: usize,

    /// Child nodes (one for each symbol)
    children: HashMap<Symbol, usize>,

    /// Parent node index
    parent: Option<usize>,

    /// Symbol that led to this node
    symbol: Option<Symbol>,

    /// Reference count
    ref_count: usize,
}

impl PPMNode {
    /// Create a new PPM node
    fn new(parent: Option<usize>, symbol: Option<Symbol>) -> Self {
        Self {
            counts: HashMap::new(),
            total: 0,
            children: HashMap::new(),
            parent,
            symbol,
            ref_count: 1,
        }
    }
}

/// PPM (Prediction by Partial Matching) language model implementation
pub struct PPMLanguageModel {
    /// Number of symbols in the alphabet
    num_symbols: usize,

    /// Maximum order of the model
    max_order: usize,

    /// Exclusion constant
    exclusion: f64,

    /// Nodes in the trie
    nodes: Vec<PPMNode>,
}

impl PPMLanguageModel {
    /// Create a new PPM language model
    pub fn new(num_symbols: usize) -> Self {
        let mut model = Self {
            num_symbols,
            max_order: 5,
            exclusion: 0.5,
            nodes: Vec::new(),
        };

        // Create root node
        model.nodes.push(PPMNode::new(None, None));

        model
    }

    /// Create a new PPM language model with custom parameters
    pub fn with_params(num_symbols: usize, max_order: usize, exclusion: f64) -> Self {
        let mut model = Self {
            num_symbols,
            max_order,
            exclusion,
            nodes: Vec::new(),
        };

        // Create root node
        model.nodes.push(PPMNode::new(None, None));

        model
    }

    /// Get a node by index
    fn get_node(&self, index: usize) -> Option<&PPMNode> {
        self.nodes.get(index)
    }

    /// Get a mutable node by index
    fn get_node_mut(&mut self, index: usize) -> Option<&mut PPMNode> {
        self.nodes.get_mut(index)
    }

    /// Add a symbol to a context
    fn add_symbol(&mut self, context: Context, symbol: Symbol) {
        // First, check if we need to create a child node
        let create_child = {
            if let Some(node) = self.get_node(context) {
                !node.children.contains_key(&symbol) && node.children.len() < self.max_order
            } else {
                false
            }
        };

        // Increment the count for this symbol
        if let Some(node) = self.get_node_mut(context) {
            let count = node.counts.entry(symbol).or_insert(0);
            *count += 1;
            node.total += 1;
        }

        // Create a child node if needed
        if create_child {
            let child_index = self.nodes.len();

            // Add the child to the parent's children map
            if let Some(node) = self.get_node_mut(context) {
                node.children.insert(symbol, child_index);
            }

            // Create the child node
            self.nodes.push(PPMNode::new(Some(context), Some(symbol)));
        }
    }

    /// Get the probability distribution for a context
    fn get_distribution(&self, context: Context, norm: usize, uniform: usize) -> Vec<usize> {
        let mut result = vec![0; self.num_symbols];
        let mut total = 0;

        // Start with the current context
        let mut current_context = context;
        let mut excluded = vec![false; self.num_symbols];

        // Process each context in the backoff chain
        while let Some(node) = self.get_node(current_context) {
            // Calculate probabilities for symbols in this context
            for (symbol, &count) in &node.counts {
                if !excluded[*symbol] {
                    let prob = (count * norm) / node.total;
                    result[*symbol] = prob;
                    total += prob;
                    excluded[*symbol] = true;
                }
            }

            // Move to the parent context
            if let Some(parent) = node.parent {
                current_context = parent;
            } else {
                break;
            }
        }

        // Add uniform distribution for symbols not seen in any context
        if uniform > 0 {
            let remaining = norm - total;
            let num_unseen = excluded.iter().filter(|&&x| !x).count();

            if num_unseen > 0 {
                let prob_per_symbol = remaining / num_unseen;

                for (i, &excluded) in excluded.iter().enumerate() {
                    if !excluded {
                        result[i] = prob_per_symbol;
                    }
                }
            }
        }

        result
    }
}

impl LanguageModel for PPMLanguageModel {
    fn create_empty_context(&self) -> Context {
        // Return the root context
        0
    }

    fn clone_context(&mut self, context: Context) -> Context {
        // Increment the reference count
        if let Some(node) = self.get_node_mut(context) {
            node.ref_count += 1;
        }

        context
    }

    fn release_context(&mut self, context: Context) {
        // Decrement the reference count
        if let Some(node) = self.get_node_mut(context) {
            node.ref_count -= 1;

            // TODO: Clean up nodes with zero reference count
        }
    }

    fn enter_symbol(&mut self, context: Context, symbol: Symbol) -> Context {
        if symbol >= self.num_symbols {
            return 0;
        }

        // Find the child node for this symbol
        if let Some(node) = self.get_node(context) {
            if let Some(&child) = node.children.get(&symbol) {
                // Return the child context
                return child;
            }
        }

        // If no child exists, return the root context
        0
    }

    fn learn_symbol(&mut self, context: Context, symbol: Symbol) {
        if symbol >= self.num_symbols {
            return;
        }

        // Add the symbol to the current context
        self.add_symbol(context, symbol);

        // Add the symbol to all parent contexts
        let mut current_context = context;

        while let Some(node) = self.get_node(current_context) {
            if let Some(parent) = node.parent {
                self.add_symbol(parent, symbol);
                current_context = parent;
            } else {
                break;
            }
        }
    }

    fn get_probs(&self, context: Context, norm: usize, uniform: usize) -> Vec<usize> {
        self.get_distribution(context, norm, uniform)
    }

    fn num_symbols(&self) -> usize {
        self.num_symbols
    }
}
