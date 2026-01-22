//! Helper functions for rendering UI components.

use std::{collections::VecDeque, time::Duration};

use crate::{Event, EventTextEntry, RecordError, RecordInput, TerminalKind};

/// Generate a one-line description of a binary file change.
pub fn make_binary_description(hash: &str, num_bytes: u64) -> String {
    format!("{hash} ({num_bytes} bytes)")
}

/// Reads input events from the terminal using `crossterm`.
///
/// Its default implementation of `edit_commit_message` returns the provided
/// message unchanged.
#[derive(Default)]
pub struct CrosstermInput {
    input_entry: bool,
}

impl RecordInput for CrosstermInput {
    fn terminal_kind(&self) -> TerminalKind {
        TerminalKind::Crossterm
    }

    fn next_events(&mut self) -> Result<Vec<Event>, RecordError> {
        // Ensure we block for at least one event.
        let first_event =
            crossterm::event::read().map_err(|err| RecordError::ReadInput(err.into()))?;
        let mut events = vec![self.parse_event(first_event)];
        // Some events, like scrolling, are generated more quickly than
        // we can render the UI. In those cases, batch up all available
        // events and process them before the next render.
        while crossterm::event::poll(Duration::ZERO)
            .map_err(|err| RecordError::ReadInput(err.into()))?
        {
            let event =
                crossterm::event::read().map_err(|err| RecordError::ReadInput(err.into()))?;
            events.push(event.into());
        }
        Ok(events)
    }

    fn edit_commit_message(&mut self, message: &str) -> Result<String, RecordError> {
        Ok(message.to_owned())
    }

    fn enable_input_entry(&mut self) {
        self.input_entry = true;
    }

    fn disable_input_entry(&mut self) {
        self.input_entry = false;
    }
}

impl CrosstermInput {
    fn parse_event(&self, event: crossterm::event::Event) -> Event {
        if self.input_entry {
            if let crossterm::event::Event::Key(crossterm::event::KeyEvent {
                code,
                modifiers:
                    crossterm::event::KeyModifiers::NONE | crossterm::event::KeyModifiers::SHIFT,
                kind: _,
                state: _,
            }) = event
            {
                if let Some(c) = code.as_char() {
                    return Event::TextEntry(EventTextEntry::Char(c));
                } else if code.is_backspace() {
                    return Event::TextEntry(EventTextEntry::Backspace);
                }
            }
        }
        event.into()
    }
}

/// Reads events from the provided sequence of events.
pub struct TestingInput {
    /// The width of the virtual terminal in columns.
    pub width: usize,

    /// The height of the virtual terminal in columns.
    pub height: usize,

    /// The sequence of events to emit.
    pub events: Box<dyn Iterator<Item = Event>>,

    /// Commit messages to use when the commit editor is opened.
    pub commit_messages: VecDeque<String>,
}

impl TestingInput {
    /// Helper function to construct a `TestingInput`.
    pub fn new(
        width: usize,
        height: usize,
        events: impl IntoIterator<Item = Event> + 'static,
    ) -> Self {
        Self {
            width,
            height,
            events: Box::new(events.into_iter()),
            commit_messages: Default::default(),
        }
    }
}

impl RecordInput for TestingInput {
    fn terminal_kind(&self) -> TerminalKind {
        let Self {
            width,
            height,
            events: _,
            commit_messages: _,
        } = self;
        TerminalKind::Testing {
            width: *width,
            height: *height,
        }
    }

    fn next_events(&mut self) -> Result<Vec<Event>, RecordError> {
        Ok(vec![self.events.next().unwrap_or(Event::None)])
    }

    fn edit_commit_message(&mut self, _message: &str) -> Result<String, RecordError> {
        self.commit_messages
            .pop_front()
            .ok_or_else(|| RecordError::Other("No more commit messages available".to_string()))
    }

    fn enable_input_entry(&mut self) {}

    fn disable_input_entry(&mut self) {}
}
