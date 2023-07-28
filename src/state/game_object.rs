use crate::util::{
    input_manager::{InputSemantic, InputState, InputManager},
    data_generator::{self, OpType, DifficultyPools, SetConfig, Board}
};
use fraction::Fraction;
use ggez::{graphics::{self, Text, Drawable, Canvas, Color}, mint::Point2};
use std::collections::HashMap;
use queues::*;

pub trait GameObject: AsGameObject {
    fn draw(&mut self, _canvas: &mut Canvas) {}
    fn get_depth(&self) -> i32;
    fn update(&mut self) {}
}
pub trait ControllableGameObject : GameObject {
    fn process_input(&mut self, _input_manager: &InputManager) {}
}

pub trait AsGameObject {
    fn as_game_object(&self) -> &dyn GameObject;
    fn as_mut_game_object(&mut self) -> &mut dyn GameObject;
}
impl <T: GameObject> AsGameObject for T {
    fn as_game_object(&self) -> &dyn GameObject {
        self
    }
    fn as_mut_game_object(&mut self) -> &mut dyn GameObject {
        self
    }
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

pub struct GameController {
    board: BoardContainer,
    xpos: f32,
    ypos: f32,
    x_spacing: f32,
    objects: Vec<Box<dyn GameObject>>,
    seq_initialized: bool
}
impl GameController {
    pub fn new(board: BoardContainer) -> GameController {
        GameController {
            board,
            xpos: 480.0,//TODO: pass in center coordinates externally
            ypos: 270.0,
            x_spacing: 20.0,
            objects: Vec::new(),
            seq_initialized: false
        }
    }
    fn load_board(&mut self) {
        self.objects.clear();
        match self.board.get_next_board() {
            Some(b) => {
                //NOTE: Game Controller is too generic for this. We should construct a board layout manager that has specifically hotbar, target, and workbench fields of fixed size and type.
                let layout = BoardLayout::new(self.xpos, self.ypos, self.x_spacing, b.input.len()); 

                //hotbar
                for (item, pos) in b.input.iter().zip(layout.hotbar_pos_vec.iter()) {
                    self.objects.push(Box::new(VisibleNumber::new(Some(Fraction::from(*item)), pos.x, pos.y, 0, Color::WHITE)));
                }

                //target
                self.objects.push(Box::new(VisibleNumber::new(Some(Fraction::from(b.target)), layout.target_pos.x, layout.target_pos.y, 0, Color::RED)));

                //workbench
                let first_pos = layout.workbench_pos_vec.get(0).expect("Could not get first position from workbench position vector.");
                let second_pos = layout.workbench_pos_vec.get(1).expect("Could not get second position from workbench position vector.");
                let third_pos = layout.workbench_pos_vec.get(2).expect("Could not get third position from workbench position vector.");
                self.objects.push(Box::new(VisibleNumber::new(None, first_pos.x, first_pos.y, 0, Color::WHITE)));
                self.objects.push(Box::new(VisibleOperation::new(second_pos.x, second_pos.y, 0, Color::WHITE)));
                self.objects.push(Box::new(VisibleNumber::new(None, third_pos.x, third_pos.y, 0, Color::WHITE)));
            },
            None => { self.reinitialize(); }
        }
    }
    fn reinitialize(&mut self) {
        self.board.generate_new_board_sequence();
        self.load_board();
    }
}
impl GameObject for GameController {
    fn update(&mut self) {
        if !self.seq_initialized {
            self.reinitialize();
            self.seq_initialized = true;
        }
    }
    fn get_depth(&self) -> i32 { return 0; }
    fn draw(&mut self, _canvas: &mut Canvas) {
        for o in self.objects.as_mut_slice() {
            o.draw(_canvas);
        }
    }
}
impl ControllableGameObject for GameController {
    fn process_input(&mut self, _input_manager: &InputManager) {
        if _input_manager.get_input_state(InputSemantic::Accept) == InputState::Pressed {
            self.load_board();
            //println!("Accept pressed! (From inside of GameController)")
        }
    }
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
    value: Option<Fraction>,
    render_text: RenderText
}
impl VisibleNumber {
    pub fn new(value: Option<Fraction>, x: f32, y: f32, depth: i32, text_color: Color) -> VisibleNumber {
        let text;
        match value {
            Some(val) => { text = val.to_string() },
            None => { text = "".to_string() }
        }
        VisibleNumber {
            value,
            render_text: RenderText::new(x, y, depth, &text, text_color)
        }
    }
    pub fn update_value(&mut self, new_value: Option<Fraction>) {
        self.value = new_value;
        match self.value {
            Some(val) => {self.render_text.set_text(val.to_string());},
            None => { self.render_text.set_text("".to_string()) }
        }
    }
}
impl GameObject for VisibleNumber {
    fn get_depth(&self) -> i32 { self.render_text.transform.depth }
    fn draw(&mut self, _canvas: &mut Canvas) {
        self.render_text.draw(_canvas);
    }
}

pub struct BoardLayout {
    hotbar_pos_vec: Vec<Point2<f32>>,
    workbench_pos_vec: Vec<Point2<f32>>,
    target_pos: Point2<f32>
}
impl BoardLayout {
    pub fn new(center_x: f32, center_y: f32, target_offset: f32, items: usize) -> BoardLayout {
        let hll = HorizontalListLayout {
            transform: Transform {x: center_x, y: center_y, depth: 0},
            x_spacing: 40.0
        };
        BoardLayout {
            hotbar_pos_vec: hll.get_points(items),
            workbench_pos_vec: hll.get_points(3),
            target_pos: Point2 { x: center_x, y: center_y-target_offset}
        }
    }
}
pub struct HorizontalListLayout {
    transform: Transform,
    x_spacing: f32
}
impl HorizontalListLayout {
    pub fn get_points(&self, input_size: usize) -> Vec<Point2<f32>> {
        let mut vec = Vec::new();
        let width = (input_size as f32)*self.x_spacing;
        let left = self.transform.x-width/2.0;
        let mut i = 0.0;
        while i < width {
            vec.push(Point2{ x: left+i, y: self.transform.y });
            i+=self.x_spacing;
        }
        return vec;
    }
}