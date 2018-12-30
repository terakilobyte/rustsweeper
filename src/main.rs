//! Basic hello world example.

use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{self, MouseButton};
use ggez::graphics;
use ggez::graphics::Point2;
use ggez::{Context, ContextBuilder, GameResult};
use rand::prelude::*;
use std::collections::VecDeque;
use std::{env, path};

const CELL_SIDE: f32 = 80.;
const SPACING: f32 = 80.;
const BOARD_SIDE: u8 = 9;
const CTRL_PANEL_WIDTH: f32 = 350.;
const PHI: f32 = 1.618;

enum GameOverState {
    Solved,
    Failed,
}

#[derive(Clone)]
struct Cell {
    position: Point2,
    rust_count: u8,
    is_rust: bool,
    is_hidden: bool,
    game_over: bool,
    is_flagged: bool,
}

impl Cell {
    fn new(position: Point2, is_rust: bool) -> Self {
        Cell {
            position,
            rust_count: 0,
            is_rust,
            is_hidden: true,
            game_over: false,
            is_flagged: false,
        }
    }

    fn calculate_rust_count(&mut self, board: &Vec<Vec<Cell>>) {
        let mut rusts_found = 0;
        'outer: for i in -1..=1 {
            for j in -1..=1 {
                if self.position[0] == 0. && i == -1
                    || self.position[0] == BOARD_SIDE as f32 - 1. && i == 1
                {
                    continue 'outer;
                }
                if self.position[1] == 0. && j == -1
                    || self.position[1] == BOARD_SIDE as f32 - 1. && j == 1
                {
                    continue;
                }
                if i != 0 || j != 0 {
                    if board[(self.position[0] as i8 + i) as usize]
                        [(self.position[1] as i8 + j) as usize]
                        .is_rust
                    {
                        rusts_found += 1;
                    }
                }
            }
        }
        self.rust_count = rusts_found;
    }
}
// First we make a structure to contain the game's state
struct MainState {
    frames: usize,
    board: Vec<Vec<Cell>>,
    image: graphics::Image,
    flag: graphics::Image,
    font: graphics::Font,
    happy_image: graphics::Image,
    game_over: Option<GameOverState>,
    did_sleep: bool,
    did_reveal: bool,
    first_click: bool,
    reset_location: (f32, f32),
    reset_dims: (f32, f32),
}

fn spawn_board() -> Vec<Vec<Cell>> {
    let mut rng = rand::thread_rng();
    let mut starting_states: Vec<bool> = (0..=81).map(|n| n % 9 == 0).collect();
    starting_states.shuffle(&mut rng);
    let mut board = vec![];
    for i in 0..9 {
        let mut inner = vec![];
        for j in 0..9 {
            let cell = Cell::new(Point2::new(i as f32, j as f32), starting_states[i * 9 + j]);
            inner.push(cell);
        }
        board.push(inner);
    }
    let tmp_board = board.clone();
    for i in 0..9 {
        for j in 0..9 {
            board[i][j].calculate_rust_count(&tmp_board);
        }
    }
    board
}

impl MainState {
    pub fn new(ctx: &mut Context) -> GameResult<MainState> {
        let flag = graphics::Image::new(ctx, "/nope_square.resized.jpg").unwrap();
        let font = graphics::Font::new(ctx, "/FiraCode-Bold.ttf", 30)?;
        let happy_image = graphics::Image::new(ctx, "/ferris_happy.resized.jpg").unwrap();
        let image = graphics::Image::new(ctx, "/cpp.resized.jpg").unwrap();

        let dims = CELL_SIDE * BOARD_SIDE as f32;
        Ok(MainState {
            frames: 0,
            board: spawn_board(),
            image,
            flag,
            font,
            game_over: None,
            did_sleep: false,
            did_reveal: false,
            first_click: true,
            happy_image,
            reset_location: (dims + 100., 100.),
            reset_dims: (200., 200. / PHI),
        })
    }

    fn reset(&mut self) {
        self.board = spawn_board();
        self.game_over = None;
        self.did_sleep = false;
        self.did_reveal = false;
        self.first_click = true;
    }
}

