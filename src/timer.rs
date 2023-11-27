use std::time::{Duration, Instant};

pub enum Mode {
    Once,
    Repeating,
}

pub struct Timer {
    duration: Duration,
    start_time: Option<Instant>,
    mode: Mode,
    running: bool,
}

impl Timer {
    pub fn new(duration: Duration, mode: Mode) -> Self {
        Self {
            duration,
            start_time: None,
            mode,
            running: false,
        }
    }

    pub fn start(&mut self) {
        self.running = true;
        self.start_time = Some(Instant::now());
    }

    pub fn finished(&mut self) -> bool {
        if !(self.running && self.start_time.is_some()) {
            false
        } else if self.start_time.unwrap().elapsed() >= self.duration {
            if let Mode::Repeating = self.mode {
                self.update()
            }
            true
        } else {
            false
        }
    }

    pub fn time_left(&mut self) -> Duration {
        if self.running {
            self.duration - self.start_time.unwrap().elapsed()
        } else {
            self.duration
        }
    }

    pub fn update(&mut self) {
        self.start_time = Some(Instant::now()
            + Duration::from_nanos(
                self.start_time.unwrap().elapsed().as_nanos() as u64 % self.duration.as_nanos() as u64,
            ));
    }
}
