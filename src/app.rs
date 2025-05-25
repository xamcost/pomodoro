use crate::ascii_images;
use crossterm::event;
use ratatui::{layout, style::Stylize, symbols, text, widgets, DefaultTerminal, Frame};
use std::io;
use std::sync::mpsc;
use std::time;
use tui_big_text;

enum Event {
    Key(event::KeyEvent),
    Tick,
}

pub struct App {
    pomo: pomodoro_tui::Pomodoro,
    exit: bool,
    tx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,
    hide_image: bool,
}

impl App {
    pub fn new(work_min: u64, break_min: u64, hide_image: bool) -> Self {
        let (tx, rx) = mpsc::channel();
        App {
            pomo: pomodoro_tui::Pomodoro::new((work_min, 0), (break_min, 0)),
            exit: false,
            tx,
            rx,
            hide_image,
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
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

    pub fn handle_inputs(&self) {
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

    pub fn start_or_pause(&mut self) {
        self.pomo.start_or_pause();
    }

    fn draw(&self, frame: &mut Frame) {
        let (work_size, work_pixel, break_size, break_pixel) = match self.pomo.state() {
            pomodoro_tui::PomodoroState::Work => (
                8,
                tui_big_text::PixelSize::Full,
                4,
                tui_big_text::PixelSize::Quadrant,
            ),
            pomodoro_tui::PomodoroState::Break => (
                4,
                tui_big_text::PixelSize::Quadrant,
                8,
                tui_big_text::PixelSize::Full,
            ),
        };

        let area = frame.area();

        let block = self.get_block_widget();
        frame.render_widget(block, area);

        let (lcenter, rtop, rbottom) = self.get_layout(area, work_size, break_size);

        if !self.hide_image {
            let ascii_img = self.get_ascii_image_widget();
            frame.render_widget(ascii_img, lcenter);
        }

        let (work_timer, break_timer) = self.get_timer_widgets(work_pixel, break_pixel);
        frame.render_widget(work_timer, rtop);
        frame.render_widget(break_timer, rbottom);
    }

    fn get_layout(
        &self,
        area: layout::Rect,
        work_size: u16,
        break_size: u16,
    ) -> (layout::Rect, layout::Rect, layout::Rect) {
        let (ascii_width, timer_width) = if !self.hide_image { (50, 50) } else { (0, 100) };
        let horizontal = layout::Layout::horizontal([
            layout::Constraint::Percentage(ascii_width),
            layout::Constraint::Percentage(timer_width),
        ]);
        let [left, right] = horizontal.areas(area);

        let left_layout = layout::Layout::vertical([
            layout::Constraint::Fill(1),
            layout::Constraint::Length(10),
            layout::Constraint::Fill(1),
        ]);
        let [_, lcenter, _] = left_layout.areas(left);

        let right_layout = layout::Layout::vertical([
            layout::Constraint::Fill(1),
            layout::Constraint::Length(work_size),
            layout::Constraint::Length(break_size),
            layout::Constraint::Fill(1),
        ]);
        let [_, rtop, rbottom, _] = right_layout.areas(right);

        (lcenter, rtop, rbottom)
    }

    fn get_block_widget(&self) -> widgets::Block {
        let start_pause = match self.pomo.is_running() {
            true => "Pause ",
            false => "Start ",
        };

        let title = text::Line::from(" Pomodoro ".bold());
        let instructions = text::Line::from(vec![
            start_pause.into(),
            "<S>".blue().bold(),
            " Reset ".into(),
            "<R>".blue().bold(),
            " Quit ".into(),
            "<Q/Esc> ".blue().bold(),
        ]);
        widgets::Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(symbols::border::THICK)
    }

    fn get_ascii_image_widget(&self) -> widgets::Paragraph {
        let ascii_image: Vec<text::Line> = match self.pomo.state() {
            pomodoro_tui::PomodoroState::Work => ascii_images::computer(),
            pomodoro_tui::PomodoroState::Break => ascii_images::sleeping_cat(),
        }
        .into_iter()
        .map(text::Line::from)
        .collect();

        widgets::Paragraph::new(ascii_image).alignment(layout::Alignment::Center)
    }

    fn get_timer_widgets(
        &self,
        work_pixel: tui_big_text::PixelSize,
        break_pixel: tui_big_text::PixelSize,
    ) -> (tui_big_text::BigText, tui_big_text::BigText) {
        let work_timer = tui_big_text::BigText::builder()
            .pixel_size(work_pixel)
            .lines(vec![self.pomo.work_time().blue().into()])
            .centered()
            .build();
        let break_timer = tui_big_text::BigText::builder()
            .pixel_size(break_pixel)
            .lines(vec![self.pomo.break_time().green().into()])
            .centered()
            .build();
        (work_timer, break_timer)
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
