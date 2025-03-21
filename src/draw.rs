use crate::sprites::GameMode;
use crate::{sprites, MenuState};

use super::board::{BoardSettings, MENU_HEIGHT_PERCENT};
use super::game::{Engine, TileState};
use super::sprites::{BlockKind, BombKind, FaceKind, Number, Sprite, SpriteManager};
use ggez::graphics::{Canvas, Color, DrawMode, DrawParam, Mesh, PxScale, Rect, Text};
use ggez::graphics::{Drawable, TextFragment};
use ggez::Context;

const TILE_PX: f32 = 24.0;

pub fn draw_tiles(
    _ctx: &mut Context,
    canvas: &mut Canvas,
    engine: &Engine,
    sprite_manager: &SpriteManager,
    tile_size: (f32, f32),
    board_settings: &BoardSettings,
    screen_dim: (f32, f32),
    current_tile_idx: Option<usize>,
) {
    let img_w = tile_size.0 / TILE_PX;
    let img_h = tile_size.1 / TILE_PX;

    let (w, _) = board_settings.dimensions();
    for (i, tile) in engine.get_tiles().iter().enumerate() {
        let x = (i % w as usize) as f32 * tile_size.0;
        let y = ((i / w as usize) as f32 * tile_size.1) + (screen_dim.1 * MENU_HEIGHT_PERCENT);

        let is_current_tile = current_tile_idx == Some(i);

        let img = match tile.state {
            TileState::Block(false) if tile.is_bomb() && is_current_tile => sprite_manager
                .get(Sprite::Block(BlockKind::Revealed))
                .unwrap(),

            TileState::Block(false) if engine.is_lost() && tile.is_bomb() => sprite_manager
                .get(Sprite::Bomb(BombKind::NotRevealed))
                .unwrap(),

            TileState::Block(true) if engine.is_lost() && !tile.is_bomb() => sprite_manager
                .get(Sprite::Bomb(BombKind::FlaggedWrong))
                .unwrap(),

            TileState::Block(_) if is_current_tile => sprite_manager
                .get(Sprite::Block(BlockKind::Revealed))
                .unwrap(),

            // Normal block - either flagged or solid
            TileState::Block(flag) => {
                let kind = if flag {
                    BlockKind::Flagged
                } else {
                    BlockKind::Solid
                };
                sprite_manager.get(Sprite::Block(kind)).unwrap()
            }

            TileState::Revealed if tile.num_adjacent_bombs() > 0 && !tile.is_bomb() => {
                let bomb_count = tile.num_adjacent_bombs() as u8;
                sprite_manager
                    .get(Sprite::Digit(Number(bomb_count)))
                    .unwrap()
            }

            TileState::Revealed if tile.is_bomb() => {
                sprite_manager.get(Sprite::Bomb(BombKind::Clicked)).unwrap()
            }

            TileState::Revealed => sprite_manager
                .get(Sprite::Block(BlockKind::Revealed))
                .unwrap(),
        };
        img.draw(
            canvas,
            DrawParam::new().dest_rect(Rect::new(x, y, img_w, img_h)),
        );
    }
}

fn draw_face(
    ctx: &mut Context,
    canvas: &mut Canvas,
    engine: &Engine,
    sprite_manager: &SpriteManager,
    tile_size: (f32, f32),
    board_settings: &BoardSettings,
    screen_dim: (f32, f32),
    current_tile_idx: Option<usize>,
    menu_state: &mut MenuState,
) {
    let menu_h = screen_dim.1 * MENU_HEIGHT_PERCENT;
    let menu_size = menu_h * 0.9;
    let menu_img_size = menu_size / TILE_PX;

    let face = sprite_manager
        .get(Sprite::Face(menu_state.face_kind))
        .unwrap();

    let face_x = screen_dim.0 / 2.0 - (0.5 * menu_size);
    let face_y = (0.1 * menu_h) / 2.0;

    let face_rect = Rect::new(face_x, face_y, menu_img_size, menu_img_size);
    menu_state.face_rect = Some(Rect::new(face_x, face_y, menu_size, menu_size));
    face.draw(canvas, DrawParam::new().dest_rect(face_rect));
}

