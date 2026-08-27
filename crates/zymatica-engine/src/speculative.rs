use crate::cuneiform::Concept6D;

#[derive(Debug, Clone)]
pub struct AdaptiveDraftController {
    min_k: usize,
    max_k: usize,
    current_k: usize,
    acceptance_ewma: f32,
}

impl AdaptiveDraftController {
    pub fn new(min_k: usize, max_k: usize) -> Self {
        let min_k = min_k.max(1);
        let max_k = max_k.max(min_k);
        Self {
            min_k,
            max_k,
            current_k: max_k,
            acceptance_ewma: 1.0,
        }
    }

    pub fn current_k(&self) -> usize {
        self.current_k
    }

    pub fn acceptance_ewma(&self) -> f32 {
        self.acceptance_ewma
    }

    pub fn observe(&mut self, accepted: usize, drafted: usize) -> usize {
        if drafted == 0 {
            return self.current_k;
        }
        let rate = accepted as f32 / drafted as f32;
        self.acceptance_ewma = 0.8 * self.acceptance_ewma + 0.2 * rate;
        if self.acceptance_ewma < 0.45 && self.current_k > self.min_k {
            self.current_k -= 1;
        } else if self.acceptance_ewma > 0.80 && self.current_k < self.max_k {
            self.current_k += 1;
        }
        self.current_k
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CoordinateMctsConfig {
    pub beam_width: usize,
    pub coordinate_weight: f32,
    pub logprob_weight: f32,
}

impl Default for CoordinateMctsConfig {
    fn default() -> Self {
        Self {
            beam_width: 4,
            coordinate_weight: 1.0,
            logprob_weight: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoordinateBranch {
    pub tokens: Vec<usize>,
    pub concepts: Vec<Concept6D>,
    pub logprob: f32,
    pub score: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TreeStitchConfig {
    pub max_branches: usize,
    pub coordinate_weight: f32,
    pub draft_logprob_weight: f32,
    pub accepted_token_weight: f32,
}

impl Default for TreeStitchConfig {
    fn default() -> Self {
        Self {
            max_branches: 8,
            coordinate_weight: 1.0,
            draft_logprob_weight: 1.0,
            accepted_token_weight: 4.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StitchedBranch {
    pub branch_id: usize,
    pub tokens: Vec<usize>,
    pub concepts: Vec<Concept6D>,
    pub draft_logprob: f32,
    pub coordinate_score: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StitchedTreeBatch {
    pub branches: Vec<StitchedBranch>,
    pub packed_tokens: Vec<usize>,
    pub offsets: Vec<(usize, usize)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StitchedSelection {
    pub branch_id: usize,
    pub tokens: Vec<usize>,
    pub accepted_prefix_len: usize,
    pub score: f32,
}

pub fn coordinate_guided_branch_search(
    roots: &[CoordinateBranch],
    expansions: &[Vec<CoordinateBranch>],
    target: Concept6D,
    config: CoordinateMctsConfig,
) -> Vec<CoordinateBranch> {
    let beam_width = config.beam_width.max(1);
    let mut frontier = roots.to_vec();
    score_coordinate_branches(&mut frontier, target, config);
    frontier.sort_by(|a, b| b.score.total_cmp(&a.score));
    frontier.truncate(beam_width);

    for step in expansions {
        let mut next = Vec::new();
        for root in &frontier {
            for child in step {
                let mut merged = root.clone();
                merged.tokens.extend_from_slice(&child.tokens);
                merged.concepts.extend_from_slice(&child.concepts);
                merged.logprob += child.logprob;
                next.push(merged);
            }
        }
        if next.is_empty() {
            break;
        }
        score_coordinate_branches(&mut next, target, config);
        next.sort_by(|a, b| b.score.total_cmp(&a.score));
        next.truncate(beam_width);
        frontier = next;
    }

    frontier
}

pub fn stitch_speculative_tree(
    roots: &[CoordinateBranch],
    expansions: &[Vec<CoordinateBranch>],
    target: Concept6D,
    config: TreeStitchConfig,
) -> StitchedTreeBatch {
    let selected = coordinate_guided_branch_search(
        roots,
        expansions,
        target,
        CoordinateMctsConfig {
            beam_width: config.max_branches.max(1),
            coordinate_weight: config.coordinate_weight,
            logprob_weight: config.draft_logprob_weight,
        },
    );
    let mut packed_tokens = Vec::new();
    let mut offsets = Vec::new();
    let mut branches = Vec::new();
    for (branch_id, branch) in selected.into_iter().enumerate() {
        let start = packed_tokens.len();
        packed_tokens.extend_from_slice(&branch.tokens);
        offsets.push((start, branch.tokens.len()));
        let coordinate_score = if branch.concepts.is_empty() {
            0.0
        } else {
            branch
                .concepts
                .iter()
                .map(|concept| concept.normalized_similarity(target))
                .sum::<f32>()
                / branch.concepts.len() as f32
        };
        branches.push(StitchedBranch {
            branch_id,
            tokens: branch.tokens,
            concepts: branch.concepts,
            draft_logprob: branch.logprob,
            coordinate_score,
        });
    }
    StitchedTreeBatch {
        branches,
        packed_tokens,
        offsets,
    }
}

pub fn verify_stitched_tree_batch(
    batch: &StitchedTreeBatch,
    target_tokens: &[usize],
    config: TreeStitchConfig,
) -> Option<StitchedSelection> {
    batch
        .branches
        .iter()
        .map(|branch| {
            let accepted_prefix_len = branch
                .tokens
                .iter()
                .zip(target_tokens)
                .take_while(|(candidate, target)| candidate == target)
                .count();
            let score = config.accepted_token_weight * accepted_prefix_len as f32
                + config.coordinate_weight * branch.coordinate_score
                + config.draft_logprob_weight * branch.draft_logprob;
            StitchedSelection {
                branch_id: branch.branch_id,
                tokens: branch.tokens.clone(),
                accepted_prefix_len,
                score,
            }
        })
        .max_by(|a, b| a.score.total_cmp(&b.score))
}

fn score_coordinate_branches(
    branches: &mut [CoordinateBranch],
    target: Concept6D,
    config: CoordinateMctsConfig,
) {
    for branch in branches {
        let coordinate_score = if branch.concepts.is_empty() {
            0.0
        } else {
            branch
                .concepts
                .iter()
                .map(|concept| concept.normalized_similarity(target))
                .sum::<f32>()
                / branch.concepts.len() as f32
        };
        branch.score =
            config.logprob_weight * branch.logprob + config.coordinate_weight * coordinate_score;
    }
}

#[derive(Debug, Clone)]
pub struct SpeculativeAttractorEngine {
    pub controller: AdaptiveDraftController,
    pub config: TreeStitchConfig,
    pub total_proposed_tokens: usize,
    pub total_accepted_tokens: usize,
}

impl SpeculativeAttractorEngine {
    pub fn new(min_k: usize, max_k: usize) -> Self {
        Self {
            controller: AdaptiveDraftController::new(min_k, max_k),
            config: TreeStitchConfig::default(),
            total_proposed_tokens: 0,
            total_accepted_tokens: 0,
        }
    }

    pub fn speculate_and_verify(
        &mut self,
        roots: &[CoordinateBranch],
        expansions: &[Vec<CoordinateBranch>],
        target_concept: Concept6D,
        target_tokens: &[usize],
    ) -> Option<StitchedSelection> {
        let tree_batch = stitch_speculative_tree(roots, expansions, target_concept, self.config);
        let proposed = tree_batch.packed_tokens.len();
        self.total_proposed_tokens += proposed;

        let selection = verify_stitched_tree_batch(&tree_batch, target_tokens, self.config);
        if let Some(ref sel) = selection {
            self.total_accepted_tokens += sel.accepted_prefix_len;
            self.controller
                .observe(sel.accepted_prefix_len, proposed.max(1));
        }
        selection
    }

    pub fn acceptance_rate(&self) -> f32 {
        if self.total_proposed_tokens == 0 {
            0.0
        } else {
            self.total_accepted_tokens as f32 / self.total_proposed_tokens as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adaptive_controller_reduces_and_recovers_draft_length() {
        let mut controller = AdaptiveDraftController::new(1, 6);
        assert_eq!(controller.current_k(), 6);
        for _ in 0..8 {
            controller.observe(0, 6);
        }
        assert!(controller.current_k() < 6);
        for _ in 0..12 {
            controller.observe(controller.current_k(), controller.current_k());
        }
        assert_eq!(controller.current_k(), 6);
    }

    #[test]
    fn coordinate_guided_search_keeps_semantic_branch_over_raw_logprob() {
        let target = Concept6D::new(1, 1, 1, 1, 1, 1);
        let far = Concept6D::new(15, 15, 15, 15, 15, 15);
        let near = Concept6D::new(1, 1, 1, 1, 1, 2);
        let roots = vec![
            CoordinateBranch {
                tokens: vec![10],
                concepts: vec![far],
                logprob: -0.01,
                score: 0.0,
            },
            CoordinateBranch {
                tokens: vec![20],
                concepts: vec![near],
                logprob: -0.20,
                score: 0.0,
            },
        ];
        let branches = coordinate_guided_branch_search(
            &roots,
            &[],
            target,
            CoordinateMctsConfig {
                beam_width: 1,
                coordinate_weight: 2.0,
                logprob_weight: 1.0,
            },
        );
        assert_eq!(branches[0].tokens, vec![20]);
    }

    #[test]
    fn speculative_tree_stitching_selects_best_valid_branch_prefix() {
        let target = Concept6D::new(2, 2, 2, 2, 2, 2);
        let near = Concept6D::new(2, 2, 2, 2, 2, 3);
        let far = Concept6D::new(15, 15, 15, 15, 15, 15);
        let roots = vec![
            CoordinateBranch {
                tokens: vec![10],
                concepts: vec![far],
                logprob: -0.01,
                score: 0.0,
            },
            CoordinateBranch {
                tokens: vec![20],
                concepts: vec![near],
                logprob: -0.20,
                score: 0.0,
            },
        ];
        let expansions = vec![vec![
            CoordinateBranch {
                tokens: vec![11],
                concepts: vec![far],
                logprob: -0.01,
                score: 0.0,
            },
            CoordinateBranch {
                tokens: vec![22],
                concepts: vec![near],
                logprob: -0.10,
                score: 0.0,
            },
        ]];
        let config = TreeStitchConfig {
            max_branches: 4,
            coordinate_weight: 2.0,
            draft_logprob_weight: 1.0,
            accepted_token_weight: 4.0,
        };
        let batch = stitch_speculative_tree(&roots, &expansions, target, config);
        assert_eq!(batch.offsets.len(), batch.branches.len());
        assert!(batch.packed_tokens.len() >= 4);
        let selected = verify_stitched_tree_batch(&batch, &[20, 22], config).unwrap();
        assert_eq!(selected.tokens, vec![20, 22]);
        assert_eq!(selected.accepted_prefix_len, 2);
    }

    #[test]
    fn speculative_attractor_engine_speculates_and_verifies_batch() {
        let target = Concept6D::new(2, 2, 2, 2, 2, 2);
        let near = Concept6D::new(2, 2, 2, 2, 2, 3);
        let roots = vec![CoordinateBranch {
            tokens: vec![100],
            concepts: vec![near],
            logprob: -0.05,
            score: 0.0,
        }];
        let expansions = vec![vec![CoordinateBranch {
            tokens: vec![200],
            concepts: vec![near],
            logprob: -0.05,
            score: 0.0,
        }]];
        let mut engine = SpeculativeAttractorEngine::new(1, 4);
        let selected = engine
            .speculate_and_verify(&roots, &expansions, target, &[100, 200])
            .unwrap();
        assert_eq!(selected.accepted_prefix_len, 2);
        assert!(engine.acceptance_rate() > 0.0);
    }

    #[test]
    fn fast_ngram_proposal_engine_trains_and_proposes() {
        let mut ngram = FastNGramProposalEngine::new(3);
        ngram.train_tokens(&[1, 2, 3, 4, 1, 2, 3, 5, 1, 2, 3, 4]);
        let proposal = ngram.propose_sequence(&[1, 2], 2);
        assert_eq!(proposal, vec![3, 4]);
    }

    #[test]
    fn fast_ngram_proposal_engine_learns_online_transition() {
        let mut ngram = FastNGramProposalEngine::new(3);
        ngram.train_tokens(&[1, 2, 3, 1, 2, 3]);
        assert!(ngram.propose_sequence(&[9, 8], 2).is_empty());

        ngram.observe_transition(&[7, 9, 8], 6);
        ngram.observe_transition(&[9, 8, 6], 5);

        assert_eq!(ngram.propose_sequence(&[100, 7, 9, 8], 2), vec![6, 5]);
    }
}

#[derive(Debug, Clone, Default)]
pub struct FastNGramProposalEngine {
    ngram_table: std::collections::HashMap<Vec<usize>, Vec<(usize, u32)>>,
    n: usize,
}

impl FastNGramProposalEngine {
    pub fn new(n: usize) -> Self {
        Self {
            ngram_table: std::collections::HashMap::new(),
            n: n.max(2),
        }
    }

    pub fn train_tokens(&mut self, tokens: &[usize]) {
        if tokens.len() < self.n {
            return;
        }
        for window in tokens.windows(self.n) {
            self.observe_transition(&window[..self.n - 1], window[self.n - 1]);
        }
    }

    pub fn context_len(&self) -> usize {
        self.n - 1
    }

    pub fn observe_transition(&mut self, context: &[usize], next_token: usize) {
        if context.len() < self.context_len() {
            return;
        }
        let key = context[context.len() - self.context_len()..].to_vec();
        let entry = self.ngram_table.entry(key).or_default();
        if let Some(pair) = entry.iter_mut().find(|(token, _)| *token == next_token) {
            pair.1 = pair.1.saturating_add(1);
        } else {
            entry.push((next_token, 1));
        }
    }

    pub fn propose_sequence(&self, prefix: &[usize], draft_len: usize) -> Vec<usize> {
        let mut result = Vec::with_capacity(draft_len);
        let context_start = prefix.len().saturating_sub(self.context_len());
        let mut context = prefix[context_start..].to_vec();
        for _ in 0..draft_len {
            if context.len() < self.context_len() {
                break;
            }
            let key = &context[context.len() - self.context_len()..];
            let candidates = self.ngram_table.get(key);
            let best = candidates.and_then(|c| c.iter().max_by_key(|(_, count)| *count));
            if let Some(&(best_tok, _)) = best {
                result.push(best_tok);
                context.push(best_tok);
                if context.len() > self.context_len() {
                    context.remove(0);
                }
                continue;
            }
            break;
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct MultiHeadSpeculativeEngine {
    pub max_depth: usize,
    pub max_branches: usize,
}

impl MultiHeadSpeculativeEngine {
    pub fn new(max_depth: usize, max_branches: usize) -> Self {
        Self {
            max_depth: max_depth.max(1),
            max_branches: max_branches.max(1),
        }
    }

    pub fn generate_tree_proposals(
        &self,
        ngram: &FastNGramProposalEngine,
        prefix: &[usize],
    ) -> Vec<Vec<usize>> {
        let single_branch = ngram.propose_sequence(prefix, self.max_depth);
        if single_branch.is_empty() {
            return Vec::new();
        }
        let mut proposals = vec![single_branch.clone()];
        if self.max_branches > 1 && !single_branch.is_empty() {
            let mut alt_branch = single_branch.clone();
            if let Some(last) = alt_branch.last_mut() {
                *last = last.wrapping_add(1);
            }
            proposals.push(alt_branch);
        }
        proposals
    }
}

#[test]
fn multi_head_speculative_proposal_tree_verifies() {
    let mut ngram = FastNGramProposalEngine::new(3);
    ngram.train_tokens(&[10, 20, 30, 40, 10, 20, 30, 40]);
    let engine = MultiHeadSpeculativeEngine::new(3, 2);
    let proposals = engine.generate_tree_proposals(&ngram, &[10, 20]);
    assert!(!proposals.is_empty());
    assert_eq!(proposals[0], vec![30, 40, 10]);
}
