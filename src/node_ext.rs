use super::*;

pub(crate) trait NodeExt {
  fn color(&self) -> Color;
}

impl NodeExt for Node<'_> {
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
