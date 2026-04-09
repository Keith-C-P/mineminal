use crate::engine::Engine;
use crate::engine::Signal;

use std::cell::RefCell;

use crossterm::event::{
    Event as CEvent, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::Frame;
use ratatui::layout::Rect;
use std::rc::Rc;

pub trait State {
    fn handle_input(&mut self, event: CEvent, root_area: Rect) -> Option<Box<dyn State>>;
    fn render(&self, frame: &mut Frame);
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
    fn handle_input(&mut self, input: CEvent, root_area: Rect) -> Option<Box<dyn State>> {
        match input {
            CEvent::Mouse(mouse_event) => match mouse_event {
                MouseEvent {
                    kind, row, column, ..
                } => match kind {
                    MouseEventKind::Down(mouse_button) => match mouse_button {
                        MouseButton::Left => {
                            let mut engine_mut = self.engine.borrow_mut();
                            let Some((board_x, board_y)) =
                                engine_mut.screen_to_board(column, row, root_area)
                            else {
                                return None;
                            };
                            let click_cell = engine_mut.get_board_cell(board_x, board_y);
                            if click_cell.flagged {
                                return None;
                            }
                            if click_cell.revealed {
                                engine_mut.start_peek(column, row, root_area);
                            } else {
                                engine_mut.start_reveal(column, row, root_area);
                            }
                            return None;
                        }
                        MouseButton::Right => {
                            let mut engine_mut = self.engine.borrow_mut();
                            let Some((board_x, board_y)) =
                                engine_mut.screen_to_board(column, row, root_area)
                            else {
                                return None;
                            };
                            let click_cell = engine_mut.get_board_cell(board_x, board_y);
                            if !click_cell.revealed {
                                engine_mut.toggle_flag(column, row, root_area);
                            }
                            return None;
                        }
                        _ => return None,
                    },
                    MouseEventKind::Up(mouse_button) => match mouse_button {
                        MouseButton::Left => {
                            let mut engine_mut = self.engine.borrow_mut();
                            let Some((board_x, board_y)) =
                                engine_mut.screen_to_board(column, row, root_area)
                            else {
                                return None;
                            };
                            let click_cell = engine_mut.get_board_cell(board_x, board_y);
                            let mut result;
                            if click_cell.revealed {
                                result = engine_mut.end_peek();
                            } else {
                                result = engine_mut.end_reveal();
                            }
                            match result {
                                Signal::Kill => return Some(Box::new(LoseState::new())),
                                _ => return None,
                            }
                            return None;
                        }
                        _ => return None,
                    },
                    MouseEventKind::Drag(mouse_button) => match mouse_button {
                        MouseButton::Left => {
                            let mut engine_mut = self.engine.borrow_mut();
                            engine_mut.cancel_peek();
                            engine_mut.end_reveal();
                            return None;
                        }
                        _ => return None,
                    },
                    _ => return None,
                },
            },
            CEvent::Key(key) => match key {
                KeyEvent {
                    code: KeyCode::Esc,
                    kind: KeyEventKind::Press,
                    ..
                } => {
                    panic!("Quit")
                }
                _ => todo!(),
            },
            _ => todo!(),
        };
    }

    fn render(&self, frame: &mut Frame) {
        self.engine.borrow().draw(frame);
    }
}

pub struct LoseState;

impl LoseState {
    pub fn new() -> Self {
        LoseState
    }
}

impl State for LoseState {
    fn handle_input(&mut self, event: CEvent, _: Rect) -> Option<Box<dyn State>> {
        match event {
            CEvent::Key(key) => match key {
                KeyEvent {
                    code: KeyCode::Esc,
                    kind: KeyEventKind::Press,
                    ..
                } => {
                    panic!("Quit")
                }
                _ => todo!(),
            },
            _ => todo!(),
        }
        None
    }
    fn render(&self, frame: &mut Frame) {}
}
