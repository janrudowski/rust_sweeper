use rand::Rng;
use std::cmp::PartialEq;
use std::ops::Add;

#[derive(Clone, Copy, Debug)]
pub struct Position(pub i32, pub i32);

impl Position {
    pub fn from_index(index: usize, board_width: usize) -> Self {
        let x = index % board_width;
        let y = index / board_width;
        Position(x as i32, y as i32)
    }

    fn to_index(&self, board_width: usize) -> usize {
        self.1 as usize * board_width + self.0 as usize
    }
}

const ADJACENT_OFFSETS: [Position; 8] = [
    Position(-1, -1),
    Position(0, -1),
    Position(1, -1),
    Position(1, 0),
    Position(1, 1),
    Position(0, 1),
    Position(-1, 1),
    Position(-1, 0),
];
impl Add for Position {
    type Output = Position;

    fn add(self, rhs: Self) -> Self::Output {
        Position(self.0 + rhs.0, self.1 + rhs.1)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum TileState {
    Revealed,
    // flagged = true
    Block(bool),
}
#[derive(Clone, Debug)]
pub struct Tile {
    pub state: TileState,
    has_bomb: bool,
    adjacent_bombs: u32,
}
impl Tile {
    pub fn is_bomb(&self) -> bool {
        self.has_bomb
    }

    pub fn num_adjacent_bombs(&self) -> u32 {
        self.adjacent_bombs
    }

    fn no_adjacent_bombs(&self) -> bool {
        self.adjacent_bombs == 0
    }

    fn is_flagged(&self) -> bool {
        match self.state {
            TileState::Block(true) => true,
            _ => false,
        }
    }

    pub fn is_revealed(&self) -> bool {
        self.state == TileState::Revealed
    }
}

struct Board {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
    tiles_left: usize,
    num_bombs: i32,
}

impl Board {
    pub fn new(width: usize, height: usize, num_bombs: usize) -> Self {
        let mut board = Self {
            width,
            height,
            tiles: Vec::with_capacity(width * height),
            tiles_left: width * height,
            num_bombs: num_bombs as i32,
        };
        board.create_tiles();
        board
    }
    fn create_tiles(&mut self) {
        self.tiles = vec![
            Tile {
                state: TileState::Block(false),
                has_bomb: false,
                adjacent_bombs: 0
            };
            self.width * self.height
        ];
    }

    fn insert_bombs(&mut self, safe_position: Position) {
        let mut rng = rand::rng();
        let mut bombs_placed = 0;

        let mut safe_positions = vec![safe_position];

        for x in ADJACENT_OFFSETS.iter() {
            let pos = safe_position + *x;
            if pos.0 >= 0 && pos.0 < self.width as i32 && pos.1 >= 0 && pos.1 < self.height as i32 {
                safe_positions.push(pos);
            }
        }

        let safe_idxs = safe_positions
            .iter()
            .map(|pos| pos.to_index(self.width))
            .collect::<Vec<_>>();

        while bombs_placed < self.num_bombs {
            let idx = rng.random_range(0..self.width * self.height);
            let tile = &mut self.tiles[idx];
            if tile.has_bomb || safe_idxs.contains(&idx) {
                continue;
            }
            tile.has_bomb = true;
            bombs_placed += 1;
        }
    }

    fn position_out_of_bounds(&self, pos: &Position) -> bool {
        pos.0 < 0 || pos.0 >= self.width as i32 || pos.1 < 0 || pos.1 >= self.height as i32
    }

    fn calculate_adjacent_bombs(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let pos = Position(x as i32, y as i32);
                let idx = pos.to_index(self.width);

                if self.tiles[idx].has_bomb {
                    continue;
                }

                let mut count = 0;
                for offset in ADJACENT_OFFSETS.iter() {
                    let adj_pos = pos + *offset;
                    if self.position_out_of_bounds(&adj_pos) {
                        continue;
                    }

                    let adj_idx = adj_pos.to_index(self.width);
                    if self.tiles[adj_idx].has_bomb {
                        count += 1;
                    }
                }

                self.tiles[idx].adjacent_bombs = count;
            }
        }
    }

