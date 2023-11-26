#[derive(Clone, Copy, Debug)]
pub enum Button {
    MoveDown,
    MoveLeft,
    MoveRight,
    RotateClockwise,
    Drop,
    Quit,
}

pub trait Controller {
    type Key;

    fn key_to_button(&self, key: Self::Key) -> Option<Button>;
}
