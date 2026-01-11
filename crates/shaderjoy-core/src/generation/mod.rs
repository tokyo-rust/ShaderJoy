//! Shader generation and evolution logic.

pub mod controller;
pub mod lineage;
pub mod specimen;

pub use specimen::{MutationType, Specimen, SpecimenStatus};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationSession {
    pub id: Uuid,
    pub name: Option<String>,
    pub prompt_words: Vec<String>,
    pub specimens: Vec<Specimen>,
    pub current_generation: u32,
    pub created_at: DateTime<Utc>,
    pub saved: bool,
}

impl GenerationSession {
    pub fn new(prompt_words: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: None,
            prompt_words,
            specimens: Vec::new(),
            current_generation: 0,
            created_at: Utc::now(),
            saved: false,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn specimens_for_generation(&self, gen: u32) -> Vec<&Specimen> {
        self.specimens
            .iter()
            .filter(|s| s.generation == gen)
            .collect()
    }

    pub fn specimens_for_generation_mut(&mut self, gen: u32) -> Vec<&mut Specimen> {
        self.specimens
            .iter_mut()
            .filter(|s| s.generation == gen)
            .collect()
    }

    pub fn selected_specimen(&self) -> Option<&Specimen> {
        self.specimens
            .iter()
            .find(|s| s.status == SpecimenStatus::Selected)
    }

    pub fn valid_specimens(&self) -> Vec<&Specimen> {
        self.specimens
            .iter()
            .filter(|s| s.status.is_renderable())
            .collect()
    }

    pub fn current_generation_specimens(&self) -> Vec<&Specimen> {
        self.specimens_for_generation(self.current_generation)
    }

    pub fn add_specimen(&mut self, specimen: Specimen) {
        self.specimens.push(specimen);
    }

    pub fn advance_generation(&mut self) {
        self.current_generation += 1;
    }

    pub fn deselect_all(&mut self) {
        for specimen in &mut self.specimens {
            if specimen.status == SpecimenStatus::Selected {
                specimen.status = SpecimenStatus::Valid;
            }
        }
    }

    pub fn select_specimen(&mut self, id: Uuid) -> bool {
        self.deselect_all();

        if let Some(specimen) = self.specimens.iter_mut().find(|s| s.id == id) {
            if specimen.status.is_renderable() {
                specimen.mark_selected();
                return true;
            }
        }
        false
    }

    pub fn specimen_count(&self) -> usize {
        self.specimens.len()
    }

    pub fn generation_count(&self) -> u32 {
        self.current_generation + 1
    }
}

impl Default for GenerationSession {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_new() {
        let session = GenerationSession::new(vec!["plasma".to_string(), "fire".to_string()]);
        assert_eq!(session.prompt_words, vec!["plasma", "fire"]);
        assert_eq!(session.current_generation, 0);
        assert!(session.specimens.is_empty());
        assert!(!session.saved);
    }

    #[test]
    fn test_session_with_name() {
        let session =
            GenerationSession::new(vec!["test".to_string()]).with_name("my-shader");
        assert_eq!(session.name, Some("my-shader".to_string()));
    }

    #[test]
    fn test_specimens_for_generation() {
        let mut session = GenerationSession::new(vec![]);
        
        let mut specimen_gen0 = Specimen::new_generating(vec![], 0);
        specimen_gen0.mark_valid();
        session.add_specimen(specimen_gen0);

        let mut specimen_gen1 = Specimen::new_generating(vec![], 1);
        specimen_gen1.mark_valid();
        session.add_specimen(specimen_gen1);

        assert_eq!(session.specimens_for_generation(0).len(), 1);
        assert_eq!(session.specimens_for_generation(1).len(), 1);
        assert_eq!(session.specimens_for_generation(2).len(), 0);
    }

    #[test]
    fn test_select_specimen() {
        let mut session = GenerationSession::new(vec![]);
        
        let mut specimen = Specimen::new_generating(vec![], 0);
        specimen.set_code("// code".to_string());
        specimen.mark_valid();
        let id = specimen.id;
        session.add_specimen(specimen);

        assert!(session.select_specimen(id));
        assert_eq!(session.selected_specimen().unwrap().id, id);
    }
}
