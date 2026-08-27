use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixValue {
    pub cache_pages: Vec<usize>,
    pub page_generations: Vec<u64>,
    pub token_len: usize,
}

#[derive(Debug, Clone, Default)]
struct PrefixNode {
    children: BTreeMap<usize, usize>,
    value: Option<PrefixValue>,
}

#[derive(Debug, Clone, Default)]
pub struct PrefixRadixCache {
    nodes: Vec<PrefixNode>,
}

impl PrefixRadixCache {
    pub fn new() -> Self {
        Self {
            nodes: vec![PrefixNode::default()],
        }
    }

    pub fn insert(&mut self, tokens: &[usize], value: PrefixValue) {
        let mut node_idx = 0;
        for token in tokens {
            let existing = self.nodes[node_idx].children.get(token).copied();
            node_idx = if let Some(child) = existing {
                child
            } else {
                let child = self.nodes.len();
                self.nodes.push(PrefixNode::default());
                self.nodes[node_idx].children.insert(*token, child);
                child
            };
        }
        self.nodes[node_idx].value = Some(value);
    }

    pub fn longest_match(&self, tokens: &[usize]) -> Option<(usize, PrefixValue)> {
        let mut node_idx = 0;
        let mut best = self.nodes[0].value.clone().map(|v| (0, v));
        for (depth, token) in tokens.iter().enumerate() {
            let Some(child) = self.nodes[node_idx].children.get(token).copied() else {
                break;
            };
            node_idx = child;
            if let Some(value) = self.nodes[node_idx].value.clone() {
                best = Some((depth + 1, value));
            }
        }
        best
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestState {
    Prefill,
    Decode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InferenceRequest {
    pub id: u64,
    pub prompt_tokens: Vec<usize>,
    pub generated_tokens: usize,
    pub max_new_tokens: usize,
    pub priority: u8,
}

impl InferenceRequest {
    pub fn state(&self) -> RequestState {
        if self.generated_tokens == 0 {
            RequestState::Prefill
        } else {
            RequestState::Decode
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedRequest {
    pub id: u64,
    pub state: RequestState,
    pub prompt_tokens: usize,
    pub reusable_prefix_tokens: usize,
    pub billable_tokens: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchPlan {
    pub requests: Vec<PlannedRequest>,
    pub total_billable_tokens: usize,
}

#[derive(Debug, Clone)]
pub struct RuntimeScheduler {
    pub prefix_cache: PrefixRadixCache,
    pub max_batch_tokens: usize,
}

impl RuntimeScheduler {
    pub fn new(max_batch_tokens: usize) -> Self {
        Self {
            prefix_cache: PrefixRadixCache::new(),
            max_batch_tokens,
        }
    }

    pub fn plan_batch(&self, requests: &[InferenceRequest]) -> BatchPlan {
        let mut sorted = requests.to_vec();
        sorted.sort_by_key(|req| (std::cmp::Reverse(req.priority), req.id));

        let mut planned = Vec::new();
        let mut total = 0;
        for req in sorted {
            let reusable = self
                .prefix_cache
                .longest_match(&req.prompt_tokens)
                .map(|(len, _)| len)
                .unwrap_or(0);
            let billable = match req.state() {
                RequestState::Prefill => req.prompt_tokens.len().saturating_sub(reusable),
                RequestState::Decode => 1,
            };
            if total + billable > self.max_batch_tokens && !planned.is_empty() {
                break;
            }
            total += billable;
            planned.push(PlannedRequest {
                id: req.id,
                state: req.state(),
                prompt_tokens: req.prompt_tokens.len(),
                reusable_prefix_tokens: reusable,
                billable_tokens: billable,
            });
        }

        BatchPlan {
            requests: planned,
            total_billable_tokens: total,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_cache_returns_longest_match() {
        let mut cache = PrefixRadixCache::new();
        cache.insert(
            &[1, 2, 3],
            PrefixValue {
                cache_pages: vec![10],
                page_generations: vec![1],
                token_len: 3,
            },
        );
        cache.insert(
            &[1, 2, 3, 4, 5],
            PrefixValue {
                cache_pages: vec![10, 11],
                page_generations: vec![1, 2],
                token_len: 5,
            },
        );
        let (len, value) = cache.longest_match(&[1, 2, 3, 4, 9]).unwrap();
        assert_eq!(len, 3);
        assert_eq!(value.cache_pages, vec![10]);
    }

    #[test]
    fn scheduler_accounts_for_prefix_reuse() {
        let mut scheduler = RuntimeScheduler::new(8);
        scheduler.prefix_cache.insert(
            &[7, 8, 9],
            PrefixValue {
                cache_pages: vec![1],
                page_generations: vec![1],
                token_len: 3,
            },
        );
        let requests = vec![
            InferenceRequest {
                id: 1,
                prompt_tokens: vec![7, 8, 9, 10, 11],
                generated_tokens: 0,
                max_new_tokens: 8,
                priority: 5,
            },
            InferenceRequest {
                id: 2,
                prompt_tokens: vec![1, 2, 3, 4],
                generated_tokens: 1,
                max_new_tokens: 8,
                priority: 3,
            },
        ];
        let plan = scheduler.plan_batch(&requests);
        assert_eq!(plan.total_billable_tokens, 3);
        assert_eq!(plan.requests[0].reusable_prefix_tokens, 3);
        assert_eq!(plan.requests[0].billable_tokens, 2);
        assert_eq!(plan.requests[1].billable_tokens, 1);
    }
}
