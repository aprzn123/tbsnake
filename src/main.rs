extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;


use std::collections::VecDeque;

const GRID_SIZE_X: u16 = 20;
const GRID_SIZE_Y: u16 = 20;
const TILE_SIZE: u16 = 30;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right
}

impl Direction {
    fn opposite(a: Direction, b: Direction) -> bool {
        match a {
            Direction::Up => b == Direction::Down,
            Direction::Down => b == Direction::Up,
            Direction::Left => b == Direction::Right,
            Direction::Right => b == Direction::Left
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct GridPosition { x: i32, y: i32}

impl GridPosition {
    fn new(x: i32, y: i32) -> GridPosition { GridPosition { x, y } }
}

struct Snake { pos: VecDeque<GridPosition>, facing: Direction, remaining_extensions: i32 }

impl Snake {
    fn new(pos: VecDeque<GridPosition>, facing: Direction) -> Self {
        Snake { pos, facing, remaining_extensions: 0 }
    }
    fn point(&mut self, dir: Direction) -> bool {
        if !Direction::opposite(self.facing, dir) || self.pos.len() == 1 {
            self.facing = dir; 
            return true
        }
        false
    }

    fn draw<T: sdl2::render::RenderTarget>(&self, canvas: &mut sdl2::render::Canvas<T>) {
        canvas.set_draw_color(Color::RGB(200, 0, 0));
        // for i in self.pos.iter() {
        //     canvas.fill_rect(sdl2::rect::Rect::new(
        //         i.x * TILE_SIZE as i32 + 2,
        //         i.y * TILE_SIZE as i32 + 2,
        //         (TILE_SIZE - 4) as u32,
        //         (TILE_SIZE - 4) as u32
        //     )).unwrap();
        // }
        for i in 0..(self.pos.len() - 1) {
            canvas.draw_line(
                sdl2::rect::Point::new(
                    self.pos[i].x * TILE_SIZE as i32 + TILE_SIZE as i32 / 2,
                    self.pos[i].y * TILE_SIZE as i32 + TILE_SIZE as i32 / 2,
                ), 
                sdl2::rect::Point::new(
                    self.pos[i + 1].x * TILE_SIZE as i32 + TILE_SIZE as i32 / 2,
                    self.pos[i + 1].y * TILE_SIZE as i32 + TILE_SIZE as i32 / 2,
                ),
            ).unwrap();
        }
    }

    fn forward(&mut self, dist: u32) {
        for _ in 0..dist {
            let front = self.pos.front().unwrap().clone();
            self.pos.push_front(match self.facing {
                Direction::Up => GridPosition { x: front.x, y: front.y - 1 },
                Direction::Down => GridPosition { x: front.x, y: front.y + 1 },
                Direction::Left => GridPosition { x: front.x - 1, y: front.y },
                Direction::Right => GridPosition { x: front.x + 1, y: front.y },
            });
            if self.remaining_extensions == 0 {
                self.pos.pop_back();
            } else {
                self.remaining_extensions -= 1;
            }
        }
    }

    fn extend(&mut self, i: i32) {
        self.remaining_extensions += i;
    }

    fn head(&self) -> GridPosition {
        self.pos.front().unwrap().clone()
    }

    fn step(&mut self) {
        self.forward(1);
    }
}

struct Food {
    pos: GridPosition,
    power: i32
}

impl Food {
    fn new(pos: GridPosition, power: i32) -> Food {
        Food { pos, power }
    }

    fn randomize(power: i32) -> Food {
        Food { pos: GridPosition {
            x: StdRng::from_entropy().gen_range(0..GRID_SIZE_X) as i32,
            y: StdRng::from_entropy().gen_range(0..GRID_SIZE_Y) as i32,
        }, power }
    }

    fn eaten(&self, snake: &mut Snake) {
        snake.extend(self.power);
    }
    
    fn draw<T: sdl2::render::RenderTarget>(&self, canvas: &mut sdl2::render::Canvas<T>) {
        canvas.set_draw_color(Color::RGB(0, 0, 150));
        canvas.fill_rect(sdl2::rect::Rect::new(
            self.pos.x * TILE_SIZE as i32 + 2,
            self.pos.y * TILE_SIZE as i32 + 2,
            (TILE_SIZE - 4) as u32,
            (TILE_SIZE - 4) as u32
        )).unwrap();
    }
}

struct GameState {
    player: Snake,
    food: Vec<Food>,
}

impl GameState {
    fn step(&mut self) {
        self.player.step();
        let mut to_replace: Vec<usize> = Vec::new();
        for i in 0..self.food.len() {
            let food = &self.food[i];
            if food.pos == self.player.head() {
                food.eaten(&mut self.player);
                to_replace.push(i);
            }
        }
        for i in to_replace {
            self.food[i] = Food::randomize(1);
        }
    }

    fn draw<T: sdl2::render::RenderTarget>(&self, canvas: &mut sdl2::render::Canvas<T>) {
        self.player.draw(canvas);
        for food in self.food.iter() {
            food.draw(canvas);
        }
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Turn-Based Snake", (TILE_SIZE * GRID_SIZE_X) as u32, (TILE_SIZE * GRID_SIZE_Y) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut game = GameState {
        player: Snake::new(VecDeque::from([GridPosition::new(10, 10)]), Direction::Right),
        food: vec![Food::randomize(1),Food::randomize(1),Food::randomize(1),]
    };

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        let next_frame_time = std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
        canvas.set_draw_color(Color::RGB(100, 200, 0));
        canvas.clear();
        
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => { break 'running },
                Event::KeyDown { keycode, .. } => match keycode {
                    None => {},
                    Some(k) => match k {
                        Keycode::Escape => { break 'running },
                        Keycode::Up => { if game.player.point(Direction::Up) {game.step();} },
                        Keycode::Down => { if game.player.point(Direction::Down) {game.step();} },
                        Keycode::Left => { if game.player.point(Direction::Left) {game.step();} },
                        Keycode::Right => { if game.player.point(Direction::Right) {game.step();} },
                        _ => { }
                    }
                },
                _ => { },
            }
        }

        game.draw(&mut canvas);
        canvas.present();
        std::thread::sleep(next_frame_time.duration_since(std::time::Instant::now()));
    }
}
