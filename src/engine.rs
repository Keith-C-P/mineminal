use crate::gameboard::{Cell, CellContent, CellState, GameBoard};
use crate::gameinfo::GameInfo;
use crate::utils::Utils;

use log::debug;
use ratatui::layout::Rect;

pub enum Signal {
    Alive,
    Kill,
    Restart,
    Exit,
    Win,
    Pause,
    Resume,
}

pub struct Chord {
    pub is_active: bool,
    pub area: Rect,
    pub board_coord: (usize, usize),
}

impl Chord {
    pub fn new() -> Self {
        Chord {
            is_active: false,
            area: Rect::new(0, 0, 0, 0),
            board_coord: (0, 0),
        }
    }
}

pub struct Engine {
    pub gameboard: GameBoard,
    pub game_info: GameInfo,
    pub chord: Chord,
    pub reveal_coord: Option<(usize, usize)>,
    pub first_click: bool,
    pub killed_by: Option<[Option<(usize, usize)>; 8]>,
}

impl Engine {
    pub fn new(width: usize, height: usize) -> Self {
        let gameboard = GameBoard::new(height, width);
        Engine {
            gameboard,
            game_info: GameInfo::new(0),
            chord: Chord::new(),
            reveal_coord: None,
            first_click: false,
            killed_by: None,
        }
    }

    pub fn lmb_down(&mut self, click_x: u16, click_y: u16, root_area: Rect) -> Signal {
        let Some((board_x, board_y)) = self.screen_to_board(click_x, click_y, root_area) else {
            return Signal::Alive;
        };
        let cell = self.gameboard.board[board_y][board_x];
        match cell.state {
            CellState::Flagged => (),
            CellState::Revealed => self.start_chord((board_x, board_y), root_area),
            CellState::Unrevealed => self.start_reveal((board_x, board_y)),
        }
        Signal::Alive
    }

    pub fn lmb_up(&mut self, click_x: u16, click_y: u16, root_area: Rect) -> Signal {
        let Some((board_x, board_y)) = self.screen_to_board(click_x, click_y, root_area) else {
            return Signal::Alive;
        };
        let cell = self.gameboard.board[board_y][board_x];
        let result = if matches!(cell.state, CellState::Revealed) {
            self.end_chord()
        } else {
            self.end_reveal()
        };
        result
    }

    fn first_lmb(&mut self, (board_x, board_y): (usize, usize)) {
        self.gameboard.scatter_bombs(40, (board_x, board_y));
        self.gameboard.fill_info();
        self.game_info.three_bv = self.gameboard.calculate_difficulty();
        self.game_info.time.start();
    }

    fn start_reveal(&mut self, (board_x, board_y): (usize, usize)) {
        self.reveal_coord = Some((board_x, board_y));
    }

    pub fn end_reveal(&mut self) -> Signal {
        match self.reveal_coord {
            Some((x, y)) => {
                if !self.first_click {
                    self.first_lmb((x, y));
                    self.first_click = true;
                }

                if let CellContent::Bomb = self.gameboard.board[y][x].content {
                    debug!("Killed by Bomb Reveal");
                    self.game_info.record_active_clicks(1);
                    self.killed_by = Some([Some((x, y)), None, None, None, None, None, None, None]);
                    return Signal::Kill;
                }

                let count = self.reveal_at((x, y));
                let total_revealed = self.gameboard.count_revealed_cells();
                debug!("Revealed {} cell(s) ({} total)", count, total_revealed);

                //TODO: check if flagged cells are matching, dk if its necassary
                if self.gameboard.height * self.gameboard.width - self.gameboard.num_bombs
                    == total_revealed
                {
                    return Signal::Win;
                }
            }
            None => {}
        }

        self.reveal_coord = None;
        Signal::Alive
    }

    pub fn toggle_flag(&mut self, click_x: u16, click_y: u16, frame_area: Rect) {
        let Some(board_coord) = self.screen_to_board(click_x, click_y, frame_area) else {
            return;
        };
        let (is_wasted, clicks) = self.gameboard.toggle_flag_at(board_coord);
        match (is_wasted, clicks) {
            (true, x) => self.game_info.record_wasted_flags(x),
            (false, x) => self.game_info.record_active_flags(x),
        }
    }

