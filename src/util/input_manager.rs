use ggez::input;
#[derive(Eq, Hash, PartialEq)]
pub enum InputSemantic {
    Up,
    Down,
    Left,
    Right,
    Accept,
    Back
}

#[derive(Eq, PartialEq)]
pub enum InputType {
    Keyboard(input::keyboard::KeyCode),
    Mouse(input::mouse::MouseButton)
}

pub trait InputProcessor {
    fn process_input_pressed(&mut self);
    fn process_input_released(&mut self);
    fn process_input(&self);
    fn get_input_state(&self) -> InputState;
    fn has_input(&self, input: &InputType) -> bool;
}

pub struct KeyboardInputProcessor {
    state: InputState,
    input_list: Vec<ggez::input::keyboard::KeyCode>
}
impl KeyboardInputProcessor {
    pub fn new(input_list: Vec<ggez::input::keyboard::KeyCode>) -> KeyboardInputProcessor {
        KeyboardInputProcessor {
            state: InputState::AtRest,
            input_list: input_list,
        }
    }
}
impl InputProcessor for KeyboardInputProcessor {
    fn process_input_pressed(&mut self) {
        self.state = InputState::Pressed;
    }
    fn process_input_released(&mut self) {
        self.state = InputState::Released;
    }
    fn process_input(&self) {} //All this function does is roll the state over from "pressed" to "held" and "released" to "atrest"
    fn get_input_state(&self) -> InputState {
        self.state
    }
    fn has_input(&self, input: &InputType) -> bool {
        for i in self.input_list.as_slice() {
            if InputType::Keyboard(*i) == *input {
                return true
            }
        }
        false
    }
}


#[derive(Clone, Copy)]
pub enum InputState {
    Pressed,
    Held,
    Released,
    AtRest
}

pub struct InputManager<'a, T: InputProcessor> {
    input_map: std::collections::HashMap<InputSemantic, usize>,
    input_processors: Vec<&'a mut Box<T>>,
}
impl<'a, T: InputProcessor> InputManager<'a, T> {
    pub fn new() -> InputManager<'a, T> {
        InputManager { 
            input_map: std::collections::HashMap::new(), 
            input_processors: Vec::new(),
        }
    }
    pub fn register_input(&mut self, semantic: InputSemantic, input_processor: &'a mut Box<T>) {
        self.input_processors.push(input_processor);
        self.input_map.insert(semantic, self.input_processors.len());
    }
    pub fn get_input_state(&self, semantic: &InputSemantic) -> Option<InputState> {
        match self.input_map.get(semantic) {
            Some(index) => {
                match self.input_processors.get(*index) {
                    Some(wrapper) => Some(wrapper.get_input_state()),
                    None => None
                }
            },
            None => None
        }
    }
    pub fn process_input_pressed(&mut self, input: &InputType) {
        for i in self.input_processors.as_mut_slice() {
            if i.has_input(input) {
                i.process_input_pressed();
            }
        }
    }
    pub fn process_input_released(&mut self, input: &InputType) {
        for i in self.input_processors.as_mut_slice() {
            if i.has_input(input) {
                i.process_input_released();
            }
        }
    }
    pub fn process_input(&self) {
        for i in self.input_processors.as_slice() {
            i.process_input();
        }
    }
    /* TODO: Implement this
    pub fn get_input_state_by_semantic(&self, semantic: InputSemantic) -> InputState {

    }
     */
}