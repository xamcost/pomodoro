use crossterm::event;
use ratatui::{layout, style::Stylize, symbols, text, widgets, DefaultTerminal, Frame};
use std::io;
use std::sync::mpsc;
use std::time;
use tui_big_text;

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
    pomo: pomodoro::Pomodoro,
    exit: bool,
    tx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        App {
            pomo: pomodoro::Pomodoro::new(1, 1),
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
                Ok(Event::Tick) => self.pomo.check_and_switch(),
                _ => (),
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

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
        frame.render_widget(block, area);

        let (work_size, work_pixel, break_size, break_pixel) = match self.pomo.state() {
            pomodoro::PomodoroState::Work => (
                8,
                tui_big_text::PixelSize::Full,
                4,
                tui_big_text::PixelSize::Quadrant,
            ),
            pomodoro::PomodoroState::Break => (
                4,
                tui_big_text::PixelSize::Quadrant,
                8,
                tui_big_text::PixelSize::Full,
            ),
        };

        let vertical = layout::Layout::vertical([
            layout::Constraint::Fill(1),
            layout::Constraint::Length(work_size),
            layout::Constraint::Length(break_size),
            layout::Constraint::Fill(1),
        ]);
        let [_, top, bottom, _] = vertical.areas(area);

        let work_timer = tui_big_text::BigText::builder()
            .pixel_size(work_pixel)
            .lines(vec![self.pomo.work_time().blue().into()])
            .centered()
            .build();
        frame.render_widget(work_timer, top);

        let break_timer = tui_big_text::BigText::builder()
            .pixel_size(break_pixel)
            .lines(vec![self.pomo.break_time().green().into()])
            .centered()
            .build();
        frame.render_widget(break_timer, bottom);
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
                self.pomo.start();
            }
            event::KeyCode::Esc => self.exit = true,
            event::KeyCode::Char('q') => self.exit = true,
            _ => (),
        }
    }
}
