#[derive(Debug)]
pub enum HDirection {
    Left,
    Right,
}

#[derive(Debug)]
pub enum VDirection {
    Up,
    Down,
}

impl HDirection {
    pub fn opposite(&self) -> Self {
        match self {
            HDirection::Left => HDirection::Right,
            HDirection::Right => HDirection::Left,
        }
    }
}

#[derive(Debug, Default)]
pub enum PlayerState {
    #[default]
    Idle,
    Moving(HDirection),
    Dashing(HDirection),
    Flying(VDirection),
}

#[derive(Debug, Default)]
pub struct Player {
    pub x: usize,
    pub y: usize,
    pub move_state: PlayerState,
}

impl Player {
    pub fn position(&self) -> (usize, usize) {
        (self.x, self.y)
    }
}
