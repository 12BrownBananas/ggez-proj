mod util;
mod components;

use std::ptr::eq;

use ggez::*;
use ggez::conf::{WindowMode, WindowSetup};
use ggez::graphics::{Canvas, Color};
use ggez::input::keyboard::KeyInput;

use util::input_manager;
use util::data_generator;

use components::game_object;
use fraction::Fraction;

enum ObjectGroup {
    Controllable,
    General
}
struct RenderInstruction {
    group: ObjectGroup,
    index: usize,
    depth: i32
}

struct GameState {
    input_manager: util::input_manager::InputManager,
    objects: Vec<Box<dyn game_object::GameObject>>,
    controllables: Vec<Box<dyn game_object::ControllableGameObject>>
}
impl GameState {
    fn new() -> GameState {
        GameState {
            input_manager: get_any4_input_manager(),
            objects: Vec::new(),
            controllables: Vec::new()
        }
    }
    fn collect_render_instructions(&self) -> Vec<RenderInstruction> {
        let mut instructions = Vec::new();
        let mut i: usize = 0;
        while i < self.objects.len() {
            let o = self.objects.get(i).expect("");
            instructions.push(RenderInstruction { group: ObjectGroup::General, index: i, depth:  o.get_depth() });
            i+=1;
        }
        let mut i: usize = 0;
        while i < self.controllables.len() {
            let o = self.controllables.get(i).expect("");
            instructions.push(RenderInstruction { group: ObjectGroup::Controllable, index: i, depth: o.get_depth() });
            i+=1;
        }
        instructions.sort_unstable_by(|a, b| a.depth.cmp(&b.depth));
        return instructions;
    }
    fn process_render_instruction(&mut self, instruction: RenderInstruction, canvas: &mut Canvas) {
        match instruction.group {
            ObjectGroup::Controllable => {
                let o = self.controllables.get_mut(instruction.index).expect("");
                o.draw(canvas);
            },
            ObjectGroup::General => {
                let o = self.objects.get_mut(instruction.index).expect("");
                o.draw(canvas);
            }
        }
    }
    fn add_object(&mut self, obj: Box<dyn game_object::GameObject>) -> &Box<dyn game_object::GameObject> {
        let idx = self.objects.len();
        self.objects.push(obj);
        return self.objects.get(idx).unwrap();
    }
    fn remove_object(&mut self, obj: &Box<dyn game_object::GameObject>) {
        let index = self.objects.iter().position(|x| eq(x, obj));
        match index {
            Some(idx) => { 
                self.objects.remove(idx);
            },
            None => {}
        }
    }
    fn add_controllable(&mut self, obj: Box<dyn game_object::ControllableGameObject>) -> &Box<dyn game_object::ControllableGameObject> {
        let idx = self.controllables.len();
        self.controllables.push(obj);
        return self.controllables.get(idx).unwrap();
    }
    fn remove_controllable(&mut self, obj: &Box<dyn game_object::ControllableGameObject>) {
        let index = self.controllables.iter().position(|x| eq(x, obj));
        match index {
            Some(idx) => { 
                self.objects.remove(idx);
            },
            None => {}
        }
    }
}
impl ggez::event::EventHandler<GameError> for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;
        while ctx.time.check_update_time(DESIRED_FPS) {}
        for o in self.controllables.as_mut_slice() {
            o.process_input(&self.input_manager);
            o.as_mut_game_object().update();
        }
        for o in self.objects.as_mut_slice() {
            o.update();
        }
        self.input_manager.process_input();
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::from([0.1, 0.2, 0.3, 1.0]));
        for i in self.collect_render_instructions() {
            self.process_render_instruction(i, &mut canvas);
        }
        canvas.finish(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        input: KeyInput,
        _repeated: bool
    ) -> Result<(), GameError> {
        if !_repeated {
            match input.keycode {
                Some(key) => { self.input_manager.process_input_pressed(input_manager::InputType::Keyboard(key)); },
                None => {}
            }
        }
        Ok(())
    }
    fn key_up_event(
        &mut self,
        _ctx: &mut Context,
        input: KeyInput
    ) -> Result<(), GameError> {
        match input.keycode {
            Some(key) => { self.input_manager.process_input_released(input_manager::InputType::Keyboard(key)); },
            None => {}
        }
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: ggez::input::mouse::MouseButton,
        _x: f32,
        _y: f32,
    ) -> Result<(), GameError> {
        self.input_manager.process_input_pressed(input_manager::InputType::Mouse(_button));
        Ok(())
    }

    /// A mouse button was released
    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        _button: ggez::input::mouse::MouseButton,
        _x: f32,
        _y: f32,
    ) -> Result<(), GameError> {
        self.input_manager.process_input_released(input_manager::InputType::Mouse(_button));
        Ok(())
    }
}


