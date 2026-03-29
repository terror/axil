use super::*;

#[derive(Clap, Debug)]
#[clap(author)]
pub(crate) struct Arguments {
  file: PathBuf,
}

impl Arguments {
  pub(crate) fn run(self) -> Result {
    App::new(self.file)?.run()
  }
}
