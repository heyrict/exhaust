use crossterm::input::{InputEvent, KeyEvent};
use std::sync::mpsc;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, SelectableList, Text, Widget};
use tui::Frame;

use crate::app::*;
use crate::event::*;
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
            AppRoute::DoExam => ExamWidget::new(self.app).draw(frame, content),
        };
    }

    pub fn propagate(state: &App, event: Messages, tx: mpsc::Sender<Messages>) -> Option<Messages> {
        // Propagation
        match state.route {
            AppRoute::Home => HomeWidget::propagate(state, event, tx),
            AppRoute::DoExam => ExamWidget::propagate(state, event, tx),
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
        const UNDERLINE_STYLE: Style = Style {
            fg: Color::Reset,
            bg: Color::Reset,
            modifier: Modifier::UNDERLINED,
        };
        const HIGHLIGHT_STYLE: Style = Style {
            fg: Color::Gray,
            bg: Color::Black,
            modifier: Modifier::empty(),
        };

        let messages: [Text; 9] = [
            Text::raw("Welcome! Choose a file to start:\n\n["),
            Text::styled("q", UNDERLINE_STYLE),
            Text::raw(": Quit] | ["),
            Text::styled("u", UNDERLINE_STYLE),
            Text::raw(": Upper] | ["),
            match self.app.home.open_mode {
                OpenMode::ReadOnly => Text::styled("ReadOnly", HIGHLIGHT_STYLE),
                OpenMode::Write => Text::styled("ReadOnly", Style::default()),
            },
            Text::raw("|"),
            match self.app.home.open_mode {
                OpenMode::ReadOnly => Text::styled("Write", Style::default()),
                OpenMode::Write => Text::styled("Write", HIGHLIGHT_STYLE),
            },
            Text::raw("]"),
        ];
        let paths = self.app.home.get_paths().unwrap();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(6)].as_ref())
            .margin(1)
            .split(content);
        let _welcome = Paragraph::new(messages.iter())
            .block(Block::default().borders(Borders::ALL))
            .render(frame, chunks[0]);
        let _items = SelectableList::default()
            .items(
                &paths
                    .iter()
                    .map(|path| {
                        let path_str = path.to_str().unwrap_or("???");
                        match path.is_dir() {
                            true => format!("{}/", path_str),
                            false => path_str.to_owned(),
                        }
                    })
                    .collect::<Vec<String>>(),
            )
            .select(self.app.home.current_selected)
            .highlight_symbol(">")
            .highlight_style(Style::default().modifier(Modifier::REVERSED))
            .render(frame, chunks[1]);
    }

    pub fn propagate(
        _state: &App,
        event: Messages,
        tx: mpsc::Sender<Messages>,
    ) -> Option<Messages> {
        match event {
            Messages::Input(InputEvent::Keyboard(key)) => match key {
                KeyEvent::Enter => {
                    tx.send(Messages::LoadFile).unwrap();
                    None
                }
                KeyEvent::Char('j') | KeyEvent::Down => {
                    tx.send(Messages::UpdateHomeSelected(UpdateHomeSelectedEvent::Next))
                        .unwrap();
                    None
                }
                KeyEvent::Char('k') | KeyEvent::Up => {
                    tx.send(Messages::UpdateHomeSelected(UpdateHomeSelectedEvent::Prev))
                        .unwrap();
                    None
                }
                KeyEvent::Char('g') => {
                    tx.send(Messages::UpdateHomeSelected(UpdateHomeSelectedEvent::Home))
                        .unwrap();
                    None
                }
                KeyEvent::Char('G') => {
                    tx.send(Messages::UpdateHomeSelected(UpdateHomeSelectedEvent::End))
                        .unwrap();
                    None
                }
                KeyEvent::Char('u') => {
                    tx.send(Messages::LoadUpperDirectory).unwrap();
                    None
                }
                KeyEvent::Char('r') => {
                    tx.send(Messages::SetOpenMode(OpenMode::ReadOnly)).unwrap();
                    None
                }
                KeyEvent::Char('w') => {
                    tx.send(Messages::SetOpenMode(OpenMode::Write)).unwrap();
                    None
                }
                KeyEvent::Char('q') => {
                    tx.send(Messages::Quit).unwrap();
                    None
                }
                _ => Some(event),
            },
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
        let sidebar_length = self.app.config.items_per_line * 4 + 1;
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            // Main View and Sidebar
            .constraints([Constraint::Min(30), Constraint::Length(sidebar_length)].as_ref())
            .split(content);

        ItemWidget::new(self.app).draw(frame, main_chunks[0]);
        ExamItemsWidget::new(self.app).draw(frame, main_chunks[1]);
    }

    pub fn propagate(state: &App, event: Messages, tx: mpsc::Sender<Messages>) -> Option<Messages> {
        match &event {
            Messages::Input(InputEvent::Keyboard(KeyEvent::Char('q'))) => {
                tx.send(Messages::ChangeRoute(AppRoute::Home)).unwrap();
                None
            }
            Messages::Input(InputEvent::Keyboard(KeyEvent::Char(' '))) => {
                tx.send(Messages::ToggleExamResult).unwrap();
                None
            }
            _ => ExamItemsWidget::propagate(state, event, tx.clone())
                .and_then(|event| ItemWidget::propagate(state, event, tx)),
        }
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
            AppRoute::DoExam => {
                let exam = self.app.exam.as_ref().unwrap();
                let item: &Item = exam
                    .question_at(exam.display.question_index)
                    .expect("Item out of range!");

                match item {
                    Item::Question(question) => {
                        QuestionWidget::new(self.app, &question, &exam.display).draw(frame, content)
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn propagate(state: &App, event: Messages, tx: mpsc::Sender<Messages>) -> Option<Messages> {
        match &state.route {
            AppRoute::DoExam => {
                let exam = state.exam.as_ref().unwrap();
                match exam.question_at(exam.display.question_index) {
                    Some(item) => match item {
                        Item::Question(_) => QuestionWidget::propagate(state, event, tx),
                        _ => Some(event),
                    },
                    _ => Some(event),
                }
            }
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
        let question_title = format!(
            "Question ({}/{})",
            &self.display.question_index + 1,
            &self.app.exam.as_ref().unwrap().num_questions()
        );

        // Question + Selections
        let two_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(12)].as_ref())
            .split(content);

        let three_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Min(10),
                    Constraint::Length(12),
                    Constraint::Min(10),
                ]
                .as_ref(),
            )
            .split(content);

        match self.app.exam.as_ref().unwrap().display.display_answer {
            false => {
                // Question
                Paragraph::new([Text::raw(&self.question.question)].iter())
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(&question_title),
                    )
                    .wrap(true)
                    .render(frame, two_chunks[0]);

                // Selections
                let selections_state = self
                    .question
                    .selections
                    .iter()
                    .enumerate()
                    .take(8)
                    .map(|(index, sel)| {
                        let selection_flag: u8 = 0b1 << index;
                        ToggleButtonState {
                            text: sel.text.clone(),
                            selected: self.question.user_selection.is_selected(selection_flag),
                        }
                    })
                    .collect();

                ToggleButtons::new(selections_state).render(frame, two_chunks[1]);
            }
            true => {
                // Question
                let question_text = [Text::raw(&self.question.question)];
                let mut question_block = Paragraph::new(question_text.iter())
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(&question_title),
                    )
                    .wrap(true);

                // Selection
                let selections_state = self
                    .question
                    .selections
                    .iter()
                    .enumerate()
                    .take(8)
                    .map(|(index, sel)| {
                        let selection_flag: u8 = 0b1 << index;
                        let selected = self.question.user_selection.is_selected(selection_flag);

                        ToggleButtonState {
                            text: format!(
                                "[{}] {}",
                                match sel.should_select {
                                    true => '√',
                                    false => match selected {
                                        true => '×',
                                        false => ' ',
                                    },
                                },
                                &sel.text
                            ),
                            selected,
                        }
                    })
                    .collect();
                let mut selections_block = ToggleButtons::new(selections_state);

                // Answer
                let answer = match self.question.answer.as_ref() {
                    Some(answer) => answer,
                    None => "",
                };
                let answer_text = [Text::raw(answer)];
                let mut answer_block = Paragraph::new(answer_text.iter())
                    .block(Block::default().borders(Borders::TOP).title("­Answer"))
                    .wrap(true);

                match self.question.answer.is_some() {
                    true => {
                        // Question + Selection
                        question_block.render(frame, three_chunks[0]);
                        selections_block.render(frame, three_chunks[1]);
                        answer_block.render(frame, three_chunks[2]);
                    }
                    false => {
                        // Question + Selection + Answer
                        question_block.render(frame, two_chunks[0]);
                        selections_block.render(frame, two_chunks[1]);
                    }
                }
            }
        }
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
        let exam = self.app.exam.as_ref().unwrap();
        let num_questions = exam.num_questions();
        let current_index = exam.display.question_index;

        let selections_height = if num_questions as u16 % self.app.config.items_per_line == 0 {
            num_questions as u16 / self.app.config.items_per_line
        } else {
            num_questions as u16 / self.app.config.items_per_line + 1
        };
        let scroll_pos = if content.height >= selections_height + 2 {
            0
        } else {
            let diff = 2 + selections_height - content.height;
            let mut a = (diff as usize * current_index) as f32;
            a /= num_questions as f32;
            a.round() as u16
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

        let qitems = exam.questions.iter().enumerate();
        let items_per_line = self.app.config.items_per_line;

        match exam.display.display_answer {
            false => qitems.for_each(|(index, item)| {
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
                if (index + 1) as u16 % items_per_line == 0 {
                    texts.push(Text::raw("\n"));
                } else {
                    texts.push(Text::raw(" "));
                }
            }),
            true => qitems.for_each(|(index, item)| {
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
                            style.fg = style.bg;
                            style.bg = Color::Rgb(209, 162, 226);
                        };
                        texts.push(Text::styled(format!("{:3}", &index + 1), style));
                    }
                    Item::Card(_) => {
                        let mut style = PENDING_STYLE;
                        if current_index == index {
                            style.modifier = Modifier::BOLD;
                            style.fg = style.bg;
                            style.bg = Color::Magenta;
                        };
                        texts.push(Text::styled(format!("{:3}", &index + 1), style));
                    }
                };

                // Separator
                if (index + 1) as u16 % items_per_line == 0 {
                    texts.push(Text::raw("\n"));
                } else {
                    texts.push(Text::raw(" "));
                }
            }),
        }

        Paragraph::new(texts.iter())
            .block(Block::default().borders(Borders::ALL).title("Items"))
            .scroll(scroll_pos)
            .render(frame, content);
    }

    pub fn propagate(
        _state: &App,
        event: Messages,
        tx: mpsc::Sender<Messages>,
    ) -> Option<Messages> {
        match event {
            Messages::Input(InputEvent::Keyboard(key)) => match key {
                KeyEvent::Char('>') | KeyEvent::Char('n') => {
                    tx.send(Messages::UpdateQuestionIndex(
                        UpdateQuestionIndexEvent::Next,
                    ))
                    .unwrap();
                    None
                }
                KeyEvent::Char('<') | KeyEvent::Char('p') => {
                    tx.send(Messages::UpdateQuestionIndex(
                        UpdateQuestionIndexEvent::Prev,
                    ))
                    .unwrap();
                    None
                }
                _ => Some(event),
            },
            _ => Some(event),
        }
    }
}
