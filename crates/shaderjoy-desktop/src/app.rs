//! Main Iced application.

use std::sync::Arc;
use std::time::Instant;

use iced::widget::{button, column, container, row, text, text_input};
use iced::window;
use iced::{Element, Length, Subscription, Task, Theme};
use uuid::Uuid;

use crate::shader_widget::DEFAULT_FRAGMENT_SHADER;
use crate::ui::grid::{GridMessage, ShaderGrid};
use shaderjoy_core::config::AppConfig;
use shaderjoy_core::generation::{generate_nonce_words, GenerationSession, Specimen};
use shaderjoy_core::llm::{create_llm_client, LlmClient, ShaderGenerationRequest};

#[derive(Debug, Clone)]
pub enum Message {
    Tick(Instant),
    PromptChanged(String),
    Generate,
    GridMessage(GridMessage),
    ShaderGenerated {
        index: usize,
        wgsl_code: String,
        nonce_words: Vec<String>,
    },
    GenerationFailed {
        index: usize,
        error: String,
    },
    Evolve,
}

impl From<GridMessage> for Message {
    fn from(msg: GridMessage) -> Self {
        Message::GridMessage(msg)
    }
}

pub struct ShaderJoyApp {
    #[allow(dead_code)]
    config: AppConfig,
    prompt: Option<String>,
    grid: ShaderGrid,
    session: GenerationSession,
    is_generating: bool,
    start_time: Instant,
    status_message: String,
    llm_client: Option<Arc<dyn LlmClient>>,
}

impl ShaderJoyApp {
    pub fn new(config: AppConfig) -> (Self, Task<Message>) {
        let grid_size = config.grid_size;
        let grid = ShaderGrid::new(grid_size.rows, grid_size.cols);

        let llm_client = create_llm_client(&config.llm_provider).ok();

        let app = Self {
            config,
            prompt: None,
            grid,
            session: GenerationSession::default(),
            is_generating: false,
            start_time: Instant::now(),
            status_message: "Ready".to_string(),
            llm_client,
        };

        (app, Task::none())
    }

