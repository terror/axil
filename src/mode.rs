#[derive(Debug, Default, PartialEq)]
pub(crate) enum Mode {
  #[default]
  Normal,
  Query,
  Search,
}
