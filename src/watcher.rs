use super::*;

pub(crate) struct Watcher {
  _inner: notify::RecommendedWatcher,
}

impl Watcher {
  pub(crate) fn new(path: &Path, tx: &Sender<ChannelEvent>) -> Result<Self> {
    let watcher_tx = tx.clone();

    let mut inner = notify::recommended_watcher(
      move |res: Result<notify::Event, notify::Error>| {
        if let Ok(event) = res {
          if event.kind.is_modify() {
            let _ = watcher_tx.send(ChannelEvent::FileChanged);
          }
        }
      },
    )?;

    notify::Watcher::watch(
      &mut inner,
      path,
      notify::RecursiveMode::NonRecursive,
    )?;

    Ok(Self { _inner: inner })
  }
}
