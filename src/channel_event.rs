pub(crate) enum ChannelEvent {
  Crossterm(crossterm::event::Event),
  FileChanged,
}