fn flood_fill(board: &mut Vec<Vec<Cell>>, cell_x: usize, cell_y: usize, ignore_rules: bool) {
    let mut queue: VecDeque<Cell> = VecDeque::new();
    queue.push_front(board[cell_x][cell_y].clone());

    while let Some(cell) = queue.pop_front() {
        if !cell.is_rust && !cell.is_flagged || ignore_rules {
            let (x, y) = (cell.position[0] as usize, cell.position[1] as usize);
            board[x][y].is_hidden = false;
            if ignore_rules {
                board[x][y].game_over = true;
                board[x][y].is_flagged = false;
            }
            'outer: for i in -1..=1 {
                for j in -1..=1 {
                    if cell.position[0] == 0. && i == -1
                        || cell.position[0] == BOARD_SIDE as f32 - 1. && i == 1
                    {
                        continue 'outer;
                    }
                    if cell.position[1] == 0. && j == -1
                        || cell.position[1] == BOARD_SIDE as f32 - 1. && j == 1
                    {
                        continue;
                    }
                    if i != 0 || j != 0 {
                        if ignore_rules
                            && !board[(cell.position[0] as i8 + i) as usize]
                                [(cell.position[1] as i8 + j) as usize]
                                .game_over
                        {
                            let new_cell = board[(cell.position[0] as i8 + i) as usize]
                                [(cell.position[1] as i8 + j) as usize]
                                .clone();
                            queue.push_back(new_cell);
                        } else if !board[(cell.position[0] as i8 + i) as usize]
                            [(cell.position[1] as i8 + j) as usize]
                            .is_flagged
                            && board[(cell.position[0] as i8 + i) as usize]
                                [(cell.position[1] as i8 + j) as usize]
                                .rust_count
                                == 0
                            && board[(cell.position[0] as i8 + i) as usize]
                                [(cell.position[1] as i8 + j) as usize]
                                .is_hidden
                        {
                            let new_cell = board[(cell.position[0] as i8 + i) as usize]
                                [(cell.position[1] as i8 + j) as usize]
                                .clone();
                            queue.push_back(new_cell);
                        } else if !board[(cell.position[0] as i8 + i) as usize]
                            [(cell.position[1] as i8 + j) as usize]
                            .is_flagged
                            && !board[(cell.position[0] as i8 + i) as usize]
                                [(cell.position[1] as i8 + j) as usize]
                                .is_rust
                        {
                            board[(cell.position[0] as i8 + i) as usize]
                                [(cell.position[1] as i8 + j) as usize]
                                .is_hidden = false;
                        }
                    }
                }
            }
        }
    }
}

