use crate::engine::Engine;
use crate::state::PlayingState;
use crate::state::State;

use crossterm::event::Event as CEvent;
use ratatui::Frame;
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::rc::Rc;

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
        if let Some(state) = &self.state {
            state.render(frame);
        }
    }

    pub fn handle_input(&mut self, event: CEvent, root_area: Rect) {
        if let Some(state) = self.state.as_mut() {
            let signal = state.handle_input(event, root_area);
            if let Some(next_state) = state.update(signal) {
                self.state = Some(next_state);
            }
        }
    }
}
