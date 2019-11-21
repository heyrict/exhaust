use crossterm::input::{InputEvent, KeyEvent};
use std::sync::mpsc;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};
use tui::Frame;

use crate::app::*;
use crate::event::{Messages, UpdateQuestionIndexEvent};
use crate::toggle_buttons::*;

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

    pub fn propagate(state: &App, event: Messages, tx: mpsc::Sender<Messages>) -> Option<Messages> {
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

    pub fn propagate(
        _state: &App,
        event: Messages,
        tx: mpsc::Sender<Messages>,
    ) -> Option<Messages> {
        match event {
            Messages::Input(InputEvent::Keyboard(KeyEvent::Enter)) => {
                tx.send(Messages::ChangeRoute(AppRoute::DoExam(
                    DoExamDisplay::default(),
                )))
                .unwrap();
                None
            }
            _ => Some(event),
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
            .constraints([Constraint::Min(30), Constraint::Length(18)].as_ref())
            .split(content);

        ItemWidget::new(self.app).draw(frame, main_chunks[0]);
        ExamItemsWidget::new(self.app).draw(frame, main_chunks[1]);
    }

    pub fn propagate(state: &App, event: Messages, tx: mpsc::Sender<Messages>) -> Option<Messages> {
        ExamItemsWidget::propagate(state, event, tx.clone())
            .and_then(|event| ItemWidget::propagate(state, event, tx))
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

    pub fn propagate(state: &App, event: Messages, tx: mpsc::Sender<Messages>) -> Option<Messages> {
        match &state.route {
            AppRoute::DoExam(display) => match &state.exam.question_at(display.question_index) {
                Some(item) => match item {
                    Item::Question(_) => QuestionWidget::propagate(state, event, tx),
                    _ => Some(event),
                },
                _ => Some(event),
            },
            _ => Some(event),
        }
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
                        selected: self.question.user_selection.is_selected(selection_flag),
                    }
                })
                .collect();
            let _selections = ToggleButtons::new(selections_state).render(frame, main_chunks[1]);
        };
    }

    pub fn propagate(
        _state: &App,
        event: Messages,
        tx: mpsc::Sender<Messages>,
    ) -> Option<Messages> {
        match event {
            Messages::Input(InputEvent::Keyboard(key)) => match key {
                KeyEvent::Char('a') | KeyEvent::Char('A') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::A))
                        .unwrap();
                    None
                }
                KeyEvent::Char('b') | KeyEvent::Char('B') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::B))
                        .unwrap();
                    None
                }
                KeyEvent::Char('c') | KeyEvent::Char('C') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::C))
                        .unwrap();
                    None
                }
                KeyEvent::Char('d') | KeyEvent::Char('D') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::D))
                        .unwrap();
                    None
                }
                KeyEvent::Char('e') | KeyEvent::Char('E') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::E))
                        .unwrap();
                    None
                }
                KeyEvent::Char('f') | KeyEvent::Char('F') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::F))
                        .unwrap();
                    None
                }
                KeyEvent::Char('g') | KeyEvent::Char('G') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::G))
                        .unwrap();
                    None
                }
                KeyEvent::Char('h') | KeyEvent::Char('H') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::H))
                        .unwrap();
                    None
                }
                _ => Some(event),
            },
            _ => Some(event),
        }
    }
}

pub struct ExamItemsWidget<'a> {
    app: &'a App,
}

impl<'a> ExamItemsWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        ExamItemsWidget { app }
    }

    pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, content: Rect) {
        let mut texts: Vec<Text> = vec![];
        let current_index = if let AppRoute::DoExam(display) = &self.app.route {
            display.question_index
        } else {
            0
        };

        const PENDING_STYLE: Style = Style {
            fg: Color::Black,
            bg: Color::Gray,
            modifier: Modifier::empty(),
        };
        const CURRENT_STYLE: Style = Style {
            fg: Color::Gray,
            bg: Color::Magenta,
            modifier: Modifier::BOLD,
        };
        const DONE_STYLE: Style = Style {
            fg: Color::White,
            bg: Color::Blue,
            modifier: Modifier::empty(),
        };
        const WRONG_STYLE: Style = Style {
            fg: Color::White,
            bg: Color::Red,
            modifier: Modifier::empty(),
        };
        const CORRECT_STYLE: Style = Style {
            fg: Color::White,
            bg: Color::Green,
            modifier: Modifier::empty(),
        };

        let qitems = self.app.exam.questions.iter().enumerate();

        match self.app.exam.result {
            ExamResult::Pending => qitems.for_each(|(index, item)| {
                // Text
                if index == current_index {
                    texts.push(Text::styled(format!("{:3}", &index + 1), CURRENT_STYLE));
                } else {
                    let style = match item {
                        Item::Question(question) => match question.get_result() {
                            QuestionResult::Pending => PENDING_STYLE,
                            _ => DONE_STYLE,
                        },
                        _ => DONE_STYLE,
                    };
                    texts.push(Text::styled(format!("{:3}", &index + 1), style));
                }

                // Separator
                if index % 4 == 3 {
                    texts.push(Text::raw("\n"));
                } else {
                    texts.push(Text::raw(" "));
                }
            }),
            ExamResult::Done => qitems.for_each(|(index, item)| {
                // Text
                match item {
                    Item::Question(question) => {
                        let mut style = match question.get_result() {
                            QuestionResult::Correct => CORRECT_STYLE,
                            QuestionResult::Wrong => WRONG_STYLE,
                            QuestionResult::Pending => PENDING_STYLE,
                            QuestionResult::Done => DONE_STYLE,
                        };
                        if current_index == index {
                            style.modifier = Modifier::BOLD;
                            style.fg = Color::Magenta;
                        };
                        texts.push(Text::styled(format!("{:3}", &index + 1), style));
                    }
                    Item::Card(_) => {
                        let mut style = PENDING_STYLE;
                        if current_index == index {
                            style.modifier = Modifier::BOLD;
                            style.fg = Color::Magenta;
                        };
                        texts.push(Text::styled(format!("{:3}", &index + 1), style));
                    }
                };

                // Separator
                if index % 4 == 3 {
                    texts.push(Text::raw("\n"));
                } else {
                    texts.push(Text::raw(" "));
                }
            }),
        }

        Paragraph::new(texts.iter())
            .block(Block::default().borders(Borders::ALL))
            .render(frame, content);
    }

    pub fn propagate(
        _state: &App,
        event: Messages,
        tx: mpsc::Sender<Messages>,
    ) -> Option<Messages> {
        match event {
            Messages::Input(InputEvent::Keyboard(KeyEvent::Char('>'))) => {
                tx.send(Messages::UpdateQuestionIndex(
                    UpdateQuestionIndexEvent::Next,
                ))
                .unwrap();
                None
            }
            Messages::Input(InputEvent::Keyboard(KeyEvent::Char('<'))) => {
                tx.send(Messages::UpdateQuestionIndex(
                    UpdateQuestionIndexEvent::Prev,
                ))
                .unwrap();
                None
            }
            _ => Some(event),
        }
    }
}