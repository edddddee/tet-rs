use crate::controls::Controller;

pub trait GameImplementation: Controller {
    fn new() -> Self;
    fn handle_input(&mut self, key: <Self as Controller>::Key);
    fn on_update(&mut self);
    // TODO: rendering to screen or terminal, run() function, on_exit() or quit() function
    fn on_setup(&mut self);
    fn run(&mut self);
    fn is_running(&self) -> bool;
    fn quit(&mut self);
}
