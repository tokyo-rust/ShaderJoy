//! Grid UI component for shader display.

use std::cell::RefCell;

use tracing::debug;

use iced::widget::{button, column, container, row};
use iced::{Element, Length};
use uuid::Uuid;

use crate::shader_widget::{shader_widget, ShaderData};
use shaderjoy_core::generation::GenerationSession;
use shaderjoy_core::shaders::uniforms::ShaderUniforms;

/// Cached ShaderData with the specimen ID it was built from.
#[derive(Clone)]
struct CachedShaderData {
    specimen_id: Uuid,
    data: ShaderData,
}

#[derive(Debug, Clone)]
pub enum GridMessage {
    CellClicked(usize),
}

pub struct ShaderGrid {
    pub rows: u32,
    pub cols: u32,
    /// Maps cell index to specimen UUID - single source of truth is GenerationSession
    pub cell_specimens: Vec<Option<Uuid>>,
    pub shader_errors: Vec<Option<String>>,
    pub uniforms: ShaderUniforms,
    /// Cached ShaderData per cell - avoids recreating every frame.
    /// Uses RefCell for interior mutability in view().
    shader_data_cache: RefCell<Vec<Option<CachedShaderData>>>,
}

impl ShaderGrid {
    pub fn new(rows: u32, cols: u32) -> Self {
        let cell_count = (rows * cols) as usize;
        Self {
            rows,
            cols,
            cell_specimens: vec![None; cell_count],
            shader_errors: vec![None; cell_count],
            uniforms: ShaderUniforms::default(),
            shader_data_cache: RefCell::new(vec![None; cell_count]),
        }
    }

    pub fn cell_count(&self) -> usize {
        (self.rows * self.cols) as usize
    }

    /// Associates a cell with a specimen UUID
    pub fn set_specimen(&mut self, index: usize, specimen_id: Uuid) {
        if index < self.cell_specimens.len() {
            self.cell_specimens[index] = Some(specimen_id);
        }
    }

    /// Sets an error message for a cell
    pub fn set_error(&mut self, index: usize, error: String) {
        if index < self.shader_errors.len() {
            self.shader_errors[index] = Some(error);
        }
    }

    pub fn clear(&mut self) {
        for specimen in &mut self.cell_specimens {
            *specimen = None;
        }
        for error in &mut self.shader_errors {
            *error = None;
        }
        for cached in self.shader_data_cache.borrow_mut().iter_mut() {
            *cached = None;
        }
    }

    pub fn update_uniforms(&mut self, uniforms: ShaderUniforms) {
        self.uniforms = uniforms;
    }

    pub fn filled_count(&self) -> usize {
        self.cell_specimens.iter().filter(|s| s.is_some()).count()
    }

    /// Gets the specimen UUID at a given cell index
    pub fn specimen_at(&self, index: usize) -> Option<Uuid> {
        self.cell_specimens.get(index).and_then(|s| *s)
    }

    /// Returns cached ShaderData if specimen ID matches, otherwise creates and caches new one.
    fn get_or_create_shader_data(
        &self,
        index: usize,
        specimen: &shaderjoy_core::generation::Specimen,
        is_selected: bool,
        compilation_error: Option<String>,
    ) -> ShaderData {
        let mut cache = self.shader_data_cache.borrow_mut();

        let needs_rebuild = match cache.get(index) {
            Some(Some(cached)) => cached.specimen_id != specimen.id,
            _ => true,
        };

        if needs_rebuild {
            debug!(
                cell_index = index,
                specimen_id = %specimen.id,
                code_len = specimen.wgsl_code.len(),
                "ShaderData cache miss - rebuilding"
            );
            let data = ShaderData::new(specimen.wgsl_code.clone(), specimen.id);
            cache[index] = Some(CachedShaderData {
                specimen_id: specimen.id,
                data: data.clone(),
            });
        }

        cache[index]
            .as_ref()
            .expect("cache entry just created")
            .data
            .clone()
            .with_uniforms(self.uniforms)
            .with_selected(is_selected)
            .with_compilation_error(compilation_error)
    }

    pub fn view<'a, Message>(&'a self, session: &'a GenerationSession) -> Element<'a, Message>
    where
        Message: 'static + Clone + From<GridMessage>,
    {
        let mut grid_rows: Vec<Element<'a, Message>> = Vec::new();

        for row_idx in 0..self.rows {
            let mut row_cells: Vec<Element<'a, Message>> = Vec::new();

            for col_idx in 0..self.cols {
                let cell_idx = (row_idx * self.cols + col_idx) as usize;
                let cell = self.view_cell(cell_idx, session);
                row_cells.push(cell);
            }

            let row_element = row(row_cells)
                .spacing(4)
                .width(Length::Fill)
                .height(Length::FillPortion(1));

            grid_rows.push(row_element.into());
        }

        column(grid_rows)
            .spacing(4)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_cell<'a, Message>(
        &'a self,
        index: usize,
        session: &'a GenerationSession,
    ) -> Element<'a, Message>
    where
        Message: 'static + Clone + From<GridMessage>,
    {
        let specimen_id = self.cell_specimens.get(index).and_then(|s| *s);
        let specimen =
            specimen_id.and_then(|id| session.lineage.specimens.iter().find(|s| s.1.id == id));
        let is_selected = specimen
            .map(|s| s.1.status == shaderjoy_core::generation::SpecimenStatus::Selected)
            .unwrap_or(false);
        let error = self.shader_errors.get(index).and_then(|e| e.as_ref());

        let content: Element<'a, Message> = if let Some((_, specimen)) = specimen {
            let data = self.get_or_create_shader_data(index, specimen, is_selected, error.cloned());

            shader_widget(data)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            container(
                iced::widget::text("Empty")
                    .size(12)
                    .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        };

        let cell_button = button(content)
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .padding(2)
            .on_press(GridMessage::CellClicked(index).into());

        let style = if is_selected {
            container::bordered_box
        } else {
            container::rounded_box
        };

        container(cell_button)
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .padding(2)
            .style(style)
            .into()
    }
}

impl Default for ShaderGrid {
    fn default() -> Self {
        Self::new(3, 3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_grid_new() {
        let grid = ShaderGrid::new(3, 3);
        assert_eq!(grid.cell_count(), 9);
        assert!(grid.cell_specimens.iter().all(|s| s.is_none()));
    }

    #[test]
    fn test_shader_grid_set_specimen() {
        let mut grid = ShaderGrid::new(2, 2);
        let id = Uuid::new_v4();
        grid.set_specimen(0, id);
        assert_eq!(grid.cell_specimens[0], Some(id));
        assert!(grid.cell_specimens[1].is_none());
    }

    #[test]
    fn test_shader_grid_filled_count() {
        let mut grid = ShaderGrid::new(3, 3);
        assert_eq!(grid.filled_count(), 0);
        grid.set_specimen(0, Uuid::new_v4());
        grid.set_specimen(2, Uuid::new_v4());
        assert_eq!(grid.filled_count(), 2);
    }

    #[test]
    fn test_shader_grid_clear() {
        let mut grid = ShaderGrid::new(2, 2);
        grid.set_specimen(0, Uuid::new_v4());
        grid.set_error(1, "test error".to_string());
        grid.clear();
        assert!(grid.cell_specimens.iter().all(|s| s.is_none()));
        assert!(grid.shader_errors.iter().all(|e| e.is_none()));
    }
}
