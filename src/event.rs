use super::*;

#[derive(Debug, PartialEq)]
pub(crate) enum Event {
  ClearSearch,
  Click { row: u16 },
  EnterQuery,
  EnterSearch,
  FileChanged,
  InputBackspace,
  InputCancel,
  InputChar(char),
  InputConfirm,
  JumpToMatch { forward: bool },
  MoveDown,
  MoveLeft,
  MoveRight,
  MoveToBottom,
  MoveToTop,
  MoveUp,
  Quit,
  ScrollDown,
  ScrollUp,
  ToggleCollapse,
  ToggleHelp,
  ToggleSelect,
  Yank,
}

impl Event {
  pub(crate) fn from_crossterm(
    event: &crossterm::event::Event,
    mode: &Mode,
  ) -> Option<Self> {
    match event {
      crossterm::event::Event::Key(key) => Self::from_key(key, mode),
      crossterm::event::Event::Mouse(mouse) => Self::from_mouse(*mouse),
      _ => None,
    }
  }

  fn from_input_key(event: &KeyEvent) -> Option<Self> {
    match event.code {
      KeyCode::Enter => Some(Self::InputConfirm),
      KeyCode::Esc => Some(Self::InputCancel),
      KeyCode::Backspace => Some(Self::InputBackspace),
      KeyCode::Char(c) => Some(Self::InputChar(c)),
      _ => None,
    }
  }

  fn from_key(event: &KeyEvent, mode: &Mode) -> Option<Self> {
    match mode {
      Mode::Normal => Self::from_normal_key(event),
      Mode::Search | Mode::Query => Self::from_input_key(event),
    }
  }

  fn from_mouse(event: crossterm::event::MouseEvent) -> Option<Self> {
    match event.kind {
      MouseEventKind::Down(MouseButton::Left) => {
        Some(Self::Click { row: event.row })
      }
      MouseEventKind::ScrollUp => Some(Self::ScrollUp),
      MouseEventKind::ScrollDown => Some(Self::ScrollDown),
      _ => None,
    }
  }

  fn from_normal_key(event: &KeyEvent) -> Option<Self> {
    match event {
      KeyEvent {
        code: KeyCode::Char('q'),
        ..
      } => Some(Self::Quit),
      KeyEvent {
        code: KeyCode::Char('k'),
        ..
      } => Some(Self::MoveUp),
      KeyEvent {
        code: KeyCode::Char('j'),
        ..
      } => Some(Self::MoveDown),
      KeyEvent {
        code: KeyCode::Char('h'),
        ..
      } => Some(Self::MoveLeft),
      KeyEvent {
        code: KeyCode::Char('l'),
        ..
      } => Some(Self::MoveRight),
      KeyEvent {
        code: KeyCode::Char(' '),
        ..
      } => Some(Self::ToggleSelect),
      KeyEvent {
        code: KeyCode::Enter,
        ..
      } => Some(Self::ToggleCollapse),
      KeyEvent {
        code: KeyCode::Char('u'),
        modifiers: KeyModifiers::CONTROL,
        ..
      } => Some(Self::ScrollUp),
      KeyEvent {
        code: KeyCode::Char('d'),
        modifiers: KeyModifiers::CONTROL,
        ..
      } => Some(Self::ScrollDown),
      KeyEvent {
        code: KeyCode::Char('/'),
        ..
      } => Some(Self::EnterSearch),
      KeyEvent {
        code: KeyCode::Char('n'),
        ..
      } => Some(Self::JumpToMatch { forward: true }),
      KeyEvent {
        code: KeyCode::Char('N'),
        ..
      } => Some(Self::JumpToMatch { forward: false }),
      KeyEvent {
        code: KeyCode::Char('?'),
        ..
      } => Some(Self::ToggleHelp),
      KeyEvent {
        code: KeyCode::Char('g'),
        ..
      } => Some(Self::MoveToTop),
      KeyEvent {
        code: KeyCode::Char('G'),
        ..
      } => Some(Self::MoveToBottom),
      KeyEvent {
        code: KeyCode::Char('y'),
        ..
      } => Some(Self::Yank),
      KeyEvent {
        code: KeyCode::Esc, ..
      } => Some(Self::ClearSearch),
      KeyEvent {
        code: KeyCode::Char(':'),
        ..
      } => Some(Self::EnterQuery),
      _ => None,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn char_differs_by_mode() {
    assert_eq!(
      Event::from_key(&key(KeyCode::Char('q')), &Mode::Normal),
      Some(Event::Quit),
    );

    assert_eq!(
      Event::from_key(&key(KeyCode::Char('q')), &Mode::Search),
      Some(Event::InputChar('q')),
    );
  }

  fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
  }

  #[test]
  fn from_crossterm_ignores_resize() {
    assert_eq!(
      Event::from_crossterm(
        &crossterm::event::Event::Resize(80, 24),
        &Mode::Normal,
      ),
      None,
    );
  }

  #[test]
  fn input_keys() {
    #[track_caller]
    fn case(event: KeyEvent, mode: Mode, expected: Event) {
      assert_eq!(Event::from_key(&event, &mode), Some(expected));
    }

    for mode in [Mode::Search, Mode::Query] {
      case(key(KeyCode::Enter), mode, Event::InputConfirm);
    }

    for mode in [Mode::Search, Mode::Query] {
      case(key(KeyCode::Esc), mode, Event::InputCancel);
    }

    for mode in [Mode::Search, Mode::Query] {
      case(key(KeyCode::Backspace), mode, Event::InputBackspace);
    }

    for mode in [Mode::Search, Mode::Query] {
      case(key(KeyCode::Char('a')), mode, Event::InputChar('a'));
    }
  }

  #[test]
  fn input_unbound_key() {
    assert_eq!(Event::from_key(&key(KeyCode::Tab), &Mode::Search), None,);
  }

  fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
  }

