mod util;
mod state;

use ggez::*;
use ggez::conf::{WindowMode, WindowSetup};
use ggez::graphics::Color;

use fraction::Fraction;

use util::data_generator;

use state::game_object;
use state::state::GameState;

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

