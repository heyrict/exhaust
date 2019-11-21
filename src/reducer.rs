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
                let question = state.exam.question_at_mut(index)?;
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
                let max_index = state.exam.num_questions() - 1;
                let next_index = match &evt {
                    UpdateQuestionIndexEvent::Next => {
                        if display.question_index < max_index {
                            display.question_index + 1
                        } else {
                            max_index
                        }
                    }
                    UpdateQuestionIndexEvent::Prev => {
                        if display.question_index > 0 {
                            display.question_index - 1
                        } else {
                            0
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
        _ => Some(event),
    }
}
