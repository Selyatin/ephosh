pub struct Cursor {
    x: u16,
    y: u16,
}
impl Cursor {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
    pub fn move_left(&mut self) {
        self.x -= 1;
    }
    pub fn move_right(&mut self) {
        self.x += 1;
    }
    pub fn get_x(&self) -> u16 {
        self.x
    }
    pub fn get_y(&self) -> u16 {
        self.y
    }
    pub fn move_cursor(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }
}
