use crate::shader_gen::config::ShaderGenConfig;
use crate::shader_gen::error::Result;
use crate::shader_gen::generate_with_retry;
use rand::Rng;
use rand::prelude::IndexedRandom;
use random_word::Lang;

/// Returns a random word from a huge default word bank or a provided one.
pub fn random_prompt_word(rng: &mut impl Rng, word_bank: Option<&[String]>) -> String {
    if let Some(words) = word_bank {
        words.choose(rng).map(|s| s.clone()).unwrap_or_default()
    } else {
        let all_words = random_word::all(Lang::En);
        all_words
            .choose(rng)
            .map(|s| s.to_string())
            .unwrap_or_default()
    }
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
    let total_tasks = (permutation_cnt as f64 * config.over_subscribe).ceil() as usize;
    println!(
        "Starting generation step. Permutations: {}, Over-subscribe: {}, Total tasks: {}",
        permutation_cnt, config.over_subscribe, total_tasks
    );

    let mut tasks = Vec::with_capacity(total_tasks);

    let base_words = {
        let mut rng = rand::rng();
        if parent.is_none() {
            Some(
                (0..config.prompt_word_count)
                    .map(|_| random_prompt_word(&mut rng, config.word_bank.as_deref()))
                    .collect::<Vec<String>>(),
            )
        } else {
            None
        }
    };

    {
        let mut rng = rand::rng();
        for i in 0..total_tasks {
            let (prompt_words, parent_shader) = if let Some(ref parent) = parent {
                let words_to_freeze = ((config.prompt_word_count as f64 * config.frozen_word_ratio)
                    .round() as usize)
                    .max(1)
                    .min(parent.prompt_words.len());

                let mut frozen: Vec<String> = parent
                    .prompt_words
                    .choose_multiple(&mut rng, words_to_freeze)
                    .cloned()
                    .collect();

                let new_word_count = config.prompt_word_count.saturating_sub(words_to_freeze);
                for _ in 0..new_word_count {
                    frozen.push(random_prompt_word(&mut rng, config.word_bank.as_deref()));
                }
                (frozen, parent.code.clone())
            } else {
                (
                    (0..config.prompt_word_count)
                        .map(|_| random_prompt_word(&mut rng, config.word_bank.as_deref()))
                        .collect(),
                    String::new(),
                )
            };

            println!(
                "Spawning generation task {}/{} with words: {:?}",
                i + 1,
                total_tasks,
                prompt_words
            );

            let config = ShaderGenConfig {
                parent_shader: Some(parent_shader),
                ..config.clone()
            };
            let words = prompt_words.clone();
            let task = tokio::spawn(async move {
                let code = generate_with_retry(&config, &words, 1).await?;
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

    let mut valid_results = Vec::new();
    let mut errors = Vec::new();

    for (i, task) in tasks.into_iter().enumerate() {
        match task.await {
            Ok(result) => {
                println!("Task {} finished.", i + 1);
                match result {
                    Ok(specimen) => valid_results.push(Ok(specimen)),
                    Err(e) => errors.push(Err(e)),
                }
            }
            Err(e) => {
                println!("Task {} failed join: {:?}", i + 1, e);
                errors.push(Err(crate::shader_gen::error::ShaderGenError::LlmError(
                    e.to_string(),
                )))
            }
        }
    }

    println!(
        "Generation step complete. Total: {}, Valid: {}, Errors: {}",
        total_tasks,
        valid_results.len(),
        errors.len()
    );

    // Combine valid results first, then errors if we need to fill up to permutation_cnt
    let mut final_results = valid_results;
    if final_results.len() < permutation_cnt {
        // Append errors to fill the gap if needed, or just return what we have
        let needed = permutation_cnt - final_results.len();
        final_results.extend(errors.into_iter().take(needed));
    } else {
        // Truncate if we have too many valid ones (though we might want to keep them?)
        // For now, let's just take the first permutation_cnt valid ones
        final_results.truncate(permutation_cnt);
    }

    final_results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_each_permutation_gets_different_words() {
        let mut rng = rand::rng();
        let word_count = 5;
        let permutation_cnt = 10;

        let word_sets: Vec<Vec<String>> = (0..permutation_cnt)
            .map(|_| {
                (0..word_count)
                    .map(|_| random_prompt_word(&mut rng, None))
                    .collect()
            })
            .collect();

        for i in 0..permutation_cnt {
            for j in (i + 1)..permutation_cnt {
                assert_ne!(
                    word_sets[i], word_sets[j],
                    "Word sets {} and {} should be different",
                    i, j
                );
            }
        }
    }

    #[test]
    fn test_random_prompt_word_returns_nonempty() {
        let mut rng = rand::rng();
        for _ in 0..100 {
            let word = random_prompt_word(&mut rng, None);
            assert!(!word.is_empty(), "Word should not be empty");
        }
    }
}
