use crate::engine::Signal;
use crate::game::GameContext;
use crate::renderer::{GameBoardWidget, InfoWidget, LoseBoardWidget, Three7SegmentWidget};
use crate::utils::Utils;

use crossterm::event::{
    Event as CEvent, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
};
use log::debug;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders};

pub enum Transition {
    ToPlaying,
    ToPause,
    Resume,
    ToLose,
    ToWin,
    Restart,
}

pub trait State {
    fn handle_input(&mut self, ctx: &mut GameContext, event: CEvent, root_area: Rect) -> Signal;
    fn render(&self, ctx: &GameContext, frame: &mut Frame);
    fn update(&self, signal: Signal) -> Option<Transition>;
}
pub struct PlayingState;

impl State for PlayingState {
    fn handle_input(&mut self, ctx: &mut GameContext, input: CEvent, root_area: Rect) -> Signal {
        match input {
            CEvent::Mouse(MouseEvent {
                kind, row, column, ..
            }) => {
                let engine = &mut ctx.engine;

                let total_revealed = engine.gameboard.count_revealed_cells();
                let Some((bx, by)) = engine.screen_to_board(column, row, root_area) else {
                    debug!("Revealed {} cells ({} total)", 0, total_revealed);
                    return Signal::Alive;
                };

                let cell = engine.gameboard.board[by][bx];

                debug!("Revealed {} cells ({} total)", 0, total_revealed);
                match kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        if cell.flagged {
                            return Signal::Alive;
                        }
                        if cell.revealed {
                            engine.start_peek(column, row, root_area);
                        } else {
                            engine.start_reveal(column, row, root_area);
                        }
                    }
                    MouseEventKind::Down(MouseButton::Right) => {
                        if !cell.revealed {
                            engine.toggle_flag(column, row, root_area);
                        }
                    }
                    MouseEventKind::Up(MouseButton::Left) => {
                        let result = if cell.revealed {
                            engine.end_peek()
                        } else {
                            engine.end_reveal()
                        };
                        return result;
                    }
                    MouseEventKind::Drag(MouseButton::Left) => {
                        engine.cancel_peek();
                        engine.end_reveal();
                        return Signal::Alive;
                    }
                    _ => {}
                }
                return Signal::Alive;
            }
            CEvent::Key(KeyEvent {
                code: KeyCode::Esc,
                kind: KeyEventKind::Press,
                ..
            }) => panic!("Quit"),
            CEvent::Key(KeyEvent {
                code: KeyCode::Char(' '),
                kind: KeyEventKind::Press,
                ..
            }) => return Signal::Restart,
            _ => {}
        }
        Signal::Alive
    }

    fn render(&self, ctx: &GameContext, frame: &mut Frame) {
        ctx.engine.draw(frame);
        let root_area = frame.area();
        let bomb_widget_area = Utils::top_left(root_area, 9 + 2, 3 + 2);

        let bomb_block = Block::default().borders(Borders::ALL);
        let bomb_inner = bomb_block.inner(bomb_widget_area);

        frame.render_widget(bomb_block, bomb_widget_area);
        frame.render_widget(
            Three7SegmentWidget::new(
                ctx.engine.gameboard.num_bombs as isize - ctx.engine.gameboard.num_flags as isize,
            ),
            bomb_inner,
        );
    }

    fn update(&self, signal: Signal) -> Option<Transition> {
        match signal {
            Signal::Kill => Some(Transition::ToLose),
            Signal::Restart => Some(Transition::Restart),
            Signal::Win => Some(Transition::ToWin),
            Signal::Alive => None,
            _ => None,
        }
    }
}

pub struct LoseState;

impl State for LoseState {
    fn handle_input(&mut self, _ctx: &mut GameContext, event: CEvent, _: Rect) -> Signal {
        match event {
            CEvent::Key(key) => match key {
                KeyEvent {
                    code: KeyCode::Esc,
                    kind: KeyEventKind::Press,
                    ..
                } => Signal::Exit,
                KeyEvent {
                    code: KeyCode::Char(' '),
                    kind: KeyEventKind::Press,
                    ..
                } => Signal::Restart,
                _ => Signal::Alive,
            },
            _ => Signal::Alive,
        }
    }

    fn render(&self, ctx: &GameContext, frame: &mut Frame) {
        let engine = &ctx.engine;
        let (width, height) = ctx.engine.game_info.dimensions();
        let root_area = frame.area();

        let area = Utils::center_right(root_area, (width + 2) as u16, (height + 2) as u16);
        // area.x -= 1; // FIXME accounting for right edge which will overflow

        let block = Block::default().borders(Borders::ALL).title("You Lose :( ");
        let inner = block.inner(area);

        let board_area = Utils::center(
            root_area,
            engine.gameboard.cols as u16,
            engine.gameboard.rows as u16,
        );

        frame.render_widget(block, area);
        frame.render_widget(InfoWidget::new(&ctx.engine.game_info), inner);
        frame.render_widget(
            LoseBoardWidget::new(&ctx.engine.gameboard, (0, 0)),
            board_area,
        );
    }

    fn update(&self, signal: Signal) -> Option<Transition> {
        match signal {
            Signal::Exit => todo!("Exit"),
            Signal::Restart => Some(Transition::Restart),
            _ => None,
        }
    }
}
pub struct WinState;

impl State for WinState {
    fn handle_input(&mut self, _ctx: &mut GameContext, event: CEvent, _root_area: Rect) -> Signal {
        match event {
            CEvent::Key(key) => match key {
                KeyEvent {
                    code: KeyCode::Esc,
                    kind: KeyEventKind::Press,
                    ..
                } => Signal::Exit,
                KeyEvent {
                    code: KeyCode::Char(' '),
                    kind: KeyEventKind::Press,
                    ..
                } => Signal::Restart,
                _ => Signal::Alive,
            },
            _ => Signal::Alive,
        }
    }

    fn render(&self, ctx: &GameContext, frame: &mut Frame) {
        let engine = &ctx.engine;
        let (width, height) = ctx.engine.game_info.dimensions();
        let root_area = frame.area();

        let area = Utils::center_right(root_area, (width + 2) as u16, (height + 2) as u16);

        let block = Block::default().borders(Borders::ALL).title("You Win :) ");
        let inner = block.inner(area);

        let board_area = Utils::center(
            root_area,
            engine.gameboard.cols as u16,
            engine.gameboard.rows as u16,
        );

        frame.render_widget(block, area);
        frame.render_widget(InfoWidget::new(&ctx.engine.game_info), inner);
        frame.render_widget(
            GameBoardWidget::new(&ctx.engine.gameboard, None),
            board_area,
        );
    }

    fn update(&self, signal: Signal) -> Option<Transition> {
        match signal {
            Signal::Exit => todo!("Exit"),
            Signal::Restart => Some(Transition::Restart),
            _ => None,
        }
    }
}
