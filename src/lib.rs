use notify_rust::Notification;
use std::fmt;
use std::time;

struct Timer {
    duration: time::Duration,
    start_time: Option<time::Instant>,
}

impl Timer {
    fn new(minutes: u64, seconds: u64) -> Self {
        let duration = time::Duration::from_secs(minutes * 60 + seconds);
        Timer {
            duration,
            start_time: None,
        }
    }

    fn start(&mut self) {
        self.start_time = Some(time::Instant::now());
    }

    fn stop(&mut self) {
        self.start_time = None;
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

pub enum PomodoroState {
    Work,
    Break,
}

pub struct Pomodoro {
    work_timer: Timer,
    break_timer: Timer,
    state: PomodoroState,
}

impl Pomodoro {
    pub fn new(work_time: (u64, u64), break_time: (u64, u64)) -> Self {
        Pomodoro {
            work_timer: Timer::new(work_time.0, work_time.1),
            break_timer: Timer::new(break_time.0, break_time.1),
            state: PomodoroState::Work,
        }
    }

    pub fn break_time(&self) -> String {
        self.break_timer.to_string()
    }

    pub fn work_time(&self) -> String {
        self.work_timer.to_string()
    }

    pub fn state(&self) -> &PomodoroState {
        &self.state
    }

    pub fn start(&mut self) {
        match self.state {
            PomodoroState::Work => {
                self.work_timer.start();
            }
            PomodoroState::Break => {
                self.break_timer.start();
            }
        }
    }

    pub fn check_and_switch(&mut self) {
        let (current_timer, next_timer, next_state, message) = match self.state {
            PomodoroState::Work => (
                &mut self.work_timer,
                &mut self.break_timer,
                PomodoroState::Break,
                "Time for a break!",
            ),
            PomodoroState::Break => (
                &mut self.break_timer,
                &mut self.work_timer,
                PomodoroState::Work,
                "Back to work!",
            ),
        };

        if current_timer.remaining() == time::Duration::from_secs(0) {
            current_timer.stop();
            next_timer.start();
            self.state = next_state;
            self.show_notification("Pomodoro Timer", message);
        }
    }

    fn show_notification(&self, summary: &str, body: &str) {
        Notification::new()
            .summary(summary)
            .body(body)
            .timeout(5)
            .show()
            .unwrap();
    }
}

fn get_min_sec_from_duration(duration: time::Duration) -> (u64, u64) {
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    (minutes, seconds)
}
