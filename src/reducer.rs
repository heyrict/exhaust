use crate::app::*;
use crate::event::*;
use libflate::gzip::{Decoder, Encoder};
use std::fs::File;
use std::io::Read;
use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;

pub fn reduce(state: &mut App, event: Messages, tx: mpsc::Sender<Messages>) -> Option<Messages> {
    // Route handler
    match event {
        Messages::ChangeRoute(route) => {
            // Saves data when quit from DoExam route
            if let AppRoute::DoExam = &state.route {
                if let OpenMode::AutoSave = &state.home.open_mode {
                    save_state(&state, tx.clone());
                }
            };

            state.route = route;
            None
        }
        Messages::ToggleSelection(sel) => match &state.route {
            AppRoute::DoExam => {
                let exam = &state.exam.as_ref().unwrap();
                let index = exam.display.question_index;
                let question = state.exam.as_mut().unwrap().question_at_mut(index)?;
                let modified = match question {
                    Item::Question(q) => {
                        if !q.has_selection(sel) {
                            false
                        } else {
                            if q.num_should_selects() == 1usize {
                                if q.user_selection == sel {
                                    false
                                } else {
                                    q.user_selection = sel;
                                    true
                                }
                            } else {
                                q.user_selection ^= sel;
                                true
                            }
                        }
                    }
                    _ => false,
                };

                if modified {
                    // Once data changed, set `unsaved_changes` to true
                    state.exam.as_mut().map(|exam| {
                        exam.unsaved_changes = true;
                    });
                    // Saves data after updating selections.
                    // This process should not block the main thread.
                    if let OpenMode::AutoSave = &state.home.open_mode {
                        save_state(&state, tx.clone());
                    }
                }

                None
            }
            _ => Some(event),
        },
        Messages::UpdateQuestionIndex(evt) => match &state.route {
            AppRoute::DoExam => {
                let exam = state.exam.as_ref().unwrap();
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
                    // Update question index
                    exam.display.question_index = next_index;

                    // Reset scroll position
                    exam.display.question_scroll_pos = 0;
                });
                None
            }
            _ => None,
        },
        Messages::ScrollQuestion(pos) => {
            state.exam.as_mut().map(move |exam: &mut Exam| {
                exam.display.question_scroll_pos = pos;
            });
            None
        }
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
                let current_selected = match state.home.list_state.selected() {
                    Some(k) => k,
                    None => {
                        state.home.list_state.select(Some(0));
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
                state.home.list_state.select(Some(next_index));
                None
            }
            _ => None,
        },
        Messages::UpdateJumpboxValue(value) => {
            state.exam.as_mut().map(|exam| {
                exam.jumpbox_value = value;
            });
            None
        }
        Messages::LoadFile => {
            let filename = state.home.get_selected_path()?;
            match filename.is_dir() {
                true => {
                    state.home.current_path = filename.to_path_buf();
                    state.home.list_state.select(Some(0));
                }
                false => match filename.extension() {
                    Some(ext) => match ext.to_str() {
                        Some("exhaust") | Some("gz") => {
                            let file = File::open(filename).expect("Unable to open file");
                            let tx = tx.clone();

                            thread::spawn(move || {
                                let mut decoder =
                                    Decoder::new(&file).expect("Unable to decompress the file");
                                let mut contents = String::new();
                                decoder
                                    .read_to_string(&mut contents)
                                    .expect("Unable to decode the file with utf-8");

                                let exam: Exam = serde_json::from_str(&contents)
                                    .expect("Unable to convert file to string");
                                tx.send(Messages::FileLoaded(exam)).unwrap();
                                tx.send(Messages::ChangeRoute(AppRoute::DoExam)).unwrap();
                            });
                        }
                        Some("json") => {
                            // Parse Json
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
                        _ => {}
                    },
                    _ => {}
                },
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
        Messages::ModalAction(action) => match action {
            ModalActions::Open => {
                state.modal.show_save_model = true;
                None
            }
            ModalActions::Okay => {
                save_state(&state, tx.clone());
                state.modal.show_save_model = false;
                None
            }
            ModalActions::Cancel => {
                state.modal.show_save_model = false;
                None
            }
        },
        Messages::UnsavedChanges(uc) => {
            state.exam.as_mut().map(|exam| {
                exam.unsaved_changes = uc;
            });
            None
        }
        _ => Some(event),
    }
}

pub fn save_state(state: &App, tx: mpsc::Sender<Messages>) -> Option<JoinHandle<()>> {
    // Save data on selection change
    let exam_copy = state.exam.clone();
    let maybe_filename = state.home.get_selected_path();
    let pretty_printing = state.config.pretty_printing;

    Some(thread::spawn(move || {
        maybe_filename.map(|filename| {
            let file = File::create(&filename)
                .expect(&format!("Error opening {}", &filename.to_str().unwrap()));
            match filename.extension() {
                Some(ext) => match ext.to_str() {
                    Some("json") => match pretty_printing {
                        false => serde_json::to_writer(&file, &exam_copy),
                        true => serde_json::to_writer_pretty(&file, &exam_copy),
                    }
                    .expect(&format!("Error writing {}", &filename.to_str().unwrap())),
                    Some("exhaust") | Some("gz") => {
                        let mut encoder = Encoder::new(file).expect("Unable to initialize encoder");
                        match pretty_printing {
                            false => serde_json::to_writer(&mut encoder, &exam_copy),
                            true => serde_json::to_writer_pretty(&mut encoder, &exam_copy),
                        }
                        .expect("Unable to write to file");
                        encoder.finish();
                    }
                    _ => {}
                },
                _ => {}
            };
            tx.send(Messages::UnsavedChanges(false)).unwrap();
        });
    }))
}
