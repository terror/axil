use {
  executable_path::executable_path,
  pretty_assertions::assert_eq,
  std::{
    fs,
    io::Write,
    iter::once,
    process::{Command, Stdio},
    str,
  },
  unindent::Unindent,
  Expected::*,
};

enum Expected {
  Contains(String),
  Exact(String),
}

struct Test<'a> {
  arguments: Vec<String>,
  expected_status: i32,
  expected_stderr: Expected,
  expected_stdout: String,
  files: Vec<(&'a str, &'a str)>,
  stdin: Option<&'a str>,
  tempdir: tempfile::TempDir,
}

impl<'a> Test<'a> {
  fn argument(self, argument: &str) -> Self {
    Self {
      arguments: self
        .arguments
        .into_iter()
        .chain(once(argument.to_string()))
        .collect(),
      ..self
    }
  }

  fn command(&self) -> Command {
    let mut command = Command::new(executable_path("axil"));

    command
      .env("NO_COLOR", "1")
      .env("RUST_BACKTRACE", "0")
      .current_dir(self.tempdir.path());

    for argument in &self.arguments {
      command.arg(argument);
    }

    command
  }

  fn expected_status(self, expected_status: i32) -> Self {
    Self {
      expected_status,
      ..self
    }
  }

  fn expected_stderr(self, expected: Expected) -> Self {
    Self {
      expected_stderr: expected,
      ..self
    }
  }

  fn expected_stdout(self, stdout: &str) -> Self {
    Self {
      expected_stdout: stdout.unindent(),
      ..self
    }
  }

  fn file(self, path: &'a str, content: &'a str) -> Self {
    Self {
      files: self
        .files
        .into_iter()
        .chain(std::iter::once((path, content)))
        .collect(),
      ..self
    }
  }

  fn new() -> Self {
    Self {
      arguments: Vec::new(),
      expected_status: 0,
      expected_stderr: Exact(String::new()),
      expected_stdout: String::new(),
      files: Vec::new(),
      stdin: None,
      tempdir: tempfile::tempdir().unwrap(),
    }
  }

  fn run(self) {
    for (path, content) in &self.files {
      let path = self.tempdir.path().join(path);

      if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
      }

      fs::write(&path, content).unwrap();
    }

    let mut command = self.command();

    if self.stdin.is_some() {
      command.stdin(Stdio::piped());
    }

    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let child = command.spawn().unwrap();

    if let Some(stdin_content) = self.stdin {
      let mut stdin = child.stdin.as_ref().unwrap();
      stdin.write_all(stdin_content.as_bytes()).unwrap();
    }

    let output = child.wait_with_output().unwrap();

    let stdout = str::from_utf8(&output.stdout).unwrap().to_string();
    let stderr = str::from_utf8(&output.stderr).unwrap().to_string();

    let status = output.status.code().unwrap();

    assert_eq!(status, self.expected_status, "unexpected exit status");

    match &self.expected_stderr {
      Exact(expected) => assert_eq!(stderr, *expected, "unexpected stderr"),
      Contains(needle) => assert!(
        stderr.contains(needle.as_str()),
        "stderr does not contain expected substring\nstderr: {stderr}\nexpected substring: {needle}",
      ),
    }

    assert_eq!(stdout, self.expected_stdout, "unexpected stdout");
  }

  fn stdin(self, stdin: &'a str) -> Self {
    Self {
      stdin: Some(stdin),
      ..self
    }
  }
}

#[test]
fn language_flag_override() {
  Test::new()
    .file("foo.txt", "fn bar() {}")
    .argument("foo.txt")
    .argument("--language")
    .argument("rust")
    .expected_stdout(
      "
      source_file [0:0..0:11]
        function_item [0:0..0:11]
          fn [0:0..0:2] \"fn\"
          identifier [0:3..0:6] \"bar\"
          parameters [0:6..0:8]
            ( [0:6..0:7] \"(\"
            ) [0:7..0:8] \")\"
          block [0:9..0:11]
            { [0:9..0:10] \"{\"
            } [0:10..0:11] \"}\"
      ",
    )
    .run();
}