    pub fn title(_state: &Self) -> String {
        "ShaderJoy - Select a shader that Sparks Joy!".to_string()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick(_now) => {
                let elapsed = self.start_time.elapsed().as_secs_f32();
                let mut uniforms = self.grid.uniforms;
                uniforms.update_time(elapsed);
                self.grid.update_uniforms(uniforms);
                Task::none()
            }

            Message::PromptChanged(prompt) => {
                self.prompt = Some(prompt.trim().to_string());
                Task::none()
            }

            Message::Generate => {
                self.grid.clear();
                self.is_generating = true;
                self.status_message = "Generating...".to_string();

                let user_prompt = self.prompt.clone();

                let nonce_word_count = self.config.generation.nonce_word_count;
                self.session = GenerationSession::new(user_prompt.clone(), nonce_word_count);

                let mut tasks = Vec::new();
                let cell_count = self.grid.cell_count();

                for index in 0..cell_count {
                    if let Some(ref client) = self.llm_client {
                        let client = Arc::clone(client);
                        let user_prompt = user_prompt.clone();
                        let nonce_words = generate_nonce_words(nonce_word_count);

                        tasks.push(Task::perform(
                            async move {
                                let request =
                                    ShaderGenerationRequest::new(user_prompt, nonce_words.clone());
                                match client.generate_shader(request).await {
                                    Ok(response) => (index, Ok((response.wgsl_code, nonce_words))),
                                    Err(e) => (index, Err(e.to_string())),
                                }
                            },
                            move |(idx, result)| match result {
                                Ok((wgsl_code, nonce_words)) => Message::ShaderGenerated {
                                    index: idx,
                                    wgsl_code,
                                    nonce_words,
                                },
                                Err(error) => Message::GenerationFailed { index: idx, error },
                            },
                        ));
                    } else {
                        self.add_demo_specimen(index);
                    }
                }

                if tasks.is_empty() {
                    self.is_generating = false;
                    self.status_message = format!("Generated {} shaders (demo mode)", cell_count);
                    Task::none()
                } else {
                    Task::batch(tasks)
                }
            }

            Message::ShaderGenerated {
                index,
                wgsl_code,
                nonce_words,
            } => {
                let specimen_id = self.add_specimen(index, wgsl_code, &nonce_words);
                self.grid.set_specimen(index, specimen_id);

                let filled = self.grid.filled_count();
                let total = self.grid.cell_count();

                if filled >= total {
                    self.is_generating = false;
                    self.status_message = format!("Generated {}/{} shaders", filled, total);
                } else {
                    self.status_message = format!("Generating... {}/{}", filled, total);
                }

                Task::none()
            }

            Message::GenerationFailed { index, error } => {
                tracing::warn!("Generation failed for cell {}: {}", index, error);
                self.add_demo_specimen(index);
                self.grid.set_error(index, error);

                let filled = self.grid.filled_count();
                let total = self.grid.cell_count();

                if filled >= total {
                    self.is_generating = false;
                    self.status_message = format!("Generated {}/{} (some failed)", filled, total);
                }

                Task::none()
            }

            Message::GridMessage(GridMessage::CellClicked(index)) => {
                if let Some(specimen_id) = self.grid.specimen_at(index) {
                    let is_currently_selected = self
                        .session
                        .lineage
                        .specimens
                        .iter()
                        .find(|s| s.1.id == specimen_id)
                        .map(|s| s.1.status == shaderjoy_core::generation::SpecimenStatus::Selected)
                        .unwrap_or(false);

                    if is_currently_selected {
                        self.session.deselect_all();
                        self.status_message = "Selection cleared".to_string();
                    } else {
                        self.session.select_specimen(specimen_id);
                        self.status_message = format!("Selected cell {}", index);
                    }
                }
                Task::none()
            }

            Message::Evolve => {
                if self.session.selected_specimen().is_some() {
                    self.session.advance_generation();
                    self.status_message = format!(
                        "Evolution to generation {} (not yet implemented)",
                        self.session.current_generation
                    );
                } else {
                    self.status_message = "Select a shader first".to_string();
                }
                Task::none()
            }
        }
    }

    fn add_specimen(&mut self, index: usize, wgsl_code: String, nonce_words: &[String]) -> Uuid {
        let mut specimen = Specimen::new_generating(
            self.session.user_prompt.clone(),
            nonce_words.to_vec(),
            self.session.current_generation,
        );
        let id = specimen.id;
        specimen.set_code(wgsl_code);
        specimen.mark_valid();
        self.session.add_specimen(specimen);
        self.grid.set_specimen(index, id);
        id
    }

    fn add_demo_specimen(&mut self, index: usize) {
        self.add_specimen(index, DEFAULT_FRAGMENT_SHADER.to_string(), &[]);
    }

    fn has_selection(&self) -> bool {
        self.session.selected_specimen().is_some()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let prompt_input = text_input(
            "Enter prompt words (e.g., plasma fire nebula)...",
            self.prompt.as_ref().unwrap_or(&String::new()),
        )
        .on_input(Message::PromptChanged)
        .padding(10)
        .width(Length::Fill);

        let generate_button = button(text("Generate"))
            .on_press_maybe(if self.is_generating {
                None
            } else {
                Some(Message::Generate)
            })
            .padding(10);

        let evolve_button = button(text("Evolve"))
            .on_press_maybe(if self.has_selection() && !self.is_generating {
                Some(Message::Evolve)
            } else {
                None
            })
            .padding(10);

        let controls = row![prompt_input, generate_button, evolve_button]
            .spacing(10)
            .padding(10);

        let grid_view = self.grid.view::<Message>(&self.session);

        let status = container(text(&self.status_message).size(14))
            .padding(10)
            .width(Length::Fill);

        let content = column![controls, grid_view, status]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        window::frames().map(Message::Tick)
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }
}
