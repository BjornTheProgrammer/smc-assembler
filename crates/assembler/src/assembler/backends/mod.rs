use arbitrary_int::{u2, u4};
use clap::{ValueEnum, builder::PossibleValue};
use strum::VariantArray;
use strum_macros::VariantArray;
use thiserror::Error;

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

#[derive(Error, Debug, Clone)]
pub enum RegisterError {
    #[error("Invalid BatPU2 register: r{0} (valid: r0-r15)")]
    InvalidBatPU2(u8),
    #[error("Invalid TauAnalyzers register: r{0} (valid: r0-r3)")]
    InvalidTauAnalyzers(u8),
}

/// Backend-specific register types
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Register {
    /// BatPU2: 16 registers (r0-r15), 4-bit encoding
    BatPU2(u4),
    /// TauAnalyzers: 4 banked registers (r0-r3), 2-bit encoding
    /// (7 total via banking, but instruction only encodes 2 bits)
    TauAnalyzers(u2),
}

impl Register {
    pub fn batpu2(val: u8) -> Result<Self, RegisterError> {
        if val > 15 {
            Err(RegisterError::InvalidBatPU2(val))
        } else {
            Ok(Register::BatPU2(u4::new(val)))
        }
    }

    pub fn tau(val: u8) -> Result<Self, RegisterError> {
        if val > 3 {
            Err(RegisterError::InvalidTauAnalyzers(val))
        } else {
            Ok(Register::TauAnalyzers(u2::new(val)))
        }
    }

    /// Get raw value for encoding
    pub fn value(&self) -> u8 {
        match self {
            Register::BatPU2(r) => r.value(),
            Register::TauAnalyzers(r) => r.value(),
        }
    }
}
