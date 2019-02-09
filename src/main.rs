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
const YELLOW: [f32; 4] = [1.0, 0.8, 0.0, 1.0];
const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const RESOLUTION: [u32; 2] = [800, 600];

const NOISE: f64 = 0.01;
const FRICTION: f64 = 0.009;
const BOUNCE_FACTOR: f64 = 0.5;
const GRAVITY: f64 = 9.8;

const SQUARE_SIZE_MAX: f64 = 70.0;
const SQUARE_SIZE_MIN: f64 = 25.0;
const JETPACK_COST: f64 = 1.5;
const JETPACK_COOLDOWN: f64 = 0.5;
const JETPACK_COOLDOWN_OVERHEAT: f64 = 0.25;

pub struct App {
    gl: GlGraphics, //OpenGL backend
    position: [f64; 2],
    velocity: [f64; 2],
    square_size: f64,
    cooldown_timer: f64,
    overheated: bool,
    keys: HashSet<Key>, //currently pressed keys
}

impl App {
    fn new(gl: GlGraphics) -> Self {
        App {
            gl: gl,
            position: [(RESOLUTION[0]/2 - 25) as f64, (RESOLUTION[1]/2 - 25) as f64],
            velocity: [3.0, 3.0],
            square_size: 50.0,
            cooldown_timer: 1.0,
            overheated: false,
            keys: HashSet::new()
        }
    }
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;
        //player render info
        let square = rectangle::square(0.0, 0.0, self.square_size);
        let (tx, ty) = (self.position[0], self.position[1]);
        //jetpack render info
        let jetpack_size = [30.0, 80.0];
        let jetpack_fill = -jetpack_size[1] * self.cooldown_timer;
        let jetpack_shape = [0.0, jetpack_size[1], jetpack_size[0], jetpack_fill];
        //if overheated, color entire bar Red
        let jetpack_color = if !self.overheated {
            if self.cooldown_timer >= 0.5 { GREEN } else { YELLOW }
         } else { RED };

        self.gl.draw(args.viewport(), |c, gl| {
            clear([1.0; 4], gl);
            //player
            rectangle(BLUE, square, c.transform.trans(tx, ty), gl);
            //jetpack heat bar
            rectangle(jetpack_color, jetpack_shape, c.transform.trans(10.0, 10.0), gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        self.process_input(args);
        self.apply_physics(args);
        //cooldown jetpack
        if self.cooldown_timer < 1.0 {
            self.cooldown_timer += if self.overheated {
                //cooldown at a slower rate if overheated
                args.dt*JETPACK_COOLDOWN_OVERHEAT
            } else {
                args.dt*JETPACK_COOLDOWN
            };
        } else if self.overheated { //reset flag after cooling down completely
            self.overheated = false;
        }
    }

    fn apply_physics(&mut self, args: &UpdateArgs) {
        //simple gravity
        self.velocity[1] += GRAVITY*args.dt;
        //needed to properly space bounces
        let (future_x, future_y) = (self.position[0]+self.velocity[0], self.position[1]+self.velocity[1]);
        let (max_x, max_y) = ((RESOLUTION[0] as f64)-self.square_size, (RESOLUTION[1] as f64)-self.square_size);
        let (max_x_noise, max_y_noise) = (max_x*(1.0-NOISE), max_y*(1.0-NOISE));
        //bounce if the future position is invalid
        if future_x<0.0 || future_x>max_x {
            self.velocity[0] *= -BOUNCE_FACTOR;
        }
        if future_y>max_y {
            self.velocity[1] *= -BOUNCE_FACTOR;
        }
        //update position and velocity
        self.position[0] += self.velocity[0];
        self.position[1] += self.velocity[1];
        //only apply friction if player is in contact with walls or floor
        if self.position[0]<=0.0 || self.position[0]>=max_x_noise {
                self.velocity[1] -= self.velocity[1] * FRICTION;
        }
        if self.position[1]>=max_y_noise {
                self.velocity[0] -= self.velocity[0] * FRICTION;
        }
    }

    fn process_input(&mut self, args: &UpdateArgs) {
        //if a button is pressed, accelerate at a constant rate
        if self.keys.contains(&Key::Right) {
            self.velocity[0] += 0.1;
        }
        if self.keys.contains(&Key::Left) {
            self.velocity[0] -= 0.1;
        }
        if self.keys.contains(&Key::Up) {
            //use boost to go Up, while cooldown_timer is not empty
            let cooldown_timer_cost = args.dt*JETPACK_COST;
            //simulates weight, based on square size heuristic
            let weight_decay = 1.0+(50.0-self.square_size)*0.02;
            if !self.overheated && self.cooldown_timer-cooldown_timer_cost > 0.0 {
                self.cooldown_timer -= cooldown_timer_cost;
                self.velocity[1] -= 0.15*weight_decay;
            } else if !self.overheated {
                self.overheated = true;
            }
        }
        if self.keys.contains(&Key::Q) {
            //make player larger and heavier
            if self.square_size < SQUARE_SIZE_MAX {
                self.position[1] -= 0.5;
                self.square_size += 0.5;
            }
        }
        if self.keys.contains(&Key::W) {
            //make player smaller and lighter
            if self.square_size > SQUARE_SIZE_MIN {
                self.square_size -= 0.5;
            }
        }
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    let mut window: Window = WindowSettings::new("Jetpack Game", RESOLUTION)
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    //holds state information of our application
    let mut app = App::new(GlGraphics::new(opengl));

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        //use a HashMap to manage button presses
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
