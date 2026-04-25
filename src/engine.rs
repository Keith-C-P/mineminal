use crate::gameboard::{Cell, CellContent, GameBoard};
use crate::renderer::{GameBoardWidget, PeekWidget};
use crate::timer::Timer;
use crate::utils::Utils;

use log::debug;
use ratatui::Frame;
use ratatui::layout::Rect;
use std::fmt::Display;

pub enum Signal {
    Alive,
    Kill,
    Restart,
    Exit,
    Win,
    Pause,
    Resume,
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

pub struct GameInfo {
    pub three_bv: usize,
    pub time: Timer,
    clicks: usize,
    redundant_clicks: usize,
}

impl<'a> Display for GameInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //TODO: Add Efficiency, Speed
        let time = format!("Time: {:.3}s", self.time.elapsed().as_secs_f64());
        let difficulty = format!("3BV: {}", self.three_bv);
        let clicks = format!("Clicks: {}+{}", self.clicks, self.redundant_clicks);
        let info = format!("{}\n{}\n{}\n", time, difficulty, clicks);
        write!(f, "{}", info)
    }
}

impl GameInfo {
    pub fn new(difficulty: usize) -> Self {
        Self {
            three_bv: difficulty,
            time: Timer::new(),
            clicks: 0,
            redundant_clicks: 0,
        }
    }

    pub fn dimensions(&self) -> (usize, usize) {
        let height = format!("{}", self).lines().count();
        let width = format!("{}", self)
            .lines()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(0);

        (width, height)
    }
}

pub struct Engine {
    pub gameboard: GameBoard,
    pub game_info: GameInfo,
    peek: Peek,
    reveal_coord: Option<(usize, usize)>,
    first_click: bool,
}

impl Engine {
    pub fn new(width: usize, height: usize) -> Self {
        let gameboard = GameBoard::new(height, width);
        Engine {
            gameboard,
            game_info: GameInfo::new(0),
            peek: Peek::new(),
            reveal_coord: None,
            first_click: false,
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let root_area = frame.area();
        let board_area = Utils::center(
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

    fn first_click(&mut self, (board_x, board_y): (usize, usize)) {
        self.gameboard.scatter_bombs(40, (board_x, board_y));
        self.gameboard.fill_info();
        self.game_info.three_bv = self.gameboard.calculate_difficulty();
        self.game_info.time.start();
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
                if !self.first_click {
                    self.first_click((x, y));
                    self.first_click = true;
                }

                match self.gameboard.board[y][x].content {
                    CellContent::Bomb => signal = Signal::Kill,
                    _ => signal = Signal::Alive,
                }
                let count = self.reveal_at((x, y));
                let total_revealed = self.gameboard.count_revealed_cells();
                if self.gameboard.rows * self.gameboard.cols - self.gameboard.num_bombs
                    == total_revealed
                {
                    debug!("Revealed {} cells ({} total)", count, total_revealed);
                    return Signal::Win;
                }
            }
            None => {
                let total_revealed = self.gameboard.count_revealed_cells();
                debug!("Revealed {} cells ({} total)", 0, total_revealed);
            }
        }

        self.reveal_coord = None;
        signal
    }

    pub fn toggle_flag(&mut self, click_x: u16, click_y: u16, frame_area: Rect) {
        let Some(board_coord) = self.screen_to_board(click_x, click_y, frame_area) else {
            return;
        };
        self.gameboard.toggle_flag_at(board_coord);
    }

    pub fn start_peek(&mut self, click_x: u16, click_y: u16, frame_area: Rect) {
        let Some((board_x, board_y)) = self.screen_to_board(click_x, click_y, frame_area) else {
            return;
        };

        let pane = Utils::center(
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
            if self
                .gameboard
                .count_surrounding_flags(self.peek.board_coord)
                != n
            {
                return Signal::Alive;
            }
        }

        if self.gameboard.flags_match(self.peek.board_coord) {
            self.reveal_neighbors(self.peek.board_coord);
        } else {
            return Signal::Kill;
        }

        let total_revealed = self.gameboard.count_revealed_cells();
        if self.gameboard.rows * self.gameboard.cols - self.gameboard.num_bombs == total_revealed {
            return Signal::Win;
        }

        Signal::Alive
    }

    pub fn cancel_peek(&mut self) {
        if !self.peek.is_peeking {
            return;
        }
        self.peek.is_peeking = false;
    }

    fn reveal_at(&mut self, (board_x, board_y): (usize, usize)) -> usize {
        let click_cell = &mut self.gameboard.board[board_y][board_x];
        if !click_cell.revealed {
            match click_cell.content {
                CellContent::Safe(0) => {
                    let count = self.propagate_from((board_x, board_y));
                    return count;
                }
                CellContent::Safe(_) => {
                    click_cell.revealed = true;
                    return 1;
                }
                CellContent::Bomb => return 0,
            }
        }
        0
    }

    fn propagate_from(&mut self, (x, y): (usize, usize)) -> usize {
        if self.gameboard.board[y][x].revealed {
            return 0;
        }

        let source_cell_content = self.gameboard.board[y][x].content;
        match source_cell_content {
            CellContent::Bomb => return 0,
            CellContent::Safe(0) => {
                self.gameboard.board[y][x].revealed = true;
                let mut count = 1;
                for offset_x in -1..=1 {
                    for offset_y in -1..=1 {
                        if let (Some(nx), Some(ny)) = (
                            x.checked_add_signed(offset_x),
                            y.checked_add_signed(offset_y),
                        ) {
                            if nx < self.gameboard.cols && ny < self.gameboard.rows {
                                count += self.propagate_from((nx, ny));
                            }
                        }
                    }
                }
                count
            }
            CellContent::Safe(_) => {
                self.gameboard.board[y][x].revealed = true;
                1
            }
        }
    }

    fn reveal_neighbors(&mut self, (x, y): (usize, usize)) -> usize {
        let mut count = 0;
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
                        _ => count += self.reveal_at((nx, ny)),
                    }
                }
            }
        }
        count
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
}
