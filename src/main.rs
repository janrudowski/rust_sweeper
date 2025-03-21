use ggez::event::{self, EventHandler, MouseButton};
use ggez::graphics::{self, Color, FontData, Rect};
use ggez::{Context, ContextBuilder, GameError, GameResult};

mod board;
mod draw;
mod game;
mod sprites;

use board::{calculate_tile_size, get_tile_index, BoardSettings, EASY_BOARD};
use game::{Engine, Position};
use sprites::{FaceKind, SpriteManager};

fn main() -> GameResult {
    let builder = ContextBuilder::new("rust_sweeper", "Jan Rudowski")
        .window_setup(ggez::conf::WindowSetup::default().title("Rust Sweeper"))
        .window_mode(
            ggez::conf::WindowMode::default()
                .resizable(true)
                .min_dimensions(EASY_BOARD.screen_width, EASY_BOARD.screen_height)
                .dimensions(EASY_BOARD.screen_width, EASY_BOARD.screen_height),
        );

    let (mut ctx, event_loop) = builder.build()?;

    ctx.fs
        .add_zip_file(std::io::Cursor::new(include_bytes!("../resources.zip")))?;

    let game: MainState = MainState::new(&mut ctx);

    event::run(ctx, event_loop, game)
}

pub struct MenuState {
    pub time_passed: u32,
    pub face_kind: FaceKind,
    pub face_rect: Option<Rect>,
    pub easy_button_rect: Option<Rect>,
    pub medium_button_rect: Option<Rect>,
    pub hard_button_rect: Option<Rect>,
}

impl MenuState {
    pub fn new() -> Self {
        Self {
            time_passed: 0,
            face_kind: FaceKind::Smile,
            face_rect: None,
            easy_button_rect: None,
            medium_button_rect: None,
            hard_button_rect: None,
        }
    }

    pub fn get_time_passed(&self) -> u32 {
        self.time_passed
    }

    pub fn is_face_clicked(&self, x: f32, y: f32) -> bool {
        if let Some(rect) = self.face_rect {
            rect.contains([x, y])
        } else {
            false
        }
    }

    pub fn get_difficulty_button_clicked(&self, x: f32, y: f32) -> Option<&str> {
        if let Some(rect) = self.easy_button_rect {
            if rect.contains([x, y]) {
                return Some("easy");
            }
        }

        if let Some(rect) = self.medium_button_rect {
            if rect.contains([x, y]) {
                return Some("medium");
            }
        }

        if let Some(rect) = self.hard_button_rect {
            if rect.contains([x, y]) {
                return Some("hard");
            }
        }

        None
    }
}

struct MainState {
    engine: Engine,
    tile_size: (f32, f32),
    board_settings: BoardSettings,
    screen_dim: (f32, f32),
    sprite_manager: SpriteManager,
    current_tile_idx: Option<usize>,
    menu_state: MenuState,
    accumulated_time: f32,
}

impl MainState {
    pub fn reset_game(&mut self, board_settings: BoardSettings, ctx: &mut Context) {
        let (screen_width, screen_height) = board_settings.screen_dimensions();

        let (board_w, board_h) = board_settings.screen_dimensions();

        ctx.gfx
            .set_mode(
                ggez::conf::WindowMode::default()
                    .resizable(true)
                    .min_dimensions(board_w, board_h)
                    .dimensions(board_w, board_h),
            )
            .unwrap();

        self.screen_dim = (screen_width, screen_height);

        let (tile_size, engine) = MainState::new_game(&board_settings, screen_width, screen_height);

        self.engine = engine;
        self.tile_size = tile_size;
        self.board_settings = board_settings;
        self.menu_state.face_kind = FaceKind::Smile;
        self.menu_state.time_passed = 0;
        self.accumulated_time = 0.0;
        self.current_tile_idx = None;
    }

    pub fn new(ctx: &mut Context) -> MainState {
        let board_settings = EASY_BOARD;
        let screen_dim = (board_settings.screen_width, board_settings.screen_height);

        let (tile_size, engine) = MainState::new_game(&board_settings, screen_dim.0, screen_dim.1);
        let sprite_manager = SpriteManager::new(ctx, "/sprites");

        let font_dir = ctx.fs.read_dir("/resources/assets").unwrap();

        let font_path = font_dir
            .filter(|item| item.extension().map(|s| s == "ttf").unwrap_or(false))
            .next()
            .unwrap();

        let font_data = FontData::from_path(ctx, &font_path).unwrap();

        ctx.gfx.add_font("pressStart2P", font_data);

        MainState {
            engine,
            tile_size,
            board_settings,
            screen_dim,
            sprite_manager,
            current_tile_idx: None,
            menu_state: MenuState::new(),
            accumulated_time: 0.0,
        }
    }

