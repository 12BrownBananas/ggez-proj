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

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum InputType {
    Keyboard(input::keyboard::KeyCode),
    Mouse(input::mouse::MouseButton)
}

pub trait InputProcessor {
    fn process_input_pressed(&mut self);
    fn process_input_released(&mut self);
    fn process_input(&mut self);
    fn get_input_state(&self) -> InputState;
    fn has_input(&self, input: InputType) -> bool;
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
    fn process_input(&mut self) {
        match self.state {
            InputState::Pressed => {self.state = InputState::Held},
            InputState::Released => {self.state = InputState::AtRest},
            _ => {}
        }
    } //All this function does is roll the state over from "pressed" to "held" and "released" to "atrest"
    fn get_input_state(&self) -> InputState {
        self.state
    }
    fn has_input(&self, input: InputType) -> bool {
        for i in self.input_list.as_slice() {
            if InputType::Keyboard(*i) == input {
                return true;
            }
        }
        false
    }
}

pub struct MouseInputProcessor {
    state: InputState,
    input_list: Vec<ggez::input::mouse::MouseButton>
}
impl MouseInputProcessor {
    pub fn new(input_list: Vec<ggez::input::mouse::MouseButton>) -> MouseInputProcessor {
        MouseInputProcessor {
            state: InputState::AtRest,
            input_list: input_list
        }
    }
}
impl InputProcessor for MouseInputProcessor {
    fn process_input_pressed(&mut self) {
        self.state = InputState::Pressed;
    }
    fn process_input_released(&mut self) {
        self.state = InputState::Released;
    }
    fn process_input(&mut self) {
        match self.state {
            InputState::Pressed => {self.state = InputState::Held},
            InputState::Released => {self.state = InputState::AtRest},
            _ => {}
        }
    } //All this function does is roll the state over from "pressed" to "held" and "released" to "atrest"
    fn get_input_state(&self) -> InputState {
        self.state
    }
    fn has_input(&self, input: InputType) -> bool {
        for i in self.input_list.as_slice() {
            if InputType::Mouse(*i) == input {
                return true;
            }
        }
        false
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InputState {
    AtRest,
    Released,
    Pressed,
    Held,
}

pub struct InputManager {
    input_map: std::collections::HashMap<InputSemantic, Vec<usize>>,
    input_processors: Vec<Box<dyn InputProcessor>>,
}
impl InputManager {
    pub fn new() -> InputManager {
        InputManager { 
            input_map: std::collections::HashMap::new(), 
            input_processors: Vec::new(),
        }
    }
    fn get_highest_priority_input(&self, list_of_inputs: &Vec<InputState>) -> InputState {
        let mut previous_best = InputState::AtRest;
        for &i in list_of_inputs {
            if previous_best < i {
                previous_best = i;
            }
        }
        return previous_best;
    }
    pub fn register_input(&mut self, semantic: InputSemantic, input_processor: Box<dyn InputProcessor>) {
        self.input_processors.push(input_processor);
        match self.input_map.get_mut(&semantic) {
            Some(vector) => { vector.push(self.input_processors.len()-1); }
            None => { self.input_map.insert(semantic, vec!(self.input_processors.len()-1)); }
        }
    }
    pub fn get_input_state(&self, semantic: InputSemantic) -> Option<InputState> {
        match self.input_map.get(&semantic) {
            Some(index_vec) => {
                let mut input_accumulator = Vec::new();
                for &index in index_vec {
                    match self.input_processors.get(index) {
                        Some(wrapper) => { input_accumulator.push(wrapper.get_input_state()); },
                        None => {}
                    }
                }
                if input_accumulator.len() > 0 {
                    return Some(self.get_highest_priority_input(&input_accumulator));
                }
                None
            },
            None => None
        }
    }
    pub fn process_input_pressed(&mut self, input: InputType) {
        for i in self.input_processors.as_mut_slice() {
            if i.has_input(input) {
                i.process_input_pressed();
            }
        }
    }
    pub fn process_input_released(&mut self, input: InputType) {
        for i in self.input_processors.as_mut_slice() {
            if i.has_input(input) {
                i.process_input_released();
            }
        }
    }
    pub fn process_input(&mut self) {
        for i in self.input_processors.as_mut_slice() {
            i.process_input();
        }
    }
    /* TODO: Implement this
    pub fn get_input_state_by_semantic(&self, semantic: InputSemantic) -> InputState {

    }
     */
}