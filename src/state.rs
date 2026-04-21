use crate::engine::Engine;
use crate::engine::Signal;
use crate::gameboard::GameBoard;
use crate::renderer::GameBoardWidget;
use crate::renderer::LoseWidget;
use crate::utils::Utils;

use crossterm::event::{
    Event as CEvent, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders};
use std::cell::RefCell;
use std::rc::Rc;

pub trait State {
    fn handle_input(&mut self, event: CEvent, root_area: Rect) -> Signal;
    fn render(&self, frame: &mut Frame);
    fn update(&self, signal: Signal) -> Option<Box<dyn State>>;
}
pub struct PlayingState {
    engine: Rc<RefCell<Engine>>,
}

impl PlayingState {
    pub fn new(engine: Rc<RefCell<Engine>>) -> Self {
        PlayingState { engine }
    }
}

impl State for PlayingState {
    fn handle_input(&mut self, input: CEvent, root_area: Rect) -> Signal {
        match input {
            CEvent::Mouse(MouseEvent {
                kind, row, column, ..
            }) => {
                let mut engine = self.engine.borrow_mut();

                let Some((bx, by)) = engine.screen_to_board(column, row, root_area) else {
                    return Signal::Alive;
                };

                let cell = engine.get_board_cell(bx, by);

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
            }) => {
                panic!("Quit")
            }
            _ => {}
        }
        Signal::Alive
    }

    fn render(&self, frame: &mut Frame) {
        self.engine.borrow().draw(frame);
    }

    fn update(&self, signal: Signal) -> Option<Box<dyn State>> {
        match signal {
            Signal::Kill => Some(Box::new(LoseState::new(Rc::clone(&self.engine)))),
            Signal::Alive => None,
            _ => None,
        }
    }
}

pub struct LoseState {
    engine: Rc<RefCell<Engine>>,
}

impl LoseState {
    pub fn new(engine: Rc<RefCell<Engine>>) -> Self {
        LoseState { engine }
    }
}

impl State for LoseState {
    fn handle_input(&mut self, event: CEvent, _: Rect) -> Signal {
        match event {
            CEvent::Key(key) => match key {
                KeyEvent {
                    code: KeyCode::Esc,
                    kind: KeyEventKind::Press,
                    ..
                } => {
                    panic!("Quit")
                }
                _ => Signal::Alive,
            },
            _ => Signal::Alive,
        }
    }

    fn render(&self, frame: &mut Frame) {
        let engine = self.engine.borrow();
        let (width, height) = engine.game_info.dimensions();

        let mut area = Utils::center_right(frame.area(), (width + 2) as u16, (height + 2) as u16);
        area.x -= 1; // FIXME accounting for right edge which will overflow

        let block = Block::default().borders(Borders::ALL).title("You Lose :( ");
        let inner = block.inner(area);

        frame.render_widget(block, area);
        frame.render_widget(LoseWidget::new(&engine.game_info), inner);
        engine.draw(frame);
    }

    fn update(&self, signal: Signal) -> Option<Box<dyn State>> {
        None
    }
}
