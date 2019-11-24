use crate::app::Exam;
use crate::app::{AppRoute, OpenMode, SelectionFlags};

use std::sync::mpsc;
use std::thread;

use crossterm::input::{input, InputEvent};

#[derive(Debug)]
pub enum UpdateQuestionIndexEvent {
    Next,
    Prev,
    Set(usize),
}

#[derive(Debug)]
pub enum UpdateHomeSelectedEvent {
    Next,
    Prev,
    Home,
    End,
}

#[derive(Debug)]
pub enum Messages {
    Input(InputEvent),
    ChangeRoute(AppRoute),
    UpdateQuestionIndex(UpdateQuestionIndexEvent),
    ScrollQuestion(u16),
    UpdateHomeSelected(UpdateHomeSelectedEvent),
    ToggleSelection(SelectionFlags),
    LoadFile,
    LoadUpperDirectory,
    FileLoaded(Exam),
    SetOpenMode(OpenMode),
    ToggleExamResult,
    Quit,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    pub tx: mpsc::Sender<Messages>,
    rx: mpsc::Receiver<Messages>,
    input_handle: thread::JoinHandle<()>,
}

impl Events {
    pub fn new() -> Events {
        let (tx, rx) = mpsc::channel();
        let input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let input = input();
                let mut reader = input.read_sync();
                loop {
                    if let Some(evt) = reader.next() {
                        if let Err(_) = tx.send(Messages::Input(evt)) {
                            return;
                        }
                    }
                }
            })
        };
        Events {
            tx,
            rx,
            input_handle,
        }
    }

    pub fn next(&self) -> Result<Messages, mpsc::RecvError> {
        self.rx.recv()
    }
}
