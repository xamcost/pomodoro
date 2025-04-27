use clap::Parser;
use ratatui;
use std::io;

mod app;
mod ascii_images;

#[derive(Parser)]
#[clap(about = "A simple Pomodoro timer")]
#[clap(long_about = None)]
struct Args {
    #[arg(short, long, default_value = "25")]
    work: u64,
    #[arg(short, long, default_value = "5")]
    break_time: u64,
    #[arg(short = 'i', long = "hide-image", default_value = "false")]
    hide_image: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let terminal = ratatui::init();
    let mut app = app::App::new(args.work, args.break_time, args.hide_image);
    app.handle_inputs();
    app.start_or_pause();
    let result = app.run(terminal);
    ratatui::restore();
    result
}