  fn mouse(kind: MouseEventKind, row: u16) -> crossterm::event::MouseEvent {
    crossterm::event::MouseEvent {
      kind,
      column: 0,
      row,
      modifiers: KeyModifiers::NONE,
    }
  }

  #[test]
  fn mouse_click() {
    assert_eq!(
      Event::from_mouse(mouse(MouseEventKind::Down(MouseButton::Left), 5)),
      Some(Event::Click { row: 5 }),
    );
  }

  #[test]
  fn mouse_scroll() {
    assert_eq!(
      Event::from_mouse(mouse(MouseEventKind::ScrollUp, 0)),
      Some(Event::ScrollUp),
    );

    assert_eq!(
      Event::from_mouse(mouse(MouseEventKind::ScrollDown, 0)),
      Some(Event::ScrollDown),
    );
  }

  #[test]
  fn mouse_unhandled() {
    assert_eq!(Event::from_mouse(mouse(MouseEventKind::Moved, 0)), None,);
  }

  #[test]
  fn normal_keys() {
    #[track_caller]
    fn case(event: KeyEvent, expected: Event) {
      assert_eq!(Event::from_key(&event, &Mode::Normal), Some(expected),);
    }

    case(
      key(KeyCode::Char('n')),
      Event::JumpToMatch { forward: true },
    );

    case(
      key(KeyCode::Char('N')),
      Event::JumpToMatch { forward: false },
    );

    case(ctrl('d'), Event::ScrollDown);
    case(ctrl('u'), Event::ScrollUp);
    case(key(KeyCode::Char(' ')), Event::ToggleSelect);
    case(key(KeyCode::Char('/')), Event::EnterSearch);
    case(key(KeyCode::Char('?')), Event::ToggleHelp);
    case(key(KeyCode::Char('G')), Event::MoveToBottom);
    case(key(KeyCode::Char('g')), Event::MoveToTop);
    case(key(KeyCode::Char('h')), Event::MoveLeft);
    case(key(KeyCode::Char('j')), Event::MoveDown);
    case(key(KeyCode::Char('k')), Event::MoveUp);
    case(key(KeyCode::Char('l')), Event::MoveRight);
    case(key(KeyCode::Char('q')), Event::Quit);
    case(key(KeyCode::Char('y')), Event::Yank);
    case(key(KeyCode::Enter), Event::ToggleCollapse);
    case(key(KeyCode::Char(':')), Event::EnterQuery);
    case(key(KeyCode::Esc), Event::ClearSearch);
  }

  #[test]
  fn normal_unbound_key() {
    assert_eq!(
      Event::from_key(&key(KeyCode::Char('z')), &Mode::Normal),
      None,
    );
  }
}
