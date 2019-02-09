extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use std::collections::HashSet;

const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const RESOLUTION: [u32; 2] = [800, 600];
const FRICTION: f32 = 0.005;
const BOUNCE_FACTOR: f32 = 0.5;
const GRAVITY: f64 = 9.8;
const BOOST_COST: f64 = 1.5;
const BOOST_REGEN: f64 = 0.4;

pub struct App {
    gl: GlGraphics, //OpenGL backend
    player_x: i32,
    player_y: i32,
    player_vx: f32,
    player_vy: f32,
    square_size: f32,
    boost_timer: f64,
    keys: HashSet<Key>, //currently pressed keys
}

impl App {
    fn new(gl: GlGraphics) -> Self {
        App {
            gl: gl,
            player_x: (RESOLUTION[0]/2 - 25) as i32,
            player_y: (RESOLUTION[1]/2 - 25) as i32,
            player_vx: 3.0,
            player_vy: 3.0,
            square_size: 50.0,
            boost_timer: 1.0,
            keys: HashSet::new()
        }
    }
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let square = rectangle::square(0.0, 0.0, self.square_size.into());
        let (player_x, player_y): (f64, f64) = (self.player_x.into(), self.player_y.into());
        let (tx, ty) = (player_x, player_y);
        let jetpack_fill = -50.0 * self.boost_timer as f64;
        let jetpack_color = if self.boost_timer >= 1.0 { GREEN } else { RED };

        self.gl.draw(args.viewport(), |c, gl| {
            clear([1.0; 4], gl);
            let transform = c.transform.trans(tx, ty);
            rectangle(BLUE, square, transform, gl);
            rectangle(jetpack_color, [10.0, 60.0, 30.0, jetpack_fill], c.transform, gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        self.process_input(args);
        //replenish boost
        if self.boost_timer < 1.0 {
            let boost_timer_heal = args.dt*BOOST_REGEN;
            self.boost_timer += boost_timer_heal;
        }
        //gravity
        self.player_vy += (GRAVITY*args.dt) as f32;
        //needed to properly space bounces
        let (future_x, future_y) = (self.player_x+(self.player_vx as i32), self.player_y+(self.player_vy as i32));
        let (max_x, max_y) = ((RESOLUTION[0] as f32)-self.square_size, (RESOLUTION[1] as f32)-self.square_size);
        //bounce if the future position is invalid
        if future_x<0 || future_x>(max_x as i32) {
            self.player_vx *= -BOUNCE_FACTOR;
        }
        if future_y>(max_y as i32) {
            self.player_vy *= -BOUNCE_FACTOR;
        }
        //update position and velocity
        self.player_x += self.player_vx as i32;
        self.player_y += self.player_vy as i32;
        //only apply friction if player is in contact with walls or floor
        let sq_size_i32 = self.square_size as u32;
        if self.player_x<=0 || self.player_x>=(RESOLUTION[0]-sq_size_i32) as i32 {
                self.player_vy -= self.player_vy * FRICTION;
        }
        if self.player_y>=(RESOLUTION[1]-sq_size_i32) as i32 {
                self.player_vx -= self.player_vx * FRICTION;
        }
    }

    fn process_input(&mut self, args: &UpdateArgs) {
        //if a button is pressed, accelerate at a constant rate
        if self.keys.contains(&Key::Right) {
            self.player_vx += 0.1;
        }
        if self.keys.contains(&Key::Left) {
            self.player_vx -= 0.1;
        }
        if self.keys.contains(&Key::Up) {
            let boost_timer_cost = args.dt*BOOST_COST;
            if self.boost_timer-boost_timer_cost > 0.0 {
                self.boost_timer -= boost_timer_cost;
                self.player_vy -= 0.15;
            }
        }
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    let mut window: Window = WindowSettings::new("graphics-test",RESOLUTION)
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    //holds state information of our application
    let mut app = App::new(GlGraphics::new(opengl));

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(Button::Keyboard(key)) = e.press_args() {
            app.keys.insert(key);
        }
        if let Some(Button::Keyboard(key)) = e.release_args() {
            app.keys.remove(&key);
        }
        if let Some(u) = e.update_args() {
            app.update(&u);
        }
        if let Some(r) = e.render_args() {
            app.render(&r);
        }
    }

}