    fn start_chord(&mut self, (board_x, board_y): (usize, usize), frame_area: Rect) {
        let pane = Utils::center(
            frame_area,
            self.gameboard.width as u16,
            self.gameboard.height as u16,
        );

        // 3x3 around clicked cell, clipped at board edges.
        let min_bx = board_x.saturating_sub(1);
        let min_by = board_y.saturating_sub(1);
        let max_bx = (board_x + 1).min(self.gameboard.width - 1);
        let max_by = (board_y + 1).min(self.gameboard.height - 1);

        let area_x = pane.x + min_bx as u16;
        let area_y = pane.y + min_by as u16;
        let area_w = (max_bx - min_bx + 1) as u16;
        let area_h = (max_by - min_by + 1) as u16;

        self.chord = Chord {
            is_active: true,
            area: Rect::new(area_x, area_y, area_w, area_h),
            board_coord: (board_x, board_y),
        };
    }

    pub fn end_chord(&mut self) -> Signal {
        if !self.chord.is_active {
            return Signal::Alive;
        }
        self.chord.is_active = false;
        let (board_x, board_y) = self.chord.board_coord;

        let click_cell = self.gameboard.board[board_y][board_x];

        //skip revealing neighbors if all neighbors are already revealed
        let revealed_neighbors = self
            .gameboard
            .get_surrounding_cells(self.chord.board_coord)
            .iter()
            .filter(|cell| match cell {
                Some((
                    _,
                    Cell {
                        state: CellState::Revealed,
                        ..
                    },
                )) => true,
                _ => false,
            })
            .count();

        if revealed_neighbors == 8 {
            debug!(
                "Useless LMB at ({},{})",
                self.chord.board_coord.0, self.chord.board_coord.1
            );
            self.game_info.record_wasted_clicks(1);
            return Signal::Alive;
        }

        // skip revealing neighbors if number of surrounding flags dont match
        if let CellContent::Safe(n) = click_cell.content {
            if self
                .gameboard
                .count_surrounding_flags(self.chord.board_coord)
                != n
            {
                debug!("Useless Chord");
                self.game_info.record_wasted_chords(1);
                return Signal::Alive;
            }
        }

        // check if flags match
        self.game_info.record_active_chords(1);
        if let Some(coords) = self.gameboard.flags_match(self.chord.board_coord) {
            debug!("Killed by Flags Mismatch");
            self.killed_by = Some(coords);
            return Signal::Kill;
        } else {
            let count = self.reveal_neighbors(self.chord.board_coord);
            let total_revealed = self.gameboard.count_revealed_cells();
            debug!(
                "Chord Revealed {} cell(s) ({} total)",
                count, total_revealed
            );
        }

        // check win
        let total_revealed = self.gameboard.count_revealed_cells();
        if self.gameboard.height * self.gameboard.width - self.gameboard.num_bombs == total_revealed
        {
            return Signal::Win;
        }

        Signal::Alive
    }

    pub fn cancel_peek(&mut self) {
        if !self.chord.is_active {
            return;
        }
        self.chord.is_active = false;
    }

    fn reveal_at(&mut self, (board_x, board_y): (usize, usize)) -> usize {
        let click_cell = &mut self.gameboard.board[board_y][board_x];
        match click_cell.state {
            CellState::Unrevealed => match click_cell.content {
                CellContent::Safe(0) => {
                    let count = self.propagate_from((board_x, board_y));
                    return count;
                }
                CellContent::Safe(_) => {
                    click_cell.state = CellState::Revealed;
                    return 1;
                }
                CellContent::Bomb => return 0,
            },
            _ => {}
        }
        0
    }

    fn propagate_from(&mut self, (x, y): (usize, usize)) -> usize {
        if let CellState::Revealed = self.gameboard.board[y][x].state {
            return 0;
        }

        let source_cell_content = self.gameboard.board[y][x].content;
        match source_cell_content {
            CellContent::Bomb => return 0,
            CellContent::Safe(0) => {
                self.gameboard.board[y][x].state = CellState::Revealed;
                let mut count = 1;
                for offset_x in -1..=1 {
                    for offset_y in -1..=1 {
                        if let (Some(nx), Some(ny)) = (
                            x.checked_add_signed(offset_x),
                            y.checked_add_signed(offset_y),
                        ) {
                            if nx < self.gameboard.width && ny < self.gameboard.height {
                                count += self.propagate_from((nx, ny));
                            }
                        }
                    }
                }
                count
            }
            CellContent::Safe(_) => {
                self.gameboard.board[y][x].state = CellState::Revealed;
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
                if nx < self.gameboard.width && ny < self.gameboard.height {
                    match self.gameboard.board[ny][nx] {
                        Cell {
                            state: CellState::Revealed,
                            ..
                        } => continue,
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
            self.gameboard.width,
            self.gameboard.height,
            root_area,
        )
    }
}
