use crate::colors::Colors;
use crate::gameboard::{Cell, CellContent, CellState, GameBoard};
use crate::gameinfo::GameInfo;
use crate::utils::Utils;

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;
use ratatui::{buffer::Buffer, widgets::Widget};

pub enum GameBoardState {
    Lose,
    Win,
    Playing,
}
pub struct GameBoardWidget<'a> {
    gameboard: &'a GameBoard,
    gameboard_state: GameBoardState,
    colors: &'a Colors,
    reveal: Option<(usize, usize)>,
    killed_by: Option<[Option<(usize, usize)>; 8]>,
}

impl<'a> GameBoardWidget<'a> {
    pub fn new(
        gameboard: &'a GameBoard,
        gameboard_state: GameBoardState,
        colors: &'a Colors,
        reveal_coord: Option<(usize, usize)>,
        killed_by: Option<[(usize, usize); 8]>,
    ) -> Self {
        GameBoardWidget {
            gameboard,
            gameboard_state,
            colors,
            reveal: reveal_coord,
            killed_by: None,
        }
    }

    pub fn lose(
        gameboard: &'a GameBoard,
        colors: &'a Colors,
        killed_by: [Option<(usize, usize)>; 8],
    ) -> Self {
        Self {
            gameboard,
            gameboard_state: GameBoardState::Lose,
            colors,
            reveal: None,
            killed_by: Some(killed_by),
        }
    }

    pub fn win(gameboard: &'a GameBoard, colors: &'a Colors) -> Self {
        Self {
            gameboard,
            gameboard_state: GameBoardState::Win,
            colors,
            reveal: None,
            killed_by: None,
        }
    }

    pub fn playing(
        gameboard: &'a GameBoard,
        colors: &'a Colors,
        reveal_coord: Option<(usize, usize)>,
    ) -> Self {
        Self {
            gameboard,
            gameboard_state: GameBoardState::Playing,
            colors,
            reveal: reveal_coord,
            killed_by: None,
        }
    }
}

impl<'a> Widget for GameBoardWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        for (y, row) in self.gameboard.board.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                match self.gameboard_state {
                    GameBoardState::Lose => {
                        let (symbol, style) = match cell {
                            Cell {
                                state: CellState::Flagged,
                                ..
                            } => {
                                let is_killed_flag = self
                                    .killed_by
                                    .is_some_and(|coords| coords.contains(&Some((x, y))));
                                if is_killed_flag {
                                    ("F", self.colors.kill_flag())
                                } else {
                                    ("F", self.colors.flag())
                                }
                            }
                            Cell {
                                content: CellContent::Bomb,
                                ..
                            } => {
                                let is_killed_bomb = self
                                    .killed_by
                                    .is_some_and(|coords| coords.contains(&Some((x, y))));
                                if is_killed_bomb {
                                    ("*", self.colors.kill_bomb())
                                } else {
                                    ("*", self.colors.bomb())
                                }
                            }

                            Cell {
                                state: CellState::Unrevealed,
                                ..
                            } => ("▇", self.colors.foreground()),
                            Cell { content, .. } => match content {
                                CellContent::Safe(n) => match n {
                                    1 => ("1", self.colors.number(*n)),
                                    2 => ("2", self.colors.number(*n)),
                                    3 => ("3", self.colors.number(*n)),
                                    4 => ("4", self.colors.number(*n)),
                                    5 => ("5", self.colors.number(*n)),
                                    6 => ("6", self.colors.number(*n)),
                                    7 => ("7", self.colors.number(*n)),
                                    8 => ("8", self.colors.number(*n)),
                                    _ => (" ", self.colors.number(*n)),
                                },
                                _ => ("?", self.colors.flag()),
                            },
                        };
                        let draw_x = area.x + x as u16;
                        let draw_y = area.y + y as u16;
                        buf.set_string(draw_x, draw_y, symbol, style)
                    }
                    GameBoardState::Win => {
                        let (symbol, style) = match cell {
                            Cell {
                                state: CellState::Flagged,
                                ..
                            } => ("F", self.colors.flag()),
                            Cell {
                                state: CellState::Unrevealed,
                                ..
                            } => ("F", self.colors.flag()),
                            Cell { content, .. } => match content {
                                CellContent::Safe(n) => match n {
                                    1 => ("1", self.colors.number(*n)),
                                    2 => ("2", self.colors.number(*n)),
                                    3 => ("3", self.colors.number(*n)),
                                    4 => ("4", self.colors.number(*n)),
                                    5 => ("5", self.colors.number(*n)),
                                    6 => ("6", self.colors.number(*n)),
                                    7 => ("7", self.colors.number(*n)),
                                    8 => ("8", self.colors.number(*n)),
                                    _ => (" ", self.colors.number(*n)),
                                },
                                _ => ("?", self.colors.flag()),
                            },
                        };
                        let draw_x = area.x + x as u16;
                        let draw_y = area.y + y as u16;
                        buf.set_string(draw_x, draw_y, symbol, style);
                    }
                    GameBoardState::Playing => {
                        let (symbol, style) = match cell {
                            Cell {
                                state: CellState::Flagged,
                                ..
                            } => ("F", self.colors.flag()),
                            Cell {
                                state: CellState::Unrevealed,
                                ..
                            } => ("▇", self.colors.foreground()),
                            Cell { content, .. } => match content {
                                CellContent::Safe(n) => match n {
                                    1 => ("1", self.colors.number(*n)),
                                    2 => ("2", self.colors.number(*n)),
                                    3 => ("3", self.colors.number(*n)),
                                    4 => ("4", self.colors.number(*n)),
                                    5 => ("5", self.colors.number(*n)),
                                    6 => ("6", self.colors.number(*n)),
                                    7 => ("7", self.colors.number(*n)),
                                    8 => ("8", self.colors.number(*n)),
                                    _ => (" ", self.colors.number(*n)),
                                },
                                _ => ("?", self.colors.flag()),
                            },
                        };
                        let draw_x = area.x + x as u16;
                        let draw_y = area.y + y as u16;
                        buf.set_string(draw_x, draw_y, symbol, style);
                    }
                }
            }
        }

        match self.reveal {
            Some((x, y)) => {
                let draw_x = area.x + x as u16;
                let draw_y = area.y + y as u16;
                buf.set_string(draw_x, draw_y, " ", Style::default().bg(Color::DarkGray));
            }
            None => (),
        }
    }
}

