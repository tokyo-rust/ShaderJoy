/// A collection of current specimen and a lineage of parents.
#[derive(Debug, Clone)]
pub struct Generation {
    current: Vec<Specimen>,
    lineage: Vec<Vec<Specimen>>,
}

/// A specimen of a single shader.
#[derive(Debug, Clone)]
pub struct Specimen {
    code: String,
}

impl Generation {
    pub fn new() -> Self {
        Self {
            current: vec![],
            lineage: vec![],
        }
    }

    pub fn go_forward(&mut self, index: usize) {
        let Some(parent) = self.current.get(index) else {
            return;
        };
        self.generate(parent.clone());
    }

    pub fn go_back(&mut self) {
        let Some(parent) = self.lineage.pop() else {
            return;
        };
        self.current = parent;
    }

    pub fn generate(&mut self, parent: Specimen) {
        let mut current = vec![];
        std::mem::swap(&mut current, &mut self.current);
        self.lineage.push(current);
        // TODO: use the Oracle to generate new specimen from the parent
        self.current = todo!();
    }
}
