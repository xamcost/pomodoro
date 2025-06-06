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
    #[arg(short = 'b', long = "break", default_value = "5")]
    break_time: u64,
    #[arg(short = 'i', long = "hide-image", default_value = "false")]
    hide_image: bool,
    #[arg(short = 's', long = "sound")]
    sound: Option<String>,
    #[arg(short = 'n', long = "no-sound", help="default to false")]
    no_sound: Option<bool>
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let terminal = ratatui::init();

    // Pass the whole structure make it easier to manage 
    let mut app = app::App::new(&args);

    app.handle_inputs();
    app.start_or_pause();
    let result = app.run(terminal);
    ratatui::restore();
    result
}
