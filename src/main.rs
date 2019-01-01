//! Basic hello world example.

use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{self, MouseButton};
use ggez::graphics;
use ggez::graphics::{Mesh, Point2};
use ggez::{Context, ContextBuilder, GameResult};
use rand::prelude::*;
use std::collections::VecDeque;
use std::{env, path};

const DEFAULT_CELL_DIMS: f32 = 80.0;
const DEFAULT_CELLS_ROW: u8 = 9;
const CTRL_PANEL_WIDTH: f32 = 350.0;
const PHI: f32 = 1.0618;

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
    scaling: f32,
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
            scaling: 0.0,
        }
    }

    fn calculate_rust_count(&mut self, board: &Vec<Vec<Cell>>) {
        let mut rusts_found = 0;
        'outer: for i in -1..=1 {
            for j in -1..=1 {
                if self.position.x == 0.0 && i == -1
                    || self.position.x == DEFAULT_CELLS_ROW as f32 - 1.0 && i == 1
                {
                    continue 'outer;
                }
                if self.position.y == 0.0 && j == -1
                    || self.position.y == DEFAULT_CELLS_ROW as f32 - 1.0 && j == 1
                {
                    continue;
                }
                if i != 0 || j != 0 {
                    if board[(self.position.x as i8 + i) as usize]
                        [(self.position.y as i8 + j) as usize]
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
    reset_button: graphics::Rect,
    mesh: Mesh,
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

        let dims = DEFAULT_CELL_DIMS * DEFAULT_CELLS_ROW as f32;
        let reset_button = graphics::Rect::new(dims + 100.0, 100.0, 200.0, 200.0 / PHI);

        let mut mb = graphics::MeshBuilder::new();
        for i in 0..DEFAULT_CELLS_ROW {
            for j in 0..DEFAULT_CELLS_ROW {
                let x1 = i as f32 * DEFAULT_CELL_DIMS as f32;
                let x2 = i as f32 * DEFAULT_CELL_DIMS as f32 + DEFAULT_CELL_DIMS;
                let y1 = j as f32 * DEFAULT_CELL_DIMS as f32;
                let y2 = j as f32 * DEFAULT_CELL_DIMS as f32 + DEFAULT_CELL_DIMS;
                let lines = vec![
                    Point2::new(x1, y1),
                    Point2::new(x2, y1),
                    Point2::new(x2, y2),
                    Point2::new(x1, y2),
                ];
                mb.polygon(graphics::DrawMode::Line(1.0), &lines);
            }
        }

        let mesh = mb.build(ctx).unwrap();

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
            reset_location: (dims + 100.0, 100.0),
            reset_button,
            mesh,
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

/// if ignore_rules is true (in the event a bomb was clicked on), flood fill
/// actually performs a flood fill. Otherwise
/// this implements standard minesweeper rules
fn flood_fill(board: &mut Vec<Vec<Cell>>, cell_x: usize, cell_y: usize, ignore_rules: bool) {
    let mut queue: VecDeque<Cell> = VecDeque::new();
    queue.push_front(board[cell_x][cell_y].clone());

    while let Some(cell) = queue.pop_front() {
        if !cell.is_rust && !cell.is_flagged || ignore_rules {
            let (x, y) = (cell.position.x as usize, cell.position.y as usize);
            board[x][y].is_hidden = false;
            if ignore_rules {
                board[x][y].game_over = true;
                board[x][y].is_flagged = false;
            }
            'outer: for i in -1..=1 {
                for j in -1..=1 {
                    if cell.position.x == 0.0 && i == -1
                        || cell.position.x == DEFAULT_CELLS_ROW as f32 - 1.0 && i == 1
                    {
                        continue 'outer;
                    }
                    if cell.position.y == 0.0 && j == -1
                        || cell.position.y == DEFAULT_CELLS_ROW as f32 - 1.0 && j == 1
                    {
                        continue;
                    }
                    if i != 0 || j != 0 {
                        if ignore_rules
                            && !board[(cell.position.x as i8 + i) as usize]
                                [(cell.position.y as i8 + j) as usize]
                                .game_over
                        {
                            let new_cell = board[(cell.position.x as i8 + i) as usize]
                                [(cell.position.y as i8 + j) as usize]
                                .clone();
                            queue.push_back(new_cell);
                        } else if !board[(cell.position.x as i8 + i) as usize]
                            [(cell.position.y as i8 + j) as usize]
                            .is_flagged
                            && board[(cell.position.x as i8 + i) as usize]
                                [(cell.position.y as i8 + j) as usize]
                                .rust_count
                                == 0
                            && board[(cell.position.x as i8 + i) as usize]
                                [(cell.position.y as i8 + j) as usize]
                                .is_hidden
                        {
                            let new_cell = board[(cell.position.x as i8 + i) as usize]
                                [(cell.position.y as i8 + j) as usize]
                                .clone();
                            queue.push_back(new_cell);
                        } else if !board[(cell.position.x as i8 + i) as usize]
                            [(cell.position.y as i8 + j) as usize]
                            .is_flagged
                            && !board[(cell.position.x as i8 + i) as usize]
                                [(cell.position.y as i8 + j) as usize]
                                .is_rust
                        {
                            board[(cell.position.x as i8 + i) as usize]
                                [(cell.position.y as i8 + j) as usize]
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
        let click_point = Point2::new(x as f32, y as f32);
        let cell_x = (x / DEFAULT_CELL_DIMS as i32) as usize;
        let cell_y = (y / DEFAULT_CELL_DIMS as i32) as usize;
        if cell_x > (DEFAULT_CELLS_ROW - 1) as usize {
            if self.reset_button.contains(click_point) {
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
                // ignore clicks on flagged cells
                if self.board[cell_x][cell_y].is_flagged {
                    return;
                }
                // if count is 0, trigger flood fill following rules
                if self.board[cell_x][cell_y].rust_count == 0 && !self.board[cell_x][cell_y].is_rust
                {
                    flood_fill(&mut self.board, cell_x, cell_y, false);
                // if this is a bomb, trigger flood fill ignoring rules (true flood fill)
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
        let dims = DEFAULT_CELL_DIMS * DEFAULT_CELLS_ROW as f32;
        graphics::set_color(ctx, graphics::Color::from_rgb(21, 4, 12))?;
        graphics::line(
            ctx,
            &[Point2::new(dims + 5.0, 0.0), Point2::new(dims + 5.0, dims)],
            10.0,
        )?;
        graphics::rectangle(ctx, graphics::DrawMode::Fill, self.reset_button)?;
        graphics::set_color(ctx, graphics::WHITE)?;
        let text = graphics::Text::new(ctx, &"RESET", &self.font)?;
        let f_w = text.width() as f32;
        let f_h = text.height() as f32;
        let center = graphics::Point2::new(
            self.reset_location.0 + (self.reset_button.w / 2.0 - f_w / 2.0),
            self.reset_location.1 + (self.reset_button.h / 2.0 - f_h / 2.0),
        );
        graphics::draw(ctx, &text, center, 0.0)?;

        match self.game_over {
            Some(GameOverState::Solved) => {
                graphics::set_color(ctx, graphics::BLACK)?;
                graphics::draw(ctx, &self.mesh, Point2::new(0.0, 0.0), 0.0).unwrap();
                graphics::set_color(ctx, graphics::WHITE)?;

                for i in 0..9 {
                    for j in 0..9 {
                        if self.board[i][j].is_rust {
                            let dest_point = graphics::Point2::new(
                                self.board[i][j].position.x * DEFAULT_CELL_DIMS,
                                self.board[i][j].position.y * DEFAULT_CELL_DIMS,
                            );
                            graphics::draw(ctx, &self.happy_image, dest_point, 0.0)?;
                        } else {
                            // self.draw_text(ctx);
                            let cell = &self.board[i][j];
                            let shown_num = if cell.rust_count == 0 {
                                "".to_owned()
                            } else {
                                format!("{}", cell.rust_count)
                            };
                            let text = graphics::Text::new(ctx, &shown_num, &self.font)?;
                            let f_w = text.width() as f32;
                            let f_h = text.height() as f32;
                            let center = graphics::Point2::new(
                                cell.position.x * DEFAULT_CELL_DIMS
                                    + (DEFAULT_CELL_DIMS / 2.0 - f_w / 2.0),
                                cell.position.y * DEFAULT_CELL_DIMS
                                    + (DEFAULT_CELL_DIMS / 2.0 - f_h / 2.0),
                            );
                            graphics::draw(ctx, &text, center, 0.0)?;
                        }
                    }
                }
            }
            _ => {
                let mut correct = 0;
                for i in 0..DEFAULT_CELLS_ROW as usize {
                    for j in 0..DEFAULT_CELLS_ROW as usize {
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
                                cell.position.x * DEFAULT_CELL_DIMS,
                                cell.position.y * DEFAULT_CELL_DIMS,
                            );
                            graphics::draw(ctx, &self.flag, dest_point, 0.0)?;
                        } else if !cell.is_hidden && cell.is_rust {
                            let dest_point = graphics::Point2::new(
                                cell.position.x * DEFAULT_CELL_DIMS,
                                cell.position.y * DEFAULT_CELL_DIMS,
                            );
                            graphics::draw(ctx, &self.image, dest_point, 0.0)?;
                        }
                        // Drawing the border of every cell
                        graphics::set_color(ctx, graphics::BLACK)?;
                        graphics::draw(ctx, &self.mesh, Point2::new(0.0, 0.0), 0.0).unwrap();

                        graphics::set_color(ctx, graphics::WHITE)?;
                        if !cell.is_rust && (!cell.is_flagged || !cell.is_hidden) {
                            // self.draw_text(ctx);
                            let cell = &self.board[i][j];
                            let shown_num = if cell.rust_count == 0 {
                                "".to_owned()
                            } else {
                                format!("{}", cell.rust_count)
                            };
                            let text = graphics::Text::new(ctx, &shown_num, &self.font)?;
                            let f_w = text.width() as f32;
                            let f_h = text.height() as f32;
                            let center = graphics::Point2::new(
                                cell.position.x * DEFAULT_CELL_DIMS
                                    + (DEFAULT_CELL_DIMS / 2.0 - f_w / 2.0),
                                cell.position.y * DEFAULT_CELL_DIMS
                                    + (DEFAULT_CELL_DIMS / 2.0 - f_h / 2.0),
                            );
                            graphics::draw(ctx, &text, center, 0.0)?;
                        }

                        // drawing the cell cover if the cell is hidden
                        if cell.is_hidden && !cell.is_flagged {
                            graphics::rectangle(
                                ctx,
                                graphics::DrawMode::Fill,
                                graphics::Rect::new(
                                    cell.position.x * DEFAULT_CELL_DIMS + 1.0,
                                    cell.position.y * DEFAULT_CELL_DIMS + 1.0,
                                    DEFAULT_CELL_DIMS - 2.0,
                                    DEFAULT_CELL_DIMS - 2.0,
                                ),
                            )?;
                        }
                    }
                }
            }
        }
        // Drawable items are drawn from their top-left corner.
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
            (DEFAULT_CELL_DIMS * DEFAULT_CELLS_ROW as f32 + CTRL_PANEL_WIDTH) as u32,
            (DEFAULT_CELL_DIMS * DEFAULT_CELLS_ROW as f32) as u32,
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
