use clap::Parser;
use crossterm::event;
use ratatui::{layout, style::Stylize, symbols, text, widgets, DefaultTerminal, Frame};
use std::io;
use std::sync::mpsc;
use std::time;
use tui_big_text;

mod ascii_images;

fn main() -> io::Result<()> {
    let args = Args::parse();
    let terminal = ratatui::init();
    let mut app = App::new(args.work, args.break_time);
    app.handle_inputs();
    app.pomo.start_or_pause();
    let result = app.run(terminal);
    ratatui::restore();
    result
}

#[derive(Parser)]
#[clap(about = "A simple Pomodoro timer")]
#[clap(long_about = None)]
struct Args {
    #[arg(short, long, default_value = "25")]
    work: u64,
    #[arg(short, long, default_value = "5")]
    break_time: u64,
}

enum Event {
    Key(event::KeyEvent),
    Tick,
}

struct App {
    pomo: pomodoro::Pomodoro,
    exit: bool,
    tx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,
}

impl App {
    fn new(work_min: u64, break_min: u64) -> Self {
        let (tx, rx) = mpsc::channel();
        App {
            pomo: pomodoro::Pomodoro::new((work_min, 0), (break_min, 0)),
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
        let start_pause = match self.pomo.is_running() {
            true => "Pause ",
            false => "Start ",
        };

        let area = frame.area();

        let title = text::Line::from(" Pomodoro ".bold());
        let instructions = text::Line::from(vec![
            start_pause.into(),
            "<S>".blue().bold(),
            " Reset ".into(),
            "<R>".blue().bold(),
            " Quit ".into(),
            "<Q/Esc> ".blue().bold(),
        ]);
        let block = widgets::Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(symbols::border::THICK);
        frame.render_widget(block, area);

        let (work_size, work_pixel, break_size, break_pixel, ascii_image) = match self.pomo.state()
        {
            pomodoro::PomodoroState::Work => (
                8,
                tui_big_text::PixelSize::Full,
                4,
                tui_big_text::PixelSize::Quadrant,
                ascii_images::computer().into_iter().map(text::Line::from),
            ),
            pomodoro::PomodoroState::Break => (
                4,
                tui_big_text::PixelSize::Quadrant,
                8,
                tui_big_text::PixelSize::Full,
                ascii_images::sleeping_cat()
                    .into_iter()
                    .map(text::Line::from),
            ),
        };

        let horizontal = layout::Layout::horizontal([
            layout::Constraint::Percentage(50),
            layout::Constraint::Percentage(50),
        ]);
        let [left, right] = horizontal.areas(area);

        let left_layout = layout::Layout::vertical([
            layout::Constraint::Fill(1),
            layout::Constraint::Length(10),
            layout::Constraint::Fill(1),
        ]);
        let [_, lcenter, _] = left_layout.areas(left);

        let ascii_image: Vec<text::Line> = ascii_image.collect();
        let work_ascii = widgets::Paragraph::new(ascii_image).alignment(layout::Alignment::Center);
        frame.render_widget(work_ascii, lcenter);

        let right_layout = layout::Layout::vertical([
            layout::Constraint::Fill(1),
            layout::Constraint::Length(work_size),
            layout::Constraint::Length(break_size),
            layout::Constraint::Fill(1),
        ]);
        let [_, rtop, rbottom, _] = right_layout.areas(right);

        let work_timer = tui_big_text::BigText::builder()
            .pixel_size(work_pixel)
            .lines(vec![self.pomo.work_time().blue().into()])
            .centered()
            .build();
        frame.render_widget(work_timer, rtop);

        let break_timer = tui_big_text::BigText::builder()
            .pixel_size(break_pixel)
            .lines(vec![self.pomo.break_time().green().into()])
            .centered()
            .build();
        frame.render_widget(break_timer, rbottom);
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
                self.pomo.start_or_pause();
            }
            event::KeyCode::Char('r') => {
                self.pomo.reset();
            }
            event::KeyCode::Esc => self.exit = true,
            event::KeyCode::Char('q') => self.exit = true,
            _ => (),
        }
    }
}
