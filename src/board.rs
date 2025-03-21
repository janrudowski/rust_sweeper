use super::game::Position;
use ggez::graphics::Rect;

#[derive(Clone)]
pub struct BoardSettings {
    pub w: f32,
    pub h: f32,
    num_bombs: usize,
    pub screen_width: f32,
    pub screen_height: f32,
}

impl BoardSettings {
    pub const fn new(
        w: f32,
        h: f32,
        num_bombs: usize,
        screen_width: f32,
        screen_height: f32,
    ) -> Self {
        Self {
            w,
            h,
            num_bombs,
            screen_width,
            screen_height,
        }
    }

    pub fn dimensions(&self) -> (f32, f32) {
        (self.w, self.h)
    }

    pub fn num_bombs(&self) -> usize {
        self.num_bombs
    }

    pub fn screen_dimensions(&self) -> (f32, f32) {
        (self.screen_width, self.screen_height)
    }

    pub fn easy() -> Self {
        EASY_BOARD.clone()
    }

    pub fn medium() -> Self {
        MEDIUM_BOARD.clone()
    }

    pub fn hard() -> Self {
        HARD_BOARD.clone()
    }
}

pub const EASY_SCREEN_WIDTH: f32 = 800.0;
pub const EASY_SCREEN_HEIGHT: f32 = 600.0;

pub const MEDIUM_SCREEN_WIDTH: f32 = 1000.0;
pub const MEDIUM_SCREEN_HEIGHT: f32 = 700.0;

pub const HARD_SCREEN_WIDTH: f32 = 1200.0;
pub const HARD_SCREEN_HEIGHT: f32 = 800.0;

pub const EASY_BOARD: BoardSettings =
    BoardSettings::new(8.0, 8.0, 10, EASY_SCREEN_WIDTH, EASY_SCREEN_HEIGHT);
pub const MEDIUM_BOARD: BoardSettings =
    BoardSettings::new(16.0, 16.0, 40, MEDIUM_SCREEN_WIDTH, MEDIUM_SCREEN_HEIGHT);
pub const HARD_BOARD: BoardSettings =
    BoardSettings::new(30.0, 16.0, 99, HARD_SCREEN_WIDTH, HARD_SCREEN_HEIGHT);

pub const MENU_HEIGHT_PERCENT: f32 = 0.15;

pub fn calculate_tile_size(w: f32, h: f32, (x, y): (f32, f32)) -> (f32, f32) {
    let game_area_height = h * (1.0 - MENU_HEIGHT_PERCENT);
    (w / x, game_area_height / y)
}

pub fn get_tile_index(
    mouse_x: f32,
    mouse_y: f32,
    screen_dim: (f32, f32),
    tile_size: (f32, f32),
    board_settings: &BoardSettings,
) -> Option<usize> {
    let adjusted_y = mouse_y - (screen_dim.1 * MENU_HEIGHT_PERCENT);

    if mouse_y < screen_dim.1 * MENU_HEIGHT_PERCENT {
        return None;
    }

    let (tile_x, tile_y) = tile_size;

    let board_x = (mouse_x / tile_x) as usize;
    let board_y = (adjusted_y / tile_y) as usize;

    if board_x < board_settings.w as usize && board_y < board_settings.h as usize {
        let index = board_y * board_settings.w as usize + board_x;
        Some(index)
    } else {
        None
    }
}

pub fn get_tile_position(
    mouse_x: f32,
    mouse_y: f32,
    screen_dim: (f32, f32),
    tile_size: (f32, f32),
    board_settings: &BoardSettings,
) -> Option<Position> {
    let adjusted_y = mouse_y - (screen_dim.1 * MENU_HEIGHT_PERCENT);

    if mouse_y < screen_dim.1 * MENU_HEIGHT_PERCENT {
        return None;
    }

    let (tile_x, tile_y) = tile_size;
    let x = (mouse_x / tile_x) as i32;
    let y = (adjusted_y / tile_y) as i32;
    if x < 0 || x >= board_settings.w as i32 || y < 0 || y >= board_settings.h as i32 {
        return None;
    }
    Some(Position(x, y))
}

pub fn get_tile_rect(
    index: usize,
    board_width: f32,
    tile_size: (f32, f32),
    screen_dim: (f32, f32),
) -> Rect {
    let x = (index % board_width as usize) as f32 * tile_size.0;
    let y = ((index / board_width as usize) as f32 * tile_size.1)
        + (screen_dim.1 * MENU_HEIGHT_PERCENT);

    Rect::new(x, y, tile_size.0, tile_size.1)
}
