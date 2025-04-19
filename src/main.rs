use crossterm::event::{self, Event};
use ratatui::{buffer, layout, style::Stylize, symbols, text, widgets, DefaultTerminal, Frame};
use std::io;

fn main() -> io::Result<()> {
    let terminal = ratatui::init();
    let mut app = App::new();
    let result = app.run(terminal);
    ratatui::restore();
    result
}

pub struct App {
    timer: pomodoro::Timer,
    exit: bool,
}

impl App {
    pub fn new() -> Self {
        App {
            timer: pomodoro::Timer::new(25),
            exit: false,
        }
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
            event::KeyCode::Char('s') => {
                self.timer.start();
            }
            event::KeyCode::Esc => self.exit = true,
            event::KeyCode::Char('q') => self.exit = true,
            _ => (),
        }
    }
}

impl widgets::Widget for &App {
    fn render(self, area: layout::Rect, buf: &mut buffer::Buffer) {
        let title = text::Line::from(" Pomodoro ".bold());
        let instructions = text::Line::from(vec![
            "Start ".into(),
            "<S>".blue().bold(),
            " Quit ".into(),
            "<Q/Esc> ".blue().bold(),
        ]);
        let block = widgets::Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(symbols::border::THICK);

        let counter_text = text::Text::from(vec![text::Line::from(vec![
            "Value: ".into(),
            format!("{}", self.timer).yellow(),
        ])]);

        widgets::Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