    fn new_game(board_settings: &BoardSettings, w: f32, h: f32) -> ((f32, f32), Engine) {
        let board_size = board_settings.dimensions();
        let num_bombs = board_settings.num_bombs();
        let engine = Engine::new(board_size, num_bombs);
        (calculate_tile_size(w, h, board_size), engine)
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if self.engine.is_won() {
            self.menu_state.face_kind = FaceKind::Cool;
        } else if self.engine.is_lost() {
            self.menu_state.face_kind = FaceKind::Dead;
        }
        if !self.engine.is_lost() && !self.engine.is_won() {
            let delta = ctx.time.delta().as_secs_f32();
            self.accumulated_time += delta;

            if self.accumulated_time >= 1.0 {
                self.menu_state.time_passed += 1;
                self.accumulated_time -= 1.0;
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::from_rgb(150, 150, 150));

        draw::draw_menu(
            ctx,
            &mut canvas,
            &self.engine,
            &self.sprite_manager,
            self.tile_size,
            &self.board_settings,
            self.screen_dim,
            self.current_tile_idx,
            &mut self.menu_state,
        );

        draw::draw_tiles(
            ctx,
            &mut canvas,
            &self.engine,
            &self.sprite_manager,
            self.tile_size,
            &self.board_settings,
            self.screen_dim,
            self.current_tile_idx,
        );

        canvas.finish(ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        let tile_idx = get_tile_index(x, y, self.screen_dim, self.tile_size, &self.board_settings);
        match button {
            MouseButton::Left => {
                if let Some(face_rect) = self.menu_state.face_rect {
                    if face_rect.contains([x, y]) {
                        self.reset_game(self.board_settings.clone(), ctx);
                        return Ok(());
                    }
                }

                if let Some(difficulty) = self.menu_state.get_difficulty_button_clicked(x, y) {
                    match difficulty {
                        "easy" => self.reset_game(BoardSettings::easy(), ctx),
                        "medium" => self.reset_game(BoardSettings::medium(), ctx),
                        "hard" => self.reset_game(BoardSettings::hard(), ctx),
                        _ => {}
                    }
                    return Ok(());
                }

                if !self.engine.is_lost() && !self.engine.is_won() {
                    self.current_tile_idx = tile_idx;
                    if tile_idx.is_some() {
                        self.menu_state.face_kind = FaceKind::Surprised;
                    }
                }
            }
            MouseButton::Right if !self.engine.is_lost() && !self.engine.is_won() => {
                if let Some(tile_idx) = tile_idx {
                    let pos = Position::from_index(tile_idx, self.board_settings.w as usize);
                    self.engine.flag(pos);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        _x: f32,
        _y: f32,
    ) -> Result<(), GameError> {
        match button {
            MouseButton::Left => {
                if let Some(tile_idx) = self.current_tile_idx {
                    let pos = Position::from_index(tile_idx, self.board_settings.w as usize);
                    self.engine.reveal(pos);
                    self.menu_state.face_kind = FaceKind::Smile;
                    self.current_tile_idx = None;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        x: f32,
        y: f32,
        _dx: f32,
        _dy: f32,
    ) -> Result<(), GameError> {
        if let Some(_) = self.current_tile_idx {
            if let Some(new_idx) =
                get_tile_index(x, y, self.screen_dim, self.tile_size, &self.board_settings)
            {
                self.current_tile_idx = Some(new_idx);
            } else {
                self.current_tile_idx = None;
                self.menu_state.face_kind = FaceKind::Smile;
            }
        }
        Ok(())
    }

    fn resize_event(
        &mut self,
        _ctx: &mut Context,
        width: f32,
        height: f32,
    ) -> Result<(), GameError> {
        self.screen_dim = (width, height);
        let board_size = self.board_settings.dimensions();
        self.tile_size = calculate_tile_size(width, height, board_size);
        Ok(())
    }
}
