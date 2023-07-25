use crate::util::input_manager::{InputSemantic, InputState, InputManager};
use crate::util::data_generator::{self, OpType, DifficultyPools, SetConfig, Board};
use fraction::Fraction;
use ggez::graphics::{self, Text, Drawable, Canvas, Color};
use std::collections::HashMap;
use queues::*;

pub trait GameObject {
    fn process_input(&mut self, _input_manager: &InputManager) {}
    fn draw(&mut self, _canvas: &mut Canvas) {}
    fn get_depth(&self) -> i32;
    fn update(&mut self) {}
}

pub struct Transform {
    x: f32,
    y: f32,
    depth: i32
}

pub struct BoardContainer {
    pool_map: HashMap<String, DifficultyPools>,
    config: SetConfig,
    sequence: Queue<Board>
}
impl BoardContainer {
    pub fn new(pool_map: HashMap<String, DifficultyPools>, config: SetConfig) -> BoardContainer {
        BoardContainer {
            pool_map,
            config,
            sequence: Queue::new()
        }
    }
    pub fn generate_new_board_sequence(&mut self) {
        match data_generator::get_set_of_inputs(self.pool_map.clone(), &self.config) {
            Ok(res) => {
                for board in res {
                    self.sequence.add(board).expect("Unable to add board to queue (in BoardContainer)");
                }
            },
            Err(_) => {}
        }
    }
    pub fn get_next_board(&mut self) -> Option<Board> {
        match self.sequence.remove() {
            Ok(b) => {Some(b)},
            Err(_) => None
        }
    }
}
impl GameObject for BoardContainer {
    fn process_input(&mut self, _input_manager: &InputManager) {
        if _input_manager.get_input_state(InputSemantic::Accept) == InputState::Pressed {
            println!("Accept pressed! (From inside of BoardContainer)")
        }
    }
    fn get_depth(&self) -> i32 {return 0;}
}

pub struct RenderText {
    transform: Transform,
    text: Text,
    text_color: Color
}
impl RenderText {
    pub fn new(x: f32, y: f32, depth: i32, text: &str, color: Color) -> RenderText {
        RenderText { transform: Transform{x, y, depth}, text: Text::new(text), text_color: color }
    }
    pub fn set_text(&mut self, new_text: String) {
        self.text = Text::new(&new_text);
    }
    pub fn set_text_color(&mut self, text_color: Color) {
        self.text_color = text_color;
    }
    pub fn set_pos(&mut self, new_x: f32, new_y: f32) {
        self.transform.x = new_x;
        self.transform.y = new_y;
    }
    pub fn draw(&mut self, _canvas: &mut Canvas) {
        self.text.draw(
            _canvas, 
            graphics::DrawParam::from([self.transform.x, self.transform.y]).color(self.text_color)
        );
    }
}

pub struct VisibleOperation {
    value: OpType,
    render_text: RenderText
}
impl VisibleOperation {
    pub fn new(x: f32, y: f32, depth: i32, color: Color) -> VisibleOperation {
        VisibleOperation {
            value: OpType::None,
            render_text: RenderText::new(x, y, depth, "", color)
        }
    }
    pub fn set_operation(&mut self, new_operation_type: OpType) {
        self.value = new_operation_type;
        self.render_text.set_text(self.get_string_representation_for_operation());
    }
    fn get_string_representation_for_operation(&self) -> String {
        match self.value {
            OpType::Plus => { "+".to_string() },
            OpType::Minus => { "-".to_string() },
            OpType::Multiply => { "x".to_string() },
            OpType::Divide => { "/".to_string() },
            OpType::None => { "".to_string() }
        }
    }
}
impl GameObject for VisibleOperation {
    fn get_depth(&self) -> i32 { self.render_text.transform.depth }
    fn draw(&mut self, _canvas: &mut Canvas) {

    }
}

pub struct VisibleNumber {
    value: Fraction,
    render_text: RenderText
}
impl VisibleNumber {
    pub fn new(value: Fraction, x: f32, y: f32, depth: i32, text_color: Color) -> VisibleNumber {
        VisibleNumber {
            value,
            render_text: RenderText::new(x, y, depth, &value.to_string(), text_color)
        }
    }
    pub fn update_value(&mut self, new_value: Fraction) {
        self.value = new_value;
        self.render_text.set_text(self.value.to_string());
    }
}
impl GameObject for VisibleNumber {
    fn get_depth(&self) -> i32 { self.render_text.transform.depth }
    fn draw(&mut self, _canvas: &mut Canvas) {
        self.render_text.draw(_canvas);
    }
}