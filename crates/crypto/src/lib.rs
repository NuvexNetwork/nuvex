#![forbid(unsafe_code)]
//! Cryptography boundary.
//!
//! Nuvex does not implement a VRF. The selected verifier is recorded in
//! ADR 0004. This crate only reports that choice.

#![no_std]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CryptoAvailability {
    /// No algorithm has been selected. Callers must not treat this as a failed proof.
    Unavailable { reason: &'static str },
    /// A named library verifies this construction. `audit` is `None` when no
    /// report was found. That is not a claim that the library was reviewed.
    Available {
        algorithm: &'static str,
        library: &'static str,
        version: &'static str,
        audit: Option<&'static str>,
    },
}

pub const VRF: CryptoAvailability = CryptoAvailability::Available {
    algorithm: "ECVRF-EDWARDS25519-SHA512-TAI",
    library: "solana-ecvrf",
    version: "0.0.1",
    audit: None,
};

#[cfg(test)]
mod tests {
    use super::{CryptoAvailability, VRF};

    #[test]
    fn vrf_names_the_selected_library_and_records_no_audit() {
        assert_eq!(
            VRF,
            CryptoAvailability::Available {
                algorithm: "ECVRF-EDWARDS25519-SHA512-TAI",
                library: "solana-ecvrf",
                version: "0.0.1",
                audit: None,
            }
        );
    }
}
