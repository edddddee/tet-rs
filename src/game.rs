use crate::controls::Controller;

pub trait GameImplementation: Controller {
    fn new() -> Self;
    fn handle_input(&mut self, key: <Self as Controller>::Key);
    fn on_update(&mut self);
    fn on_setup(&mut self);
    fn run(&mut self);
    fn is_running(&self) -> bool;
    fn quit(&mut self);
}
