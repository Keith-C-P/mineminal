use crate::engine::Engine;
use crossterm::event::{
    Event as CEvent, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::Frame;
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::rc::Rc;

trait State {
    fn handle_input(&mut self, event: CEvent, root_area: Rect) -> Option<Box<dyn State>>;
    fn update(&mut self) -> Option<Box<dyn State>>;
    fn render(&mut self, frame: &mut Frame);
}

pub struct Game {
    state: Option<Box<dyn State>>,
    engine: Rc<RefCell<Engine>>,
}

impl Game {
    pub fn new() -> Self {
        let engine = Rc::new(RefCell::new(Engine::new(16, 16)));
        Self {
            engine,
            state: None,
        }
    }

    pub fn run(&mut self) {
        self.state = Some(Box::new(PlayingState::new(Rc::clone(&self.engine))));
    }

    pub fn draw(&self, frame: &mut Frame) {
        if let Some(_) = &self.state {
            self.engine.borrow().draw(frame);
        }
    }

    pub fn handle_input(&mut self, event: CEvent, root_area: Rect) {
        if let Some(state) = self.state.as_mut() {
            if let Some(next_state) = state.handle_input(event, root_area) {
                self.state = Some(next_state);
            }
        }
    }
}

// struct TitleState;
struct PlayingState {
    engine: Rc<RefCell<Engine>>,
}
// struct Paused;
// struct Lose;
// struct Win;
// struct Title;

// impl State for TitleState {
//     fn handle_input(&mut self, input: CEvent) -> Option<Box<dyn State>> {
//         match input {
//             CEvent::Key(key) => match key.code {
//                 KeyCode::Enter => Some(Box::new(PlayingState::new())),
//                 KeyCode::Char('q') => None,
//                 _ => None,
//             },
//             _ => None,
//         }
//     }

//     fn render(&self, frame: &mut Frame) {}
// }

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
                            if click_cell.revealed {
                                engine_mut.end_peek();
                            } else {
                                engine_mut.end_reveal();
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

    fn update(&mut self) -> Option<Box<dyn State>> {
        None
    }

    fn render(&mut self, frame: &mut Frame) {
        self.engine.borrow_mut().draw(frame);
    }
}
