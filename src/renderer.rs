use crate::engine::GameInfo;
use crate::gameboard::{Cell, CellContent, CellState, GameBoard};
use crate::utils::Utils;

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;
use ratatui::{buffer::Buffer, widgets::Widget};

pub struct GameBoardWidget<'a> {
    gameboard: &'a GameBoard,
    reveal: Option<(usize, usize)>,
}

impl<'a> GameBoardWidget<'a> {
    pub fn new(board: &'a GameBoard, reveal_coord: Option<(usize, usize)>) -> Self {
        GameBoardWidget {
            gameboard: board,
            reveal: reveal_coord,
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
                let (symbol, style) = match cell {
                    Cell {
                        state: CellState::Flagged,
                        ..
                    } => ("F", Style::default().bg(Color::DarkGray)),
                    Cell {
                        state: CellState::Unrevealed,
                        ..
                    } => ("▇", Style::default().bg(Color::DarkGray)),
                    Cell { content, .. } => match content {
                        CellContent::Bomb => ("*", Style::default().bg(Color::DarkGray)),
                        CellContent::Safe(n) => {
                            let text = match n {
                                //TODO add colours
                                0 => " ",
                                1 => "1",
                                2 => "2",
                                3 => "3",
                                4 => "4",
                                5 => "5",
                                6 => "6",
                                7 => "7",
                                8 => "8",
                                _ => "?",
                            };
                            (text, Style::default().bg(Color::DarkGray))
                        }
                    },
                };
                let draw_x = area.x + x as u16;
                let draw_y = area.y + y as u16;
                buf.set_string(draw_x, draw_y, symbol, style);
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
            self.gameboard.cols,
            self.gameboard.rows,
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
            self.gameboard.cols,
            self.gameboard.rows,
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

pub struct LoseBoardWidget<'a> {
    gameboard: &'a GameBoard,
    killed_by: (usize, usize),
}

impl<'a> LoseBoardWidget<'a> {
    pub fn new(gameboard: &'a GameBoard, killed_by: (usize, usize)) -> Self {
        Self {
            gameboard,
            killed_by,
        }
    }
}

impl<'a> Widget for LoseBoardWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        for (y, row) in self.gameboard.board.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                //TODO implement colors and killed by
                let (symbol, style) = match cell {
                    Cell {
                        state: CellState::Flagged,
                        ..
                    } => ("F", Style::default().bg(Color::DarkGray)),
                    Cell {
                        content: CellContent::Bomb,
                        ..
                    } => ("*", Style::default().bg(Color::DarkGray)),
                    Cell {
                        state: CellState::Revealed,
                        ..
                    } => ("▇", Style::default().bg(Color::DarkGray)),
                    Cell {
                        content: CellContent::Safe(number),
                        ..
                    } => {
                        let text = match number {
                            0 => " ",
                            1 => "1",
                            2 => "2",
                            3 => "3",
                            4 => "4",
                            5 => "5",
                            6 => "6",
                            7 => "7",
                            8 => "8",
                            _ => "?",
                        };
                        (text, Style::default().bg(Color::DarkGray))
                    }
                };
                let draw_x = area.x + x as u16;
                let draw_y = area.y + y as u16;
                buf.set_string(draw_x, draw_y, symbol, style);
            }
        }
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
