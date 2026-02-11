use clap::ValueEnum;

use crate::core::geoul::TraceTier;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum TraceTierArg {
    #[value(name = "T-OFF")]
    TOff,
    #[value(name = "T-PATCH")]
    TPatch,
    #[value(name = "T-ALRIM")]
    TAlrim,
    #[value(name = "T-FULL")]
    TFull,
}

impl TraceTierArg {
    pub fn to_core(self) -> TraceTier {
        match self {
            TraceTierArg::TOff => TraceTier::Off,
            TraceTierArg::TPatch => TraceTier::Patch,
            TraceTierArg::TAlrim => TraceTier::Alrim,
            TraceTierArg::TFull => TraceTier::Full,
        }
    }
}
