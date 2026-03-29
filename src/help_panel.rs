use super::*;

pub(crate) struct HelpPanel;

impl HelpPanel {
  const BINDINGS: &[(&str, &str)] = &[
    ("j", "Move to first child"),
    ("k", "Move to parent"),
    ("h", "Move to previous sibling"),
    ("l", "Move to next sibling"),
    ("g", "Move to top"),
    ("G", "Move to bottom"),
    ("Ctrl-u", "Scroll up"),
    ("Ctrl-d", "Scroll down"),
    ("Enter", "Toggle collapse"),
    ("Space", "Toggle select"),
    ("/", "Search"),
    ("n", "Next match"),
    ("N", "Previous match"),
    (":", "Tree-sitter query"),
    ("y", "Yank node text"),
    ("Esc", "Clear search"),
    ("?", "Toggle help"),
    ("q", "Quit"),
  ];
}

impl Widget for HelpPanel {
  #[allow(clippy::cast_possible_truncation)]
  fn render(self, area: Rect, buf: &mut Buffer) {
    let key_width = Self::BINDINGS
      .iter()
      .map(|(k, _)| k.len())
      .max()
      .unwrap_or(0);

    let content_width = Self::BINDINGS
      .iter()
      .map(|(k, d)| k.len() + 3 + d.len())
      .max()
      .unwrap_or(0)
      + 4;

    let content_height = Self::BINDINGS.len() + 2;

    let width = (content_width as u16).min(area.width);
    let height = (content_height as u16).min(area.height);

    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;

    let panel_area = Rect::new(x, y, width, height);

    for row in panel_area.y..panel_area.y + panel_area.height {
      for col in panel_area.x..panel_area.x + panel_area.width {
        buf[(col, row)].reset();
      }
    }

    let lines = Self::BINDINGS
      .iter()
      .map(|(key, desc)| {
        Line::from(vec![
          Span::styled(
            format!("{key:>key_width$}"),
            Style::default()
              .fg(Color::Yellow)
              .add_modifier(Modifier::BOLD),
          ),
          Span::styled(" - ", Style::default().fg(Color::DarkGray)),
          Span::styled(*desc, Style::default().fg(Color::White)),
        ])
      })
      .collect::<Vec<_>>();

    Paragraph::new(lines)
      .block(
        Block::default()
          .borders(Borders::ALL)
          .title("Help")
          .title_style(
            Style::default()
              .fg(Color::Cyan)
              .add_modifier(Modifier::BOLD),
          )
          .border_style(Style::default().fg(Color::DarkGray)),
      )
      .render(panel_area, buf);
  }
}
