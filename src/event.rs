use crate::app::Exam;
use crate::app::{AppRoute, OpenMode, SelectionFlags};

use std::sync::mpsc;
use std::thread;

use crossterm::event::{read, Event, KeyEvent};

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
    Input(KeyEvent),
    Resize,
    ChangeRoute(AppRoute),
    UpdateQuestionIndex(UpdateQuestionIndexEvent),
    ScrollQuestion(u16),
    UpdateHomeSelected(UpdateHomeSelectedEvent),
    UpdateJumpboxValue(u16),
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
    _input_handle: thread::JoinHandle<()>,
}

impl Events {
    pub fn new() -> Events {
        let (tx, rx) = mpsc::channel();
        let _input_handle = {
            let tx = tx.clone();
            thread::spawn(move || loop {
                if let Ok(event) = read() {
                    match event {
                        Event::Key(keyevent) => {
                            if let Err(_) = tx.send(Messages::Input(keyevent)) {
                                return;
                            }
                        }
                        Event::Resize(_, _) => {
                            if let Err(_) = tx.send(Messages::Resize) {
                                return;
                            }
                        }
                        _ => {}
                    }
                }
            })
        };
        Events {
            tx,
            rx,
            _input_handle,
        }
    }

    pub fn next(&self) -> Result<Messages, mpsc::RecvError> {
        self.rx.recv()
    }
}
