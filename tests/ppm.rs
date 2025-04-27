use dasher_core::model::{PPMLanguageModel, PPMOrder, PPMNode};
use std::rc::Rc;
use std::cell::RefCell;

#[test]
fn test_ppm_basic() {
    let mut model = PPMLanguageModel::new(PPMOrder::Two);
    let mut ctx = String::new();
    let max_order = model.max_order().value() as usize;
    for c in "hello".chars() {
        model.enter_symbol(&ctx, c);
        if ctx.len() == max_order {
            ctx.remove(0);
        }
        ctx.push(c);
    }
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
    let mut ctx = String::new();
    let max_order = model.max_order().value() as usize;
    for _ in 0..3 {
        for c in "ab".chars() {
            model.enter_symbol(&ctx, c);
            if ctx.len() == max_order {
                ctx.remove(0);
            }
            ctx.push(c);
        }
    }
    fn print_trie(node: &Rc<RefCell<dasher_core::model::PPMNode>>, prefix: String) {
        let node_ref = node.borrow();
        for (k, child) in &node_ref.children {
            let new_prefix = format!("{}{}", prefix, k);
            print_trie(child, new_prefix);
        }
    }
    // Optionally: print_trie(&model.root, String::new());
}
