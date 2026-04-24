use std::time::{Duration, Instant};

pub struct Timer {
    start: Option<Instant>,
    elapsed: Duration,
    running: bool,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: None,
            elapsed: Duration::new(0, 0),
            running: false,
        }
    }
    pub fn start(&mut self) {
        if !self.running {
            self.start = Some(Instant::now());
            self.running = true;
        }
    }
    pub fn pause(&mut self) {
        if self.running {
            if let Some(start) = self.start {
                self.elapsed += start.elapsed();
            }
            self.start = None;
            self.running = false;
        }
    }
    pub fn elapsed(&self) -> Duration {
        if self.running {
            if let Some(start) = self.start {
                return self.elapsed + start.elapsed();
            }
        }
        self.elapsed
    }
}
