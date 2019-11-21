#[macro_use]
extern crate bitflags;

mod app;
mod event;
mod reducer;
mod toggle_buttons;
mod ui;
mod widget;

use app::*;

use std::error::Error;

use event::Messages;
use tui::widgets::{Block, Borders, Widget};

use crossterm::{
    input::{InputEvent, KeyEvent},
    screen::AlternateScreen,
};
use tui::backend::CrosstermBackend;
use tui::Terminal;

use std::io::stdout;

fn main() -> Result<(), Box<dyn Error>> {
    let screen = AlternateScreen::to_alternate(true)?;
    let backend = CrosstermBackend::with_alternate_screen(stdout(), screen)?;
    let events = event::Events::new();
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let mut app = get_sample_app();

    terminal.clear()?;

    loop {
        let mut app_widget = ui::AppWidget::new(&app);
        terminal.draw(|mut f| {
            let size = f.size();
            Block::default()
                .title("EXHAUST")
                .borders(Borders::ALL)
                .render(&mut f, size);

            let main_size = f.size();
            //main_size.y += 1;
            //main_size.height -= 1;
            //main_size.x += 1;
            //main_size.width -= 1;

            app_widget.draw(&mut f, main_size);
        })?;
        let next_event = events.next()?;
        match next_event {
            Messages::Input(InputEvent::Keyboard(KeyEvent::Char('Q'))) => break,
            Messages::Quit => break,
            _ => {
                reducer::reduce(&mut app, next_event)
                    .map(|evt: Messages| ui::AppWidget::propagate(&app, evt, events.tx.clone()));
            }
        }
    }

    Ok(())
}