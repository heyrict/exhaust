use crate::event::Messages;
use crossterm::input::{InputEvent, KeyEvent};
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};
use tui::Frame;

use crate::app::*;
use crate::toggle_buttons::*;
use std::sync::mpsc;

pub struct AppWidget<'a> {
    app: &'a App,
}

impl<'a> AppWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        AppWidget { app }
    }

    pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, content: Rect) {
        match &self.app.route {
            AppRoute::Home => HomeWidget::new(self.app).draw(frame, content),
            AppRoute::DoExam(_) => ExamWidget::new(self.app).draw(frame, content),
        };
    }

    pub fn propagate(state: &App, event: Messages, tx: mpsc::Sender<Messages>) -> bool {
        // Propagation
        match state.route {
            AppRoute::Home => HomeWidget::propagate(state, event, tx),
            AppRoute::DoExam(_) => ExamWidget::propagate(state, event, tx),
        }
    }
}

pub struct HomeWidget<'a> {
    app: &'a App,
}

impl<'a> HomeWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        HomeWidget { app }
    }

    pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, content: Rect) {
        let _test = Paragraph::new([Text::raw("Welcome!\n\nPress Enter to Start")].iter())
            .block(Block::default().borders(Borders::ALL))
            .render(frame, content);
    }

    pub fn propagate(_state: &App, event: Messages, tx: mpsc::Sender<Messages>) -> bool {
        match event {
            Messages::Input(InputEvent::Keyboard(KeyEvent::Enter)) => {
                tx.send(Messages::ChangeRoute(AppRoute::DoExam(
                    DoExamDisplay::default(),
                )))
                .unwrap();
                false
            }
            _ => true,
        }
    }
}

pub struct ExamWidget<'a> {
    app: &'a App,
}

impl<'a> ExamWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        ExamWidget { app }
    }

    pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, content: Rect) {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            // Main View and Sidebar
            .constraints([Constraint::Min(30), Constraint::Max(16)].as_ref())
            .split(content);

        ItemWidget::new(self.app).draw(frame, main_chunks[0]);
        Paragraph::new([Text::raw("Answer")].iter()).render(frame, main_chunks[1]);
    }

    pub fn propagate(state: &App, event: Messages, tx: mpsc::Sender<Messages>) -> bool {
        ItemWidget::propagate(state, event, tx);
        false
    }
}

pub struct ItemWidget<'a> {
    app: &'a App,
}

impl<'a> ItemWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        ItemWidget { app }
    }

    pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, content: Rect) {
        match &self.app.route {
            AppRoute::DoExam(display) => {
                let item: &Item = self
                    .app
                    .exam
                    .question_at(display.question_index)
                    .expect("Item out of range!");

                match item {
                    Item::Question(question) => {
                        QuestionWidget::new(self.app, &question, &display).draw(frame, content)
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn propagate(state: &App, event: Messages, tx: mpsc::Sender<Messages>) -> bool {
        if let AppRoute::DoExam(display) = &state.route {
            if let Some(item) = &state.exam.question_at(display.question_index) {
                match item {
                    Item::Question(_) => {
                        QuestionWidget::propagate(state, event, tx);
                        return false;
                    }
                    _ => {}
                }
            }
        };
        true
    }
}

pub struct QuestionWidget<'a> {
    app: &'a App,
    question: &'a Question,
    display: &'a DoExamDisplay,
}

impl<'a> QuestionWidget<'a> {
    pub fn new(app: &'a App, question: &'a Question, display: &'a DoExamDisplay) -> Self {
        QuestionWidget {
            app,
            question,
            display,
        }
    }

    pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, content: Rect) {
        if self.display.display_answer {
        } else {
            // Question + Selection + Answer
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(10), Constraint::Min(16)].as_ref())
                .split(content);
            let question_title = format!(
                "Question ({}/{})",
                &self.display.question_index + 1,
                &self.app.exam.num_questions()
            );
            let _question = Paragraph::new([Text::raw(&self.question.question)].iter())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(&question_title),
                )
                .wrap(true)
                .render(frame, main_chunks[0]);
            let selections_state = self
                .question
                .selections
                .iter()
                .enumerate()
                .map(|(index, sel)| {
                    let selection_flag: u8 = 0b1 << index;
                    ToggleButtonState {
                        text: &sel.text,
                        selected: (selection_flag & self.question.user_selection.bits()) != 0,
                    }
                })
                .collect();
            let _selections = ToggleButtons::new(selections_state).render(frame, main_chunks[1]);
        };
    }

    pub fn propagate(_state: &App, event: Messages, tx: mpsc::Sender<Messages>) -> bool {
        match event {
            Messages::Input(InputEvent::Keyboard(key)) => match key {
                KeyEvent::Char('a') | KeyEvent::Char('A') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::A))
                        .unwrap();
                    false
                }
                KeyEvent::Char('b') | KeyEvent::Char('B') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::B))
                        .unwrap();
                    false
                }
                KeyEvent::Char('c') | KeyEvent::Char('C') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::C))
                        .unwrap();
                    false
                }
                KeyEvent::Char('d') | KeyEvent::Char('D') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::D))
                        .unwrap();
                    false
                }
                KeyEvent::Char('e') | KeyEvent::Char('E') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::E))
                        .unwrap();
                    false
                }
                KeyEvent::Char('f') | KeyEvent::Char('F') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::F))
                        .unwrap();
                    false
                }
                KeyEvent::Char('g') | KeyEvent::Char('G') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::G))
                        .unwrap();
                    false
                }
                KeyEvent::Char('h') | KeyEvent::Char('H') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::H))
                        .unwrap();
                    false
                }
                _ => true,
            },
            _ => true,
        }
    }
}
