use crate::app::*;
use crate::event::Messages;

pub fn reduce(state: &mut App, event: Messages) -> Option<Messages> {
    // Route handler
    match event {
        Messages::ChangeRoute(route) => {
            state.route = route;
            None
        }
        Messages::ToggleSelection(sel) => {
            if let AppRoute::DoExam(display) = &state.route {
                let index = display.question_index;
                let question = state.exam.question_at_mut(index)?;
                match question {
                    Item::Question(q) => {
                        if q.num_should_selects() == 1usize {
                            q.user_selection = sel;
                        } else {
                            q.user_selection ^= sel;
                        }
                        return None;
                    }
                    _ => {}
                };
            };
            Some(event)
        }
        _ => Some(event),
    }
}
