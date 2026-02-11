use clap::ValueEnum;

use crate::runtime::OpenMode;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum OpenModeArg {
    #[value(name = "deny")]
    Deny,
    #[value(name = "record")]
    Record,
    #[value(name = "replay")]
    Replay,
}

impl OpenModeArg {
    pub fn to_runtime(self) -> OpenMode {
        match self {
            OpenModeArg::Deny => OpenMode::Deny,
            OpenModeArg::Record => OpenMode::Record,
            OpenModeArg::Replay => OpenMode::Replay,
        }
    }
}
