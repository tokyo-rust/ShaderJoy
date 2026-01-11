//! Specimen (individual shader) definition.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecimenStatus {
    Generating,
    Validating,
    Valid,
    Invalid,
    Failed,
    Selected,
}

impl SpecimenStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            SpecimenStatus::Valid | SpecimenStatus::Failed | SpecimenStatus::Selected
        )
    }

    pub fn is_renderable(&self) -> bool {
        matches!(self, SpecimenStatus::Valid | SpecimenStatus::Selected)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutationType {
    PromptWords(Vec<String>),
}

impl std::fmt::Display for MutationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            MutationType::PromptWords(words) => format!("Prompt word change: {:?}", words),
        };
        write!(f, "{}", name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Specimen {
    pub id: Uuid,
    pub wgsl_code: String,
    pub prompt_words: Vec<String>,
    pub generation: u32,
    pub parent_id: Option<Uuid>,
    pub parent: Option<Arc<Specimen>>,
    pub mutation_type: Option<MutationType>,
    pub created_at: DateTime<Utc>,
    pub status: SpecimenStatus,
}

impl Specimen {
    pub fn new_generating(prompt_words: Vec<String>, generation: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            wgsl_code: String::new(),
            prompt_words,
            generation,
            parent_id: None,
            parent: None,
            mutation_type: None,
            created_at: Utc::now(),
            status: SpecimenStatus::Generating,
        }
    }

    pub fn new_mutation(
        parent: &Arc<Specimen>,
        mutation_type: MutationType,
        generation: u32,
    ) -> Self {
        let prompt_words = match &mutation_type {
            MutationType::PromptWords(pw) => pw.clone(),
            // _ => parent.prompt_words.clone(),
        };
        Self {
            id: Uuid::new_v4(),
            wgsl_code: String::new(),
            prompt_words: prompt_words,
            generation,
            parent_id: Some(parent.id),
            parent: Some(parent.clone()),
            mutation_type: Some(mutation_type),
            created_at: Utc::now(),
            status: SpecimenStatus::Generating,
        }
    }

    pub fn set_code(&mut self, wgsl_code: String) {
        self.wgsl_code = wgsl_code;
        self.status = SpecimenStatus::Validating;
    }

    pub fn mark_valid(&mut self) {
        self.status = SpecimenStatus::Valid;
    }

    pub fn mark_invalid(&mut self) {
        self.status = SpecimenStatus::Invalid;
    }

    pub fn mark_failed(&mut self) {
        self.status = SpecimenStatus::Failed;
    }

    pub fn mark_selected(&mut self) {
        self.status = SpecimenStatus::Selected;
    }

    pub fn is_initial_generation(&self) -> bool {
        self.generation == 0
    }

    pub fn has_parent(&self) -> bool {
        self.parent_id.is_some()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn test_specimen_new_generating() {
        let specimen = Specimen::new_generating(vec!["plasma".to_string()], 0);
        assert_eq!(specimen.status, SpecimenStatus::Generating);
        assert_eq!(specimen.generation, 0);
        assert!(specimen.parent_id.is_none());
        assert!(specimen.mutation_type.is_none());
    }

    #[test]
    fn test_specimen_mutation() {
        let parent = Arc::new(Specimen::new_generating(vec!["fire".to_string()], 0));
        let child = Specimen::new_mutation(
            &parent,
            MutationType::PromptWords(parent.prompt_words.clone()),
            1,
        );

        assert_eq!(child.parent_id, Some(parent.id));
        assert_eq!(
            child.mutation_type,
            Some(MutationType::PromptWords(parent.prompt_words.clone()))
        );
        assert_eq!(child.generation, 1);
        assert_eq!(child.prompt_words, parent.prompt_words);
    }

    #[test]
    fn test_specimen_status_transitions() {
        let mut specimen = Specimen::new_generating(vec![], 0);
        assert!(!specimen.status.is_renderable());

        specimen.set_code("// wgsl code".to_string());
        assert_eq!(specimen.status, SpecimenStatus::Validating);

        specimen.mark_valid();
        assert!(specimen.status.is_renderable());
        assert!(specimen.status.is_terminal());
    }
}
