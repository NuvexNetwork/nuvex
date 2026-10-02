#![forbid(unsafe_code)]
//! Proof vocabulary.
//!
//! `Vrf` is the RFC 9381 encoding named in ADR 0004. Byte lengths live in
//! `nuvex-vrf` so this crate does not depend on the verifier.

#![no_std]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProofKind {
    Vrf,
}

impl ProofKind {
    pub const fn is_specified(self) -> bool {
        match self {
            Self::Vrf => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProofKind;

    #[test]
    fn vrf_proof_format_is_specified() {
        assert!(ProofKind::Vrf.is_specified());
    }
}
