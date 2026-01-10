use crate::shader_gen::config::ShaderGenConfig;
use crate::shader_gen::error::Result;
use crate::shader_gen::generate_with_retry;
use rand::prelude::IndexedRandom;
use random_word::Lang;

fn random_prompt_word() -> String {
    random_word::r#gen(Lang::En).to_string()
}

/// A collection of current specimen and a lineage of parents.
#[derive(Debug, Clone)]
pub struct Generation {
    current: Vec<Specimen>,
    lineage: Vec<Vec<Specimen>>,
}

/// A specimen of a single shader.
#[derive(Debug, Clone)]
pub struct Specimen {
    /// The shader code which is the result of the shader generation.
    code: String,
    /// The words which describe the latent space to prompt the LLM to generate shader.
    prompt_words: Vec<String>,
    /// The generation of this parent starting from index 1 (0 implying there was no parent).
    generation: u32,
}

impl Generation {
    pub fn new() -> Self {
        Self {
            current: vec![],
            lineage: vec![],
        }
    }

    pub async fn go_forward(&mut self, index: usize, config: &ShaderGenConfig) {
        let Some(parent) = self.current.get(index).cloned() else {
            return;
        };
        self.generate(Some(parent), 25, config).await; //GOAT, should be grid size - 1
    }

    pub fn go_back(&mut self) {
        let Some(parent) = self.lineage.pop() else {
            return;
        };
        self.current = parent;
    }

    pub async fn generate(
        &mut self,
        parent: Option<Specimen>,
        permutation_cnt: usize,
        config: &ShaderGenConfig,
    ) -> Vec<Result<Specimen>> {
        let mut current = vec![];
        std::mem::swap(&mut current, &mut self.current);
        self.lineage.push(current);

        let mut rng = rand::rng();

        let generation_num = parent.as_ref().map_or(1, |p| p.generation + 1);

        let mut tasks = Vec::with_capacity(permutation_cnt);

        for _ in 0..permutation_cnt {
            let prompt_words = if let Some(ref parent) = parent {
                let words_to_freeze =
                    config.prompt_word_count / 2usize.pow(parent.generation.min(10));
                let words_to_freeze = words_to_freeze.max(1).min(parent.prompt_words.len());

                let mut frozen: Vec<String> = parent
                    .prompt_words
                    .choose_multiple(&mut rng, words_to_freeze)
                    .cloned()
                    .collect();

                let new_word_count = config.prompt_word_count.saturating_sub(words_to_freeze);
                for _ in 0..new_word_count {
                    frozen.push(random_prompt_word());
                }
                frozen
            } else {
                (0..config.prompt_word_count)
                    .map(|_| random_prompt_word())
                    .collect()
            };

            let config = config.clone();
            let words = prompt_words.clone();
            let task = tokio::spawn(async move {
                let code = generate_with_retry(&config, &words, 3).await?;
                Ok(Specimen {
                    code,
                    prompt_words: words,
                    generation: generation_num,
                })
            });
            tasks.push(task);
        }

        let mut results = Vec::with_capacity(permutation_cnt);
        for task in tasks {
            match task.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(crate::shader_gen::error::ShaderGenError::LlmError(
                    e.to_string(),
                ))),
            }
        }

        self.current = results
            .iter()
            .filter_map(|r| r.as_ref().ok().cloned())
            .collect();

        results
    }
}
