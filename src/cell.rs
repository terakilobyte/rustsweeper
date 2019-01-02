use ggez::graphics::Point2;
#[derive(Clone, Debug)]
pub struct Cell {
    pub position: Point2,
    pub rust_count: u8,
    pub is_rust: bool,
    pub is_hidden: bool,
    pub game_over: bool,
    pub is_flagged: bool,
    pub scaling: f32,
}

impl Cell {
    pub fn new(position: Point2, is_rust: bool) -> Self {
        Cell {
            position,
            rust_count: 0,
            is_rust,
            is_hidden: true, //TODO: change this back to true
            game_over: false,
            is_flagged: false,
            scaling: 0.0,
        }
    }
}
