use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    BDT,
    SGD,
    USDC,
}

impl Currency {
    /// Returns the decimal exponent (precision) for internal representation.
    pub fn precision(&self) -> u32 {
        match self {
            Currency::USD | Currency::EUR | Currency::GBP | Currency::BDT | Currency::SGD => 2,
            Currency::USDC => 6,
        }
    }

    /// Multiplier to convert whole units into atomic integer units.
    pub fn atomic_multiplier(&self) -> u128 {
        10u128.pow(self.precision())
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
