#[derive(Debug)]
pub enum PlayerDirection {
    Left,
    Right,
}

impl PlayerDirection {
    pub fn opposite(&self) -> Self {
        match self {
            PlayerDirection::Left => PlayerDirection::Right,
            PlayerDirection::Right => PlayerDirection::Left,
        }
    }
}

#[derive(Debug, Default)]
pub enum PlayerState {
    #[default]
    Idle,
    Moving(PlayerDirection),
    Dashing(PlayerDirection),
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
