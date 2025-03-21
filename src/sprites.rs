use ggez::graphics::{Image, Rect};
use std::{collections::HashMap, fs, path::Path};

pub struct Number(pub u8);

impl From<u32> for Number {
    fn from(n: u32) -> Self {
        Number(n as u8)
    }
}

impl Number {
    pub fn new(n: u8) -> Option<Self> {
        match n {
            1..=8 => Some(Number(n)),
            _ => None,
        }
    }
}

pub enum GameMode {
    Easy,
    Medium,
    Hard,
}

pub enum BombKind {
    Clicked,
    FlaggedWrong,
    NotRevealed,
}

pub enum BlockKind {
    Revealed,
    Solid,
    Flagged,
}

#[derive(Clone, Copy)]
pub enum FaceKind {
    Smile,
    Dead,
    Surprised,
    Cool,
}
pub enum Sprite {
    Digit(Number),
    Bomb(BombKind),
    Block(BlockKind),
    Face(FaceKind),
    GameMode(GameMode),
}

pub struct SpriteManager {
    sprites: HashMap<String, Image>,
}

impl SpriteManager {
    pub fn new(ctx: &ggez::Context, _path: &str) -> Self {
        let images = ctx
            .fs
            .read_dir("/resources/sprites")
            .unwrap()
            .map(|path| {
                let file_name: String = path.file_name().unwrap().to_string_lossy().into();
                let image = Image::from_path(ctx, &path);
                (file_name, image)
            })
            .filter(|item| item.1.is_ok())
            .map(|item| (item.0, item.1.unwrap()))
            .collect::<Vec<_>>();
        let mut sprites = HashMap::new();

        for (file, image) in images {
            sprites.insert(file, image);
        }

        Self { sprites }
    }

    pub fn get(&self, sprite: Sprite) -> Option<&Image> {
        match sprite {
            Sprite::Digit(n) => self.sprites.get(&format!("{}.png", n.0)),
            Sprite::Bomb(b) => match b {
                BombKind::Clicked => self.sprites.get("bomb_clicked.png"),
                BombKind::FlaggedWrong => self.sprites.get("bomb_flagged_wrong.png"),
                BombKind::NotRevealed => self.sprites.get("bomb.png"),
            },
            Sprite::Block(b) => match b {
                BlockKind::Revealed => self.sprites.get("revealed.png"),
                BlockKind::Solid => self.sprites.get("block.png"),
                BlockKind::Flagged => self.sprites.get("block_flagged.png"),
            },
            Sprite::Face(f) => match f {
                FaceKind::Smile => self.sprites.get("smile.png"),
                FaceKind::Dead => self.sprites.get("dead.png"),
                FaceKind::Surprised => self.sprites.get("surprised.png"),
                FaceKind::Cool => self.sprites.get("cool.png"),
            },
            Sprite::GameMode(m) => match m {
                GameMode::Easy => self.sprites.get("easy.png"),
                GameMode::Medium => self.sprites.get("mid.png"),
                GameMode::Hard => self.sprites.get("hard.png"),
            },
        }
    }
}
