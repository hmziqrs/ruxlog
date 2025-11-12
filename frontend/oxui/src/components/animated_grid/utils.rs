pub struct GridCalculator;

impl GridCalculator {
    /// Find optimal cell size that maximizes coverage
    /// Searches from max_size down to min_size
    pub fn find_optimal_cell_size(width: f64, height: f64, min_size: f64, max_size: f64) -> f64 {
        if width <= 0.0 || height <= 0.0 {
            return min_size;
        }

        let mut best_size = max_size;
        let mut best_coverage = 0.0;

        let mut size = max_size;
        while size >= min_size {
            let coverage = Self::calculate_coverage(width, height, size);
            if coverage > best_coverage {
                best_coverage = coverage;
                best_size = size;
            }
            size -= 1.0;
        }

        best_size
    }

    /// Calculate coverage score for a given cell size
    fn calculate_coverage(width: f64, height: f64, cell_size: f64) -> f64 {
        let num_cols = (width / cell_size).floor();
        let num_rows = (height / cell_size).floor();

        // Total covered area (same cell_size for both dimensions for 1:1 ratio)
        num_cols * cell_size * num_rows * cell_size
    }

    /// Calculate grid lines for given dimensions and cell size
    /// Returns (vertical_lines, horizontal_lines)
    pub fn calculate_grid_lines(width: f64, height: f64, cell_size: f64) -> (Vec<f64>, Vec<f64>) {
        if width <= 0.0 || height <= 0.0 || cell_size <= 0.0 {
            return (Vec::new(), Vec::new());
        }

        let num_cols = (width / cell_size).ceil() as usize;
        let num_rows = (height / cell_size).ceil() as usize;

        let vertical_lines: Vec<f64> = (0..=num_cols).map(|i| i as f64 * cell_size).collect();

        let horizontal_lines: Vec<f64> = (0..=num_rows).map(|i| i as f64 * cell_size).collect();

        (vertical_lines, horizontal_lines)
    }

    /// Complete grid calculation: find optimal size and calculate lines
    pub fn calculate_optimal_grid(
        width: f64,
        height: f64,
        min_size: f64,
        max_size: f64,
    ) -> (f64, Vec<f64>, Vec<f64>) {
        let cell_size = Self::find_optimal_cell_size(width, height, min_size, max_size);
        let (vertical_lines, horizontal_lines) =
            Self::calculate_grid_lines(width, height, cell_size);
        (cell_size, vertical_lines, horizontal_lines)
    }
}
