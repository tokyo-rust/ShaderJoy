//! Specimen (individual shader) definition.

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
    /// Changed the random nonce words used to vary LLM output
    NonceWords(Vec<String>),
    /// Changed the user-provided steering prompt
    UserPrompt(Option<String>),
}

impl std::fmt::Display for MutationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MutationType::NonceWords(words) => write!(f, "Nonce word change: {:?}", words),
            MutationType::UserPrompt(prompt) => {
                write!(
                    f,
                    "User prompt change: {:?}",
                    prompt.as_deref().unwrap_or("(none)")
                )
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Specimen {
    pub id: Uuid,
    pub wgsl_code: String,
    /// Optional user-provided prompt to steer the generation direction
    pub user_prompt: Option<String>,
    /// Random words that act as nonces to randomize LLM output.  These act as a
    /// kind of "salt" or latent space for the LLM to explore.
    pub nonce_words: Vec<String>,
    pub generation: u32,
    pub parent_id: Option<Uuid>,
    pub mutation_type: Option<MutationType>,
    pub created_at: DateTime<Utc>,
    pub status: SpecimenStatus,
}

impl Specimen {
    pub fn new_generating(
        user_prompt: Option<String>,
        nonce_words: Vec<String>,
        generation: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            wgsl_code: String::new(),
            user_prompt,
            nonce_words,
            generation,
            parent_id: None,
            mutation_type: None,
            created_at: Utc::now(),
            status: SpecimenStatus::Generating,
        }
    }

    pub fn new_mutation(
        parent: &Specimen,
        mutation_type: MutationType,
        generation: u32,
    ) -> Self {
        let (user_prompt, nonce_words) = match &mutation_type {
            MutationType::NonceWords(words) => (parent.user_prompt.clone(), words.clone()),
            MutationType::UserPrompt(prompt) => (prompt.clone(), parent.nonce_words.clone()),
        };
        Self {
            id: Uuid::new_v4(),
            wgsl_code: String::new(),
            user_prompt,
            nonce_words,
            generation,
            parent_id: Some(parent.id),
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
    use super::*;

    #[test]
    fn test_specimen_new_generating() {
        let specimen =
            Specimen::new_generating(Some("cyberpunk".to_string()), vec!["plasma".to_string()], 0);
        assert_eq!(specimen.status, SpecimenStatus::Generating);
        assert_eq!(specimen.generation, 0);
        assert_eq!(specimen.user_prompt, Some("cyberpunk".to_string()));
        assert_eq!(specimen.nonce_words, vec!["plasma".to_string()]);
        assert!(specimen.parent_id.is_none());
        assert!(specimen.mutation_type.is_none());
    }

    #[test]
    fn test_specimen_new_generating_no_user_prompt() {
        let specimen =
            Specimen::new_generating(None, vec!["fire".to_string(), "wave".to_string()], 0);
        assert!(specimen.user_prompt.is_none());
        assert_eq!(specimen.nonce_words.len(), 2);
    }

    #[test]
    fn test_specimen_mutation_nonce_words() {
        let parent = Specimen::new_generating(
            Some("retro".to_string()),
            vec!["fire".to_string()],
            0,
        );
        let new_nonces = vec!["ice".to_string(), "glow".to_string()];
        let child =
            Specimen::new_mutation(&parent, MutationType::NonceWords(new_nonces.clone()), 1);

        assert_eq!(child.parent_id, Some(parent.id));
        assert_eq!(child.user_prompt, parent.user_prompt);
        assert_eq!(child.nonce_words, new_nonces);
        assert_eq!(child.generation, 1);
    }

    #[test]
    fn test_specimen_mutation_user_prompt() {
        let parent = Specimen::new_generating(
            Some("retro".to_string()),
            vec!["fire".to_string()],
            0,
        );
        let child = Specimen::new_mutation(
            &parent,
            MutationType::UserPrompt(Some("neon".to_string())),
            1,
        );

        assert_eq!(child.user_prompt, Some("neon".to_string()));
        assert_eq!(child.nonce_words, parent.nonce_words);
    }

    #[test]
    fn test_specimen_status_transitions() {
        let mut specimen = Specimen::new_generating(None, vec![], 0);
        assert!(!specimen.status.is_renderable());

        specimen.set_code("// wgsl code".to_string());
        assert_eq!(specimen.status, SpecimenStatus::Validating);

        specimen.mark_valid();
        assert!(specimen.status.is_renderable());
        assert!(specimen.status.is_terminal());
    }
}
