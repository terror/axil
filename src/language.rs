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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn language_from_extension() {
    let cases = vec![
      ("file.go", Language::Go),
      ("app.java", Language::Java),
      ("script.js", Language::JavaScript),
      ("config.json", Language::Json),
      ("build.just", Language::Just),
      ("lib.rs", Language::Rust),
      ("module.ts", Language::TypeScript),
      ("component.tsx", Language::Tsx),
    ];

    for (path_str, expected_language) in cases {
      let path = PathBuf::from(path_str);

      let result = Language::try_from(path);

      assert!(result.is_ok(), "Failed to parse {}", path_str);

      assert_eq!(
        result.unwrap(),
        expected_language,
        "Wrong language for {}",
        path_str
      );
    }
  }

  #[test]
  fn language_from_filename() {
    let path = PathBuf::from("justfile");

    let result = Language::try_from(path);
    assert!(result.is_ok());

    assert_eq!(result.unwrap(), Language::Just);
  }

  #[test]
  fn case_insensitive_extension() {
    let path = PathBuf::from("main.RS");

    let result = Language::try_from(path);

    assert!(result.is_ok());

    assert_eq!(result.unwrap(), Language::Rust);
  }

  #[test]
  fn unknown_extension() {
    let path = PathBuf::from("document.txt");

    let result = Language::try_from(path);

    assert!(result.is_err());
  }

  #[test]
  fn no_extension() {
    let path = PathBuf::from("noextension");

    let result = Language::try_from(path);

    assert!(result.is_err());
  }
}
