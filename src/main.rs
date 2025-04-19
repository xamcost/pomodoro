use crossterm::event::{self, Event};
use ratatui::{buffer, layout, style::Stylize, symbols, text, widgets, DefaultTerminal, Frame};
use std::io;
use std::time;

fn main() -> io::Result<()> {
    let terminal = ratatui::init();
    let mut app = App::new();
    let result = app.run(terminal);
    ratatui::restore();
    result
}

pub struct App {
    start_time: Option<time::Instant>,
    work_duration: time::Duration,
    exit: bool,
}

fn get_min_sec_from_duration(duration: time::Duration) -> (u64, u64) {
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    (minutes, seconds)
}

impl App {
    pub fn new() -> Self {
        App {
            start_time: Some(time::Instant::now()),
            work_duration: time::Duration::from_secs(10),
            exit: false,
        }
    }

    fn start(&mut self) {
        self.start_time = Some(time::Instant::now());
    }

    fn elapsed(&self) -> time::Duration {
        match self.start_time {
            Some(start_time) => start_time.elapsed(),
            None => time::Duration::from_secs(0),
        }
    }

    fn remaining(&self) -> time::Duration {
        if self.elapsed() >= self.work_duration {
            return time::Duration::from_secs(0);
        }
        self.work_duration - self.elapsed()
    }

    fn run(&mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_event()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_event(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) => match key_event.kind {
                event::KeyEventKind::Press => self.handle_key_event(key_event),
                _ => (),
            },
            _ => (),
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: event::KeyEvent) {
        match key_event.code {
            event::KeyCode::Esc => self.exit = true,
            event::KeyCode::Char('q') => self.exit = true,
            _ => (),
        }
    }
}

impl widgets::Widget for &App {
    fn render(self, area: layout::Rect, buf: &mut buffer::Buffer) {
        let title = text::Line::from(" Pomodoro ".bold());
        let instructions = text::Line::from(vec![" Quit ".into(), "<Q/Esc> ".blue().bold()]);
        let block = widgets::Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(symbols::border::THICK);

        let (min, sec) = get_min_sec_from_duration(self.remaining());
        let counter_text = text::Text::from(vec![text::Line::from(vec![
            "Value: ".into(),
            format!("{:0>2}:{:0>2}", min, sec).to_string().yellow(),
        ])]);

        widgets::Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
