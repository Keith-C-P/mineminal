mod colors;
mod engine;
mod game;
mod gameboard;
mod gameinfo;
mod renderer;
mod state;
mod timer;
mod utils;

use game::Game;

use crossterm::ExecutableCommand;
use ratatui::DefaultTerminal;
use ratatui::layout::Rect;
use simplelog::*;
use std::fs::File;
use std::panic;

fn main() -> color_eyre::Result<()> {
    WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        File::create("mineminal.log").unwrap(),
    )
    .unwrap();

    color_eyre::install()?;
    ratatui::run(app)?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    std::io::stdout().execute(crossterm::event::EnableMouseCapture)?;
    let mut game = Game::new();

    //Graceful shutdown on panic
    panic::set_hook(Box::new(|info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        );

        eprintln!("panic: {}", info);
    }));

    game.run();

    loop {
        let mut root_area = Rect::default();
        terminal.draw(|frame| {
            root_area = frame.area();
            game.draw(frame);
        })?;

        let event = crossterm::event::read()?;
        game.handle_input(event, root_area);
    }
}
