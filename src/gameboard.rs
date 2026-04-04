use rand::seq::SliceRandom;

#[derive(Clone, Copy)]
pub enum CellContent {
    Bomb,
    Safe(u8),
}
#[derive(Clone, Copy)]
pub struct Cell {
    pub content: CellContent,
    pub revealed: bool,
    pub flagged: bool,
}

pub struct GameBoard {
    pub board: Vec<Vec<Cell>>,
    pub rows: usize,
    pub cols: usize,
}

impl GameBoard {
    pub fn new(rows: usize, cols: usize) -> Self {
        let board: Vec<Vec<Cell>> = vec![
            vec![
                Cell {
                    content: CellContent::Safe(0),
                    revealed: false,
                    flagged: false,
                };
                cols
            ];
            rows
        ];

        GameBoard {
            board,
            rows: rows,
            cols: cols,
        }
    }

    pub fn scatter_bombs(mut self, num_bombs: usize) -> Self {
        let total = self.cols * self.rows;
        assert!(num_bombs <= total, "Too many bombs");

        let mut positions: Vec<usize> = (0..total).collect();
        positions.shuffle(&mut rand::thread_rng());
        let bomb_positions = &positions[..num_bombs];

        for &idx in bomb_positions {
            self.board[idx / self.rows][idx % self.cols].content = CellContent::Bomb;
        }
        self
    }

    pub fn fill_info(mut self) -> Self {
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

        self
    }

    fn flag_cell(mut self, x: usize, y: usize) -> Self {
        self.board[y][x].flagged = true;
        self
    }

    fn reveal_cell(mut self, x: usize, y: usize) -> Self {
        self.board[y][x].revealed = true;
        self
    }
}
