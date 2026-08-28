//! ==============================================================================
//! ZYMATICA CLASS 35: Z-MCTS (Continuous Manifold Test-Time Latent Reasoning Engine)
//! Author: Danny Bouldiez | Codebase by Devs One
//!
//! Executes Monte Carlo Tree Search directly over continuous 8D Riemannian manifold
//! trajectories prior to token generation. Enables test-time compute scaling (o1/R1 reasoning)
//! without emitting wasteful, slow text tokens during dead-end exploration.
//! ==============================================================================

/// Point in 8D Continuous Riemannian Latent Space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LatentState8D {
    pub coords: [f32; 8],
}

impl LatentState8D {
    pub fn new(coords: [f32; 8]) -> Self {
        Self { coords }
    }

    pub fn distance_to(&self, other: &Self) -> f32 {
        let weights = [1.0, 1.0, 0.75, 0.75, 0.5, 0.5, 0.25, 0.25];
        let mut sum = 0.0f32;
        for i in 0..8 {
            let d = self.coords[i] - other.coords[i];
            sum += weights[i] * d * d;
        }
        sum.sqrt()
    }

    pub fn step(&self, velocity: &[f32; 8], dt: f32) -> Self {
        let mut new_coords = [0.0f32; 8];
        for i in 0..8 {
            new_coords[i] = (self.coords[i] + velocity[i] * dt).clamp(0.0, 15.0);
        }
        Self::new(new_coords)
    }
}

/// MCTS Node in Continuous Latent Geodesic Space
#[derive(Debug, Clone)]
pub struct LatentMctsNode {
    pub state: LatentState8D,
    pub parent_idx: Option<usize>,
    pub children_indices: Vec<usize>,
    pub action_taken: [f32; 8], // Velocity tangent vector
    pub visit_count: u32,
    pub total_value: f32,
    pub prior_prob: f32,
}

impl LatentMctsNode {
    pub fn new(state: LatentState8D, parent: Option<usize>, action: [f32; 8], prior: f32) -> Self {
        Self {
            state,
            parent_idx: parent,
            children_indices: Vec::new(),
            action_taken: action,
            visit_count: 0,
            total_value: 0.0,
            prior_prob: prior,
        }
    }

    pub fn q_value(&self) -> f32 {
        if self.visit_count == 0 {
            0.0
        } else {
            self.total_value / self.visit_count as f32
        }
    }
}

/// Configuration for Z-MCTS Latent Reasoning Engine
#[derive(Debug, Clone)]
pub struct ZMctsConfig {
    pub num_simulations: usize,
    pub max_depth: usize,
    pub c_puct: f32,
    pub branch_factor: usize,
    pub step_dt: f32,
    pub curvature_penalty: f32,
}

impl Default for ZMctsConfig {
    fn default() -> Self {
        Self {
            num_simulations: 250,
            max_depth: 8,
            c_puct: 1.414,
            branch_factor: 6,
            step_dt: 0.25,
            curvature_penalty: 0.05,
        }
    }
}

/// Z-MCTS Continuous Latent Reasoning Engine
pub struct ZMctsEngine {
    pub config: ZMctsConfig,
    pub nodes: Vec<LatentMctsNode>,
}

impl ZMctsEngine {
    pub fn new(config: ZMctsConfig) -> Self {
        Self {
            config,
            nodes: Vec::new(),
        }
    }

