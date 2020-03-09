/*
 * ui.rs
 *
 * The definition of terminal user interface.
 *
 * ## Structure
 * - AppWidget
 *   - HomeWidget
 *   - ExamWidget
 *     - JumpBarWidget
 *     - ExamItemsWidget
 *     - ItemWidget
 *       - QuestionWidget
 */
use std::sync::mpsc;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Gauge, Paragraph, SelectableList, Text, Widget};
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
            fg: Color::Magenta,
            bg: Color::Reset,
            modifier: Modifier::REVERSED,
        };
        const CURRENT_PATH_STYLE: Style = Style {
            fg: Color::Green,
            bg: Color::Reset,
            modifier: Modifier::BOLD,
        };

        let pwd = self.app.home.current_path.to_str().unwrap_or("???");
        let welcome_messages: [Text; 2] = [
            Text::raw("Welcome! Choose a file to start:\n\nCurrent Path: "),
            Text::styled(pwd, CURRENT_PATH_STYLE),
        ];
        let footer_messages: [Text; 7] = [
            Text::raw("["),
            Text::styled("q", UNDERLINE_STYLE),
            Text::raw(": Quit] | ["),
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
            .constraints(
                [
                    Constraint::Length(5),
                    Constraint::Min(6),
                    Constraint::Length(2),
                ]
                .as_ref(),
            )
            .margin(1)
            .split(content);
        let _welcome = Paragraph::new(welcome_messages.iter())
            .block(Block::default().borders(Borders::ALL))
            .render(frame, chunks[0]);
        let _footer = Paragraph::new(footer_messages.iter())
            .block(Block::default().borders(Borders::TOP))
            .render(frame, chunks[2]);
        let parent_dir = self.app.home.current_path.parent();
        let _items = SelectableList::default()
            .items(
                &paths
                    .iter()
                    .map(|path| {
                        if parent_dir.map(|pd| path == pd).unwrap_or(false) {
                            return "../".to_owned();
                        };

                        let filename = path
                            .file_name()
                            .and_then(|filename| filename.to_str())
                            .unwrap_or("???");
                        match path.is_dir() {
                            true => format!("{}/", filename),
                            false => filename.to_owned(),
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
        match &event {
            Messages::Input(keyevent) => match keyevent {
                key!(Enter) => {
                    tx.send(Messages::LoadFile).unwrap();
                    None
                }
                key!('j') | key!(Down) => {
                    tx.send(Messages::UpdateHomeSelected(UpdateHomeSelectedEvent::Next))
                        .unwrap();
                    None
                }
                key!('k') | key!(Up) => {
                    tx.send(Messages::UpdateHomeSelected(UpdateHomeSelectedEvent::Prev))
                        .unwrap();
                    None
                }
                key!('g') => {
                    tx.send(Messages::UpdateHomeSelected(UpdateHomeSelectedEvent::Home))
                        .unwrap();
                    None
                }
                key!('G') => {
                    tx.send(Messages::UpdateHomeSelected(UpdateHomeSelectedEvent::End))
                        .unwrap();
                    None
                }
                key!('r') => {
                    tx.send(Messages::SetOpenMode(OpenMode::ReadOnly)).unwrap();
                    None
                }
                key!('w') => {
                    tx.send(Messages::SetOpenMode(OpenMode::Write)).unwrap();
                    None
                }
                key!('q') => {
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
            .direction(Direction::Vertical)
            .margin(1)
            // Title bar, progress bar and the rest
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(10),
                ]
                .as_ref(),
            )
            .split(content);

        // Title bar
        let filepath = self.app.home.get_selected_path().unwrap();
        let filename = filepath.file_stem().unwrap().to_str().unwrap();
        let title = match self.app.home.open_mode {
            OpenMode::ReadOnly => format!("{} [readonly]", &filename),
            OpenMode::Write => format!("{} [write]", &filename),
        };
        Paragraph::new(
            [Text::Styled(
                title.into(),
                Style::default().modifier(Modifier::BOLD),
            )]
            .iter(),
        )
        .style(Style::default().modifier(Modifier::REVERSED))
        .alignment(Alignment::Center)
        .render(frame, main_chunks[0]);

        // Progress bar
        let num_questions = self.app.exam.as_ref().unwrap().num_questions();
        let num_answered = self
            .app
            .exam
            .as_ref()
            .unwrap()
            .questions
            .iter()
            .filter(|item| match item {
                Item::Question(question) => question.user_selection.is_empty(),
                _ => false,
            })
            .collect::<Vec<&Item>>()
            .len();
        Gauge::default()
            .ratio(1f64 - num_answered as f64 / num_questions as f64)
            .style(
                Style::default()
                    .fg(Color::Rgb(147, 161, 161))
                    .bg(Color::Rgb(238, 232, 213)),
            )
            .render(frame, main_chunks[1]);

        let main_chunks = match &self.app.config.show_usage {
            // Has usage footer
            true => {
                let main_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    // Main View and Sidebar
                    .constraints([Constraint::Min(10), Constraint::Length(1)].as_ref())
                    .split(main_chunks[2]);

                let footer_messages: [Text; 1] = [Text::raw(
                    "Usage: [q: quit][a-h: toggle answer][space: toggle view]\
            [0-9: goto][n,p: change page]",
                )];
                Paragraph::new(footer_messages.iter())
                    .alignment(Alignment::Center)
                    .render(frame, main_chunks[1]);

                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(30), Constraint::Length(sidebar_length)].as_ref())
                    .split(main_chunks[0])
            }
            // Has no usage footer
            false => Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(30), Constraint::Length(sidebar_length)].as_ref())
                .split(main_chunks[2]),
        };

        ItemWidget::new(self.app).draw(frame, main_chunks[0]);

        match self.app.exam.as_ref().unwrap().jumpbox_value {
            // Do not display jumpbox if its value is zero.
            0 => {
                ExamItemsWidget::new(self.app).draw(frame, main_chunks[1]);
            }
            _ => {
                let sidebar_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    // Jumpbar and ExamItems
                    .constraints([Constraint::Length(3), Constraint::Min(5)].as_ref())
                    .split(main_chunks[1]);

                JumpBarWidget::new(self.app).draw(frame, sidebar_chunks[0]);
                ExamItemsWidget::new(self.app).draw(frame, sidebar_chunks[1]);
            }
        }
    }

    pub fn propagate(state: &App, event: Messages, tx: mpsc::Sender<Messages>) -> Option<Messages> {
        match &event {
            Messages::Input(keyevent) => match keyevent {
                key!('q') => {
                    tx.send(Messages::ChangeRoute(AppRoute::Home)).unwrap();
                    return None;
                }
                key!(' ') => {
                    tx.send(Messages::ToggleExamResult).unwrap();
                    return None;
                }
                _ => {}
            },
            _ => {}
        };
        ExamItemsWidget::propagate(state, event, tx.clone())
            .and_then(|event| JumpBarWidget::propagate(state, event, tx.clone()))
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
        const WRAPPER_SELECT: [&str; 2] = ["(", ")"];
        const WRAPPER_MULTSEL: [&str; 2] = ["[", "]"];
        let current_wrapper = if self.question.num_should_selects() == 1usize {
            WRAPPER_SELECT
        } else {
            WRAPPER_MULTSEL
        };

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
                    .scroll(self.display.question_scroll_pos)
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

                ToggleButtons::new(selections_state)
                    .wrapper(current_wrapper)
                    .render(frame, two_chunks[1]);
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
                    .scroll(self.display.question_scroll_pos)
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
                let mut selections_block =
                    ToggleButtons::new(selections_state).wrapper(current_wrapper);

                // Answer
                let answer = match self.question.answer.as_ref() {
                    Some(answer) => answer,
                    None => "",
                };
                let answer_text = [Text::raw(answer)];
                let mut answer_block = Paragraph::new(answer_text.iter())
                    .block(Block::default().borders(Borders::TOP).title("­Answer"))
                    .scroll(self.display.question_scroll_pos)
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

    pub fn propagate(state: &App, event: Messages, tx: mpsc::Sender<Messages>) -> Option<Messages> {
        match event {
            Messages::Input(keyevent) => match keyevent {
                key!('a') | key!('A') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::A))
                        .unwrap();
                    None
                }
                key!('b') | key!('B') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::B))
                        .unwrap();
                    None
                }
                key!('c') | key!('C') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::C))
                        .unwrap();
                    None
                }
                key!('d') | key!('D') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::D))
                        .unwrap();
                    None
                }
                key!('e') | key!('E') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::E))
                        .unwrap();
                    None
                }
                key!('f') | key!('F') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::F))
                        .unwrap();
                    None
                }
                key!('g') | key!('G') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::G))
                        .unwrap();
                    None
                }
                key!('h') | key!('H') => {
                    tx.send(Messages::ToggleSelection(SelectionFlags::H))
                        .unwrap();
                    None
                }
                key!('j') => {
                    state.exam.as_ref().map(|exam: &Exam| {
                        tx.send(Messages::ScrollQuestion(
                            &exam.display.question_scroll_pos + 1,
                        ))
                        .unwrap();
                    });
                    None
                }
                key!('k') => {
                    state.exam.as_ref().map(|exam: &Exam| {
                        let next_pos = match &exam.display.question_scroll_pos {
                            0 => 0,
                            _ => &exam.display.question_scroll_pos - 1,
                        };
                        tx.send(Messages::ScrollQuestion(next_pos)).unwrap();
                    });
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
            Messages::Input(keyevent) => match keyevent {
                key!('>') | key!('n') => {
                    tx.send(Messages::UpdateQuestionIndex(
                        UpdateQuestionIndexEvent::Next,
                    ))
                    .unwrap();
                    None
                }
                key!('<') | key!('p') => {
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

pub struct JumpBarWidget<'a> {
    app: &'a App,
}

impl<'a> JumpBarWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        JumpBarWidget { app }
    }

    pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, content: Rect) {
        const JUMPBOX_TEXT_STYLE: Style = Style {
            fg: Color::Black,
            bg: Color::Reset,
            modifier: Modifier::UNDERLINED,
        };

        let sidebar_inner_length = self.app.config.items_per_line * 4 - 1;
        let jumpbox_value = self.app.exam.as_ref().unwrap().jumpbox_value;
        let jumpbox_text = [
            Text::styled(
                " ".repeat(sidebar_inner_length as usize - jumpbox_value.to_string().len()),
                JUMPBOX_TEXT_STYLE,
            ),
            Text::styled(format!("{}", jumpbox_value), JUMPBOX_TEXT_STYLE),
        ];

        Paragraph::new(jumpbox_text.iter())
            .block(Block::default().borders(Borders::ALL).title("Jump to"))
            .render(frame, content);
    }

    pub fn propagate(state: &App, event: Messages, tx: mpsc::Sender<Messages>) -> Option<Messages> {
        let exam = state.exam.as_ref().unwrap();
        let jumpbox_value = exam.jumpbox_value;
        match event {
            Messages::Input(keyevent) => match keyevent {
                key!(Char(c)) => {
                    if c < '0' || c > '9' {
                        return Some(event);
                    };
                    let next_value = jumpbox_value * 10 + c.to_digit(10)? as u16;
                    if next_value > exam.num_questions() as u16 {
                        tx.send(Messages::UpdateJumpboxValue(exam.num_questions() as u16))
                            .unwrap();
                    } else {
                        tx.send(Messages::UpdateJumpboxValue(next_value)).unwrap();
                    }
                    None
                }
                key!(Enter) => {
                    // Do not handle Enter if Jumpbox is closed
                    if jumpbox_value == 0 {
                        return Some(event);
                    }

                    tx.send(Messages::UpdateQuestionIndex(
                        UpdateQuestionIndexEvent::Set(jumpbox_value as usize - 1),
                    ))
                    .unwrap();
                    tx.send(Messages::UpdateJumpboxValue(0)).unwrap();
                    None
                }
                key!(Backspace) => {
                    if jumpbox_value != 0 {
                        tx.send(Messages::UpdateJumpboxValue(jumpbox_value / 10))
                            .unwrap();
                        None
                    } else {
                        Some(event)
                    }
                }
                key!(Esc) => {
                    if jumpbox_value != 0 {
                        tx.send(Messages::UpdateJumpboxValue(0)).unwrap();
                        None
                    } else {
                        Some(event)
                    }
                }
                _ => Some(event),
            },
            _ => Some(event),
        }
    }
}
