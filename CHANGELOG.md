## [0.2.2](https://github.com/terror/axil/releases/tag/0.2.2) - 2026-03-29

### Added

- Add watch mode for interactive automatic file reload ([#12](https://github.com/terror/axil/pull/12) by [terror](https://github.com/terror))
- Add help panel with keybinding reference ([#11](https://github.com/terror/axil/pull/11) by [terror](https://github.com/terror))

### Misc

- Add integration test for multiple query patterns ([#10](https://github.com/terror/axil/pull/10) by [terror](https://github.com/terror))
- Add integration test for standard input combining `--language` and `--query` ([#9](https://github.com/terror/axil/pull/9) by [terror](https://github.com/terror))
- Cover status line prompt with unit tests ([#8](https://github.com/terror/axil/pull/8) by [terror](https://github.com/terror))
- Add integration test for unknown language ([#7](https://github.com/terror/axil/pull/7) by [terror](https://github.com/terror))
- Inline `clear_input` inside of `handle_event` ([#6](https://github.com/terror/axil/pull/6) by [terror](https://github.com/terror))
- Inline `yank` function inside `handle_event` ([#5](https://github.com/terror/axil/pull/5) by [terror](https://github.com/terror))
- Add brew installation to readme table ([#4](https://github.com/terror/axil/pull/4) by [terror](https://github.com/terror))
- Move status line into its own widget ([#3](https://github.com/terror/axil/pull/3) by [terror](https://github.com/terror))
- Lift out domain specific `Event` type ([#2](https://github.com/terror/axil/pull/2) by [terror](https://github.com/terror))
- Dispatch state updates per mode ([#1](https://github.com/terror/axil/pull/1) by [terror](https://github.com/terror))

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