    /// Search the optimal continuous reasoning trajectory from start to goal in latent space
    pub fn search_optimal_trajectory(
        &mut self,
        start_state: LatentState8D,
        goal_state: LatentState8D,
    ) -> Vec<LatentState8D> {
        self.nodes.clear();
        self.nodes
            .push(LatentMctsNode::new(start_state, None, [0.0; 8], 1.0));

        let actions = self.generate_candidate_velocities();

        for _ in 0..self.config.num_simulations {
            // 1. Selection
            let mut curr_idx = 0;
            let mut depth = 0;

            while !self.nodes[curr_idx].children_indices.is_empty() && depth < self.config.max_depth
            {
                let parent_visits = self.nodes[curr_idx].visit_count;
                let mut best_score = -f32::INFINITY;
                let mut best_child_idx = self.nodes[curr_idx].children_indices[0];

                for &child_idx in &self.nodes[curr_idx].children_indices {
                    let child = &self.nodes[child_idx];
                    let q = child.q_value();
                    let u = self.config.c_puct
                        * child.prior_prob
                        * ((parent_visits as f32).sqrt() / (1.0 + child.visit_count as f32));
                    let score = q + u;
                    if score > best_score {
                        best_score = score;
                        best_child_idx = child_idx;
                    }
                }
                curr_idx = best_child_idx;
                depth += 1;
            }

            // 2. Expansion
            if depth < self.config.max_depth && self.nodes[curr_idx].visit_count > 0 {
                let parent_state = self.nodes[curr_idx].state;
                for act in &actions {
                    let next_state = parent_state.step(act, self.config.step_dt);
                    let dist = next_state.distance_to(&goal_state);
                    let prior = (1.0 / (1.0 + dist)).max(0.01);
                    let new_node = LatentMctsNode::new(next_state, Some(curr_idx), *act, prior);
                    let new_idx = self.nodes.len();
                    self.nodes.push(new_node);
                    self.nodes[curr_idx].children_indices.push(new_idx);
                }
                if let Some(&first_child) = self.nodes[curr_idx].children_indices.first() {
                    curr_idx = first_child;
                }
            }

            // 3. Evaluation (Energy functional in Riemannian space)
            let current_state = self.nodes[curr_idx].state;
            let dist_to_goal = current_state.distance_to(&goal_state);
            let goal_reward = (10.0 / (1.0 + dist_to_goal)).min(10.0);
            let action_norm: f32 = self.nodes[curr_idx]
                .action_taken
                .iter()
                .map(|x| x * x)
                .sum();
            let value = goal_reward - self.config.curvature_penalty * action_norm;

            // 4. Backpropagation
            let mut back_idx = Some(curr_idx);
            while let Some(idx) = back_idx {
                self.nodes[idx].visit_count += 1;
                self.nodes[idx].total_value += value;
                back_idx = self.nodes[idx].parent_idx;
            }
        }

        // Extract best trajectory from root
        let mut trajectory = vec![start_state];
        let mut curr_idx = 0;
        while !self.nodes[curr_idx].children_indices.is_empty() {
            let mut most_visited = 0;
            let mut best_next_idx = self.nodes[curr_idx].children_indices[0];
            for &child_idx in &self.nodes[curr_idx].children_indices {
                if self.nodes[child_idx].visit_count > most_visited {
                    most_visited = self.nodes[child_idx].visit_count;
                    best_next_idx = child_idx;
                }
            }
            if most_visited == 0 {
                break;
            }
            trajectory.push(self.nodes[best_next_idx].state);
            curr_idx = best_next_idx;
        }

        trajectory
    }

    fn generate_candidate_velocities(&self) -> Vec<[f32; 8]> {
        let mut velocities = Vec::new();
        // Cardinal direction exploratory tangents
        for axis in 0..8 {
            let mut v_pos = [0.0f32; 8];
            v_pos[axis] = 1.0;
            velocities.push(v_pos);

            let mut v_neg = [0.0f32; 8];
            v_neg[axis] = -1.0;
            velocities.push(v_neg);
        }
        velocities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_z_mcts_latent_trajectory_optimization() {
        let config = ZMctsConfig {
            num_simulations: 100,
            max_depth: 6,
            c_puct: 1.414,
            branch_factor: 16,
            step_dt: 0.5,
            curvature_penalty: 0.01,
        };

        let mut engine = ZMctsEngine::new(config);
        let start = LatentState8D::new([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
        let goal = LatentState8D::new([5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0]);

        let trajectory = engine.search_optimal_trajectory(start, goal);
        assert!(trajectory.len() >= 2);
        assert_eq!(trajectory[0], start);

        let initial_dist = start.distance_to(&goal);
        let final_dist = trajectory.last().unwrap().distance_to(&goal);
        assert!(final_dist < initial_dist, "MCTS must navigate towards goal");
    }
}
