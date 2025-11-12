use dioxus::prelude::*;
use dioxus_time::sleep;
use std::time::Duration;

use super::GridCalculator;

const MIN_CELL_SIZE: f64 = 30.0;
const MAX_CELL_SIZE: f64 = 80.0;

#[derive(Clone, Debug, PartialEq)]
pub struct GridData {
    pub dimensions: (f64, f64),
    pub vertical_lines: Vec<f64>,
    pub horizontal_lines: Vec<f64>,
    pub middle_line: f64,
}

impl GridData {
    pub fn cols(&self) -> i32 {
        if self.vertical_lines.len() > 0 {
            (self.vertical_lines.len() - 1) as i32
        } else {
            0
        }
    }

    pub fn rows(&self) -> i32 {
        if self.horizontal_lines.len() > 0 {
            (self.horizontal_lines.len() - 1) as i32
        } else {
            0
        }
    }

    pub fn in_bounds(&self, col: i32, row: i32) -> bool {
        col >= 0 && row >= 0 && col < self.cols() && row < self.rows()
    }
}

impl Default for GridData {
    fn default() -> Self {
        Self {
            dimensions: (0.0, 0.0),
            vertical_lines: Vec::new(),
            horizontal_lines: Vec::new(),
            middle_line: 0.0,
        }
    }
}

#[derive(Clone)]
pub struct GridContext {
    pub container_ref: Signal<Option<std::rc::Rc<MountedData>>>,
    pub grid_data: Signal<GridData>,
    debounce_timer: Signal<u64>,
}

impl GridContext {
    pub fn new() -> Self {
        Self {
            container_ref: Signal::new(None),
            grid_data: Signal::new(GridData::default()),
            debounce_timer: Signal::new(0),
        }
    }

    pub fn update_dimensions(&self) {
        let mut debounce_timer = self.debounce_timer;
        let timer_id = debounce_timer() + 1;
        debounce_timer.set(timer_id);

        let mut grid_data = self.grid_data;
        let container_ref = self.container_ref;

        spawn(async move {
            sleep(Duration::from_millis(50)).await;

            if debounce_timer() != timer_id {
                return;
            }

            let rect = container_ref.peek();
            if rect.is_none() {
                return;
            }
            let data = rect.as_ref().unwrap();
            let rect = data.get_client_rect().await;
            if rect.is_err() {
                return;
            }
            let rect = rect.unwrap();

            let width = rect.size.width;
            let height = rect.size.height;

            // Calculate optimal grid
            let (_cell_size, vertical_lines, horizontal_lines) =
                GridCalculator::calculate_optimal_grid(width, height, MIN_CELL_SIZE, MAX_CELL_SIZE);

            // Find middle horizontal line
            let middle_line = if !horizontal_lines.is_empty() {
                let mid_idx = horizontal_lines.len() / 2;
                horizontal_lines[mid_idx]
            } else {
                height / 2.0
            };

            grid_data.set(GridData {
                dimensions: (width, height),
                vertical_lines,
                horizontal_lines,
                middle_line,
            });
        });
    }

    pub fn handle_mount(&self, data: std::rc::Rc<MountedData>) {
        let mut container_ref = self.container_ref;
        container_ref.set(Some(data));
        self.update_dimensions();
    }

    pub fn handle_resize(&self) {
        self.update_dimensions();
    }
}

pub fn use_grid_context() -> GridContext {
    use_context::<GridContext>()
}
