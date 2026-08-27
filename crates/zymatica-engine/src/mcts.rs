use crate::cuneiform::Concept6D;
use crate::model::{AnyKvCache, QuantizedGemmaQ8};

#[derive(Clone, Debug)]
pub struct MctsNode {
    pub token_id: usize,
    pub parent_idx: Option<usize>,
    pub children_indices: Vec<usize>,
    pub visit_count: u32,
    pub value_sum: f32,
    pub prior_prob: f32,
    pub position: usize,
    pub cache: Option<AnyKvCache>,
}

pub struct MctsTree {
    pub nodes: Vec<MctsNode>,
}

impl MctsTree {
    pub fn new(root_token_id: usize, root_position: usize, root_cache: AnyKvCache) -> Self {
        Self {
            nodes: vec![MctsNode {
                token_id: root_token_id,
                parent_idx: None,
                children_indices: Vec::new(),
                visit_count: 1,
                value_sum: 0.0,
                prior_prob: 1.0,
                position: root_position,
                cache: Some(root_cache),
            }],
        }
    }
}

fn softmax(logits: &mut [f32]) {
    let max = logits.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    let mut sum = 0.0;
    for val in logits.iter_mut() {
        *val = (*val - max).exp();
        sum += *val;
    }
    if sum > 0.0 {
        let inv_sum = 1.0 / sum;
        for val in logits.iter_mut() {
            *val *= inv_sum;
        }
    }
}

pub fn token_to_concept(token_id: usize) -> Concept6D {
    crate::cuneiform::token_id_to_concept(token_id)
}

fn concept_distance(a: Concept6D, b: Concept6D) -> f32 {
    let d0 = (a.domain as f32 - b.domain as f32).powi(2);
    let d1 = (a.subdomain as f32 - b.subdomain as f32).powi(2);
    let d2 = (a.operation as f32 - b.operation as f32).powi(2);
    let d3 = (a.modality as f32 - b.modality as f32).powi(2);
    let d4 = (a.depth as f32 - b.depth as f32).powi(2);
    let d5 = (a.polarity as f32 - b.polarity as f32).powi(2);
    (d0 + d1 + d2 + d3 + d4 + d5).sqrt()
}

