pub struct Selection {
    pub text: String,
    pub should_select: bool,
}

bitflags! {
    #[derive(Default)]
    pub struct SelectionFlags: u8 {
        const A = 0b00000001;
        const B = 0b00000010;
        const C = 0b00000100;
        const D = 0b00001000;
        const E = 0b00010000;
        const F = 0b00100000;
        const G = 0b01000000;
        const H = 0b10000000;
        const NONE = 0b0;
    }
}

impl SelectionFlags {
    pub fn is_selected(&self, index: u8) -> bool {
        self.bits() & index != 0
    }
}

pub trait HasQuestionResult {
    fn get_result(&self) -> QuestionResult;
}

pub struct Question {
    pub question: String,
    pub selections: Vec<Selection>,
    pub answer: Option<String>,
    pub user_selection: SelectionFlags,
}

impl Question {
    pub fn get_should_selects(&self) -> SelectionFlags {
        let mut result = SelectionFlags::NONE;
        self.selections
            .iter()
            .enumerate()
            .for_each(|(index, _sel)| {
                SelectionFlags::from_bits(0b1 << index).and_then(|mask| {
                    result |= mask;
                    Some(mask)
                });
            });
        result
    }
}

impl HasQuestionResult for Question {
    fn get_result(&self) -> QuestionResult {
        match self.user_selection.bits() == 0 {
            true => QuestionResult::Pending,
            false => match self.get_should_selects() == self.user_selection {
                true => QuestionResult::Correct,
                false => QuestionResult::Wrong,
            },
        }
    }
}

pub struct Card {
    question: String,
    answer: String,
}

pub enum Item {
    Question(Question),
    Card(Card),
}

pub struct Exam {
    pub questions: Vec<Item>,
    pub result: ExamResult,
}

#[derive(Clone, Debug)]
pub enum QuestionResult {
    Pending,
    Correct,
    Wrong,
    Done,
}

#[derive(Clone, Debug)]
pub enum ExamResult {
    Pending,
    Done,
}

#[derive(Clone, Debug)]
pub struct DoExamDisplay {
    pub question_index: usize,
    pub display_answer: bool,
}

impl Default for DoExamDisplay {
    fn default() -> Self {
        DoExamDisplay {
            question_index: 0,
            display_answer: false,
        }
    }
}

impl DoExamDisplay {
    pub fn set_index(&mut self, index: usize) {
        self.question_index = index;
    }
}

#[derive(Debug)]
pub enum AppRoute {
    Home,
    DoExam(DoExamDisplay),
}

pub struct App {
    pub exam: Exam,
    pub route: AppRoute,
}

impl Exam {
    pub fn question_at(&self, index: usize) -> Option<&Item> {
        self.questions.get(index)
    }

    pub fn question_at_mut(&mut self, index: usize) -> Option<&mut Item> {
        self.questions.get_mut(index)
    }

    pub fn num_questions(&self) -> usize {
        self.questions.len()
    }
}

impl Question {
    pub fn num_selections(&self) -> usize {
        self.selections.len()
    }

    pub fn num_should_selects(&self) -> usize {
        self.selections
            .iter()
            .filter(|sel| sel.should_select)
            .collect::<Vec<&Selection>>()
            .len()
    }

    pub fn has_selection(&self, selection: SelectionFlags) -> bool {
        selection.bits() >> self.selections.len() == 0
    }
}

pub fn get_sample_app() -> App {
    App {
        exam: Exam {
            questions: vec![
                Item::Question(Question {
                    question: "Game Over. Continue?".to_owned(),
                    selections: vec![
                        Selection {
                            text: "No".to_owned(),
                            should_select: false,
                        },
                        Selection {
                            text: "Yes".to_owned(),
                            should_select: true,
                        },
                    ],
                    answer: Some("You should select yes".to_owned()),
                    user_selection: Default::default(),
                }),
                Item::Question(Question {
                    question: "Select your gender".to_owned(),
                    selections: vec![
                        Selection {
                            text: "Male".to_owned(),
                            should_select: false,
                        },
                        Selection {
                            text: "Female".to_owned(),
                            should_select: false,
                        },
                    ],
                    answer: None,
                    user_selection: Default::default(),
                }),
            ],
            result: ExamResult::Pending,
        },
        route: AppRoute::Home,
    }
}
