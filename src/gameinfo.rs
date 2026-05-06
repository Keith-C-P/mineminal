use std::fmt::Display;

use crate::timer::Timer;
pub struct ActionStats {
    pub clicks: usize,
    pub flags: usize,
    pub chords: usize,
}

impl ActionStats {
    pub fn new() -> Self {
        Self {
            clicks: 0,
            flags: 0,
            chords: 0,
        }
    }

    pub fn to_clicks(&self) -> usize {
        self.clicks + self.flags + self.chords
    }
}

pub struct GameInfo {
    pub three_bv: usize,
    pub time: Timer,
    active_clicks: ActionStats,
    wasted_clicks: ActionStats,
}

impl<'a> Display for GameInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //TODO: Add Efficiency, Speed
        let time = format!("Time: {:.3}s", self.time.elapsed().as_secs_f64());
        let difficulty = format!("3BV: {}", self.three_bv);
        let clicks = format!(
            "Clicks: {}+{}",
            self.active_clicks.to_clicks(),
            self.wasted_clicks.to_clicks()
        );
        let info = format!("{}\n{}\n{}\n", time, difficulty, clicks);
        write!(f, "{}", info)
    }
}

impl GameInfo {
    pub fn new(difficulty: usize) -> Self {
        Self {
            three_bv: difficulty,
            time: Timer::new(),
            active_clicks: ActionStats::new(),
            wasted_clicks: ActionStats::new(),
        }
    }

    pub fn dimensions(&self) -> (usize, usize) {
        let height = format!("{}", self).lines().count();
        let width = format!("{}", self)
            .lines()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(0);

        (width, height)
    }

    pub fn record_active_clicks(&mut self, count: usize) {
        self.active_clicks.clicks += count;
    }
    pub fn record_wasted_clicks(&mut self, count: usize) {
        self.wasted_clicks.clicks += count;
    }
    pub fn record_active_flags(&mut self, count: usize) {
        self.active_clicks.flags += count;
    }
    pub fn record_wasted_flags(&mut self, count: usize) {
        self.wasted_clicks.flags += count;
    }
    pub fn record_active_chords(&mut self, count: usize) {
        self.active_clicks.chords += count;
    }
    pub fn record_wasted_chords(&mut self, count: usize) {
        self.wasted_clicks.chords += count;
    }
}
