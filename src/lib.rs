use std::fmt;
use std::time;

pub struct Timer {
    duration: time::Duration,
    start_time: Option<time::Instant>,
}

impl Timer {
    pub fn new(minutes: u64) -> Self {
        let duration = time::Duration::from_secs(minutes * 60);
        Timer {
            duration,
            start_time: None,
        }
    }

    pub fn start(&mut self) {
        self.start_time = Some(time::Instant::now());
    }

    fn elapsed(&self) -> time::Duration {
        match self.start_time {
            Some(start_time) => start_time.elapsed(),
            None => time::Duration::from_secs(0),
        }
    }

    fn remaining(&self) -> time::Duration {
        if self.elapsed() >= self.duration {
            return time::Duration::from_secs(0);
        }
        self.duration - self.elapsed()
    }
}

impl fmt::Display for Timer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let remaining = self.remaining();
        let (minutes, seconds) = get_min_sec_from_duration(remaining);
        write!(f, "{:02}:{:02}", minutes, seconds)
    }
}

fn get_min_sec_from_duration(duration: time::Duration) -> (u64, u64) {
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    (minutes, seconds)
}
