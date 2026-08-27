use crate::cuneiform::{Concept6D, token_id_to_concept};
use anyhow::{Result, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConceptBounds6D {
    pub min: Concept6D,
    pub max: Concept6D,
}

impl ConceptBounds6D {
    pub fn new(min: Concept6D, max: Concept6D) -> Result<Self> {
        for (axis, (lo, hi)) in min.axes().into_iter().zip(max.axes()).enumerate() {
            if lo > hi {
                bail!("concept bound axis {axis} has min {lo} greater than max {hi}");
            }
        }
        Ok(Self { min, max })
    }

    pub fn contains(self, concept: Concept6D) -> bool {
        concept
            .axes()
            .into_iter()
            .zip(self.min.axes())
            .zip(self.max.axes())
            .all(|((value, lo), hi)| value >= lo && value <= hi)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConceptConstraintMask {
    bounds: Vec<ConceptBounds6D>,
}

impl ConceptConstraintMask {
    pub fn new(bounds: Vec<ConceptBounds6D>) -> Result<Self> {
        if bounds.is_empty() {
            bail!("concept constraint mask requires at least one allowed bound");
        }
        Ok(Self { bounds })
    }

    pub fn single(bounds: ConceptBounds6D) -> Self {
        Self {
            bounds: vec![bounds],
        }
    }

    pub fn allows_concept(&self, concept: Concept6D) -> bool {
        self.bounds.iter().any(|bound| bound.contains(concept))
    }

    pub fn allows_token(&self, token_id: usize) -> bool {
        self.allows_concept(token_id_to_concept(token_id))
    }

    pub fn mask_logits_in_place(&self, logits: &mut [f32]) -> usize {
        let mut allowed = 0;
        for (token_id, logit) in logits.iter_mut().enumerate() {
            if self.allows_token(token_id) {
                allowed += 1;
            } else {
                *logit = f32::NEG_INFINITY;
            }
        }
        allowed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concept_bounds_mask_logits_by_token_coordinates() -> Result<()> {
        let target_token = 42;
        let target = token_id_to_concept(target_token);
        let bounds = ConceptBounds6D::new(target, target)?;
        let mask = ConceptConstraintMask::single(bounds);
        let mut logits = vec![1.0; 96];
        let allowed = mask.mask_logits_in_place(&mut logits);
        assert!(allowed >= 1);
        assert!(logits[target_token].is_finite());
        for (token_id, logit) in logits.iter().enumerate() {
            if token_id != target_token && token_id_to_concept(token_id) != target {
                assert!(!logit.is_finite());
            }
        }
        Ok(())
    }
}
