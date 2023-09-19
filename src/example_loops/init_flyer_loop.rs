mod aircraft;
use aircraft::Aircraft;

mod terrain;

use aerso::types::*;
use ggez::{Context, ContextBuilder, GameResult};
use ggez::event;
use ggez::glam::*;
use ggez::graphics::{self, Color};
use ggez::conf;
use ggez::event::EventHandler;

struct MainState {
    screen: graphics::ScreenImage,
    aircraft: Aircraft,
    screen_width: f32,
    screen_height: f32,
    input: Vec<f64>,
}

impl MainState {

    fn new(ctx: &mut Context) -> GameResult<MainState> {
        
        let screen = graphics::ScreenImage::new(ctx, graphics::ImageFormat::Rgba8UnormSrgb, 1.0, 1.0, 1);
        let aircraft = Aircraft::new(
            "TO",
            Vector3::zeros(),
            Vector3::new(100.0, 0.0, 0.0),
            UnitQuaternion::from_euler_angles(0.0,0.0,0.0),
            Vector3::zeros()
        );
        let (width, height) = ctx.gfx.drawable_size();
        let input = vec![0.0, 0.0, 1.0, 0.0];

        let s = MainState {
            screen,
            aircraft,
            screen_width: width,
            screen_height: height,
            input
        };

        Ok(s)
    }

}

impl EventHandler<ggez::GameError> for MainState {
    
    fn update(&mut self, _ctx: &mut Context) -> GameResult {

        const DESIRED_FPS: u32 = 60;

        while _ctx.time.check_update_time(DESIRED_FPS) {
            let dt = 1.0 / (DESIRED_FPS as f64);

            // Update aircraft state based on inputstate
            self.aircraft.aff_body.step(dt, &self.input);

            println!("States: {}", self.aircraft.statevector());
            println!("Position: {}", self.aircraft.position());
            // Check the end state
            
        }

        Ok(())
    }

    fn draw(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

}


pub fn main() -> GameResult {

    let cb = ContextBuilder::new("flyer-env", "ggez");
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
