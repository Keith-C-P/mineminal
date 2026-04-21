use core::time::Duration;

use crate::engine::GameInfo;
use crate::gameboard::CellContent;
use crate::gameboard::GameBoard;
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
                let (symbol, style) = if cell.flagged {
                    ("F", Style::default().bg(Color::DarkGray))
                } else if !cell.revealed {
                    ("▇", Style::default().bg(Color::DarkGray))
                } else {
                    match cell.content {
                        CellContent::Bomb => ("💣", Style::default().bg(Color::DarkGray)),
                        CellContent::Safe(n) => {
                            let text = match n {
                                // add colours
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
                    }
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
            Some((x, y)) => self.gameboard.board[y][x].revealed,
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
            Some((x, y)) => self.gameboard.board[y][x].flagged,
            None => false,
        }
    }
}

pub struct LoseWidget<'a> {
    game_info: &'a GameInfo,
}

impl<'a> Widget for LoseWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let game_info = format!("{}", self.game_info);
        Paragraph::new(game_info).render(area, buf);
    }
}

impl<'a> LoseWidget<'a> {
    pub fn new(game_info: &'a GameInfo) -> Self {
        LoseWidget { game_info }
    }
}
