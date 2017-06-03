
pub trait Display {
    fn clear(&mut self);
    fn draw(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool;
}
