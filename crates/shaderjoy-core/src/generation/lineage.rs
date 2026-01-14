//! Evolution lineage tracking.
//!
//! Tracks the evolutionary history of specimens, allowing traversal of
//! parent-child relationships and generation analysis.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::specimen::{Specimen, SpecimenStatus};

/// A set of all the specimens from a generation session from start to finish.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Lineage {
    pub specimens: HashMap<Uuid, Specimen>,
    /// Mapping from parent id to child ids which are keys in the [specimens].
    children: HashMap<Uuid, Vec<Uuid>>,
    /// Mapping from generation number to specimen ids.
    generations: HashMap<u32, Vec<Uuid>>,
}

impl Lineage {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a specimen to the lineage, updating the parent-child relationships and generation mapping.
    pub fn add_specimen(&mut self, specimen: Specimen) {
        let id = specimen.id;
        let generation = specimen.generation;
        let parent_id = specimen.parent_id;

        self.specimens.insert(id, specimen);

        self.generations.entry(generation).or_default().push(id);

        if let Some(parent) = parent_id {
            self.children.entry(parent).or_default().push(id);
        }
    }

    pub fn get(&self, id: &Uuid) -> Option<&Specimen> {
        self.specimens.get(id)
    }

    pub fn get_children(&self, id: Uuid) -> Vec<&Specimen> {
        self.children
            .get(&id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|child_id| self.specimens.get(child_id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_ancestors(&self, id: Uuid) -> Vec<&Specimen> {
        let mut ancestors = Vec::new();
        let mut current_id = Some(id);

        while let Some(specimen_id) = current_id {
            if let Some(specimen) = self.specimens.get(&specimen_id) {
                if specimen.parent_id.is_some() {
                    if let Some(parent) =
                        specimen.parent_id.and_then(|pid| self.specimens.get(&pid))
                    {
                        ancestors.push(parent);
                    }
                }
                current_id = specimen.parent_id;
            } else {
                break;
            }
        }

        ancestors
    }

    pub fn get_generation(&self, generation: u32) -> Vec<&Specimen> {
        self.generations
            .get(&generation)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.specimens.get(id))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    pub fn get_generation_mut(&mut self, generation: u32) -> Vec<&mut Specimen> {
        let ids: HashSet<_> = self
            .generations
            .get(&generation)
            .map(|g| g.iter().cloned().collect())
            .unwrap_or_default();

        self.specimens
            .iter_mut()
            .filter(|(uuid, _)| ids.contains(uuid))
            .map(|(_, specimen)| specimen)
            .collect()
    }

    pub fn get_valid_specimens(&self) -> Vec<&Specimen> {
        self.specimens
            .values()
            .filter(|s| s.status.is_renderable())
            .collect()
    }

    pub fn get_selected(&self) -> Option<&Specimen> {
        self.specimens
            .values()
            .find(|s| s.status == SpecimenStatus::Selected)
    }

    pub fn specimen_count(&self) -> usize {
        self.specimens.len()
    }

    pub fn generation_count(&self) -> usize {
        self.generations.len()
    }

    pub fn latest_generation(&self) -> Option<u32> {
        self.generations.keys().max().copied()
    }

    pub fn evolution_path(&self, id: Uuid) -> Vec<&Specimen> {
        let mut path = Vec::new();

        if let Some(specimen) = self.get(&id) {
            path.push(specimen);
        }

        let ancestors = self.get_ancestors(id);
        path.extend(ancestors.into_iter().rev());
        path.reverse();
        path
    }

    pub fn describe_lineage(&self, id: Uuid) -> String {
        let path = self.evolution_path(id);

        if path.is_empty() {
            return "Unknown specimen".to_string();
        }

        let descriptions: Vec<String> = path
            .iter()
            .map(|s| {
                let mutation = s
                    .mutation_type
                    .as_ref()
                    .map(|m| format!(" ({})", m))
                    .unwrap_or_default();
                format!("Gen {}{}", s.generation, mutation)
            })
            .collect();

        descriptions.join(" → ")
    }
}

#[derive(Debug, Clone)]
pub struct LineageStats {
    pub total_specimens: usize,
    pub valid_specimens: usize,
    pub failed_specimens: usize,
    pub generations: usize,
    pub avg_children_per_specimen: f32,
}

impl Lineage {
    pub fn stats(&self) -> LineageStats {
        let total = self.specimens.len();
        let valid = self
            .specimens
            .values()
            .filter(|s| s.status.is_renderable())
            .count();
        let failed = self
            .specimens
            .values()
            .filter(|s| s.status == SpecimenStatus::Failed)
            .count();

        let total_children: usize = self.children.values().map(|c| c.len()).sum();
        let parents_with_children = self.children.len();
        let avg_children = if parents_with_children > 0 {
            total_children as f32 / parents_with_children as f32
        } else {
            0.0
        };

        LineageStats {
            total_specimens: total,
            valid_specimens: valid,
            failed_specimens: failed,
            generations: self.generations.len(),
            avg_children_per_specimen: avg_children,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_specimen(generation: u32, parent_id: Option<Uuid>) -> Specimen {
        let mut s = Specimen::new_generating(None, vec!["test".to_string()], generation);
        s.parent_id = parent_id;
        s.set_code("// test".to_string());
        s.mark_valid();
        s
    }

    #[test]
    fn test_lineage_add_and_get() {
        let mut lineage = Lineage::new();
        let specimen = make_specimen(0, None);
        let id = specimen.id;

        lineage.add_specimen(specimen);

        assert!(lineage.get(&id).is_some());
        assert_eq!(lineage.specimen_count(), 1);
    }

    #[test]
    fn test_lineage_parent_child() {
        let mut lineage = Lineage::new();

        let parent = make_specimen(0, None);
        let parent_id = parent.id;
        lineage.add_specimen(parent);

        let child = make_specimen(1, Some(parent_id));
        let child_id = child.id;
        lineage.add_specimen(child);

        let children = lineage.get_children(parent_id);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, child_id);
    }

    #[test]
    fn test_lineage_ancestors() {
        let mut lineage = Lineage::new();

        let grandparent = make_specimen(0, None);
        let grandparent_id = grandparent.id;
        lineage.add_specimen(grandparent);

        let parent = make_specimen(1, Some(grandparent_id));
        let parent_id = parent.id;
        lineage.add_specimen(parent);

        let child = make_specimen(2, Some(parent_id));
        let child_id = child.id;
        lineage.add_specimen(child);

        let ancestors = lineage.get_ancestors(child_id);
        assert_eq!(ancestors.len(), 2);
    }

    #[test]
    fn test_lineage_generations() {
        let mut lineage = Lineage::new();

        lineage.add_specimen(make_specimen(0, None));
        lineage.add_specimen(make_specimen(0, None));
        lineage.add_specimen(make_specimen(1, None));

        assert_eq!(lineage.get_generation(0).len(), 2);
        assert_eq!(lineage.get_generation(1).len(), 1);
        assert_eq!(lineage.generation_count(), 2);
    }

    #[test]
    fn test_lineage_stats() {
        let mut lineage = Lineage::new();

        let parent = make_specimen(0, None);
        let parent_id = parent.id;
        lineage.add_specimen(parent);
        lineage.add_specimen(make_specimen(1, Some(parent_id)));
        lineage.add_specimen(make_specimen(1, Some(parent_id)));

        let stats = lineage.stats();
        assert_eq!(stats.total_specimens, 3);
        assert_eq!(stats.valid_specimens, 3);
        assert_eq!(stats.generations, 2);
    }
}
