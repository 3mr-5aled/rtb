use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::Frame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TabOutcome {
    Handled,
    Ignored,
    Quit,
}

#[allow(dead_code)]
pub trait TabController {
    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> TabOutcome;
    fn render(&mut self, frame: &mut Frame, area: Rect);
}
