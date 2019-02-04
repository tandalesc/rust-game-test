extern crate piston_window;

use piston_window::*;

const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const RESOLUTION: [u32; 2] = [800, 600];

fn main() {
    let mut window: PistonWindow =
        WindowSettings::new("Game Test", RESOLUTION)
        .exit_on_esc(true).build().unwrap();
    while let Some(event) = window.next() {
        window.draw_2d(&event, |_c, g| {
            clear([1.0; 4], g);
            ellipse(RED, [0.0, 0.0, 100.0, 100.0], _c.transform, g);
            //rectangle(RED, [0.0, 0.0, 100.0, 100.0], _c.transform, g);
        });
    }
}