pub fn mcts_generate(
    model: &QuantizedGemmaQ8,
    prompt_ids: &[usize],
    num_tokens: usize,
    iterations: usize,
    exploration_constant: f32,
    attractor_concept: Option<Concept6D>,
    semantic_weight: f32,
) -> Vec<usize> {
    if prompt_ids.is_empty() || num_tokens == 0 {
        return Vec::new();
    }

    let mut generated = Vec::with_capacity(num_tokens);
    let mut active_prompt = prompt_ids.to_vec();

    // 1. Warm up the base KV cache by feeding prompt tokens except the last one
    let mut base_cache = model.new_cache();
    for (i, &token_id) in active_prompt
        .iter()
        .take(active_prompt.len() - 1)
        .enumerate()
    {
        let _ = model.forward_token(token_id, i, &mut base_cache);
    }

    for _g in 0..num_tokens {
        let root_token = *active_prompt.last().unwrap();
        let root_pos = active_prompt.len() - 1;

        // Initialize tree with root token and cloned cache
        let mut tree = MctsTree::new(root_token, root_pos, base_cache.clone());

        for _iter in 0..iterations {
            // A. Selection
            let mut current_idx = 0;
            let mut path = vec![current_idx];

            while !tree.nodes[current_idx].children_indices.is_empty() {
                let parent_visit_count = tree.nodes[current_idx].visit_count;
                let ln_n_parent = (parent_visit_count as f32).ln();

                let mut best_child_idx = 0;
                let mut best_uct = f32::MIN;

                for &child_idx in &tree.nodes[current_idx].children_indices {
                    let child = &tree.nodes[child_idx];
                    let exploitation = if child.visit_count > 0 {
                        child.value_sum / child.visit_count as f32
                    } else {
                        0.0
                    };

                    let exploration = exploration_constant
                        * child.prior_prob
                        * (ln_n_parent / (1.0 + child.visit_count as f32)).sqrt();

                    let mut semantic_boost = 0.0;
                    if let Some(target) = attractor_concept {
                        let token_concept = token_to_concept(child.token_id);
                        let dist = concept_distance(token_concept, target);
                        semantic_boost = semantic_weight / (1.0 + dist);
                    }

                    let uct_score = exploitation + exploration + semantic_boost;
                    if uct_score > best_uct {
                        best_uct = uct_score;
                        best_child_idx = child_idx;
                    }
                }

                current_idx = best_child_idx;
                path.push(current_idx);
            }

            // B. Expansion
            // If the selected node has not been run through the model, do so and expand
            let node_cache_opt = tree.nodes[current_idx].cache.clone();
            if let Some(mut node_cache) = node_cache_opt {
                let token_id = tree.nodes[current_idx].token_id;
                let pos = tree.nodes[current_idx].position;

                let mut logits = model.forward_token(token_id, pos, &mut node_cache);
                softmax(&mut logits);

                // Find top-k tokens to expand (say, top 5)
                let mut token_probs: Vec<(usize, f32)> = logits.into_iter().enumerate().collect();
                token_probs
                    .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                token_probs.truncate(5);

                let next_pos = pos + 1;
                let mut expanded_indices = Vec::new();

                for (tok_id, prob) in token_probs {
                    let child_node = MctsNode {
                        token_id: tok_id,
                        parent_idx: Some(current_idx),
                        children_indices: Vec::new(),
                        visit_count: 0,
                        value_sum: 0.0,
                        prior_prob: prob,
                        position: next_pos,
                        cache: Some(node_cache.clone()),
                    };
                    let child_idx = tree.nodes.len();
                    tree.nodes.push(child_node);
                    expanded_indices.push(child_idx);
                }

                tree.nodes[current_idx].children_indices = expanded_indices;
            }

            // C. Evaluation & Backpropagation
            // For evaluation, let's use the node's prior probability / semantic reward
            let val = if let Some(target) = attractor_concept {
                let token_concept = token_to_concept(tree.nodes[current_idx].token_id);
                let dist = concept_distance(token_concept, target);
                1.0 / (1.0 + dist)
            } else {
                tree.nodes[current_idx].prior_prob
            };

            for &idx in &path {
                tree.nodes[idx].visit_count += 1;
                tree.nodes[idx].value_sum += val;
            }
        }

        // 3. Selection of the best child of the root node
        let root_children = &tree.nodes[0].children_indices;
        if root_children.is_empty() {
            break;
        }

        let mut best_next_idx = root_children[0];
        let mut max_visits = 0;

        for &child_idx in root_children {
            if tree.nodes[child_idx].visit_count > max_visits {
                max_visits = tree.nodes[child_idx].visit_count;
                best_next_idx = child_idx;
            }
        }

        let best_token = tree.nodes[best_next_idx].token_id;
        generated.push(best_token);
        active_prompt.push(best_token);

        // Advance base cache by feeding the chosen best token
        let _ = model.forward_token(best_token, root_pos + 1, &mut base_cache);
    }

    generated
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::NativeGemma;

    #[test]
    fn test_mcts_generate_mock() {
        let model_a = NativeGemma::seeded_e4b_mock(1234);
        let q8 = crate::model::QuantizedGemmaQ8::from_native(&model_a);

        let target_concept = Concept6D::new(1, 2, 3, 4, 5, 6);
        let prompt = vec![1, 2, 3];

        let generated = mcts_generate(&q8, &prompt, 2, 5, 1.4, Some(target_concept), 1.0);

        assert_eq!(generated.len(), 2);
    }

    #[test]
    fn test_mcts_semantic_attractor_biases_reachable_candidate() {
        let model_a = NativeGemma::seeded_e4b_mock(1234);
        let q8 = crate::model::QuantizedGemmaQ8::from_native(&model_a);
        let prompt = vec![1, 2, 3];

        let mut cache = q8.new_cache();
        for (pos, token_id) in prompt.iter().copied().take(prompt.len() - 1).enumerate() {
            let _ = q8.forward_token(token_id, pos, &mut cache);
        }
        let mut logits = q8.forward_token(*prompt.last().unwrap(), prompt.len() - 1, &mut cache);
        softmax(&mut logits);
        let mut top_tokens: Vec<(usize, f32)> = logits.into_iter().enumerate().collect();
        top_tokens.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        top_tokens.truncate(5);

        let target_token = top_tokens.last().unwrap().0;
        let generated = mcts_generate(
            &q8,
            &prompt,
            1,
            12,
            0.0,
            Some(token_to_concept(target_token)),
            1000.0,
        );

        assert_eq!(generated, vec![target_token]);
    }
}
