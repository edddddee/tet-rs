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

    fn key_to_button(&mut self, key: Self::Key) -> Option<Button>;
}