// Then we implement the `ggez:event::EventHandler` trait on it, which
// requires callbacks for updating and drawing the game state each frame.
//
// The `EventHandler` trait also contains callbacks for event handling
// that you can override if you wish, but the defaults are fine.
impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: i32, y: i32) {
        let cell_x = (x / CELL_SIDE as i32) as usize;
        let cell_y = (y / CELL_SIDE as i32) as usize;
        if cell_x > (BOARD_SIDE - 1) as usize {
            let click_x = x as f32 - self.reset_location.0;
            let click_y = y as f32 - self.reset_location.1;
            if click_x > 0.
                && click_x < self.reset_dims.0
                && click_y > 0.
                && click_y < self.reset_dims.1
            {
                self.reset();
            }
            return;
        }
        match button {
            MouseButton::Right => {
                if self.board[cell_x][cell_y].is_hidden {
                    self.board[cell_x][cell_y].is_flagged = !self.board[cell_x][cell_y].is_flagged
                }
            }
            _ => {
                // user should never encounter a bomb on the first click
                if self.first_click {
                    if self.board[cell_x][cell_y].is_rust {
                        let mut clicked_bomb = true;
                        while clicked_bomb {
                            self.board = spawn_board();
                            if !self.board[cell_x][cell_y].is_rust {
                                clicked_bomb = false;
                                self.first_click = false;
                            }
                        }
                    }
                    self.first_click = false;
                }
                if self.board[cell_x][cell_y].is_flagged {
                    return;
                }
                if self.board[cell_x][cell_y].rust_count == 0 && !self.board[cell_x][cell_y].is_rust
                {
                    flood_fill(&mut self.board, cell_x, cell_y, false);
                } else if self.board[cell_x][cell_y].is_rust {
                    flood_fill(&mut self.board, cell_x, cell_y, true);
                } else {
                    self.board[cell_x][cell_y].is_hidden = false;
                }
            }
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        graphics::set_background_color(ctx, graphics::Color::from_rgb(75, 27, 34));
        let dims = CELL_SIDE * BOARD_SIDE as f32;
        graphics::set_color(ctx, graphics::Color::from_rgb(21, 4, 12))?;
        graphics::line(
            ctx,
            &[Point2::new(dims + 5., 0.), Point2::new(dims + 5., dims)],
            10.,
        )?;
        graphics::rectangle(
            ctx,
            graphics::DrawMode::Fill,
            graphics::Rect::new(
                self.reset_location.0,
                self.reset_location.1,
                self.reset_dims.0,
                self.reset_dims.1,
            ),
        )?;
        graphics::set_color(ctx, graphics::WHITE)?;
        let text = graphics::Text::new(ctx, &"RESET", &self.font)?;
        let center =
            graphics::Point2::new(self.reset_location.0 + 15., self.reset_location.1 + 20.);
        graphics::draw(ctx, &text, center, 0.0)?;

        //graphics::draw(ctx, &self.image, Point2::new(400., 400.), 0.)?;
        match self.game_over {
            Some(GameOverState::Solved) => {
                for i in 0..9 {
                    for j in 0..9 {
                        if self.board[i][j].is_rust {
                            let dest_point = graphics::Point2::new(
                                self.board[i][j].position[0] * SPACING,
                                self.board[i][j].position[1] * SPACING,
                            );
                            graphics::draw(ctx, &self.happy_image, dest_point, 0.)?;
                        } else {
                            let shown_num = if self.board[i][j].rust_count == 0 {
                                "".to_owned()
                            } else {
                                format!("{}", self.board[i][j].rust_count)
                            };
                            let text = graphics::Text::new(ctx, &shown_num, &self.font)?;
                            let center = graphics::Point2::new(
                                self.board[i][j].position[0] * SPACING + 20.,
                                self.board[i][j].position[1] * SPACING + 10.,
                            );
                            graphics::draw(ctx, &text, center, 0.0)?;
                        }
                    }
                }
            }
            _ => {
                let mut correct = 0;
                for i in 0..BOARD_SIDE as usize {
                    for j in 0..BOARD_SIDE as usize {
                        let cell = &self.board[i][j];
                        if !cell.is_hidden && !cell.is_rust {
                            correct += 1;
                        }
                        if !cell.is_hidden && cell.is_rust {
                            self.game_over = Some(GameOverState::Failed);
                            correct = 0;
                        }
                        if correct == 71 {
                            self.game_over = Some(GameOverState::Solved);
                        }
                        if cell.is_flagged {
                            let dest_point = graphics::Point2::new(
                                cell.position[0] * SPACING,
                                cell.position[1] * SPACING,
                            );
                            graphics::draw(ctx, &self.flag, dest_point, 0.)?;
                        } else if !cell.is_hidden && cell.is_rust {
                            let dest_point = graphics::Point2::new(
                                cell.position[0] * SPACING,
                                cell.position[1] * SPACING,
                            );
                            graphics::draw(ctx, &self.image, dest_point, 0.)?;
                        } // Drawing the border of every cell
                        graphics::set_color(ctx, graphics::BLACK)?;
                        graphics::rectangle(
                            ctx,
                            graphics::DrawMode::Line(1.),
                            graphics::Rect::new(
                                cell.position[0] * SPACING,
                                cell.position[1] * SPACING,
                                CELL_SIDE,
                                CELL_SIDE,
                            ),
                        )?;
                        // The color must be set back to white to get the images to show up as expected
                        graphics::set_color(ctx, graphics::WHITE)?;

                        if !cell.is_rust && !cell.is_flagged {
                            let shown_num = if cell.rust_count == 0 {
                                "".to_owned()
                            } else {
                                format!("{}", cell.rust_count)
                            };
                            let text = graphics::Text::new(ctx, &shown_num, &self.font)?;
                            let center = graphics::Point2::new(
                                cell.position[0] * SPACING + 20.,
                                cell.position[1] * SPACING + 10.,
                            );
                            graphics::draw(ctx, &text, center, 0.0)?;
                        }
                        if cell.is_hidden && !cell.is_flagged {
                            graphics::rectangle(
                                ctx,
                                graphics::DrawMode::Fill,
                                graphics::Rect::new(
                                    cell.position[0] * SPACING,
                                    cell.position[1] * SPACING,
                                    CELL_SIDE - 2.,
                                    CELL_SIDE - 2.,
                                ),
                            )?;
                        }
                    }
                }
            }
        }
        // Drawables are drawn from their top-left corner.
        graphics::present(ctx);

        self.frames += 1;
        if (self.frames % 100) == 0 {
            println!("FPS: {}", ggez::timer::get_fps(ctx));
        }

        Ok(())
    }
}

// Now our main function, which does three things:
//
// * First, create a new `ggez::conf::Conf`
// object which contains configuration info on things such
// as screen resolution and window title.
// * Second, create a `ggez::game::Game` object which will
// do the work of creating our MainState and running our game.
// * Then, just call `game.run()` which runs the `Game` mainloop.
pub fn main() {
    let ctx = &mut ContextBuilder::new("Rust Sweeper", "ggez")
        .window_setup(WindowSetup::default().title("Rust Sweeper "))
        .window_mode(WindowMode::default().dimensions(
            (CELL_SIDE * BOARD_SIDE as f32 + CTRL_PANEL_WIDTH) as u32,
            (CELL_SIDE * BOARD_SIDE as f32) as u32,
        ))
        .build()
        .unwrap();

    // We add the CARGO_MANIFEST_DIR/resources to the filesystem's path
    // so that ggez will look in our cargo project directory for files.
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        ctx.filesystem.mount(&path, true);
    }

    let state = &mut MainState::new(ctx).unwrap();
    if let Err(e) = event::run(ctx, state) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
