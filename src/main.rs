use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{self, MouseButton};
use ggez::graphics;
use ggez::graphics::{DrawParam, Mesh, Point2};
use ggez::{Context, ContextBuilder, GameResult};
use std::{env, path};

const DEFAULT_CELL_DIMS: f32 = 80.0;
const DEFAULT_CELLS_ROW: usize = 9;
const CTRL_PANEL_WIDTH: f32 = 350.0;
const EASY: (usize, usize) = (10, 9);
const MEDIUM: (usize, usize) = (37, 18);
const HARD: (usize, usize) = (82, 27);

mod board;
mod cell;
use crate::board::Board;

enum GameOverState {
    Solved,
    Failed,
}

// First we make a structure to contain the game's state
struct MainState {
    // frames: usize,
    board: Board,
    image: graphics::Image,
    flag: graphics::Image,
    font: graphics::Font,
    num_font: graphics::Font,
    happy_image: graphics::Image,
    game_over: Option<GameOverState>,
    did_sleep: bool,
    did_reveal: bool,
    first_click: bool,
    reset_button: graphics::Rect,
    easy_button: graphics::Rect,
    medium_button: graphics::Rect,
    hard_button: graphics::Rect,
    mesh: Mesh,
    difficulty: (usize, usize),
    scaling: f32,
}

impl MainState {
    pub fn new(ctx: &mut Context) -> GameResult<MainState> {
        let flag = graphics::Image::new(ctx, "/nope_square.resized.jpg").unwrap();
        let font = graphics::Font::new(ctx, "/FiraCode-Bold.ttf", 30)?;
        let happy_image = graphics::Image::new(ctx, "/ferris_happy.resized.jpg").unwrap();
        let image = graphics::Image::new(ctx, "/cpp.resized.jpg").unwrap();

        // based on difficulty
        let dims = DEFAULT_CELL_DIMS * DEFAULT_CELLS_ROW as f32;

        let easy_button = graphics::Rect::new(dims + 75.0, 50.0, 250.0, 75.0);
        let medium_button = graphics::Rect::new(dims + 75.0, 150.0, 250.0, 75.0);
        let hard_button = graphics::Rect::new(dims + 75.0, 250.0, 250.0, 75.0);
        let reset_button = graphics::Rect::new(dims + 75.0, 400.0, 250.0, 75.0);

        // based on difficulty
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
        let mut board = Board::new(EASY);
        board.calculate_rust_count();

        Ok(MainState {
            // frames: 0,
            board,
            image,
            flag,
            font: font.clone(),
            num_font: font,
            game_over: None,
            did_sleep: false,
            did_reveal: false,
            first_click: true,
            happy_image,
            reset_button,
            easy_button,
            medium_button,
            hard_button,
            mesh,
            difficulty: EASY,
            scaling: 1.0,
        })
    }

