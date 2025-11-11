#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
        }
    }

    pub fn perpendicular(self) -> [Direction; 2] {
        match self {
            Direction::Left | Direction::Right => [Direction::Up, Direction::Down],
            Direction::Up | Direction::Down => [Direction::Left, Direction::Right],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnEdge {
    Left,
    Right,
    Top,
    Bottom,
}

impl SpawnEdge {
    pub fn random() -> Self {
        use super::config::random_usize;
        match random_usize(4) {
            0 => SpawnEdge::Left,
            1 => SpawnEdge::Right,
            2 => SpawnEdge::Top,
            _ => SpawnEdge::Bottom,
        }
    }
}

impl From<SpawnEdge> for Direction {
    fn from(edge: SpawnEdge) -> Self {
        match edge {
            SpawnEdge::Left => Direction::Right,
            SpawnEdge::Right => Direction::Left,
            SpawnEdge::Top => Direction::Down,
            SpawnEdge::Bottom => Direction::Up,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridCircle {
    pub id: u64,
    pub col: i32,
    pub row: i32,
    pub travel_dir: Direction,
    pub moving: bool,
    pub respawning: bool,
    pub scaling_in: bool,
    pub spawn_edge: SpawnEdge,
    pub alive: bool,
    pub just_side_stepped: bool,
    pub scale: f64,
    pub opacity: f64,
}

impl GridCircle {
    /// Check if circle is at spawn position
    pub fn is_at_spawn_position(&self, grid: &super::super::provider::GridData) -> bool {
        match self.spawn_edge {
            SpawnEdge::Left => self.col == 0,
            SpawnEdge::Right => self.col >= grid.cols() - 1,
            SpawnEdge::Top => self.row == 0,
            SpawnEdge::Bottom => self.row >= grid.rows() - 1,
        }
    }

    /// Circle just finished scaling in (3x→1x) after spawn/respawn
    pub fn is_scale_in_complete(&self, _grid: &super::super::provider::GridData) -> bool {
        self.scaling_in && self.scale == 1.0 && self.opacity == 1.0
    }

    /// Circle just finished moving to next cell
    pub fn is_movement_complete(&self) -> bool {
        !self.respawning && self.moving && self.scale == 1.0
    }

    /// Circle just finished scaling out (1x→3x) at goal edge
    pub fn is_scale_out_complete(&self) -> bool {
        !self.respawning && self.moving && self.scale >= super::config::MAX_SCALE
    }

    /// Circle is actively scaling in after spawn (should use scale transition)
    pub fn is_scaling_in_active(&self, _grid: &super::super::provider::GridData) -> bool {
        self.scaling_in
    }

    /// Circle is actively scaling out at goal (should use scale transition)
    pub fn is_scaling_out_active(&self) -> bool {
        !self.respawning && self.moving && self.scale != 1.0
    }

    /// Circle is actively moving between cells (should use movement transition)
    pub fn is_moving_active(&self) -> bool {
        !self.respawning && self.moving
    }

    /// Circle is respawning (no transition)
    pub fn is_respawning(&self) -> bool {
        self.respawning
    }
}
