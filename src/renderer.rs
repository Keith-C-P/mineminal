use crate::gameboard::CellContent;
use crate::gameboard::GameBoard;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Style};
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

    pub fn center_rect(root_area: Rect, width: u16, height: u16) -> Rect {
        // let vertical = Layout::default()
        //     .direction(Direction::Vertical)
        //     .constraints([
        //         Constraint::Percentage(50),
        //         Constraint::Length(height),
        //         Constraint::Percentage(50),
        //     ])
        //     .split(root_area);

        // let horizontal = Layout::default()
        //     .direction(Direction::Horizontal)
        //     .constraints([
        //         Constraint::Percentage(50),
        //         Constraint::Length(width),
        //         Constraint::Percentage(50),
        //     ])
        //     .split(vertical[1]);

        // horizontal[1]

        root_area.centered(Constraint::Length(width), Constraint::Length(height))
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

pub struct PeekWidget;

impl Widget for PeekWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        for y in area.y..(area.y + area.height) {
            for x in area.x..(area.x + area.width) {
                if buf[(x, y)].symbol() != "▇" {
                    buf.set_string(x, y, " ", Style::default().bg(Color::DarkGray));
                }
            }
        }
    }
}
