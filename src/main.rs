mod engine;
mod game;
mod gameboard;
mod renderer;

use game::Game;

use crossterm::ExecutableCommand;
use ratatui::DefaultTerminal;
use ratatui::layout::Rect;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(app)?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    std::io::stdout().execute(crossterm::event::EnableMouseCapture)?;
    let mut game = Game::new();

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
