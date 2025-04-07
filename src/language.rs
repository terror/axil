use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Language {
  Go,
  Java,
  JavaScript,
  Json,
  Just,
  Rust,
  Tsx,
  TypeScript,
}

impl From<Language> for TreeSitterLanguage {
  fn from(language: Language) -> Self {
    match language {
      Language::Go => tree_sitter_go::LANGUAGE.into(),
      Language::Java => tree_sitter_java::LANGUAGE.into(),
      Language::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
      Language::Json => tree_sitter_json::LANGUAGE.into(),
      Language::Just => unsafe { tree_sitter_just() },
      Language::Rust => tree_sitter_rust::LANGUAGE.into(),
      Language::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
      Language::TypeScript => {
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
      }
    }
  }
}

impl TryFrom<PathBuf> for Language {
  type Error = Error;

  fn try_from(value: PathBuf) -> Result<Self> {
    if let Some(extension) = value.extension().and_then(|ext| ext.to_str()) {
      match extension.to_lowercase().as_str() {
        "go" => return Ok(Self::Go),
        "java" => return Ok(Self::Java),
        "js" => return Ok(Self::JavaScript),
        "json" => return Ok(Self::Json),
        "just" => return Ok(Self::Just),
        "rs" => return Ok(Self::Rust),
        "ts" => return Ok(Self::TypeScript),
        "tsx" => return Ok(Self::Tsx),
        _ => {}
      }
    }

    if let Some("justfile") = value.file_name().and_then(|name| name.to_str()) {
      return Ok(Self::Just);
    }

    Err(anyhow!("Failed to detect language for path"))
  }
}
