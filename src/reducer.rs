use crate::app::*;
use crate::event::*;
use libflate::gzip::{Decoder, Encoder};
use std::fs::File;
use std::io::Read;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;
use tui::widgets::ListState;

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
        Messages::UpdateHomeSelected(evt) => {
            let paths = state.home.get_paths().unwrap();
            let max_index = if paths.len() > 0 {
                paths.len() - 1
            } else {
                return None;
            };
            let selected = state.home.list_state.selected();
            let next_index = match &evt {
                UpdateListSelectedEvent::Next => selected
                    .map(|selected| {
                        if selected < max_index {
                            selected + 1
                        } else {
                            0
                        }
                    })
                    .unwrap_or(0),
                UpdateListSelectedEvent::Prev => selected
                    .map(|selected| {
                        if selected > 0 {
                            selected - 1
                        } else {
                            max_index
                        }
                    })
                    .unwrap_or(0),
                UpdateListSelectedEvent::Home => 0,
                UpdateListSelectedEvent::End => max_index,
            };
            state.home.list_state.select(Some(next_index));
            None
        }
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
        Messages::SaveModalAction(action) => match action {
            SaveModalActions::Open(modal_state) => {
                state.modal.save_modal_state = modal_state;
                None
            }
            SaveModalActions::Quit(quit_action) => {
                state.modal.save_modal_state = SaveModalState::Hidden;

                match quit_action {
                    QuitAction::BackHome => {
                        state.route = AppRoute::Home;
                    }
                    QuitAction::QuitProgram => {
                        tx.send(Messages::Quit).unwrap();
                    }
                };
                None
            }
            SaveModalActions::Okay => {
                save_state(&state, tx.clone());
                state.modal.save_modal_state = SaveModalState::Hidden;
                None
            }
            SaveModalActions::Cancel => {
                state.modal.save_modal_state = SaveModalState::Hidden;
                None
            }
        },
        Messages::AssetsModalAction(action) => match action {
            AssetsModalActions::Open => {
                let exam = state.exam.as_ref().unwrap();
                let current_item = exam.question_at(exam.display.question_index).unwrap();
                let num_assets = current_item.get_assets().len();

                if num_assets > 0 {
                    state.modal.assets_modal_state = AssetsModalState::Show(ListState::default());
                }
                None
            }
            AssetsModalActions::Close => {
                state.modal.assets_modal_state = AssetsModalState::Hidden;
                None
            }
            AssetsModalActions::Select(evt) => {
                let exam = state.exam.as_ref().unwrap();
                let current_item = exam.question_at(exam.display.question_index).unwrap();
                let assets = current_item.get_assets();

                let max_index = if assets.len() > 0 {
                    assets.len() - 1
                } else {
                    return None;
                };

                let next_index = {
                    let selected = match &state.modal.assets_modal_state {
                        AssetsModalState::Show(list_state) => list_state.selected(),
                        AssetsModalState::Hidden => return None,
                    };

                    match &evt {
                        UpdateListSelectedEvent::Next => selected
                            .map(|selected| {
                                if selected < max_index {
                                    selected + 1
                                } else {
                                    0
                                }
                            })
                            .unwrap_or(0),
                        UpdateListSelectedEvent::Prev => selected
                            .map(|selected| {
                                if selected > 0 {
                                    selected - 1
                                } else {
                                    max_index
                                }
                            })
                            .unwrap_or(0),
                        UpdateListSelectedEvent::Home => 0,
                        UpdateListSelectedEvent::End => max_index,
                    }
                };

                if let AssetsModalState::Show(list_state) = &mut state.modal.assets_modal_state {
                    list_state.select(Some(next_index));
                };
                None
            }
            AssetsModalActions::OpenFile => {
                let launcher = &state.config.launcher;
                let current_path = &state.home.current_path;

                let assets_list_state = match &state.modal.assets_modal_state {
                    AssetsModalState::Show(list_state) => list_state,
                    AssetsModalState::Hidden => unreachable!(),
                };
                let exam = state.exam.as_ref().unwrap();
                exam.question_at(exam.display.question_index)
                    .map(|current_item| current_item.get_assets())
                    .and_then(|assets| {
                        let selected = assets_list_state.selected()?;
                        let current_file = assets.get(selected)?;
                        Command::new(launcher)
                            .arg(current_path.join(current_file))
                            .spawn()
                            .ok()
                    });

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
