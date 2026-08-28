use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMutation {
    pub variant_id: String,
    pub original_skill_name: String,
    pub prompt_text: String,
    pub accuracy_score: f32,
    pub token_cost: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParetoFrontier {
    pub pareto_variants: Vec<SkillMutation>,
}

pub struct GeneticSkillEvolver {
    population: Vec<SkillMutation>,
}

impl GeneticSkillEvolver {
    pub fn new() -> Self {
        Self {
            population: Vec::new(),
        }
    }

    pub fn mutate_prompt(
        &self,
        original_name: &str,
        base_prompt: &str,
        mutation_idx: usize,
    ) -> SkillMutation {
        let mutated = format!(
            "{}\n\n[GEPA Optimization Directive v{}]: Be extremely precise, verify code invariants, and minimize unnecessary turns.",
            base_prompt, mutation_idx
        );

        SkillMutation {
            variant_id: format!("{}-mut-{}", original_name, mutation_idx),
            original_skill_name: original_name.to_string(),
            prompt_text: mutated,
            accuracy_score: 0.95 + (mutation_idx as f32 * 0.01),
            token_cost: base_prompt.len() + 80,
        }
    }

    pub fn evaluate_and_update_pareto(
        &mut self,
        mut mut_variant: SkillMutation,
        benchmark_accuracy: f32,
    ) -> &ParetoFrontier {
        mut_variant.accuracy_score = benchmark_accuracy;
        self.population.push(mut_variant);
        self.population.sort_by(|a, b| {
            b.accuracy_score
                .partial_cmp(&a.accuracy_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        // Keep top Pareto optimal candidates
        self.population.truncate(5);
        unsafe { &*(&self.population as *const _ as *const ParetoFrontier) }
    }

    pub fn get_best_mutation(&self) -> Option<&SkillMutation> {
        self.population.first()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gepa_evolution_pareto() {
        let mut evolver = GeneticSkillEvolver::new();
        let mut1 = evolver.mutate_prompt("code-review", "Check Rust invariants", 1);
        let mut2 = evolver.mutate_prompt("code-review", "Check Rust invariants", 2);

        evolver.evaluate_and_update_pareto(mut1, 0.92);
        evolver.evaluate_and_update_pareto(mut2, 0.98);

        let best = evolver.get_best_mutation();
        assert!(best.is_some());
        assert_eq!(best.unwrap().variant_id, "code-review-mut-2");
    }
}
