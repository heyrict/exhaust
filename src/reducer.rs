use crate::app::*;
use crate::event::*;
use std::fs::read_dir;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

pub fn reduce(state: &mut App, event: Messages, tx: mpsc::Sender<Messages>) -> Option<Messages> {
    // Route handler
    match event {
        Messages::ChangeRoute(route) => {
            state.route = route;
            None
        }
        Messages::ToggleSelection(sel) => match &state.route {
            AppRoute::DoExam(display) => {
                let index = display.question_index;
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

                // Save data on selection change
                // TODO: Move save to a separate event;
                match &event {
                    Messages::ToggleSelection(_) => {
                        let exam_copy = state.exam.clone();
                        thread::spawn(move || {
                            let mut file = File::create("/tmp/temp.json")
                                .expect("Error opening /tmp/temp.json");
                            file.write_all(
                                serde_json::to_string(&exam_copy)
                                    .expect("Error converting exam to json")
                                    .as_ref(),
                            )
                            .expect("Error writing /tmp/temp.json");
                        });
                    }
                    _ => {}
                };

                returns
            }
            _ => Some(event),
        },
        Messages::UpdateQuestionIndex(evt) => match &state.route {
            AppRoute::DoExam(display) => {
                let max_index = state.exam.as_mut().unwrap().num_questions() - 1;
                let next_index = match &evt {
                    UpdateQuestionIndexEvent::Next => {
                        if display.question_index < max_index {
                            display.question_index + 1
                        } else {
                            0
                        }
                    }
                    UpdateQuestionIndexEvent::Prev => {
                        if display.question_index > 0 {
                            display.question_index - 1
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
                let new_display = DoExamDisplay {
                    question_index: next_index,
                    ..display.clone()
                };
                state.route = AppRoute::DoExam(new_display);
                None
            }
            _ => None,
        },
        Messages::ToggleExamResult => {
            state.exam.as_mut().unwrap().result = match state.exam.as_ref().unwrap().result {
                ExamResult::Done => ExamResult::Pending,
                ExamResult::Pending => ExamResult::Done,
            };
            None
        }
        Messages::UpdateHomeSelected(evt) => match &state.route {
            AppRoute::Home => {
                let paths: Vec<PathBuf> = read_dir(&state.home.current_path)
                    .unwrap()
                    .map(|path| path.unwrap().path())
                    .collect();
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
                };
                state.home.current_selected = Some(next_index);
                None
            }
            _ => None,
        },
        Messages::LoadFile => {
            let paths: Vec<PathBuf> = read_dir(&state.home.current_path)
                .unwrap()
                .map(|path| path.unwrap().path())
                .collect();
            let selected_index = match &state.home.current_selected {
                Some(i) => i,
                None => return None,
            };
            let filename = paths.get(*selected_index).expect("Index out of range");
            let mut file = File::open(filename).expect("Unable to open file");
            let tx = tx.clone();

            thread::spawn(move || {
                let mut contents = String::new();
                file.read_to_string(&mut contents)
                    .expect("Unable to read file");
                let exam: Exam =
                    serde_json::from_str(&contents).expect("Unable to convert file to string");
                tx.send(Messages::FileLoaded(exam)).unwrap();
                tx.send(Messages::ChangeRoute(AppRoute::DoExam(
                    DoExamDisplay::default(),
                )))
                .unwrap();
            });

            None
        }
        Messages::FileLoaded(exam) => {
            state.exam = Some(exam);
            None
        }
        _ => Some(event),
    }
}