fn main() {
    let mut state  = GameState::new();
    data_generator::init(1, 10, 4, false);
    let config = data_generator::SetConfig::new(10, None, Some(data_generator::value_is_positive_integer), vec!(data_generator::InputDifficulty::Easy, data_generator::InputDifficulty::Moderate, data_generator::InputDifficulty::Hard));
    
    let pool_map = data_generator::get_deserialized_input_data_pool_map().expect("");
    let res = data_generator::get_set_of_inputs(pool_map.clone(), &config);
    
    match res {
        Ok(set) => {
            println!("Generated set of boards: {{");
            let mut xpos = 0.0;
            let ypos = 10.0;
            for board in set {
                println!("   {:?}", board.get_board_info());
                for b in board.input {
                    state.add_object(Box::new(game_object::VisibleNumber::new(Fraction::from(b), xpos, ypos, 0, Color::WHITE))); //just for example
                    xpos+=20.0;
                }
            }
            println!("}}");
        },
        Err(e) => {println!("{}", e)}
    }
    state.add_controllable(Box::new(game_object::GameController::new(game_object::BoardContainer::new(pool_map, config))));

    /* Main game loop */
    let (ctx, event_loop) = ContextBuilder::new("any4", "Act-Novel")
        .window_setup(WindowSetup::default().title("Any4"))
        .window_mode(
            WindowMode::default()
                .dimensions(960.0, 540.0)
                .resizable(true)
        )
        .build()
        .unwrap();

    event::run(ctx, event_loop, state);
}

fn get_any4_input_manager() -> input_manager::InputManager {
    let mut manager = util::input_manager::InputManager::new();
    /* Control initialization */
    //Accept
    manager.register_input(
        input_manager::InputSemantic::Accept, 
        Box::new(input_manager::KeyboardInputProcessor::new(
            vec!(
                ggez::input::keyboard::KeyCode::Return, 
                ggez::input::keyboard::KeyCode::Space, 
                ggez::input::keyboard::KeyCode::NumpadEnter
            )
        ))
    );
    manager.register_input(
        input_manager::InputSemantic::Accept, 
        Box::new(input_manager::MouseInputProcessor::new(
            vec!(ggez::input::mouse::MouseButton::Left)
        ))
    );
    //Back
    manager.register_input(
        input_manager::InputSemantic::Back, 
        Box::new(input_manager::KeyboardInputProcessor::new(
            vec!(ggez::input::keyboard::KeyCode::Back)
        ))
    );
    manager.register_input(
        input_manager::InputSemantic::Back, 
        Box::new(input_manager::MouseInputProcessor::new(
            vec!(ggez::input::mouse::MouseButton::Right)
        ))
    );
    //Directional input
    manager.register_input(
        input_manager::InputSemantic::Up, 
        Box::new(input_manager::KeyboardInputProcessor::new(
            vec!(
                ggez::input::keyboard::KeyCode::W,
                ggez::input::keyboard::KeyCode::Up
            )
        ))
    );
    manager.register_input(
        input_manager::InputSemantic::Down, 
        Box::new(input_manager::KeyboardInputProcessor::new(
            vec!(
                ggez::input::keyboard::KeyCode::S, 
                ggez::input::keyboard::KeyCode::Down
            )
        ))
    );
    manager.register_input(
        input_manager::InputSemantic::Left, 
        Box::new(input_manager::KeyboardInputProcessor::new(
            vec!(
                ggez::input::keyboard::KeyCode::A, 
                ggez::input::keyboard::KeyCode::Left
            )
        ))
    );
    manager.register_input(
        input_manager::InputSemantic::Right, 
        Box::new(input_manager::KeyboardInputProcessor::new(
            vec!(
                ggez::input::keyboard::KeyCode::D, 
                ggez::input::keyboard::KeyCode::Right
            )
        ))
    );
    //Hotkeys
    manager.register_input(
        input_manager::InputSemantic::Plus, 
        Box::new(input_manager::KeyboardInputProcessor::new(
            vec!(
                ggez::input::keyboard::KeyCode::Plus
            )
        ))
    );
    manager.register_input(
        input_manager::InputSemantic::Minus, 
        Box::new(input_manager::KeyboardInputProcessor::new(
            vec!(
                ggez::input::keyboard::KeyCode::Minus
            )
        ))
    );
    manager.register_input(
        input_manager::InputSemantic::Multiply,
        Box::new(input_manager::KeyboardInputProcessor::new(
            vec!(
                ggez::input::keyboard::KeyCode::X,
                ggez::input::keyboard::KeyCode::Asterisk,
                ggez::input::keyboard::KeyCode::NumpadMultiply
            )
        ))
    );
    manager.register_input(
        input_manager::InputSemantic::Divide, 
        Box::new(input_manager::KeyboardInputProcessor::new(
            vec!(
                ggez::input::keyboard::KeyCode::Slash,
                ggez::input::keyboard::KeyCode::NumpadDivide
            )
        ))
    );
    manager.register_input(
        input_manager::InputSemantic::Hotbar1,
        Box::new(input_manager::KeyboardInputProcessor::new(
            vec!(
                ggez::input::keyboard::KeyCode::Numpad1,
                ggez::input::keyboard::KeyCode::Key1
            )
        ))
    );
    manager.register_input(
        input_manager::InputSemantic::Hotbar2,
        Box::new(input_manager::KeyboardInputProcessor::new(
            vec!(
                ggez::input::keyboard::KeyCode::Numpad2,
                ggez::input::keyboard::KeyCode::Key2
            )
        ))
    );
    manager.register_input(
        input_manager::InputSemantic::Hotbar3,
        Box::new(input_manager::KeyboardInputProcessor::new(
            vec!(
                ggez::input::keyboard::KeyCode::Numpad3,
                ggez::input::keyboard::KeyCode::Key3
            )
        ))
    );
    manager.register_input(
        input_manager::InputSemantic::Hotbar4,
        Box::new(input_manager::KeyboardInputProcessor::new(
            vec!(
                ggez::input::keyboard::KeyCode::Numpad4,
                ggez::input::keyboard::KeyCode::Key4
            )
        ))
    );
    return manager;
}