// use ggez::{Context, ContextBuilder, GameResult};
// use ggez::graphics::{self, Color};
// use ggez::event::{self, ControlFlow};
// use ggez::event::winit_event::{Event, WindowEvent};
// // use winit::event_loop::ControlFlow;

// struct MyGame {
//     global_step: u32,
//     draw_step: u32,
//     update_step: u32,
//     draw_rate: u32,
//     update_rate: u32,
// }

// impl MyGame {
//     pub fn new(_ctx: &mut Context) -> MyGame {
//         // Load/create resources such as images here.
//         MyGame {
//             global_step: 0,
//             draw_step: 0,
//             update_step: 0,
//             draw_rate: 10,
//             update_rate: 1
//         }
//     }
// }


// pub fn main() -> GameResult {

//     let (mut ctx, event_loop) = ContextBuilder::new("my_game", "Cool Game Author")
//         .build()
//         .expect("aieee, could not create ggez context!");

//     let mut my_game = MyGame::new(&mut ctx);

//     // Handle events. Refer to `winit` docs for more information.
//     event_loop.run(move |mut event, _window_target, control_flow| {
        
//         let ctx = &mut ctx;

//         if ctx.quit_requested {
//             ctx.continuing = false;
//         }

//         if !ctx.continuing {
//             *control_flow = ControlFlow::Exit;
//             return;
//         }
        
//         *control_flow = ControlFlow::Poll;

//         event::process_event(ctx, &mut event);
        
//         match event {
//             Event::WindowEvent {event, .. } => match event {
//                 WindowEvent::CloseRequested => ctx.request_quit(),
//                 _ => ()
//             },
//             Event::MainEventsCleared => {
//                 ctx.time.tick();
                
//                 // Update
//                 my_game.global_step += 1;
//                 my_game.update_step += 1;

//                 if my_game.update_step >= my_game.update_rate {
//                     println!("update self.global_step: {}", my_game.global_step);
//                     my_game.update_step = 0;
//                 }

//                 // Draw
//                 let canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);
        
//                 my_game.draw_step += 1;

//                 if my_game.draw_step >= my_game.draw_rate {
//                     println!("draw self.global_step: {}", my_game.global_step);
//                     my_game.draw_step = 0;
//                 }

//                 let _ = canvas.finish(ctx);

//             }, 
//             _ => ()
//         }
//     })
// }