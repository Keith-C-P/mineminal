use std::iter;

use log::{debug, info};
use rand::seq::SliceRandom;

#[derive(Clone, Copy)]
pub enum CellContent {
    Bomb,
    Safe(u8),
}

#[derive(Clone, Copy)]
pub enum CellState {
    Unrevealed,
    Revealed,
    Flagged,
}
#[derive(Clone, Copy)]
pub struct Cell {
    pub content: CellContent,
    pub state: CellState,
}

impl Cell {
    pub fn new() -> Self {
        Self {
            content: CellContent::Safe(0),
            state: CellState::Unrevealed,
        }
    }
}

pub struct GameBoard {
    pub board: Vec<Vec<Cell>>,
    pub height: usize,
    pub width: usize,
    pub num_bombs: usize,
    pub num_flags: usize,
}

impl GameBoard {
    pub fn new(rows: usize, cols: usize) -> Self {
        let board: Vec<Vec<Cell>> = vec![vec![Cell::new(); cols]; rows];

        GameBoard {
            board,
            height: rows,
            width: cols,
            num_bombs: 0,
            num_flags: 0,
        }
    }

    pub fn scatter_bombs(&mut self, num_bombs: usize, (origin_x, origin_y): (usize, usize)) {
        let total = self.width * self.height;
        assert!(num_bombs <= total, "Too many bombs");

        let origin = origin_y * self.width + origin_x;
        let mut positions: Vec<usize> = (0..total).collect();

        //XOR Swap, swaps first_click with end of array, guarenteeing no bomb will be at first_click location
        positions[origin] = positions[origin] ^ positions[total - 1];
        positions[total - 1] = positions[origin] ^ positions[total - 1];
        positions[origin] = positions[origin] ^ positions[total - 1];

        positions[..total - 1].shuffle(&mut rand::thread_rng());
        let bomb_positions = &positions[..num_bombs];

        for &idx in bomb_positions {
            self.board[idx / self.height][idx % self.width].content = CellContent::Bomb;
        }

        self.num_bombs = num_bombs;
    }

    pub fn fill_info(&mut self) {
        for board_y in 0..self.height {
            for board_x in 0..self.width {
                if let CellContent::Bomb = self.board[board_y][board_x].content {
                    continue;
                }

                let count: u8 = self
                    .get_surrounding_cells((board_x, board_y))
                    .iter()
                    .filter(|cell| match cell {
                        Some(Cell {
                            content: CellContent::Bomb,
                            ..
                        }) => true,
                        _ => false,
                    })
                    .count() as u8;

                self.board[board_y][board_x].content = CellContent::Safe(count);
            }
        }
    }

    pub fn count_revealed_cells(&self) -> usize {
        self.board
            .iter()
            .flatten()
            .filter(|cell| matches!(cell.state, CellState::Revealed))
            .count()
    }

    pub fn calculate_difficulty(&mut self) -> usize {
        let mut visited = vec![vec![false; self.width]; self.height];
        let mut difficulty = 0;
        for y in 0..visited.len() {
            for x in 0..visited[0].len() {
                if visited[y][x] {
                    continue;
                }
                let cell = self.board[y][x].content;
                match cell {
                    CellContent::Safe(0) => self.flood_fill(&mut visited, (x, y)),
                    CellContent::Safe(_) => visited[y][x] = true,
                    CellContent::Bomb => continue, // Don't count bombs in 3BV
                }
                difficulty += 1;
            }
        }
        info!("3BV: {difficulty}");
        difficulty
    }

