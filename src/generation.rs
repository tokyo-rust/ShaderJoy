use crate::shader_gen::config::ShaderGenConfig;
use crate::shader_gen::error::Result;
use crate::shader_gen::generate_with_retry;
use rand::prelude::IndexedRandom;
use random_word::Lang;

pub fn random_prompt_word() -> String {
    random_word::r#gen(Lang::En).to_string()
}

/// A collection of current specimen and a lineage of parents.
#[derive(Debug, Clone)]
pub struct Generation {
    pub current: Vec<Specimen>,
    lineage: Vec<Vec<Specimen>>,
}

/// A specimen of a single shader.
#[derive(Debug, Clone)]
pub struct Specimen {
    /// The shader code which is the result of the shader generation.
    pub code: String,
    /// The words which describe the latent space to prompt the LLM to generate shader.
    pub prompt_words: Vec<String>,
    /// The generation of this parent starting from index 1 (0 implying there was no parent).
    pub generation: u32,
}

impl Generation {
    pub fn new() -> Self {
        Self {
            current: vec![],
            lineage: vec![],
        }
    }

    pub fn go_back(&mut self) {
        let Some(parent) = self.lineage.pop() else {
            return;
        };
        self.current = parent;
    }

    pub fn advance(&mut self, new_specimens: Vec<Specimen>) {
        let mut old_current = vec![];
        std::mem::swap(&mut old_current, &mut self.current);
        self.lineage.push(old_current);
        self.current = new_specimens;
    }

    pub async fn generate(
        &mut self,
        parent: Option<Specimen>,
        permutation_cnt: usize,
        config: &ShaderGenConfig,
    ) -> Vec<Result<Specimen>> {
        let results = generate_specimens(parent, permutation_cnt, config).await;

        let valid_specimens: Vec<Specimen> = results
            .iter()
            .filter_map(|r| r.as_ref().ok().cloned())
            .collect();

        self.advance(valid_specimens);

        results
    }
}

pub async fn generate_specimens(
    parent: Option<Specimen>,
    permutation_cnt: usize,
    config: &ShaderGenConfig,
) -> Vec<Result<Specimen>> {
    let generation_num = parent.as_ref().map_or(1, |p| p.generation + 1);
    println!("Starting generation step. Permutations: {}", permutation_cnt);

    let mut tasks = Vec::with_capacity(permutation_cnt);

    let base_words = if parent.is_none() {
        Some(
            (0..config.prompt_word_count)
                .map(|_| random_prompt_word())
                .collect::<Vec<String>>(),
        )
    } else {
        None
    };

    {
        let mut rng = rand::rng();
        for i in 0..permutation_cnt {
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
                base_words.clone().unwrap()
            };

            println!(
                "Spawning generation task {}/{} with words: {:?}",
                i + 1,
                permutation_cnt,
                prompt_words
            );

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
    }

    println!("All tasks spawned. Awaiting results...");

    let mut results = Vec::with_capacity(permutation_cnt);
    for (i, task) in tasks.into_iter().enumerate() {
        match task.await {
            Ok(result) => {
                println!("Task {} finished.", i + 1);
                results.push(result)
            }
            Err(e) => {
                println!("Task {} failed join: {:?}", i + 1, e);
                results.push(Err(crate::shader_gen::error::ShaderGenError::LlmError(
                    e.to_string(),
                )))
            }
        }
    }

    println!("Generation step complete.");
    results
}
