//! Evolution will be a 2D genetic algorithm simulation.
//!
//! A `Vehicle` will first spawn at a random location in the window.
//! The `Vehicle` will then go on to `seek` the cursor's location.

use std::{
    fs::File,
    path::PathBuf,
};
use ggez::{
    conf,
    event,
    graphics,
    nalgebra::Point2,
    Context,
    ContextBuilder,
    GameResult,
    timer,
};
use rand::Rng;
use ron::de::from_reader;
use serde::Deserialize;

use evolution::food::{FoodConfig, Food};
use evolution::vehicle::{VehicleConfig, Vehicle};

#[derive(Debug, Deserialize)]
struct Config {
    fullscreen: bool,
    window_size: (f32, f32),
    desired_fps: u32,
    show_fps: bool,
    vehicle: VehicleConfig,
    food: FoodConfig,
    poison: FoodConfig,
}

/// The application state that keeps track of the current state of vehicles and food
struct State {
    config: Config,
    /// A collection of vehicles
    vehicles: Vec<Vehicle>,
    /// A collection of food
    food: Vec<Food>,
    /// A collection of poison
    poison: Vec<Food>,
    vehicle_image: graphics::Image,
}

impl State {
    /// Creates a new instance of the application state
    fn new(ctx: &mut Context, config: Config) -> GameResult<State> {
        // Random number generator is used for the location of the vehicle and its angle
        let mut rng = rand::thread_rng();

        let mut vehicles = Vec::new();

        // The non-default attributes of the vehicle that are to be specified before-hand
        for _ in 1..config.vehicle.quantity {
            let size = rng.gen_range(
                config.vehicle.size_range.0,
                config.vehicle.size_range.1
            );
            let max_speed = map_range(
                size,
                config.vehicle.size_range,
                config.vehicle.max_speed_range
            );
            let max_steering_force = map_range(
                size,
                config.vehicle.size_range,
                config.vehicle.max_steering_force_range
            );
            let angle = rng.gen_range(0.0, 2.0 * std::f32::consts::PI);
            let pos = Point2::new(
                rng.gen_range(0.0, config.window_size.0),
                rng.gen_range(0.0, config.window_size.1),
            );
            let dna = [
                rng.gen_range(-5.0, 5.0),
                rng.gen_range(-5.0, 5.0),
            ];

            vehicles.push(Vehicle::new(
                size,
                max_speed,
                max_steering_force,
                pos,
                angle,
                dna
            ));
        }

        let mut food = Vec::new();
        for _ in 1..config.food.quantity {
            food.push(Food {
                size: rng.gen_range(
                          config.food.size_range.0,
                          config.food.size_range.1
                      ),
                pos: Point2::new(
                    rng.gen_range(0.0, config.window_size.0),
                    rng.gen_range(0.0, config.window_size.1)
                ),
                color: /*[
                    rng.gen_range(0.0, 1.0),
                    rng.gen_range(0.0, 1.0),
                    rng.gen_range(0.0, 1.0),
                    0.8
                ]*/[0.0, 1.0, 0.0, 0.8]
            });
        }

        let mut poison = Vec::new();
        for _ in 1..config.poison.quantity {
            poison.push(Food {
                size: rng.gen_range(
                          config.poison.size_range.0,
                          config.poison.size_range.1
                      ),
                pos: Point2::new(
                    rng.gen_range(0.0, config.window_size.0),
                    rng.gen_range(0.0, config.window_size.1)
                ),
                color: [1.0, 0.0, 0.0, 0.8]
            });
        }

        let mut vehicle_image = graphics::Image::new(ctx, "/frames.png").unwrap();
        vehicle_image.set_filter(graphics::FilterMode::Nearest);

        Ok(State {
            config,
            vehicles,
            food,
            poison,
            vehicle_image,
        })
    }
}

impl event::EventHandler for State {
    /// Updates all elements of the current application state
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while timer::check_update_time(ctx, self.config.desired_fps) {
            for vehicle in self.vehicles.iter_mut() {
                vehicle.behaviors(&mut self.food, &mut self.poison);
                vehicle.update();
            }
        }

        Ok(())
    }

    /// Draws all elements of the current application state
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        for poison in self.poison.iter() {
            if let Err(error) = poison.draw(ctx) {
                return Err(error);
            }
        }

        for food in self.food.iter() {
            if let Err(error) = food.draw(ctx) {
                return Err(error);
            }
        }

        for vehicle in self.vehicles.iter_mut() {
            if let Err(error) = vehicle.draw(
                ctx,
                &self.vehicle_image,
                self.config.vehicle.max_speed_range.1
            ) {
                return Err(error);
            }
        }

        if self.config.show_fps {
            let fps = timer::fps(ctx);
            let fps_text = graphics::Text::new(format!("FPS: {:.*}", 1, fps));
            graphics::draw(ctx, &fps_text, (Point2::new(5.0, 5.0), graphics::WHITE))?;
        }

        graphics::present(ctx)?;

        Ok(())
    }
}

/// The main function :D
pub fn main() -> GameResult {
    let input_path = format!("{}/config.ron", env!("CARGO_MANIFEST_DIR"));
    let f = File::open(&input_path)?;
    let config: Config = match from_reader(f) {
        Ok(x) => x,
        Err(e) => {
            println!("Failed to load `config.ron`: {}", e);
            std::process::exit(1);
        }
    };

    let assets_dir = PathBuf::from(format!("{}/assets", env!("CARGO_MANIFEST_DIR")));

    let window_settings = if config.fullscreen {
        conf::WindowMode::default().fullscreen_type(conf::FullscreenType::True)
    } else {
        conf::WindowMode::default().dimensions(config.window_size.0, config.window_size.1)
    };

    let (ctx, event_loop) = &mut ContextBuilder::new("evolution", "Austin Baugh")
        .window_setup(conf::WindowSetup::default().title("Evolution!"))
        .window_mode(window_settings)
        .add_resource_path(assets_dir)
        .build()?;

    let state = &mut State::new(ctx, config)?;

    event::run(ctx, event_loop, state)
}

fn map_range(value: f32, range1: (f32, f32), range2: (f32, f32)) -> f32 {
    range2.0
        + (range2.1 - range2.0)
        * ((value - range1.0) / (range1.1 - range1.0))
}
