use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::current_dir;
use std::fs::read_dir;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Selection {
    pub text: String,
    #[serde(default)]
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
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Question {
    pub fn get_should_selects(&self) -> SelectionFlags {
        let mut result = SelectionFlags::NONE;
        self.selections.iter().enumerate().for_each(|(index, sel)| {
            SelectionFlags::from_bits(0b1 << index).and_then(|mask| {
                if sel.should_select {
                    result |= mask;
                }
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
#[serde(tag = "type")]
pub enum Item {
    Question(Question),
    Card(Card),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Exam {
    pub questions: Vec<Item>,
    #[serde(skip)]
    pub display: DoExamDisplay,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
    #[serde(skip)]
    pub jumpbox_value: u16,
}

#[derive(Clone, Debug)]
pub enum QuestionResult {
    Pending,
    Correct,
    Wrong,
    Done,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DoExamDisplay {
    pub question_index: usize,
    #[serde(skip)]
    pub question_scroll_pos: u16,
    pub display_answer: bool,
}

impl Default for DoExamDisplay {
    fn default() -> Self {
        DoExamDisplay {
            question_index: 0,
            question_scroll_pos: 0,
            display_answer: false,
        }
    }
}

#[derive(Debug)]
pub enum AppRoute {
    Home,
    DoExam,
}

impl Default for AppRoute {
    fn default() -> Self {
        AppRoute::Home
    }
}

#[derive(Debug)]
pub enum OpenMode {
    ReadOnly,
    Write,
}

impl Default for OpenMode {
    fn default() -> Self {
        Self::ReadOnly
    }
}

pub struct Home {
    pub exam_src: Option<PathBuf>,
    pub current_path: PathBuf,
    pub current_selected: Option<usize>,
    pub open_mode: OpenMode,
}

impl Default for Home {
    fn default() -> Self {
        Home {
            exam_src: None,
            current_path: current_dir().expect("Unable to get current directory"),
            current_selected: Some(0),
            open_mode: OpenMode::default(),
        }
    }
}

impl Home {
    pub fn get_selected_path(&self) -> Option<PathBuf> {
        let paths = self.get_paths().ok()?;
        paths
            .get(self.current_selected?)
            .map(|path| path.to_path_buf())
    }

    pub fn get_paths(&self) -> Result<Vec<PathBuf>, std::io::Error> {
        read_dir(&self.current_path).map(|result| {
            let mut paths: Vec<PathBuf> = result
                .map(|path| path.unwrap().path())
                .filter(|path| {
                    path.is_dir()
                        || match path.extension() {
                            Some(ext) => match ext.to_str() {
                                Some("json") => true,
                                Some("exhaust") | Some("gz") => true,
                                _ => false,
                            },
                            _ => false,
                        }
                })
                .collect();
            paths.sort();

            &self.current_path.parent().map(|parent_dir| {
                paths.insert(0, parent_dir.into());
            });
            paths
        })
    }
}

pub struct Config {
    pub items_per_line: u16,
}

impl Default for Config {
    fn default() -> Self {
        Config { items_per_line: 5 }
    }
}

#[derive(Default)]
pub struct App {
    pub route: AppRoute,
    pub exam: Option<Exam>,
    pub home: Home,
    pub config: Config,
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
        // `selection` should not be greater than the char that
        // the number of selections indicates.
        selection.bits() >> self.num_selections() == 0
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
