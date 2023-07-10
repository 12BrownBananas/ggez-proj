mod util;

use ggez::*;
use ggez::conf::{WindowMode, WindowSetup};
use ggez::graphics::{Color, Text};
use ggez::input::keyboard::{KeyCode, KeyInput};

use util::input_manager;

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
        let accept_check = self.input_manager.get_input_state(input_manager::InputSemantic::Accept);
        match accept_check {
            Some(state) => {
                match state {
                    input_manager::InputState::Pressed => {println!("Pressed")},
                    input_manager::InputState::Held => {println!("Held")},
                    input_manager::InputState::Released => {println!("Released")},
                    input_manager::InputState::AtRest => {println!("At Rest")}
                }
            },
            None => {}
        }
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
    let accept_key = Box::new(input_manager::KeyboardInputProcessor::new(vec!(ggez::input::keyboard::KeyCode::A, ggez::input::keyboard::KeyCode::Space)));
    state.get_input_manager().register_input(input_manager::InputSemantic::Accept, accept_key);
    let accept_mouse_button = Box::new(input_manager::MouseInputProcessor::new(vec!(ggez::input::mouse::MouseButton::Left)));
    state.get_input_manager().register_input(input_manager::InputSemantic::Accept, accept_mouse_button);
    
    let (ctx, event_loop) = ContextBuilder::new("ggez-proj", "Act-Novel")
        .window_setup(WindowSetup::default().title("ggez project"))
        .window_mode(
            WindowMode::default()
                .dimensions(960.0, 540.0)
                .resizable(true)
        )
        .build()
        .unwrap();

    event::run(ctx, event_loop, state);
}