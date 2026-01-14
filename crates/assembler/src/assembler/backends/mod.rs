use clap::{ValueEnum, builder::PossibleValue};
use strum::VariantArray;
use strum_macros::VariantArray;

#[derive(Debug, Clone, PartialEq, VariantArray)]
pub enum Backend {
    BatPU2,
    TauAnalyzersNone,
}

impl ValueEnum for Backend {
    fn value_variants<'a>() -> &'a [Self] {
        Backend::VARIANTS
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(PossibleValue::new(self.to_str()))
    }
}

impl Backend {
    pub fn to_str(&self) -> &'static str {
        match self {
            Backend::BatPU2 => "batpu2-mattbatwings-none",
            Backend::TauAnalyzersNone => "tau-analyzers-none",
        }
    }
}
