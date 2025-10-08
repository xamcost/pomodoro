use clap::Parser;
use std::io;
use std::path::PathBuf;
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
    #[arg(
        short = 'n',
        long = "no-sound",
        help = "default to false",
        default_value = "false"
    )]
    no_sound: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let sound = match &args.sound {
        Some(sound) => PathBuf::from(sound.to_owned()),
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("default_sound.mp3"),
    };

    let terminal = ratatui::init();

    // Pass the whole structure make it easier to manage
    let mut app = app::App::new(
        args.work,
        args.break_time,
        args.hide_image,
        &sound,
        args.no_sound,
    );

    app.handle_inputs();
    app.start_or_pause();
    let result = app.run(terminal);
    ratatui::restore();
    result
}
