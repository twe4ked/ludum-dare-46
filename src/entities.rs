#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wall {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Player {
    pub jumping: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
}
