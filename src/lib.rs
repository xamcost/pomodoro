use std::fmt;
use std::process;
use std::time;

struct Timer {
    duration: time::Duration,
    start_time: Option<time::Instant>,
    elapsed: time::Duration,
    is_running: bool,
}

impl Timer {
    fn new(minutes: u64, seconds: u64) -> Self {
        let duration = time::Duration::from_secs(minutes * 60 + seconds);
        Timer {
            duration,
            start_time: None,
            elapsed: time::Duration::from_secs(0),
            is_running: false,
        }
    }

    fn start_or_pause(&mut self) {
        if self.is_running {
            self.elapsed = self.elapsed();
            self.start_time = None;
        } else {
            self.start_time = Some(time::Instant::now());
        }
        self.is_running = !self.is_running;
    }

    fn reset(&mut self) {
        self.start_time = None;
        self.elapsed = time::Duration::from_secs(0);
        self.is_running = false;
    }

    fn elapsed(&self) -> time::Duration {
        match self.start_time {
            Some(start_time) => self.elapsed + start_time.elapsed(),
            None => self.elapsed,
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

    pub fn is_running(&self) -> bool {
        match self.state {
            PomodoroState::Work => self.work_timer.is_running,
            PomodoroState::Break => self.break_timer.is_running,
        }
    }

    pub fn start_or_pause(&mut self) {
        match self.state {
            PomodoroState::Work => {
                self.work_timer.start_or_pause();
            }
            PomodoroState::Break => {
                self.break_timer.start_or_pause();
            }
        }
    }

    pub fn reset(&mut self) {
        self.work_timer.reset();
        self.break_timer.reset();
        self.state = PomodoroState::Work;
    }

    pub fn check_and_switch(&mut self) {
        let (current_timer, next_timer, next_state, message) = match self.state {
            PomodoroState::Work => (
                &mut self.work_timer,
                &mut self.break_timer,
                PomodoroState::Break,
                "It's time to have a break.",
            ),
            PomodoroState::Break => (
                &mut self.break_timer,
                &mut self.work_timer,
                PomodoroState::Work,
                "It's time to research.",
            ),
        };

        if current_timer.remaining() == time::Duration::from_secs(0) {
            current_timer.reset();
            next_timer.start_or_pause();
            self.state = next_state;
            show_notification("Pomodoro Timer", message);
        }
    }
}

fn get_min_sec_from_duration(duration: time::Duration) -> (u64, u64) {
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    (minutes, seconds)
}

fn show_notification(title: &str, message: &str) {
    if cfg!(target_os = "macos") {
        match process::Command::new("osascript")
            .arg("-e")
            .arg(format!(
                "display notification \"{}\" with title \"{}\"",
                message, title
            ))
            .arg("-e")
            .arg(format!("say \"{}\" using \"Thomas\"", message))
            .output()
        {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to send notification: {}", e);
            }
        }
    }
}