    fn flood_fill(&self, visited: &mut Vec<Vec<bool>>, (x, y): (usize, usize)) {
        if visited[y][x] {
            return;
        }

        visited[y][x] = true;

        if let CellContent::Safe(0) = self.board[y][x].content {
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = match x.checked_add_signed(dx) {
                        Some(v) if v < self.width => v,
                        _ => continue,
                    };
                    let ny = match y.checked_add_signed(dy) {
                        Some(v) if v < self.height => v,
                        _ => continue,
                    };
                    if visited[ny][nx] {
                        continue;
                    }
                    match self.board[ny][nx].content {
                        CellContent::Safe(0) => {
                            self.flood_fill(visited, (nx, ny));
                        }
                        CellContent::Safe(_) => {
                            visited[ny][nx] = true;
                        }
                        CellContent::Bomb => {}
                    }
                }
            }
        }
    }

    pub fn toggle_flag_at(&mut self, (board_x, board_y): (usize, usize)) -> (bool, usize) {
        let flag_cell = &mut self.board[board_y][board_x];
        self.num_flags = self
            .num_flags
            .checked_add_signed(if matches!(flag_cell.state, CellState::Flagged) {
                -1
            } else {
                1
            })
            .unwrap_or(0);

        let (is_wasted, clicks);
        flag_cell.state = match flag_cell.state {
            CellState::Unrevealed => {
                debug!(
                    "Flagged at ({}, {}) (RMB on unrevealed cell)",
                    board_x, board_y
                );
                (is_wasted, clicks) = (false, 1);
                CellState::Flagged
            }
            CellState::Revealed => {
                debug!(
                    "Useless RMB at ({}, {}) (RMB on revealed cell)",
                    board_x, board_y
                );
                (is_wasted, clicks) = (true, 1);
                CellState::Revealed
            }
            CellState::Flagged => {
                debug!("Unflag at ({}, {}) (RMB on flagged cell)", board_x, board_y);
                (is_wasted, clicks) = (true, 2); // 2 clicks wasted here because flagging and then unflagging is 2 clicks
                CellState::Unrevealed
            }
        };
        (is_wasted, clicks)
    }

    pub fn get_peek_area(&self, board_coords: (usize, usize)) -> (usize, usize, usize, usize) {
        let (board_x, board_y) = board_coords;
        let start_x = board_x.saturating_sub(1);
        let start_y = board_y.saturating_sub(1);
        let end_x = (board_x + 2).min(self.width);
        let end_y = (board_y + 2).min(self.height);

        (start_x, start_y, end_x, end_y)
    }

    pub fn count_surrounding_flags(&self, board_coords: (usize, usize)) -> u8 {
        let surrounding_cells = self.get_surrounding_cells(board_coords);
        let flag_count = surrounding_cells
            .iter()
            .filter(|cell| match cell {
                Some(c) => matches!(c.state, CellState::Flagged),
                None => false,
            })
            .count();
        assert!(flag_count <= 8);

        flag_count as u8
    }

    pub fn get_surrounding_cells(&self, board_coords: (usize, usize)) -> [Option<Cell>; 8] {
        let (start_x, start_y, end_x, end_y) = self.get_peek_area(board_coords);
        let (origin_x, origin_y) = board_coords;
        let mut neighbors: [Option<Cell>; 8] = [None; 8];

        // neighbors are ordered like this:
        // 0 1 2
        // 3 X 4
        // 5 6 7

        for y in start_y..end_y {
            for x in start_x..end_x {
                // skip if origin
                if origin_x == x && origin_y == y {
                    continue;
                }
                let dx = x - start_x;
                let dy = y - start_y;
                let idx = dy * 3 + dx;
                let neighbor_idx = if idx < 4 { idx } else { idx - 1 };

                neighbors[neighbor_idx] = Some(self.board[y][x]);
            }
        }
        neighbors
    }

    pub fn flags_match(&self, board_coords: (usize, usize)) -> bool {
        let surrounding_cells = self.get_surrounding_cells(board_coords);
        for cell in surrounding_cells {
            if let Some(cell) = cell {
                let is_flag = matches!(cell.state, CellState::Flagged);
                let is_bomb = matches!(cell.content, CellContent::Bomb);
                if is_flag && !is_bomb {
                    return false;
                }
            }
        }
        true
    }
}
