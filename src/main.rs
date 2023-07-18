mod util;

use ggez::*;
use ggez::conf::{WindowMode, WindowSetup};
use ggez::graphics::{Color, Text};
use ggez::input::keyboard::{KeyCode, KeyInput};

use util::input_manager;
use util::data_generator;

enum DeltaTimeFormat {
    Nanos,
    Micros,
    Millis,
    Secs
}

struct GameState {
    dt: std::time::Duration,
    dt_format: DeltaTimeFormat,
    dt_text_display: Text,
    display_dt: bool,
    input_manager: util::input_manager::InputManager,
}
impl GameState {
    fn new() -> GameState {
        GameState {
            dt: std::time::Duration::new(0, 0),
            dt_format: DeltaTimeFormat::Nanos,
            dt_text_display: Text::new(""),
            display_dt: true,
            input_manager: util::input_manager::InputManager::new()
        }
    }
    pub fn get_dt_string(&self) -> String {
        let delta_time_string = match self.dt_format {
            DeltaTimeFormat::Micros => self.dt.as_micros(),
            DeltaTimeFormat::Millis => self.dt.as_millis(),
            DeltaTimeFormat::Nanos => self.dt.as_nanos(),
            DeltaTimeFormat::Secs => self.dt.as_secs().into(),
        };
        let delta_time_string_suffix = match self.dt_format {
            DeltaTimeFormat::Micros => "µs",
            DeltaTimeFormat::Millis => "ms",
            DeltaTimeFormat::Nanos => "ns",
            DeltaTimeFormat::Secs => "s",
        };
        return format!("{}{}", delta_time_string, delta_time_string_suffix);
    }
    fn get_input_manager(&mut self) -> &mut input_manager::InputManager {
        &mut self.input_manager
    }
}
impl ggez::event::EventHandler<GameError> for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;
        while ctx.time.check_update_time(DESIRED_FPS) {}
        self.dt = ctx.time.delta();
        /* You can check the input status from anywhere like this, not just the update loop */
        if self.input_manager.get_input_state(input_manager::InputSemantic::Accept) == input_manager::InputState::Pressed { println!("Accept Pressed") }
        if self.input_manager.get_input_state(input_manager::InputSemantic::Back) == input_manager::InputState::Pressed { println!("Back Pressed") }
        if self.input_manager.get_input_state(input_manager::InputSemantic::Up) == input_manager::InputState::Pressed { println!("Up Pressed") }
        if self.input_manager.get_input_state(input_manager::InputSemantic::Down) == input_manager::InputState::Pressed { println!("Down Pressed") }
        if self.input_manager.get_input_state(input_manager::InputSemantic::Left) == input_manager::InputState::Pressed { println!("Left Pressed") }
        if self.input_manager.get_input_state(input_manager::InputSemantic::Right) == input_manager::InputState::Pressed { println!("Right Pressed") }
        self.input_manager.process_input();

        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::from([0.1, 0.2, 0.3, 1.0]));
        if self.display_dt {
            self.dt_text_display.clear();
            self.dt_text_display.add(self.get_dt_string());
            canvas.draw(&self.dt_text_display, graphics::DrawParam::from([200.0, 0.0]).color(Color::WHITE),);
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
            self.input_manager.process_input_pressed(input_manager::InputType::Keyboard(input.keycode.unwrap()));
        }
        if input.keycode == Some(KeyCode::Escape) {
            self.dt_format = match self.dt_format {
                DeltaTimeFormat::Micros => DeltaTimeFormat::Millis,
                DeltaTimeFormat::Millis => DeltaTimeFormat::Nanos,
                DeltaTimeFormat::Nanos => DeltaTimeFormat::Secs,
                DeltaTimeFormat::Secs => DeltaTimeFormat::Micros,
            }
        }
        Ok(())
    }
    fn key_up_event(
        &mut self,
        _ctx: &mut Context,
        input: KeyInput
    ) -> Result<(), GameError> {
        self.input_manager.process_input_released(input_manager::InputType::Keyboard(input.keycode.unwrap()));
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

    /* Control initialization */
    //Accept
    let accept_key = Box::new(input_manager::KeyboardInputProcessor::new(vec!(ggez::input::keyboard::KeyCode::Return, ggez::input::keyboard::KeyCode::Space, ggez::input::keyboard::KeyCode::NumpadEnter)));
    state.get_input_manager().register_input(input_manager::InputSemantic::Accept, accept_key);
    let accept_mouse_button = Box::new(input_manager::MouseInputProcessor::new(vec!(ggez::input::mouse::MouseButton::Left)));
    state.get_input_manager().register_input(input_manager::InputSemantic::Accept, accept_mouse_button);
    //Back
    let back_key = Box::new(input_manager::KeyboardInputProcessor::new(vec!(ggez::input::keyboard::KeyCode::Back)));
    state.get_input_manager().register_input(input_manager::InputSemantic::Back, back_key);
    let back_mouse_button = Box::new(input_manager::MouseInputProcessor::new(vec!(ggez::input::mouse::MouseButton::Right)));
    state.get_input_manager().register_input(input_manager::InputSemantic::Back, back_mouse_button);
    //Directional input
    let up_key = Box::new(input_manager::KeyboardInputProcessor::new(vec!(ggez::input::keyboard::KeyCode::W, ggez::input::keyboard::KeyCode::Up)));
    state.get_input_manager().register_input(input_manager::InputSemantic::Up, up_key);
    let down_key = Box::new(input_manager::KeyboardInputProcessor::new(vec!(ggez::input::keyboard::KeyCode::S, ggez::input::keyboard::KeyCode::Down)));
    state.get_input_manager().register_input(input_manager::InputSemantic::Down, down_key);
    let left_key = Box::new(input_manager::KeyboardInputProcessor::new(vec!(ggez::input::keyboard::KeyCode::A, ggez::input::keyboard::KeyCode::Left)));
    state.get_input_manager().register_input(input_manager::InputSemantic::Left, left_key);
    let right_key = Box::new(input_manager::KeyboardInputProcessor::new(vec!(ggez::input::keyboard::KeyCode::D, ggez::input::keyboard::KeyCode::Right)));
    state.get_input_manager().register_input(input_manager::InputSemantic::Right, right_key);

    data_generator::init(1, 10, 4, false);
    let res = data_generator::get_input_set(data_generator::SetConfig{size: 5, target: "".to_string(), validator: Some(data_generator::value_is_positive_integer), difficulties: vec!(data_generator::InputDifficulty::Easy, data_generator::InputDifficulty::Moderate, data_generator::InputDifficulty::Hard)});
    match res {
        Ok(set) => {
            println!("Generated set of boards: {{");
            for board in set {
                println!("   {:?}", board.get_board_info())
            }
            println!("}}");
        },
        Err(e) => {println!("{}", e)}
    }

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
