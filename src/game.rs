use crate::engine::{Engine, GameInfo};
use crate::state::{LoseState, State};
use crate::state::{PlayingState, Transition};

use crossterm::event::Event as CEvent;
use ratatui::Frame;
use ratatui::layout::Rect;

pub struct GameContext {
    pub engine: Engine,
    pub game_info: GameInfo,
}

impl GameContext {
    pub fn new() -> Self {
        Self {
            engine: Engine::new(16, 16),
            game_info: GameInfo::new(0),
        }
    }

    pub fn with_size(width: usize, height: usize) -> Self {
        Self {
            engine: Engine::new(width, height),
            game_info: GameInfo::new(0),
        }
    }
}

pub struct Game {
    state: Option<Box<dyn State>>,
    ctx: Option<GameContext>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: None,
            ctx: None,
        }
    }

    pub fn run(&mut self) {
        self.ctx = Some(GameContext::new());
        self.state = Some(Box::new(PlayingState));
    }

    pub fn draw(&self, frame: &mut Frame) {
        if let Some(state) = &self.state {
            state.render(&self.ctx.as_ref().unwrap(), frame);
        }
    }

    pub fn handle_input(&mut self, event: CEvent, root_area: Rect) {
        if let Some(state) = self.state.as_mut() {
            let signal = state.handle_input(&mut self.ctx.as_mut().unwrap(), event, root_area);

            if let Some(transition) = state.update(&mut self.ctx.as_mut().unwrap(), signal) {
                match transition {
                    Transition::ToPlaying => self.state = Some(Box::new(PlayingState)),
                    Transition::ToPause => todo!("Not Implemented Pause"),
                    Transition::Resume => todo!("Not Implemented Pause"),
                    Transition::ToLose => self.state = Some(Box::new(LoseState)),
                    Transition::ToWin => todo!("Not Implemented Win"),
                    Transition::Restart => {
                        self.ctx = Some(GameContext::new());
                        self.state = Some(Box::new(PlayingState));
                    }
                }
            }
        }
    }
}
