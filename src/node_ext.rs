use super::*;

pub(crate) trait NodeExt {
  fn child_count_u32(&self) -> u32;
  fn color(&self) -> Color;
}

impl NodeExt for Node<'_> {
  #[allow(clippy::cast_possible_truncation)]
  fn child_count_u32(&self) -> u32 {
    self.child_count() as u32
  }

  fn color(&self) -> Color {
    match self.kind() {
      "assignment" => Color::Magenta,
      "comment" => Color::DarkGray,
      "function" => Color::Red,
      "identifier" | "parameter" => Color::Yellow,
      "number" => Color::Blue,
      "source_file" | "argument" => Color::Cyan,
      "string" => Color::Green,
      _ => Color::White,
    }
  }
}
