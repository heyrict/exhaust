use crate::app::*;
use crate::event::*;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;

pub fn reduce(state: &mut App, event: Messages, tx: mpsc::Sender<Messages>) -> Option<Messages> {
    // Route handler
    match event {
        Messages::ChangeRoute(route) => {
            // Saves data when quit from DoExam route
            if let AppRoute::DoExam = state.route {
                maybe_save_state(&state);
            };

            state.route = route;
            None
        }
        Messages::ToggleSelection(sel) => match &state.route {
            AppRoute::DoExam => {
                let exam = &state.exam.as_ref().unwrap();
                let index = exam.display.question_index;
                let question = state.exam.as_mut().unwrap().question_at_mut(index)?;
                let returns = match question {
                    Item::Question(q) => {
                        if !q.has_selection(sel) {
                            None
                        } else {
                            if q.num_should_selects() == 1usize {
                                q.user_selection = sel;
                            } else {
                                q.user_selection ^= sel;
                            };
                            None
                        }
                    }
                    _ => None,
                };

                // Saves data after updating selections.
                // This process should not block the main thread.
                maybe_save_state(&state);

                returns
            }
            _ => Some(event),
        },
        Messages::UpdateQuestionIndex(evt) => match &state.route {
            AppRoute::DoExam => {
                let exam = &state.exam.as_ref().unwrap();
                let question_index = exam.display.question_index;
                let max_index = exam.num_questions() - 1;
                let next_index = match &evt {
                    UpdateQuestionIndexEvent::Next => {
                        if question_index < max_index {
                            question_index + 1
                        } else {
                            0
                        }
                    }
                    UpdateQuestionIndexEvent::Prev => {
                        if question_index > 0 {
                            question_index - 1
                        } else {
                            max_index
                        }
                    }
                    UpdateQuestionIndexEvent::Set(index) => {
                        if *index > max_index {
                            max_index
                        } else {
                            *index
                        }
                    }
                };
                state.exam.as_mut().map(move |exam| {
                    exam.display.question_index = next_index;
                });
                None
            }
            _ => None,
        },
        Messages::ToggleExamResult => {
            state.exam.as_mut().map(|exam| {
                exam.display.display_answer = !exam.display.display_answer;
            });
            None
        }
        Messages::UpdateHomeSelected(evt) => match &state.route {
            AppRoute::Home => {
                let paths = state.home.get_paths().unwrap();
                let max_index = if paths.len() > 0 {
                    paths.len() - 1
                } else {
                    return None;
                };
                let current_selected = match state.home.current_selected {
                    Some(k) => k,
                    None => {
                        state.home.current_selected = Some(0);
                        return None;
                    }
                };
                let next_index = match &evt {
                    UpdateHomeSelectedEvent::Next => {
                        if current_selected < max_index {
                            current_selected + 1
                        } else {
                            0
                        }
                    }
                    UpdateHomeSelectedEvent::Prev => {
                        if current_selected > 0 {
                            current_selected - 1
                        } else {
                            max_index
                        }
                    }
                    UpdateHomeSelectedEvent::Home => 0,
                    UpdateHomeSelectedEvent::End => max_index,
                };
                state.home.current_selected = Some(next_index);
                None
            }
            _ => None,
        },
        Messages::LoadFile => {
            let filename = state.home.get_selected_path()?;
            match filename.is_dir() {
                true => {
                    state.home.current_path = filename.to_path_buf();
                    state.home.current_selected = None;
                }
                false => {
                    let mut file = File::open(filename).expect("Unable to open file");
                    let tx = tx.clone();

                    thread::spawn(move || {
                        let mut contents = String::new();
                        file.read_to_string(&mut contents)
                            .expect("Unable to read file");
                        let exam: Exam = serde_json::from_str(&contents)
                            .expect("Unable to convert file to string");
                        tx.send(Messages::FileLoaded(exam)).unwrap();
                        tx.send(Messages::ChangeRoute(AppRoute::DoExam)).unwrap();
                    });
                }
            };

            None
        }
        Messages::LoadUpperDirectory => {
            match state.home.current_path.parent() {
                Some(parent) => {
                    state.home.current_path = parent.to_path_buf();
                    state.home.current_selected = None;
                }
                None => {}
            };
            None
        }
        Messages::SetOpenMode(mode) => {
            state.home.open_mode = mode;
            None
        }
        Messages::FileLoaded(exam) => {
            state.exam = Some(exam);
            None
        }
        _ => Some(event),
    }
}

pub fn maybe_save_state(state: &App) -> Option<JoinHandle<()>> {
    // Save data on selection change
    if let OpenMode::Write = &state.home.open_mode {
        let exam_copy = state.exam.clone();
        let maybe_filename = state.home.get_selected_path();
        Some(thread::spawn(move || {
            maybe_filename.map(|filename| {
                let mut file = File::create(&filename)
                    .expect(&format!("Error opening {}", &filename.to_str().unwrap()));
                file.write_all(
                    serde_json::to_string(&exam_copy)
                        .expect("Error converting exam to json")
                        .as_ref(),
                )
                .expect(&format!("Error writing {}", &filename.to_str().unwrap()));
            });
        }))
    } else {
        None
    }
}
