#![forbid(unsafe_code)]
//! The SDK still does not send transactions. VRF proving does, and a forged
//! proof is not a result.

use nuvex_crypto::{CryptoAvailability, VRF};
use nuvex_job_types::JobType;
use nuvex_proof_types::ProofKind;
use nuvex_sdk::{prove_vrf, submit_request, ClientConfig, SdkError};
use nuvex_vrf::{verify, write_vrf_alpha, ProveError, MAX_VRF_ALPHA_LEN};
use solana_pubkey::Pubkey;

#[test]
fn the_client_does_not_send_and_a_forged_vrf_proof_fails() {
    let program = Pubkey::new_from_array([3_u8; 32]);
    let config = ClientConfig::new(
        "localnet",
        "http://127.0.0.1:8899",
        program,
        program,
        program,
    );
    let error = submit_request(&config, JobType::AiInference).expect_err("closed");
    assert_eq!(error, SdkError::NotImplemented("submit_request"));
    assert!(matches!(
        VRF,
        CryptoAvailability::Available { audit: None, .. }
    ));
    assert!(ProofKind::Vrf.is_specified());

    let secret = [9u8; 32];
    let request = Pubkey::new_from_array([4u8; 32]);
    let proved = prove_vrf(&secret, &request, JobType::Vrf, b"dice").expect("prove");
    let mut alpha = [0u8; MAX_VRF_ALPHA_LEN];
    let len = write_vrf_alpha(
        &mut alpha,
        &request.to_bytes(),
        JobType::Vrf.as_u8(),
        b"dice",
    )
    .expect("alpha");
    let mut forged = proved.proof;
    forged[0] ^= 1;
    assert_eq!(
        verify(&proved.public_key, &alpha[..len], &forged),
        Err(ProveError::Library)
    );
}