#[test]
fn missing_file_is_error() {
  Test::new()
    .argument("nonexistent.rs")
    .expected_status(1)
    .expected_stderr(Contains("os error 2".into()))
    .run();
}

#[test]
fn parse_python_file() {
  Test::new()
    .file("foo.py", "x = 1")
    .argument("foo.py")
    .expected_stdout(
      "
      module [0:0..0:5]
        expression_statement [0:0..0:5]
          assignment [0:0..0:5]
            identifier [0:0..0:1] \"x\"
            = [0:2..0:3] \"=\"
            integer [0:4..0:5] \"1\"
      ",
    )
    .run();
}

#[test]
fn parse_rust_file() {
  Test::new()
    .file("foo.rs", "fn bar() {}")
    .argument("foo.rs")
    .expected_stdout(
      "
      source_file [0:0..0:11]
        function_item [0:0..0:11]
          fn [0:0..0:2] \"fn\"
          identifier [0:3..0:6] \"bar\"
          parameters [0:6..0:8]
            ( [0:6..0:7] \"(\"
            ) [0:7..0:8] \")\"
          block [0:9..0:11]
            { [0:9..0:10] \"{\"
            } [0:10..0:11] \"}\"
      ",
    )
    .run();
}

#[test]
fn query_filters_output() {
  Test::new()
    .file("foo.rs", "fn bar() {}\nfn baz() {}")
    .argument("foo.rs")
    .argument("--query")
    .argument("(identifier) @name")
    .expected_stdout(
      "
      source_file [0:0..1:11]
        function_item [0:0..0:11]
          identifier [0:3..0:6] \"bar\"
        function_item [1:0..1:11]
          identifier [1:3..1:6] \"baz\"
      ",
    )
    .run();
}

#[test]
fn query_filters_output_multiple_patterns() {
  Test::new()
    .file("foo.rs", "fn bar() {}")
    .argument("foo.rs")
    .argument("--query")
    .argument("(identifier) @name (parameters) @params")
    .expected_stdout(
      "
      source_file [0:0..0:11]
        function_item [0:0..0:11]
          identifier [0:3..0:6] \"bar\"
          parameters [0:6..0:8]
      ",
    )
    .run();
}

#[test]
fn stdin_with_language() {
  Test::new()
    .stdin("fn bar() {}")
    .argument("--language")
    .argument("rust")
    .expected_stdout(
      "
      source_file [0:0..0:11]
        function_item [0:0..0:11]
          fn [0:0..0:2] \"fn\"
          identifier [0:3..0:6] \"bar\"
          parameters [0:6..0:8]
            ( [0:6..0:7] \"(\"
            ) [0:7..0:8] \")\"
          block [0:9..0:11]
            { [0:9..0:10] \"{\"
            } [0:10..0:11] \"}\"
      ",
    )
    .run();
}

#[test]
fn stdin_with_language_and_query() {
  Test::new()
    .stdin("fn bar() {}\nfn baz() {}")
    .argument("--language")
    .argument("rust")
    .argument("--query")
    .argument("(identifier) @name")
    .expected_stdout(
      "
      source_file [0:0..1:11]
        function_item [0:0..0:11]
          identifier [0:3..0:6] \"bar\"
        function_item [1:0..1:11]
          identifier [1:3..1:6] \"baz\"
      ",
    )
    .run();
}

#[test]
fn stdin_without_language_is_error() {
  Test::new()
    .stdin("foo")
    .expected_status(1)
    .expected_stderr(Exact(
      "error: `--language` is required when reading from stdin\n".into(),
    ))
    .run();
}

#[test]
fn unknown_extension_is_error() {
  Test::new()
    .file("foo.xyz", "bar")
    .argument("foo.xyz")
    .expected_status(1)
    .expected_stderr(Exact(
      "error: failed to detect language for path\n".into(),
    ))
    .run();
}

#[test]
fn unknown_language_is_error() {
  Test::new()
    .argument("--language")
    .argument("foo")
    .expected_status(2)
    .expected_stderr(Contains(
      "invalid value 'foo' for '--language <LANGUAGE>': unknown language `foo`"
        .into(),
    ))
    .run();
}
