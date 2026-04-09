use crate::gameboard::{Cell, CellContent, GameBoard};
use crate::renderer::{GameBoardWidget, PeekWidget};
use crate::utils::Utils;

use log::debug;
use ratatui::Frame;
use ratatui::layout::Rect;

pub enum Signal {
    Alive,
    Kill,
}

struct Peek {
    is_peeking: bool,
    area: Rect,
    board_coord: (usize, usize),
}

impl Peek {
    pub fn new() -> Self {
        Peek {
            is_peeking: false,
            area: Rect::new(0, 0, 0, 0),
            board_coord: (0, 0),
        }
    }
}

pub struct Engine {
    gameboard: GameBoard,
    peek: Peek,
    reveal_coord: Option<(usize, usize)>,
}

impl Engine {
    pub fn new(width: usize, height: usize) -> Self {
        let board = GameBoard::new(height, width)
            .scatter_bombs(width * height / 8)
            .fill_info();

        Engine {
            gameboard: board,
            peek: Peek::new(),
            reveal_coord: None,
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let root_area = frame.area();
        let board_area = Utils::center_rect(
            root_area,
            self.gameboard.cols as u16,
            self.gameboard.rows as u16,
        );

        frame.render_widget(
            GameBoardWidget::new(&self.gameboard, self.reveal_coord),
            board_area,
        );

        if self.peek.is_peeking {
            frame.render_widget(PeekWidget::new(root_area, &self.gameboard), self.peek.area);
        }
    }

    pub fn start_reveal(&mut self, click_x: u16, click_y: u16, frame_area: Rect) {
        let Some(board_coord) = self.screen_to_board(click_x, click_y, frame_area) else {
            return;
        };

        self.reveal_coord = Some(board_coord);
    }

    pub fn end_reveal(&mut self) -> Signal {
        let mut signal = Signal::Alive;
        match self.reveal_coord {
            Some((x, y)) => {
                match self.gameboard.board[y][x].content {
                    CellContent::Bomb => signal = Signal::Kill,
                    _ => signal = Signal::Alive,
                }
                self.reveal_at((x, y));
                self.reveal_coord = None;
            }
            None => self.reveal_coord = None,
        }
        signal
    }

    pub fn toggle_flag(&mut self, click_x: u16, click_y: u16, frame_area: Rect) {
        let Some(board_coord) = self.screen_to_board(click_x, click_y, frame_area) else {
            return;
        };
        self.toggle_flag_at(board_coord);
    }

    pub fn start_peek(&mut self, click_x: u16, click_y: u16, frame_area: Rect) {
        let Some((board_x, board_y)) = self.screen_to_board(click_x, click_y, frame_area) else {
            return;
        };

        let pane = Utils::center_rect(
            frame_area,
            self.gameboard.cols as u16,
            self.gameboard.rows as u16,
        );

        // 3x3 around clicked cell, clipped at board edges.
        let min_bx = board_x.saturating_sub(1);
        let min_by = board_y.saturating_sub(1);
        let max_bx = (board_x + 1).min(self.gameboard.cols - 1);
        let max_by = (board_y + 1).min(self.gameboard.rows - 1);

        let area_x = pane.x + min_bx as u16;
        let area_y = pane.y + min_by as u16;
        let area_w = (max_bx - min_bx + 1) as u16;
        let area_h = (max_by - min_by + 1) as u16;

        self.peek = Peek {
            is_peeking: true,
            area: Rect::new(area_x, area_y, area_w, area_h),
            board_coord: (board_x, board_y),
        };
    }

    pub fn end_peek(&mut self) -> Signal {
        if !self.peek.is_peeking {
            return Signal::Alive;
        }
        self.peek.is_peeking = false;

        let click_cell = self.gameboard.board[self.peek.board_coord.1][self.peek.board_coord.0];
        if let CellContent::Safe(n) = click_cell.content {
            if self.count_flags(self.peek.board_coord) != n {
                return Signal::Alive;
            }
        }

        if self.flags_match(self.peek.board_coord) {
            self.reveal_neighbors(self.peek.board_coord);
        } else {
            return Signal::Kill;
        }

        Signal::Alive
    }

    pub fn cancel_peek(&mut self) -> Signal {
        if !self.peek.is_peeking {
            return Signal::Alive;
        }
        self.peek.is_peeking = false;

        Signal::Alive
    }

    fn toggle_flag_at(&mut self, (board_x, board_y): (usize, usize)) {
        let flag_cell = &mut self.gameboard.board[board_y][board_x];

        if !flag_cell.revealed {
            flag_cell.flagged = !flag_cell.flagged;
        }
    }

    fn reveal_at(&mut self, (board_x, board_y): (usize, usize)) {
        let click_cell = &mut self.gameboard.board[board_y][board_x];
        if !click_cell.revealed {
            match click_cell.content {
                CellContent::Bomb => debug!("Kill Signal"), //lose
                CellContent::Safe(0) => {
                    self.propagate_from((board_x, board_y));
                }
                CellContent::Safe(_) => click_cell.revealed = true,
            }
        }
    }

    fn propagate_from(&mut self, (x, y): (usize, usize)) {
        if self.gameboard.board[y][x].revealed {
            return;
        }

        let source_cell_content = self.gameboard.board[y][x].content;
        match source_cell_content {
            CellContent::Bomb => return,
            CellContent::Safe(0) => {
                self.gameboard.board[y][x].revealed = true;
                for offset_x in -1..=1 {
                    for offset_y in -1..=1 {
                        if let (Some(nx), Some(ny)) = (
                            x.checked_add_signed(offset_x),
                            y.checked_add_signed(offset_y),
                        ) {
                            if nx < self.gameboard.cols && ny < self.gameboard.rows {
                                self.propagate_from((nx, ny));
                            }
                        }
                    }
                }
            }
            CellContent::Safe(_) => {
                self.gameboard.board[y][x].revealed = true;
            }
        }
    }

    fn reveal_neighbors(&mut self, (x, y): (usize, usize)) {
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let (nx, ny) = (
                    x.checked_add_signed(dx).unwrap_or(0),
                    y.checked_add_signed(dy).unwrap_or(0),
                );
                if nx < self.gameboard.cols && ny < self.gameboard.rows {
                    match self.gameboard.board[ny][nx] {
                        Cell { revealed: true, .. } => continue,
                        Cell {
                            content: CellContent::Bomb,
                            ..
                        } => continue,
                        _ => self.reveal_at((nx, ny)),
                    }
                }
            }
        }
    }

    pub fn get_peek_area(&self, board_coords: (usize, usize)) -> (usize, usize, usize, usize) {
        let (board_x, board_y) = board_coords;
        let origin_x = board_x.checked_add_signed(-1).unwrap_or(0);
        let origin_y = board_y.checked_add_signed(-1).unwrap_or(0);
        let width = if board_x + 1 >= self.gameboard.cols {
            board_x
        } else {
            board_x + 1
        };
        let height = if board_y + 1 >= self.gameboard.rows {
            board_y
        } else {
            board_y + 1
        };

        (origin_x, origin_y, width, height)
    }

    fn count_flags(&self, board_coords: (usize, usize)) -> u8 {
        let (origin_x, origin_y, width, height) = self.get_peek_area(board_coords);
        let mut flag_count = 0;

        for y in origin_y..=height {
            for x in origin_x..=width {
                if self.gameboard.board[y][x].flagged {
                    flag_count += 1;
                }
            }
        }
        assert!(flag_count <= 8);
        flag_count
    }

    fn flags_match(&self, board_coords: (usize, usize)) -> bool {
        let (origin_x, origin_y, width, height) = self.get_peek_area(board_coords);

        for y in origin_y..height {
            for x in origin_x..width {
                let is_flagged = self.gameboard.board[y][x].flagged;
                let is_bomb = match self.gameboard.board[y][x].content {
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

    pub fn screen_to_board(
        &self,
        click_x: u16,
        click_y: u16,
        root_area: Rect,
    ) -> Option<(usize, usize)> {
        Utils::screen_to_board(
            click_x,
            click_y,
            self.gameboard.cols,
            self.gameboard.rows,
            root_area,
        )
    }

    pub fn get_board_cell(&self, col: usize, row: usize) -> &Cell {
        &self.gameboard.board[row][col]
    }
}