fn draw_timer(
    ctx: &mut Context,
    canvas: &mut Canvas,
    menu_state: &MenuState,
    tile_size: (f32, f32),
    screen_dim: (f32, f32),
) {
    let menu_h = screen_dim.1 * MENU_HEIGHT_PERCENT;

    let timer_w = screen_dim.0 * 0.2;
    let timer_h = menu_h * 0.9;

    let timer_x = screen_dim.0 - timer_w - 20.0; // 20px from right edge
    let timer_y = (menu_h - timer_h) / 2.0;

    let timer_bg = Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        Rect::new(timer_x, timer_y, timer_w, timer_h),
        Color::BLACK,
    )
    .unwrap();

    canvas.draw(&timer_bg, DrawParam::default());

    let time = menu_state.get_time_passed();

    let time_text = format!("{:03}", time);

    let text_scale = PxScale {
        x: timer_w / time_text.len() as f32,
        y: timer_h,
    };

    let text_fragment = TextFragment::new(time_text)
        .color(Color::from_rgb(255, 255, 0))
        .font("pressStart2P")
        .scale(text_scale);

    let text = Text::new(text_fragment);

    let text_dimensions = text.dimensions(ctx).unwrap();
    let text_x = ((timer_w - text_dimensions.w) / 2.0) + screen_dim.0 - timer_w - 15.0;
    let text_y = (timer_h - text_dimensions.h) / 2.0 + timer_y * 2.0;

    // Draw text
    canvas.draw(&text, DrawParam::new().dest([text_x, text_y]));
}

fn draw_bombs_left(
    ctx: &mut Context,
    canvas: &mut Canvas,
    engine: &Engine,
    sprite_manager: &SpriteManager,
    tile_size: (f32, f32),
    board_settings: &BoardSettings,
    screen_dim: (f32, f32),
    current_tile_idx: Option<usize>,
    menu_state: &mut MenuState,
) {
    let menu_h = screen_dim.1 * MENU_HEIGHT_PERCENT;

    let counter_w = screen_dim.0 * 0.2;
    let counter_h = menu_h * 0.9;

    let counter_x = 20.0; // 20px from left edge
    let counter_y = (menu_h - counter_h) / 2.0;

    let counter_bg = Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        Rect::new(counter_x, counter_y, counter_w, counter_h),
        Color::BLACK,
    )
    .unwrap();

    canvas.draw(&counter_bg, DrawParam::default());

    let bombs_left = engine.bombs_left();
    let bombs_text = format!("{:03}", bombs_left);

    let text_scale = PxScale {
        x: counter_w / bombs_text.len() as f32,
        y: counter_h,
    };

    let text_fragment = TextFragment::new(bombs_text)
        .color(Color::from_rgb(255, 0, 0)) // Red text
        .font("pressStart2P")
        .scale(text_scale);

    let text = Text::new(text_fragment);

    let text_dimensions = text.dimensions(ctx).unwrap();
    let text_x = ((counter_w - text_dimensions.w) / 2.0) + counter_x;
    let text_y = (counter_h - text_dimensions.h) / 2.0 + counter_y * 2.0;

    // Draw text
    canvas.draw(&text, DrawParam::new().dest([text_x, text_y]));
}

