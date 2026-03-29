use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Language {
  Bash,
  C,
  Cpp,
  Css,
  Go,
  Html,
  Java,
  JavaScript,
  Json,
  Just,
  Python,
  Ruby,
  Rust,
  Toml,
  Tsx,
  TypeScript,
  Yaml,
}

impl Display for Language {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::Bash => write!(f, "bash"),
      Self::C => write!(f, "c"),
      Self::Cpp => write!(f, "cpp"),
      Self::Css => write!(f, "css"),
      Self::Go => write!(f, "go"),
      Self::Html => write!(f, "html"),
      Self::Java => write!(f, "java"),
      Self::JavaScript => write!(f, "javascript"),
      Self::Json => write!(f, "json"),
      Self::Just => write!(f, "just"),
      Self::Python => write!(f, "python"),
      Self::Ruby => write!(f, "ruby"),
      Self::Rust => write!(f, "rust"),
      Self::Toml => write!(f, "toml"),
      Self::Tsx => write!(f, "tsx"),
      Self::TypeScript => write!(f, "typescript"),
      Self::Yaml => write!(f, "yaml"),
    }
  }
}

impl From<Language> for TreeSitterLanguage {
  fn from(language: Language) -> Self {
    match language {
      Language::Bash => tree_sitter_bash::LANGUAGE.into(),
      Language::C => tree_sitter_c::LANGUAGE.into(),
      Language::Cpp => tree_sitter_cpp::LANGUAGE.into(),
      Language::Css => tree_sitter_css::LANGUAGE.into(),
      Language::Go => tree_sitter_go::LANGUAGE.into(),
      Language::Html => tree_sitter_html::LANGUAGE.into(),
      Language::Java => tree_sitter_java::LANGUAGE.into(),
      Language::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
      Language::Json => tree_sitter_json::LANGUAGE.into(),
      Language::Just => unsafe { tree_sitter_just() },
      Language::Python => tree_sitter_python::LANGUAGE.into(),
      Language::Ruby => tree_sitter_ruby::LANGUAGE.into(),
      Language::Rust => tree_sitter_rust::LANGUAGE.into(),
      Language::Toml => tree_sitter_toml_ng::LANGUAGE.into(),
      Language::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
      Language::TypeScript => {
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
      }
      Language::Yaml => tree_sitter_yaml::LANGUAGE.into(),
    }
  }
}

impl FromStr for Language {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    match s.to_lowercase().as_str() {
      "bash" | "sh" => Ok(Self::Bash),
      "c" => Ok(Self::C),
      "cpp" | "c++" => Ok(Self::Cpp),
      "css" => Ok(Self::Css),
      "go" => Ok(Self::Go),
      "html" => Ok(Self::Html),
      "java" => Ok(Self::Java),
      "javascript" | "js" => Ok(Self::JavaScript),
      "json" => Ok(Self::Json),
      "just" => Ok(Self::Just),
      "python" | "py" => Ok(Self::Python),
      "ruby" | "rb" => Ok(Self::Ruby),
      "rust" | "rs" => Ok(Self::Rust),
      "toml" => Ok(Self::Toml),
      "typescript" | "ts" => Ok(Self::TypeScript),
      "tsx" => Ok(Self::Tsx),
      "yaml" | "yml" => Ok(Self::Yaml),
      _ => Err(anyhow!("unknown language `{s}`")),
    }
  }
}

impl TryFrom<PathBuf> for Language {
  type Error = Error;

  fn try_from(value: PathBuf) -> Result<Self> {
    if let Some(extension) = value.extension().and_then(|ext| ext.to_str()) {
      match extension.to_lowercase().as_str() {
        "bash" | "sh" => return Ok(Self::Bash),
        "c" | "h" => return Ok(Self::C),
        "cc" | "cpp" | "cxx" | "hpp" | "hxx" => return Ok(Self::Cpp),
        "css" => return Ok(Self::Css),
        "go" => return Ok(Self::Go),
        "html" | "htm" => return Ok(Self::Html),
        "java" => return Ok(Self::Java),
        "js" => return Ok(Self::JavaScript),
        "json" => return Ok(Self::Json),
        "just" => return Ok(Self::Just),
        "py" => return Ok(Self::Python),
        "rb" => return Ok(Self::Ruby),
        "rs" => return Ok(Self::Rust),
        "toml" => return Ok(Self::Toml),
        "ts" => return Ok(Self::TypeScript),
        "tsx" => return Ok(Self::Tsx),
        "yaml" | "yml" => return Ok(Self::Yaml),
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

    case("bash", Language::Bash);
    case("sh", Language::Bash);
    case("c", Language::C);
    case("cpp", Language::Cpp);
    case("c++", Language::Cpp);
    case("css", Language::Css);
    case("go", Language::Go);
    case("html", Language::Html);
    case("java", Language::Java);
    case("javascript", Language::JavaScript);
    case("js", Language::JavaScript);
    case("json", Language::Json);
    case("just", Language::Just);
    case("python", Language::Python);
    case("py", Language::Python);
    case("ruby", Language::Ruby);
    case("rb", Language::Ruby);
    case("rust", Language::Rust);
    case("rs", Language::Rust);
    case("toml", Language::Toml);
    case("typescript", Language::TypeScript);
    case("ts", Language::TypeScript);
    case("tsx", Language::Tsx);
    case("yaml", Language::Yaml);
    case("yml", Language::Yaml);
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
      ("foo.bash", Language::Bash),
      ("foo.sh", Language::Bash),
      ("foo.c", Language::C),
      ("foo.h", Language::C),
      ("foo.cpp", Language::Cpp),
      ("foo.cc", Language::Cpp),
      ("foo.cxx", Language::Cpp),
      ("foo.hpp", Language::Cpp),
      ("foo.hxx", Language::Cpp),
      ("foo.css", Language::Css),
      ("foo.go", Language::Go),
      ("foo.html", Language::Html),
      ("foo.htm", Language::Html),
      ("foo.java", Language::Java),
      ("foo.js", Language::JavaScript),
      ("foo.json", Language::Json),
      ("foo.just", Language::Just),
      ("foo.py", Language::Python),
      ("foo.rb", Language::Ruby),
      ("foo.rs", Language::Rust),
      ("foo.toml", Language::Toml),
      ("foo.ts", Language::TypeScript),
      ("foo.tsx", Language::Tsx),
      ("foo.yaml", Language::Yaml),
      ("foo.yml", Language::Yaml),
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