pub struct PeekWidget<'a> {
    root_area: Rect,
    gameboard: &'a GameBoard,
}

impl<'a> Widget for PeekWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        for y in area.y..(area.y + area.height) {
            for x in area.x..(area.x + area.width) {
                if !self.is_revealed(x, y) && !self.is_flagged(x, y) {
                    buf.set_string(x, y, " ", Style::default().bg(Color::DarkGray));
                }
            }
        }
    }
}

impl<'a> PeekWidget<'a> {
    pub fn new(root_area: Rect, board: &'a GameBoard) -> Self {
        PeekWidget {
            root_area,
            gameboard: board,
        }
    }

    pub fn is_revealed(&self, click_x: u16, click_y: u16) -> bool {
        let coords = Utils::screen_to_board(
            click_x,
            click_y,
            self.gameboard.width,
            self.gameboard.height,
            self.root_area,
        );
        match coords {
            Some((x, y)) => matches!(self.gameboard.board[y][x].state, CellState::Revealed),
            None => false,
        }
    }

    pub fn is_flagged(&self, click_x: u16, click_y: u16) -> bool {
        let coords = Utils::screen_to_board(
            click_x,
            click_y,
            self.gameboard.width,
            self.gameboard.height,
            self.root_area,
        );
        match coords {
            Some((x, y)) => matches!(self.gameboard.board[y][x].state, CellState::Flagged),
            None => false,
        }
    }
}

pub struct InfoWidget<'a> {
    game_info: &'a GameInfo,
}

impl<'a> InfoWidget<'a> {
    pub fn new(game_info: &'a GameInfo) -> Self {
        InfoWidget { game_info }
    }
}

impl<'a> Widget for InfoWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let game_info = format!("{}", self.game_info);
        Paragraph::new(game_info).render(area, buf);
    }
}

pub struct Three7SegmentWidget {
    number: isize,
}

impl Three7SegmentWidget {
    pub fn new(number: isize) -> Self {
        Self { number }
    }
}

impl Widget for Three7SegmentWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let text = Utils::num_to_big_text(self.number);
        Paragraph::new(text)
            .style(Style::new().bold())
            .render(area, buf);
    }
}
