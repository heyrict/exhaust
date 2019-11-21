use serde::{Deserialize, Serialize};
use std::env::current_dir;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Question {
    pub question: String,
    pub selections: Vec<Selection>,
    pub answer: Option<String>,
    #[serde(default, with = "selection_flags_serde")]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Card {
    question: String,
    answer: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Item {
    Question(Question),
    Card(Card),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Exam {
    pub questions: Vec<Item>,
    #[serde(default)]
    pub result: ExamResult,
}

#[derive(Clone, Debug)]
pub enum QuestionResult {
    Pending,
    Correct,
    Wrong,
    Done,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ExamResult {
    Pending,
    Done,
}

impl Default for ExamResult {
    fn default() -> Self {
        ExamResult::Pending
    }
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

pub struct Home {
    pub exam_src: Option<PathBuf>,
    pub current_path: PathBuf,
    pub current_selected: Option<usize>,
}

pub struct App {
    pub route: AppRoute,
    pub exam: Option<Exam>,
    pub home: Home,
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
        home: Home {
            current_path: current_dir().expect("Unable to get current directory"),
            exam_src: None,
            current_selected: None,
        },
        exam: None,
        route: AppRoute::Home,
    }
}

mod selection_flags_serde {
    use super::SelectionFlags;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(data: &SelectionFlags, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(data.bits())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SelectionFlags, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bits = u8::deserialize(deserializer)?;
        SelectionFlags::from_bits(bits).ok_or(serde::de::Error::custom(
            "Error deserializing selection flags",
        ))
    }
}
