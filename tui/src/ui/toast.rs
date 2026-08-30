use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl ToastLevel {
    pub fn title(&self) -> &'static str {
        match self {
            ToastLevel::Info => "ℹ Info",
            ToastLevel::Success => "✓ Success",
            ToastLevel::Warning => "⚠ Warning",
            ToastLevel::Error => "✗ Error",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            ToastLevel::Info => Color::Cyan,
            ToastLevel::Success => Color::Green,
            ToastLevel::Warning => Color::Yellow,
            ToastLevel::Error => Color::Red,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToastMessage {
    pub message: String,
    pub level: ToastLevel,
    pub created_at: Instant,
    pub duration: Duration,
}

impl ToastMessage {
    pub fn new(message: impl Into<String>, level: ToastLevel, duration: Duration) -> Self {
        Self {
            message: message.into(),
            level,
            created_at: Instant::now(),
            duration,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }
}

#[derive(Debug, Default, Clone)]
pub struct ToastQueue {
    pub toasts: Vec<ToastMessage>,
}

impl ToastQueue {
    pub fn new() -> Self {
        Self { toasts: Vec::new() }
    }

    pub fn push(&mut self, message: impl Into<String>, level: ToastLevel, duration: Duration) {
        self.toasts.push(ToastMessage::new(message, level, duration));
    }

    pub fn cleanup_expired(&mut self) {
        self.toasts.retain(|t| !t.is_expired());
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.toasts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.toasts.is_empty()
    }
}

pub fn draw(f: &mut Frame, toast_queue: &ToastQueue, area: Rect) {
    if toast_queue.is_empty() {
        return;
    }

    let max_toasts = 4;
    let visible_toasts: Vec<&ToastMessage> = toast_queue
        .toasts
        .iter()
        .filter(|t| !t.is_expired())
        .rev()
        .take(max_toasts)
        .collect();

    if visible_toasts.is_empty() {
        return;
    }

    let toast_width = 45u16.min(area.width.saturating_sub(4));
    let toast_height = 3u16;

    for (i, toast) in visible_toasts.iter().enumerate() {
        let top_offset = area.y + 1 + (i as u16 * (toast_height + 1));
        if top_offset + toast_height >= area.height {
            break;
        }

        let left_offset = area.x + area.width.saturating_sub(toast_width + 2);
        let toast_area = Rect::new(left_offset, top_offset, toast_width, toast_height);

        f.render_widget(Clear, toast_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", toast.level.title()))
            .title_style(Style::default().fg(toast.level.color()).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(toast.level.color()));

        let text = Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled(&toast.message, Style::default().fg(Color::White)),
        ]);

        let para = Paragraph::new(text).block(block);
        f.render_widget(para, toast_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_toast_queue_expiration() {
        let mut queue = ToastQueue::new();
        queue.push("Short toast", ToastLevel::Info, Duration::from_millis(50));
        queue.push("Long toast", ToastLevel::Success, Duration::from_secs(10));

        assert_eq!(queue.len(), 2);
        assert!(!queue.toasts[0].is_expired());

        thread::sleep(Duration::from_millis(70));
        queue.cleanup_expired();

        assert_eq!(queue.len(), 1);
        assert_eq!(queue.toasts[0].message, "Long toast");
    }
}
