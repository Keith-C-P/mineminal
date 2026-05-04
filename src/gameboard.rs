use log::info;
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

pub struct GameBoard {
    pub board: Vec<Vec<Cell>>,
    pub rows: usize,
    pub cols: usize,
    pub num_bombs: usize,
    pub num_flags: usize,
}

impl GameBoard {
    pub fn new(rows: usize, cols: usize) -> Self {
        let board: Vec<Vec<Cell>> = vec![
            vec![
                Cell {
                    content: CellContent::Safe(0),
                    state: CellState::Unrevealed,
                };
                cols
            ];
            rows
        ];

        GameBoard {
            board,
            rows: rows,
            cols: cols,
            num_bombs: 0,
            num_flags: 0,
        }
    }

    pub fn scatter_bombs(&mut self, num_bombs: usize, (origin_x, origin_y): (usize, usize)) {
        let total = self.cols * self.rows;
        assert!(num_bombs <= total, "Too many bombs");

        let origin = origin_y * self.cols + origin_x;
        let mut positions: Vec<usize> = (0..total).collect();

        //XOR Swap
        positions[origin] = positions[origin] ^ positions[total - 1];
        positions[total - 1] = positions[origin] ^ positions[total - 1];
        positions[origin] = positions[origin] ^ positions[total - 1];

        positions[..total - 1].shuffle(&mut rand::thread_rng());
        let bomb_positions = &positions[..num_bombs];

        for &idx in bomb_positions {
            self.board[idx / self.rows][idx % self.cols].content = CellContent::Bomb;
        }

        self.num_bombs = num_bombs;
    }

    pub fn fill_info(&mut self) {
        let rows = self.board.len();
        let cols = self.board[0].len();

        for r in 0..rows {
            for c in 0..cols {
                if let CellContent::Bomb = self.board[r][c].content {
                    continue;
                }

                let mut count: u8 = 0;

                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dy == 0 && dx == 0 {
                            continue;
                        }

                        let nr = r as i32 + dy;
                        let nc = c as i32 + dx;

                        if nr >= 0 && nr < rows as i32 && nc >= 0 && nc < cols as i32 {
                            if let CellContent::Bomb = self.board[nr as usize][nc as usize].content
                            {
                                count += 1;
                            }
                        }
                    }
                }

                self.board[r][c].content = CellContent::Safe(count);
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
        let mut visited = vec![vec![false; self.cols]; self.rows];
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
                        Some(v) if v < self.cols => v,
                        _ => continue,
                    };
                    let ny = match y.checked_add_signed(dy) {
                        Some(v) if v < self.rows => v,
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

    pub fn toggle_flag_at(&mut self, (board_x, board_y): (usize, usize)) {
        let flag_cell = &mut self.board[board_y][board_x];
        self.num_flags = self
            .num_flags
            .checked_add_signed(if matches!(flag_cell.state, CellState::Flagged) {
                -1
            } else {
                1
            })
            .unwrap_or(0);
        flag_cell.state = match flag_cell.state {
            CellState::Flagged => CellState::Unrevealed,
            CellState::Unrevealed => CellState::Flagged,
            CellState::Revealed => CellState::Revealed,
        };
    }

    pub fn get_peek_area(&self, board_coords: (usize, usize)) -> (usize, usize, usize, usize) {
        let (board_x, board_y) = board_coords;
        let origin_x = board_x.checked_add_signed(-1).unwrap_or(0);
        let origin_y = board_y.checked_add_signed(-1).unwrap_or(0);
        let width = if board_x + 1 >= self.cols {
            board_x
        } else {
            board_x + 1
        };
        let height = if board_y + 1 >= self.rows {
            board_y
        } else {
            board_y + 1
        };

        (origin_x, origin_y, width, height)
    }

    pub fn count_surrounding_flags(&self, board_coords: (usize, usize)) -> u8 {
        let (origin_x, origin_y, width, height) = self.get_peek_area(board_coords);
        let mut flag_count = 0;

        for y in origin_y..=height {
            for x in origin_x..=width {
                if matches!(self.board[y][x].state, CellState::Flagged) {
                    flag_count += 1;
                }
            }
        }
        assert!(flag_count <= 8);

        flag_count
    }

    pub fn flags_match(&self, board_coords: (usize, usize)) -> bool {
        let (origin_x, origin_y, width, height) = self.get_peek_area(board_coords);

        for y in origin_y..height {
            for x in origin_x..width {
                let is_flagged = matches!(self.board[y][x].state, CellState::Flagged);
                let is_bomb = match self.board[y][x].content {
                    CellContent::Bomb => true,
                    _ => false,
                };
                if is_flagged && !is_bomb {
                    return false;
                }
            }
        }

        true
    }
}
