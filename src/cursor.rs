pub struct Cursor {
    pub x: u16,
    pub y: u16
}
impl Cursor {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}