    fn reset(&mut self, ctx: &mut Context, difficulty: (usize, usize)) -> GameResult<()> {
        let mut board = Board::new(difficulty);
        board.calculate_rust_count();
        let scaling = DEFAULT_CELLS_ROW as f32 / difficulty.1 as f32;
        let (_, rows) = difficulty;
        let mut mb = graphics::MeshBuilder::new();
        let font = graphics::Font::new(ctx, "/FiraCode-Bold.ttf", (30.0 * scaling) as u32)?;
        for i in 0..rows {
            for j in 0..rows {
                let x1 = i as f32 * DEFAULT_CELL_DIMS as f32 * scaling;
                let x2 =
                    i as f32 * DEFAULT_CELL_DIMS * scaling as f32 + DEFAULT_CELL_DIMS * scaling;
                let y1 = j as f32 * DEFAULT_CELL_DIMS as f32 * scaling;
                let y2 =
                    j as f32 * DEFAULT_CELL_DIMS as f32 * scaling + DEFAULT_CELL_DIMS * scaling;
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

        self.board = board;
        self.game_over = None;
        self.did_sleep = false;
        self.did_reveal = false;
        self.first_click = true;
        self.difficulty = difficulty;
        self.mesh = mesh;
        self.scaling = scaling;
        self.num_font = font;

        Ok(())
    }

    fn center_text_relative_to(&self, text: &graphics::Text, rect: &graphics::Rect) -> Point2 {
        let f_w = text.width() as f32;
        let f_h = text.height() as f32;
        graphics::Point2::new(
            rect.x + (rect.w / 2.0 - f_w / 2.0),
            rect.y + (rect.h / 2.0 - f_h / 2.0),
        )
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

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: i32, y: i32) {
        let click_point = Point2::new(x as f32, y as f32);
        let cell_x = (x as f32 / (DEFAULT_CELL_DIMS * self.scaling)) as usize;
        let cell_y = (y as f32 / (DEFAULT_CELL_DIMS * self.scaling)) as usize;
        if cell_x > (self.difficulty.1 - 1) as usize {
            if self.reset_button.contains(click_point) {
                self.reset(ctx, self.difficulty).expect("poop");
            }
            if self.easy_button.contains(click_point) {
                self.reset(ctx, EASY).expect("poop");
            }
            if self.medium_button.contains(click_point) {
                self.reset(ctx, MEDIUM).expect("poop");
            }
            if self.hard_button.contains(click_point) {
                self.reset(ctx, HARD).expect("poop");
            }
            return;
        }
        match button {
            MouseButton::Right => {
                if self.board.cells[cell_x][cell_y].is_hidden {
                    self.board.cells[cell_x][cell_y].is_flagged =
                        !self.board.cells[cell_x][cell_y].is_flagged
                }
            }
            _ => {
                // user should never encounter a bomb on the first click
                if self.first_click {
                    if self.board.cells[cell_x][cell_y].is_rust {
                        let mut clicked_bomb = true;
                        while clicked_bomb {
                            self.reset(ctx, self.difficulty).expect("poop");
                            if !self.board.cells[cell_x][cell_y].is_rust {
                                clicked_bomb = false;
                                self.first_click = false;
                            }
                        }
                    }
                    self.first_click = false;
                }
                // ignore clicks on flagged cells
                if self.board.cells[cell_x][cell_y].is_flagged {
                    return;
                }
                // if count is 0, trigger flood fill following rules
                if self.board.cells[cell_x][cell_y].rust_count == 0
                    && !self.board.cells[cell_x][cell_y].is_rust
                {
                    self.board.flood_fill(cell_x, cell_y);
                // if this is a bomb, trigger flood fill ignoring rules (true flood fill)
                } else if self.board.cells[cell_x][cell_y].is_rust {
                    self.board.flood_fill(cell_x, cell_y);
                } else {
                    self.board.cells[cell_x][cell_y].is_hidden = false;
                }
            }
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // settings panel
        let scaling = DEFAULT_CELLS_ROW as f32 / self.difficulty.1 as f32;
        let scale = Point2::new(scaling, scaling);
        graphics::clear(ctx);
        graphics::set_background_color(ctx, graphics::Color::from_rgb(75, 27, 34));
        let dims = DEFAULT_CELL_DIMS * DEFAULT_CELLS_ROW as f32;
        graphics::set_color(ctx, graphics::Color::from_rgb(21, 4, 12))?;
        // dividing line
        graphics::line(
            ctx,
            &[Point2::new(dims + 5.0, 0.0), Point2::new(dims + 5.0, dims)],
            10.0,
        )?;
        // reset button
        graphics::rectangle(ctx, graphics::DrawMode::Fill, self.reset_button)?;
        graphics::set_color(ctx, graphics::WHITE)?;
        let reset = graphics::Text::new(ctx, &"RESET", &self.font)?;
        graphics::draw(
            ctx,
            &reset,
            self.center_text_relative_to(&reset, &self.reset_button),
            0.0,
        )?;
        //easy button
        graphics::set_color(ctx, graphics::Color::from_rgb(21, 4, 12))?;
        graphics::rectangle(ctx, graphics::DrawMode::Fill, self.easy_button)?;
        graphics::set_color(ctx, graphics::WHITE)?;
        let easy = graphics::Text::new(ctx, &"EASY", &self.font)?;
        graphics::draw(
            ctx,
            &easy,
            self.center_text_relative_to(&easy, &self.easy_button),
            0.0,
        )?;

        // medium button
        graphics::set_color(ctx, graphics::Color::from_rgb(21, 4, 12))?;
        graphics::rectangle(ctx, graphics::DrawMode::Fill, self.medium_button)?;
        let medium = graphics::Text::new(ctx, &"MEDIUM", &self.font)?;
        graphics::set_color(ctx, graphics::WHITE)?;
        graphics::draw(
            ctx,
            &medium,
            self.center_text_relative_to(&medium, &self.medium_button),
            0.0,
        )?;
        // hard button
        graphics::set_color(ctx, graphics::Color::from_rgb(21, 4, 12))?;
        graphics::rectangle(ctx, graphics::DrawMode::Fill, self.hard_button)?;
        graphics::set_color(ctx, graphics::WHITE)?;
        let hard = graphics::Text::new(ctx, &"HARD", &self.font)?;
        graphics::set_color(ctx, graphics::WHITE)?;
        graphics::draw(
            ctx,
            &hard,
            self.center_text_relative_to(&hard, &self.hard_button),
            0.0,
        )?;
        // end settings panel

        match &self.game_over {
            Some(state) => {
                let cell_image = match state {
                    GameOverState::Solved => &self.happy_image,
                    GameOverState::Failed => &self.image,
                };
                graphics::set_color(ctx, graphics::BLACK)?;
                graphics::draw(ctx, &self.mesh, Point2::new(0.0, 0.0), 0.0).unwrap();
                graphics::set_color(ctx, graphics::WHITE)?;

                for i in 0..self.difficulty.1 {
                    for j in 0..self.difficulty.1 {
                        let cell = &self.board.cells[i][j];

                        if self.board.cells[i][j].is_rust {
                            let dest = graphics::Point2::new(
                                cell.position.x * DEFAULT_CELL_DIMS * scale.x,
                                cell.position.y * DEFAULT_CELL_DIMS * scale.y,
                            );
                            graphics::draw_ex(
                                ctx,
                                cell_image,
                                DrawParam {
                                    dest,
                                    scale,
                                    ..Default::default()
                                },
                            )?;
                        } else {
                            let shown_num = if cell.rust_count == 0 {
                                "".to_owned()
                            } else {
                                format!("{}", cell.rust_count)
                            };
                            let text = graphics::Text::new(ctx, &shown_num, &self.num_font)?;
                            graphics::draw_ex(
                                ctx,
                                &text,
                                DrawParam {
                                    dest: self.center_text_relative_to(
                                        &text,
                                        &graphics::Rect::new(
                                            cell.position.x * DEFAULT_CELL_DIMS * scale.x,
                                            cell.position.y * DEFAULT_CELL_DIMS * scale.y,
                                            DEFAULT_CELL_DIMS * scale.x,
                                            DEFAULT_CELL_DIMS * scale.y,
                                        ),
                                    ),
                                    ..Default::default()
                                },
                            )?;
                        }
                    }
                }
            }
            _ => {
                let mut correct = 0;
                for i in 0..self.difficulty.1 as usize {
                    for j in 0..self.difficulty.1 as usize {
                        let cell = &self.board.cells[i][j];
                        if cell.game_over {
                            self.game_over = Some(GameOverState::Failed)
                        }
                        if !cell.is_hidden && !cell.is_rust {
                            correct += 1;
                        }
                        if !cell.is_hidden && cell.is_rust {
                            self.game_over = Some(GameOverState::Failed);
                            correct = 0;
                        }
                        if correct == self.difficulty.1.pow(2) - self.difficulty.0 {
                            self.game_over = Some(GameOverState::Solved);
                        }
                        if cell.is_flagged {
                            let dest_point = graphics::Point2::new(
                                cell.position.x * DEFAULT_CELL_DIMS * scale.x,
                                cell.position.y * DEFAULT_CELL_DIMS * scale.y,
                            );
                            graphics::draw_ex(
                                ctx,
                                &self.flag,
                                graphics::DrawParam {
                                    dest: dest_point,
                                    scale,
                                    ..Default::default()
                                },
                            )?;
                        } else if !cell.is_hidden && cell.is_rust {
                            let dest_point = graphics::Point2::new(
                                cell.position.x * DEFAULT_CELL_DIMS * scale.x,
                                cell.position.y * DEFAULT_CELL_DIMS * scale.y,
                            );
                            graphics::draw_ex(
                                ctx,
                                &self.image,
                                graphics::DrawParam {
                                    dest: dest_point,
                                    scale,
                                    ..Default::default()
                                },
                            )?;
                        }
                        // Drawing the border of every cell
                        graphics::set_color(ctx, graphics::BLACK)?;
                        graphics::draw(ctx, &self.mesh, Point2::new(0.0, 0.0), 0.0).unwrap();

                        graphics::set_color(ctx, graphics::WHITE)?;
                        if !cell.is_rust && (!cell.is_flagged || !cell.is_hidden) {
                            // self.draw_text(ctx);
                            let cell = &self.board.cells[i][j];
                            let shown_num = if cell.rust_count == 0 {
                                "".to_owned()
                            } else {
                                format!("{}", cell.rust_count)
                            };
                            let text = graphics::Text::new(ctx, &shown_num, &self.num_font)?;
                            graphics::draw_ex(
                                ctx,
                                &text,
                                DrawParam {
                                    dest: self.center_text_relative_to(
                                        &text,
                                        &graphics::Rect::new(
                                            cell.position.x * DEFAULT_CELL_DIMS * scale.x,
                                            cell.position.y * DEFAULT_CELL_DIMS * scale.y,
                                            DEFAULT_CELL_DIMS * scale.x,
                                            DEFAULT_CELL_DIMS * scale.y,
                                        ),
                                    ),
                                    ..Default::default()
                                },
                            )?;
                        }

                        graphics::set_color(ctx, graphics::WHITE)?;
                        // drawing the cell cover if the cell is hidden
                        if cell.is_hidden && !cell.is_flagged {
                            let mut rect = graphics::Rect::new(
                                cell.position.x * DEFAULT_CELL_DIMS * scale.x,
                                cell.position.y * DEFAULT_CELL_DIMS * scale.y,
                                DEFAULT_CELL_DIMS - (2.0 / scale.x),
                                DEFAULT_CELL_DIMS - (2.0 / scale.y),
                            );
                            rect.scale(scale.x, scale.y);
                            graphics::rectangle(ctx, graphics::DrawMode::Fill, rect)?;
                        }
                    }
                }
            }
        }
        graphics::present(ctx);

        // self.frames += 1;
        // if (self.frames % 100) == 0 {
        //     println!("FPS: {}", ggez::timer::get_fps(ctx));
        // }

        Ok(())
    }
}

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
