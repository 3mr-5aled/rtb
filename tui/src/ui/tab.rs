use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::Frame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabOutcome {
    Handled,
    Ignored,
    Quit,
}

pub trait TabController {
    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> TabOutcome;
    fn render(&mut self, frame: &mut Frame, area: Rect);
}
