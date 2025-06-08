use std::fmt;
use std::fs;
use std::io::BufReader;
use std::path::PathBuf;
use std::process;
use std::thread;
use std::time;

use notify_rust::Notification;
use rodio::source;
use rodio::Decoder;
use rodio::OutputStream;
use rodio::Source;

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

#[derive(Debug, PartialEq)]
pub enum PomodoroState {
    Work,
    Break,
}

pub struct Pomodoro {
    work_timer: Timer,
    break_timer: Timer,
    state: PomodoroState,
    sound: PathBuf,
    no_sound: bool
}

impl Pomodoro {
    pub fn new(work_time: (u64, u64), break_time: (u64, u64), sound: PathBuf, no_sound: bool) -> Self {
        Pomodoro {
            work_timer: Timer::new(work_time.0, work_time.1),
            break_timer: Timer::new(break_time.0, break_time.1),
            state: PomodoroState::Work,
            sound,
            no_sound
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
            let sound_clone = self.sound.clone();
            if !self.no_sound {
                thread::spawn(move || {
                    sound_play(sound_clone);
                });
            }
        }

    }




}

// Maybe adding some widget to render the error some few seconds
pub fn sound_play(sound: PathBuf) {
    let (_stream, stream_handler) = match OutputStream::try_default() {
        Ok(ok) => ok,
        Err(_e) => return,
    };
    if let Ok(open_file) = fs::File::open(&sound) {
        let file = BufReader::new(open_file);
        if let Ok(sound_file) = Decoder::new(file) {
            let _ = stream_handler.play_raw(sound_file.convert_samples());
            std::thread::sleep(std::time::Duration::from_secs(3)); // Let it play
        } else {
            // Decoder::new(file) failed
        }
    } else {
        // fs::File::open(&sound) failed
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

    if cfg!(target_os = "linux") {
        
        let _ = Notification::new()
            .summary(title)
            .body(message)
            .show();

    }
}



#[cfg(test)]
mod tests {
    // This module tests the functionalities fo the Pomodoro timer.
    // Some tests for the Timer struct are included to check more
    // thoroughly the timer functionalities.
    use super::*;

    // For tests units only
    fn default_sound_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("default_sound.mp3")
    }

    #[test]
    fn test_default_sound_path_exists() {
        let path = default_sound_path();
        println!("{:?}", path);
        assert!(std::path::Path::new(&path).exists(), "Sound file does not exist!");
    }
    #[test]
    fn test_timer_start_or_pause() {
        // Given
        let mut timer = Timer::new(1, 15);
        // When
        timer.start_or_pause();
        // Then
        assert!(timer.is_running);
        assert!(timer.start_time.is_some());
        let elapsed = timer.elapsed();
        assert!(timer.elapsed() > time::Duration::from_secs(0));
        assert!(timer.remaining() < timer.duration);
        // Testing pause
        // Given
        std::thread::sleep(std::time::Duration::from_secs(1));
        // When
        timer.start_or_pause();
        // Then
        assert!(!timer.is_running);
        assert!(timer.elapsed() > elapsed + time::Duration::from_secs(1));
        assert_eq!(timer.remaining(), timer.duration - timer.elapsed());
    }

    #[test]
    fn test_timer_reset() {
        // Given
        let mut timer = Timer::new(1, 15);
        timer.start_or_pause();
        std::thread::sleep(std::time::Duration::from_secs(1));
        // When
        timer.reset();
        // Then
        assert_eq!(timer.elapsed(), time::Duration::from_secs(0));
        assert!(!timer.is_running);
        assert!(timer.start_time.is_none());
        assert_eq!(timer.remaining(), timer.duration);
    }

    #[test]
    fn test_timer_remaining() {
        // When
        let mut timer = Timer::new(0, 3);
        // Then
        assert_eq!(timer.remaining().as_secs(), 3);
        // When
        timer.start_or_pause();
        std::thread::sleep(std::time::Duration::from_secs(1));
        // Then
        assert!(timer.remaining().as_secs() > 0);
        // When
        std::thread::sleep(std::time::Duration::from_secs(3));
        let remaining = timer.remaining();
        // Then
        assert_eq!(remaining.as_secs(), 0);
    }

    #[test]
    fn test_timer_display() {
        // When
        let timer = Timer::new(1, 125);
        // Then
        assert_eq!(timer.to_string(), "03:05");
    }

    #[test]
    fn test_pomodoro_initialization() {
        // When
        //
        let sound = default_sound_path();
        let pomodoro = Pomodoro::new((25, 0), (2, 5),sound, true );
        // Then
        assert_eq!(pomodoro.work_time(), "25:00");
        assert_eq!(pomodoro.break_time(), "02:05");
        assert_eq!(*pomodoro.state(), PomodoroState::Work);
        assert!(!pomodoro.is_running());
    }

    #[test]
    fn test_pomodoro_start_or_pause() {
        // Given

        let sound = default_sound_path();
        let mut pomodoro = Pomodoro::new((0, 3), (0, 2) ,sound, true);
        // When
        pomodoro.start_or_pause();
        // Then
        assert!(pomodoro.is_running());
        assert_eq!(pomodoro.work_time(), "00:02");
        assert_eq!(pomodoro.break_time(), "00:02");
        assert_eq!(*pomodoro.state(), PomodoroState::Work);
        // When paused
        pomodoro.start_or_pause();
        // Then
        assert!(!pomodoro.is_running());
        assert_eq!(pomodoro.work_time(), "00:02");
        assert_eq!(pomodoro.break_time(), "00:02");
    }

    #[test]
    fn test_pomodoro_reset() {
        // Given
        let sound = default_sound_path();
        let mut pomodoro = Pomodoro::new((0, 3), (0, 2) ,sound, true);
        pomodoro.start_or_pause();
        std::thread::sleep(std::time::Duration::from_secs(1));
        // When
        pomodoro.reset();
        // Then
        assert_eq!(pomodoro.work_time(), "00:03");
        assert_eq!(pomodoro.break_time(), "00:02");
        assert_eq!(*pomodoro.state(), PomodoroState::Work);
        assert!(!pomodoro.is_running());
    }

    #[test]
    fn test_pomodoro_reset_from_break() {
        // Given
        
        let sound = default_sound_path();
        let mut pomodoro = Pomodoro::new((0, 3), (0, 2) ,sound, true);
        std::thread::sleep(std::time::Duration::from_secs(2));
        pomodoro.check_and_switch();
        // When
        pomodoro.reset();
        // Then
        assert_eq!(pomodoro.work_time(), "00:01");
        assert_eq!(pomodoro.break_time(), "00:05");
        assert_eq!(*pomodoro.state(), PomodoroState::Work);
        assert!(!pomodoro.is_running());
    }

    #[test]
    fn test_pomodoro_check_and_switch() {
        // Given

        let sound = default_sound_path();
        let mut pomodoro = Pomodoro::new((0, 3), (0, 2) ,sound, true);
        pomodoro.start_or_pause();
        // When
        pomodoro.check_and_switch();
        // Then
        assert_eq!(*pomodoro.state(), PomodoroState::Work);
        // When expected to switch to break
        std::thread::sleep(std::time::Duration::from_secs(2));
        pomodoro.check_and_switch();
        // Then
        assert_eq!(*pomodoro.state(), PomodoroState::Break);
        // When expected to switch to work
        std::thread::sleep(std::time::Duration::from_secs(2));
        pomodoro.check_and_switch();
        // Then
        assert_eq!(*pomodoro.state(), PomodoroState::Work);
    }

    #[test]
    fn test_get_min_sec_from_duration() {
        let duration = time::Duration::from_secs(125);
        let (minutes, seconds) = get_min_sec_from_duration(duration);
        assert_eq!(minutes, 2);
        assert_eq!(seconds, 5);
    }
}
