use crate::util::input_manager::{InputSemantic, InputState, InputManager};

pub trait GameObject {
    fn process_input(&mut self, _input_manager: &InputManager) {}
    fn draw(&mut self) {}
    fn get_depth(&self) -> i32;
    fn update(&mut self);
}

struct Transform {
    x: i32,
    y: i32,
    depth: i32
}

pub struct BoardContainer {}
impl GameObject for BoardContainer {
    fn update(&mut self) {
    }
    fn process_input(&mut self, _input_manager: &InputManager) {
        if _input_manager.get_input_state(InputSemantic::Accept) == InputState::Pressed {
            println!("Accept pressed! (From inside of BoardContainer)")
        }
    }
    fn get_depth(&self) -> i32 {return 0;}
}