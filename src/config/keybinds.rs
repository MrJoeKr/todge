pub const UP: char = 'i';
pub const DOWN: char = 'k';
pub const LEFT: char = 'j';
pub const RIGHT: char = 'l';

pub const DASH_LEFT: char = 'J';
pub const DASH_RIGHT: char = 'L';

pub const ULTIMATE: char = ' ';

pub const QUIT: char = 'q';
pub const RESTART: char = 'r';

/// Human-readable name of a key, for the instructions line.
pub fn label(key: char) -> String {
    match key {
        ' ' => "space".to_string(),
        c => c.to_string(),
    }
}
