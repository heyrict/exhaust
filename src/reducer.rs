use crate::app::*;
use crate::event::*;

pub fn reduce(state: &mut App, event: Messages) -> Option<Messages> {
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
                match question {
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
                }
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
        _ => Some(event),
    }
}
