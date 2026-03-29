## [0.2.1](https://github.com/terror/axil/releases/tag/0.2.1) - 2026-03-28

### Added

- Add `--version` flag
- Add short flags for command-line arguments

## [0.2.0](https://github.com/terror/axil/releases/tag/0.2.0) - 2026-03-28

### Added

- Add `g`/`G` keybindings for jump-to-top/bottom navigation
- Add mouse support for click-to-navigate and scroll wheel
- Add support for `Python`, `C`, `C++`, `Ruby`, `Bash`, `HTML`, `CSS`, `TOML`, and `YAML`
- Add yank-to-clipboard with flash notification on 'y' keypress
- Add tree-sitter query support for all interaction modes
- Add interactive search with `/` to filter by node kind or text
- Add stdin support with `--language` flag
- Add tracebacks to error messages
- Print syntax tree by default
- Add tests for `State` navigation and actions
- Add `NodeExt` for node coloring

### Fixed

- Preserve newlines in info panel node text
- Return `Result` from node lookup instead of panicking

### Misc

- Add changelog file
- Use dark mode for readme badges
- Track code coverage metrics
- Expand readme documentation
- Scaffold integration test suite
- Enforce a stricter set of Clippy lints
- Extract navigation state from `App` into `State`
- Handle node position on `App` instead of `NodeHandle`
- Move widgets into their own modules
- Move event loop into `App`

## [0.1.0](https://github.com/terror/axil/releases/tag/0.1.0) - 2025-04-06

Initial release 🎉
