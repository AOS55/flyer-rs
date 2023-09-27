use ggez::{Context, ContextBuilder, GameResult};
use ggez::graphics::{self, Color};
use ggez::event::{self, EventHandler};

fn main() {
    // Make a Context.
    let (mut ctx, event_loop) = ContextBuilder::new("my_game", "Cool Game Author")
        .build()
        .expect("aieee, could not create ggez context!");

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let my_game = MyGame::new(&mut ctx);

    // Run!
    event::run(ctx, event_loop, my_game);
}

struct MyGame {
    global_step: u32,
    draw_step: u32,
    update_step: u32,
    draw_rate: u32,
    update_rate: u32,
}

impl MyGame {
    pub fn new(_ctx: &mut Context) -> MyGame {
        // Load/create resources such as images here.
        MyGame {
            global_step: 0,
            draw_step: 0,
            update_step: 0,
            draw_rate: 10,
            update_rate: 1
        }
    }
}

impl EventHandler for MyGame {

    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        
        self.global_step += 1;
        self.update_step += 1;

        if self.update_step >= self.update_rate {
            println!("update self.global_step: {}", self.global_step);
            self.update_step = 0;
        }

        Ok(())

    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);
        
        self.draw_step += 1;

        if self.draw_step >= self.draw_rate {
            println!("draw self.global_step: {}", self.global_step);
            self.draw_step = 0;
        }

        canvas.finish(ctx)
    }
}
