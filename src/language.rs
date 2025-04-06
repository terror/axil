use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Language {
  Just,
}

impl From<Language> for TreeSitterLanguage {
  fn from(language: Language) -> Self {
    match language {
      Language::Just => unsafe { tree_sitter_just() },
    }
  }
}

impl TryFrom<PathBuf> for Language {
  type Error = Error;

  fn try_from(value: PathBuf) -> Result<Self> {
    if let Some(extension) = value.extension() {
      match extension.to_str() {
        Some("just") => return Ok(Self::Just),
        _ => {}
      }
    }

    if let Some(filename) = value.to_str() {
      match filename {
        "justfile" => return Ok(Self::Just),
        _ => {}
      }
    }

    Err(anyhow!("Failed to detect language for path"))
  }
}
