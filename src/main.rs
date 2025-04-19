use crossterm::event;
use ratatui::{buffer, layout, style::Stylize, symbols, text, widgets, DefaultTerminal, Frame};
use std::io;
use std::sync::mpsc;
use std::time;

fn main() -> io::Result<()> {
    let terminal = ratatui::init();
    let mut app = App::new();
    app.handle_inputs();
    let result = app.run(terminal);
    ratatui::restore();
    result
}

enum Event {
    Key(event::KeyEvent),
    Tick,
}

pub struct App {
    timer: pomodoro::Timer,
    exit: bool,
    tx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        App {
            timer: pomodoro::Timer::new(25),
            exit: false,
            tx,
            rx,
        }
    }

    fn run(&mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            match self.rx.recv() {
                Ok(Event::Key(key_event)) => self.handle_key_event(key_event),
                Ok(Event::Tick) => {}
                _ => (),
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_inputs(&self) {
        let tx = self.tx.clone();
        let tick_rate = time::Duration::from_millis(200);
        std::thread::spawn(move || {
            let mut last_tick = time::Instant::now();
            loop {
                let timeout = tick_rate.saturating_sub(last_tick.elapsed());
                if event::poll(timeout).unwrap() {
                    match event::read().unwrap() {
                        event::Event::Key(key_event) => tx.send(Event::Key(key_event)).unwrap(),
                        _ => (),
                    }
                }
                if last_tick.elapsed() >= tick_rate {
                    tx.send(Event::Tick).unwrap();
                    last_tick = time::Instant::now();
                }
            }
        });
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