    pub fn reveal_tile(&mut self, pos: Position) {
        use std::collections::VecDeque;

        let mut queue = VecDeque::new();
        queue.push_back(pos);

        while let Some(pos) = queue.pop_front() {
            let idx = pos.to_index(self.width);
            let tile = &mut self.tiles[idx];

            if tile.is_revealed() || tile.is_flagged() {
                continue;
            }

            tile.state = TileState::Revealed;
            self.tiles_left -= 1;

            if tile.no_adjacent_bombs() {
                for offset in ADJACENT_OFFSETS.iter() {
                    let adj_pos = pos + *offset;
                    if self.position_out_of_bounds(&adj_pos) {
                        continue;
                    }
                    queue.push_back(adj_pos);
                }
            }
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum GameState {
    Lost,
    Won,
    InProgress,
    FirstMove,
}
pub struct Engine {
    board: Board,
    state: GameState,
    bombs_left: i32,
}

impl Engine {
    pub fn new(board_size: (f32, f32), num_bombs: usize) -> Self {
        let (w, h) = board_size;
        let mut board = Board::new(w as usize, h as usize, num_bombs);
        board.create_tiles();

        Self {
            board,
            state: GameState::FirstMove,
            bombs_left: num_bombs as i32,
        }
    }

    pub fn bombs_left(&self) -> usize {
        if self.bombs_left < 0 {
            0
        } else {
            self.bombs_left as usize
        }
    }

    pub fn is_lost(&self) -> bool {
        self.state == GameState::Lost
    }

    pub fn is_won(&self) -> bool {
        self.state == GameState::Won
    }

    pub fn flag(&mut self, pos: Position) {
        let idx = pos.to_index(self.board.width);
        let tile = &mut self.board.tiles[idx];

        tile.state = match tile.state {
            TileState::Block(n) => {
                if n {
                    self.bombs_left += 1;
                } else {
                    self.bombs_left -= 1;
                }
                TileState::Block(!n)
            }
            TileState::Revealed => TileState::Revealed,
        };
    }

    pub fn reveal(&mut self, pos: Position) {
        if self.is_lost() || self.is_won() {
            return;
        }

        // first move is always safe
        if self.state == GameState::FirstMove {
            self.board.insert_bombs(pos);
            self.board.calculate_adjacent_bombs();
            self.state = GameState::InProgress;
        }

        let idx = pos.to_index(self.board.width);
        let tile = &self.board.tiles[idx];

        if tile.is_revealed() || tile.is_flagged() {
            return;
        }

        if tile.is_bomb() {
            self.board.tiles[idx].state = TileState::Revealed;
            self.state = GameState::Lost;
            return;
        }

        self.board.reveal_tile(pos);

        self.check_win_condition();
    }

    pub fn get_tiles(&self) -> &[Tile] {
        &self.board.tiles
    }

    fn check_win_condition(&mut self) {
        if self.board.tiles_left == self.board.num_bombs as usize {
            self.state = GameState::Won;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_count_matches_board_size() {
        let board_width = 10.0;
        let board_height = 15.0;
        let num_bombs = 20;
        let engine = Engine::new((board_width, board_height), num_bombs);

        let tile_count = engine.get_tiles().into_iter().count();
        let expected_count = (board_width * board_height) as usize;

        assert_eq!(
            tile_count, expected_count,
            "Tile count ({}) does not match the expected board size ({})",
            tile_count, expected_count
        );
    }

    #[test]
    fn test_bomb_count() {
        let board_size = (8.0, 8.0);
        let num_bombs = 10;
        let mut engine = Engine::new(board_size, num_bombs);

        engine.reveal(Position(2, 3));

        let bomb_count = engine.get_tiles().iter().filter(|t| t.is_bomb()).count();

        assert_eq!(
            bomb_count, num_bombs,
            "Bomb count ({}) does not match the expected number ({})",
            bomb_count, num_bombs
        );
    }

    #[test]
    fn test_initial_game_state() {
        let engine = Engine::new((5.0, 5.0), 5);
        assert_eq!(
            engine.state,
            GameState::FirstMove,
            "Initial game state should be InProgress"
        );
    }

    #[test]
    fn test_flag() {
        let mut engine = Engine::new((5.0, 5.0), 5);
        let pos = Position(2, 2);

        engine.flag(pos);
        let tile = engine.get_tiles().iter().nth(pos.to_index(5)).unwrap();

        assert!(tile.is_flagged(), "Tile should be flagged after flagging");
    }

    #[test]
    fn test_reveal_safe_tile() {
        let mut engine = Engine::new((5.0, 5.0), 0);
        let pos = Position(2, 2);

        engine.reveal(pos);
        let tile = engine.get_tiles().iter().nth(pos.to_index(5)).unwrap();

        assert!(
            tile.is_revealed(),
            "Tile should be revealed after revealing a safe tile"
        );
        assert_eq!(engine.state, GameState::Won, "Game should be won");
    }

    #[test]
    fn test_reveal_bomb_tile() {
        let board_size = (3.0, 3.0);
        let num_bombs = 1;
        let mut engine = Engine::new(board_size, num_bombs);

        engine.state = GameState::InProgress;

        let bomb_pos = Position(1, 1);
        let bomb_idx = bomb_pos.to_index(board_size.0 as usize);

        for tile in &mut engine.board.tiles {
            tile.has_bomb = false;
        }
        engine.board.tiles[bomb_idx].has_bomb = true;

        engine.board.calculate_adjacent_bombs();

        assert!(
            engine.get_tiles()[bomb_idx].is_bomb(),
            "Expected to find a bomb at position ({}, {})",
            bomb_pos.0,
            bomb_pos.1
        );

        engine.reveal(Position(0, 0));

        engine.reveal(bomb_pos);

        // Verify the game is lost
        assert_eq!(
            engine.state,
            GameState::Lost,
            "Game should be lost after revealing a bomb"
        );
    }

    #[test]
    fn test_unflag_tile() {
        let mut engine = Engine::new((5.0, 5.0), 5);
        let pos = Position(2, 2);

        engine.flag(pos);
        engine.flag(pos); // unflag
        let tile = engine.get_tiles().iter().nth(pos.to_index(5)).unwrap();

        assert!(
            !tile.is_flagged(),
            "Tile should not be flagged after unflagging"
        );
    }

    #[test]
    fn test_reveal_adjacent_tiles() {
        let mut engine = Engine::new((5.0, 5.0), 0); // No bombs
        let pos = Position(2, 2);

        engine.reveal(pos);
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let adjacent_pos = Position(pos.0 + dx, pos.1 + dy);
                let adjacent_tile = engine
                    .get_tiles()
                    .iter()
                    .nth(adjacent_pos.to_index(5))
                    .unwrap();
                assert!(
                    adjacent_tile.is_revealed(),
                    "Adjacent tile at {:?} should be revealed",
                    adjacent_pos
                );
            }
        }
    }

    #[test]
    fn test_game_won() {
        let board_size = (3.0, 3.0);
        let num_bombs = 1;
        let mut engine = Engine::new(board_size, num_bombs);

        for x in 0..3 {
            for y in 0..3 {
                let pos = Position(x, y);
                let tile = engine.get_tiles().iter().nth(pos.to_index(3)).unwrap();
                if !tile.is_bomb() {
                    engine.reveal(pos);
                }
            }
        }

        assert_eq!(
            engine.state,
            GameState::Won,
            "Game should be won after revealing all non-bomb tiles"
        );
    }
}
