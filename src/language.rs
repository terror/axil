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

impl Display for Language {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::Go => write!(f, "go"),
      Self::Java => write!(f, "java"),
      Self::JavaScript => write!(f, "javascript"),
      Self::Json => write!(f, "json"),
      Self::Just => write!(f, "just"),
      Self::Rust => write!(f, "rust"),
      Self::Tsx => write!(f, "tsx"),
      Self::TypeScript => write!(f, "typescript"),
    }
  }
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

impl FromStr for Language {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    match s.to_lowercase().as_str() {
      "go" => Ok(Self::Go),
      "java" => Ok(Self::Java),
      "javascript" | "js" => Ok(Self::JavaScript),
      "json" => Ok(Self::Json),
      "just" => Ok(Self::Just),
      "rust" | "rs" => Ok(Self::Rust),
      "typescript" | "ts" => Ok(Self::TypeScript),
      "tsx" => Ok(Self::Tsx),
      _ => Err(anyhow!("unknown language `{s}`")),
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

    Err(anyhow!("failed to detect language for path"))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn case_insensitive_extension() {
    let path = PathBuf::from("main.RS");

    let result = Language::try_from(path);

    assert!(result.is_ok());

    assert_eq!(result.unwrap(), Language::Rust);
  }

  #[test]
  fn from_str() {
    #[track_caller]
    fn case(s: &str, expected: Language) {
      assert_eq!(s.parse::<Language>().unwrap(), expected);
    }

    case("go", Language::Go);
    case("java", Language::Java);
    case("javascript", Language::JavaScript);
    case("js", Language::JavaScript);
    case("json", Language::Json);
    case("just", Language::Just);
    case("rust", Language::Rust);
    case("rs", Language::Rust);
    case("typescript", Language::TypeScript);
    case("ts", Language::TypeScript);
    case("tsx", Language::Tsx);
  }

  #[test]
  fn from_str_case_insensitive() {
    assert_eq!("RUST".parse::<Language>().unwrap(), Language::Rust);
    assert_eq!("Go".parse::<Language>().unwrap(), Language::Go);
  }

  #[test]
  fn from_str_unknown() {
    assert!("foo".parse::<Language>().is_err());
  }

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

      assert!(result.is_ok(), "Failed to parse {path_str}");

      assert_eq!(
        result.unwrap(),
        expected_language,
        "Wrong language for {path_str}"
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
  fn no_extension() {
    let path = PathBuf::from("noextension");

    let result = Language::try_from(path);

    assert!(result.is_err());
  }

  #[test]
  fn unknown_extension() {
    let path = PathBuf::from("document.txt");

    let result = Language::try_from(path);

    assert!(result.is_err());
  }
}
