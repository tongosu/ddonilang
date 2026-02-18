use clap::ValueEnum;

use crate::lang::parser::ParseMode;

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum LangModeArg {
    Compat,
    Strict,
}

impl LangModeArg {
    pub fn to_parse_mode(self) -> ParseMode {
        match self {
            LangModeArg::Compat => ParseMode::Compat,
            LangModeArg::Strict => ParseMode::Strict,
        }
    }
}
