#[macro_use]
extern crate bitflags;

#[macro_use]
mod macros;

mod app;
mod event;
mod reducer;
mod toggle_buttons;
mod ui;
mod widget;

use app::*;

use std::error::Error;

use event::Messages;
use tui::widgets::{Block, Borders};

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::backend::CrosstermBackend;
use tui::Terminal;

use std::io::{stdout, Write};

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = event::Events::new();
    terminal.hide_cursor()?;

    let mut app = App::default();
    app.load_config()?;

    terminal.clear()?;

    loop {
        let mut app_widget = ui::AppWidget::new(&mut app);
        terminal.draw(|mut f| {
            let size = f.size();
            f.render_widget(
                Block::default().title("EXHAUST").borders(Borders::ALL),
                size,
            );

            let main_size = f.size();
            //main_size.y += 1;
            //main_size.height -= 1;
            //main_size.x += 1;
            //main_size.width -= 1;

            app_widget.draw(&mut f, main_size);
        })?;
        let next_event = events.next()?;
        match next_event {
            Messages::Quit => {
                disable_raw_mode()?;
                execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                terminal.show_cursor()?;
                break;
            }
            _ => {
                reducer::reduce(&mut app, next_event, events.tx.clone())
                    .map(|evt: Messages| ui::AppWidget::propagate(&app, evt, events.tx.clone()));
            }
        }
    }

    Ok(())
}