fn draw_difficulty_buttons(
    ctx: &mut Context,
    canvas: &mut Canvas,
    menu_state: &mut MenuState,
    screen_dim: (f32, f32),
    _tile_size: (f32, f32),
    _sprite_manager: &SpriteManager,
) -> ggez::GameResult {
    let menu_h = screen_dim.1 * MENU_HEIGHT_PERCENT;

    let buttons_total_space = menu_h * 0.9;

    let initial_y_offset = (0.1 * menu_h) / 2.0;

    let padding = 10.0;

    let btn_h = (buttons_total_space - (2.0 * padding)) / 3.0;

    let btn_w = screen_dim.0 * 0.15;
    let btn_x = screen_dim.0 * 0.2 + 30.0;
    let btn_y = initial_y_offset;

    let easy_btn = Rect::new(btn_x, btn_y, btn_w, btn_h);
    let medium_btn = Rect::new(btn_x, btn_y + btn_h + padding, btn_w, btn_h);
    let hard_btn = Rect::new(btn_x, btn_y + 2.0 * (btn_h + padding), btn_w, btn_h);

    let easy_btn_bg = Mesh::new_rectangle(ctx, DrawMode::fill(), easy_btn, Color::BLACK).unwrap();
    let medium_btn_bg =
        Mesh::new_rectangle(ctx, DrawMode::fill(), medium_btn, Color::BLACK).unwrap();
    let hard_btn_bg = Mesh::new_rectangle(ctx, DrawMode::fill(), hard_btn, Color::BLACK).unwrap();
    let easy_scale = PxScale {
        x: btn_w / 4.0,
        y: btn_h,
    };
    let easy_text_fragment = TextFragment::new("EASY")
        .color(Color::from_rgb(255, 255, 255))
        .font("pressStart2P")
        .scale(easy_scale);
    let easy_text = Text::new(easy_text_fragment);

    let medium_scale = PxScale {
        x: btn_w / 6.0,
        y: btn_h,
    };
    let medium_text_fragment = TextFragment::new("MEDIUM")
        .color(Color::from_rgb(255, 255, 255))
        .font("pressStart2P")
        .scale(medium_scale);
    let medium_text = Text::new(medium_text_fragment);

    let hard_scale = PxScale {
        x: btn_w / 4.0,
        y: btn_h,
    };
    let hard_text_fragment = TextFragment::new("HARD")
        .color(Color::from_rgb(255, 255, 255))
        .font("pressStart2P")
        .scale(hard_scale);
    let hard_text = Text::new(hard_text_fragment);

    menu_state.easy_button_rect = Some(easy_btn);
    menu_state.medium_button_rect = Some(medium_btn);
    menu_state.hard_button_rect = Some(hard_btn);

    canvas.draw(&easy_btn_bg, DrawParam::default());
    canvas.draw(
        &easy_text,
        DrawParam::new().dest([
            btn_x + btn_w / 2.0 - easy_text.dimensions(ctx).unwrap().w / 2.0,
            btn_y + btn_h / 2.0 - easy_text.dimensions(ctx).unwrap().h / 2.0,
        ]),
    );

    canvas.draw(&medium_btn_bg, DrawParam::default());
    canvas.draw(
        &medium_text,
        DrawParam::new().dest([
            btn_x + btn_w / 2.0 - medium_text.dimensions(ctx).unwrap().w / 2.0,
            btn_y + (btn_h + padding) + btn_h / 2.0 - medium_text.dimensions(ctx).unwrap().h / 2.0,
        ]),
    );

    canvas.draw(&hard_btn_bg, DrawParam::default());
    canvas.draw(
        &hard_text,
        DrawParam::new().dest([
            btn_x + btn_w / 2.0 - hard_text.dimensions(ctx).unwrap().w / 2.0,
            btn_y + 2.0 * (btn_h + padding) + btn_h / 2.0
                - hard_text.dimensions(ctx).unwrap().h / 2.0,
        ]),
    );

    Ok(())
}

pub fn draw_menu(
    ctx: &mut Context,
    canvas: &mut Canvas,
    engine: &Engine,
    sprite_manager: &SpriteManager,
    tile_size: (f32, f32),
    board_settings: &BoardSettings,
    screen_dim: (f32, f32),
    current_tile_idx: Option<usize>,
    menu_state: &mut MenuState,
) {
    draw_face(
        ctx,
        canvas,
        engine,
        sprite_manager,
        tile_size,
        board_settings,
        screen_dim,
        current_tile_idx,
        menu_state,
    );

    draw_timer(ctx, canvas, menu_state, tile_size, screen_dim);

    draw_bombs_left(
        ctx,
        canvas,
        engine,
        sprite_manager,
        tile_size,
        board_settings,
        screen_dim,
        current_tile_idx,
        menu_state,
    );

    draw_difficulty_buttons(
        ctx,
        canvas,
        menu_state,
        screen_dim,
        tile_size,
        sprite_manager,
    )
    .unwrap();
}
