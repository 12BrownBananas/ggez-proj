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
    display_dt: bool
}
impl GameState {
    fn new() -> GameState {
        GameState {
            dt: std::time::Duration::new(0, 0),
            dt_format: DeltaTimeFormat::Nanos,
            dt_text_display: Text::new(""),
            display_dt: true,
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
}
impl ggez::event::EventHandler<GameError> for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;
        while ctx.time.check_update_time(DESIRED_FPS) {}
        self.dt = ctx.time.delta();
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
}

fn main() {
    let state = GameState::new();
    
    let mut input_manager = input_manager::InputManager::new();
    let mut accept_key = Box::new(input_manager::KeyboardInputProcessor::new(vec!(ggez::input::keyboard::KeyCode::A, ggez::input::keyboard::KeyCode::Space)));
    input_manager.register_input(input_manager::InputSemantic::Accept, &mut accept_key);
    
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
