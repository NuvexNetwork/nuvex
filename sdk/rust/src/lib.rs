#![forbid(unsafe_code)]
//! Rust SDK.
//!
//! PDA derivation and host VRF proving are implemented. This client does not
//! talk to a cluster, so `submit_request` still returns `NotImplemented`.

use nuvex_common::{
    CHALLENGE_SEED, JOB_CONFIG_SEED, NODE_SEED, PROTOCOL_SEED, REGISTRY_SEED, REQUEST_ID_LEN,
    REQUEST_SEED, REWARD_VAULT_SEED, VERIFICATION_SEED,
};
use nuvex_job_types::JobType;
use nuvex_vrf::{prove, public_key, write_vrf_alpha, ProveError, MAX_VRF_ALPHA_LEN};
use solana_pubkey::Pubkey;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SdkError {
    #[error("missing environment variable {0}")]
    MissingEnv(&'static str),
    #[error("invalid program id in {0}")]
    InvalidProgramId(&'static str),
    #[error("{0} is not implemented")]
    NotImplemented(&'static str),
    #[error("invalid PDA seed: {0}")]
    InvalidSeed(&'static str),
    #[error("VRF alpha does not fit the request binding")]
    Alpha,
    #[error("the VRF library rejected its own proof")]
    Prove,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientConfig {
    pub cluster: String,
    pub rpc_url: String,
    pub oracle_core_program_id: Pubkey,
    pub oracle_registry_program_id: Pubkey,
    pub verification_program_id: Pubkey,
}

impl ClientConfig {
    /// Loads configuration from explicit values. Callers read the environment
    /// themselves so this function does not hide a default RPC URL.
    pub fn new(
        cluster: impl Into<String>,
        rpc_url: impl Into<String>,
        oracle_core_program_id: Pubkey,
        oracle_registry_program_id: Pubkey,
        verification_program_id: Pubkey,
    ) -> Self {
        Self {
            cluster: cluster.into(),
            rpc_url: rpc_url.into(),
            oracle_core_program_id,
            oracle_registry_program_id,
            verification_program_id,
        }
    }
}

pub fn parse_pubkey(value: &str, env_name: &'static str) -> Result<Pubkey, SdkError> {
    value
        .parse()
        .map_err(|_| SdkError::InvalidProgramId(env_name))
}

pub fn protocol_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PROTOCOL_SEED], program_id)
}

pub fn registry_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[REGISTRY_SEED], program_id)
}

pub fn node_pda(program_id: &Pubkey, node_authority: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[NODE_SEED, node_authority.as_ref()], program_id)
}

pub fn request_pda(
    program_id: &Pubkey,
    requester: &Pubkey,
    request_id: &[u8],
) -> Result<(Pubkey, u8), SdkError> {
    if request_id.len() != REQUEST_ID_LEN {
        return Err(SdkError::InvalidSeed("request id must be 32 bytes"));
    }
    Ok(Pubkey::find_program_address(
        &[REQUEST_SEED, requester.as_ref(), request_id],
        program_id,
    ))
}

pub fn verification_pda(program_id: &Pubkey, request: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[VERIFICATION_SEED, request.as_ref()], program_id)
}

pub fn challenge_pda(program_id: &Pubkey, request: &Pubkey, challenger: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[CHALLENGE_SEED, request.as_ref(), challenger.as_ref()],
        program_id,
    )
}

pub fn reward_vault_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[REWARD_VAULT_SEED], program_id)
}

pub fn job_config_pda(program_id: &Pubkey, job: JobType) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[JOB_CONFIG_SEED, &[job.as_u8()]], program_id)
}

/// Request creation is not sent by this client. The program instruction exists.
pub fn submit_request(_config: &ClientConfig, _job: JobType) -> Result<(), SdkError> {
    Err(SdkError::NotImplemented("submit_request"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VrfProof {
    pub public_key: [u8; 32],
    pub output: [u8; 64],
    pub proof: [u8; 80],
}

/// Proves a VRF output bound to `request`. The secret never leaves the caller.
pub fn prove_vrf(
    secret: &[u8; 32],
    request: &Pubkey,
    job: JobType,
    input: &[u8],
) -> Result<VrfProof, SdkError> {
    let mut alpha = [0u8; MAX_VRF_ALPHA_LEN];
    let len = write_vrf_alpha(&mut alpha, &request.to_bytes(), job.as_u8(), input)
        .map_err(|_| SdkError::Alpha)?;
    let (output, proof) = prove(secret, &alpha[..len]).map_err(|error| match error {
        ProveError::Library => SdkError::Prove,
    })?;
    Ok(VrfProof {
        public_key: public_key(secret),
        output,
        proof,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        job_config_pda, protocol_pda, prove_vrf, request_pda, submit_request, ClientConfig,
        SdkError,
    };
    use nuvex_job_types::JobType;
    use solana_pubkey::Pubkey;

    fn program() -> Pubkey {
        Pubkey::new_from_array([7_u8; 32])
    }

    #[test]
    fn protocol_pda_is_deterministic() {
        let program_id = program();
        assert_eq!(protocol_pda(&program_id), protocol_pda(&program_id));
        assert_ne!(protocol_pda(&program_id).0, program_id);
    }

    #[test]
    fn job_pdas_differ_by_job_type() {
        let program_id = program();
        assert_ne!(
            job_config_pda(&program_id, JobType::Vrf).0,
            job_config_pda(&program_id, JobType::Price).0
        );
    }

    #[test]
    fn request_pda_binds_the_requester_and_rejects_short_ids() {
        let program_id = program();
        let requester = Pubkey::new_from_array([9_u8; 32]);
        let other = Pubkey::new_from_array([8_u8; 32]);
        let id = [1_u8; 32];
        let left = request_pda(&program_id, &requester, &id).expect("32 byte id");
        let right = request_pda(&program_id, &other, &id).expect("32 byte id");
        assert_ne!(left.0, right.0);
        let error = request_pda(&program_id, &requester, &[1_u8; 31]).expect_err("short id");
        assert_eq!(error, SdkError::InvalidSeed("request id must be 32 bytes"));
    }

    #[test]
    fn submit_request_is_not_implemented() {
        let config = ClientConfig::new(
            "localnet",
            "http://127.0.0.1:8899",
            program(),
            program(),
            program(),
        );
        let error = submit_request(&config, JobType::Vrf).expect_err("unimplemented");
        assert_eq!(error, SdkError::NotImplemented("submit_request"));
    }

    #[test]
    fn prove_vrf_rejects_a_tampered_proof() {
        use nuvex_vrf::{verify, write_vrf_alpha, ProveError, MAX_VRF_ALPHA_LEN};

        let secret = [5u8; 32];
        let request = Pubkey::new_from_array([6u8; 32]);
        let proved = prove_vrf(&secret, &request, JobType::Vrf, b"dice").expect("prove");
        let mut alpha = [0u8; MAX_VRF_ALPHA_LEN];
        let len = write_vrf_alpha(
            &mut alpha,
            &request.to_bytes(),
            JobType::Vrf.as_u8(),
            b"dice",
        )
        .expect("alpha");
        assert_eq!(
            verify(&proved.public_key, &alpha[..len], &proved.proof),
            Ok(proved.output)
        );
        let mut forged = proved.proof;
        forged[79] ^= 1;
        assert_eq!(
            verify(&proved.public_key, &alpha[..len], &forged),
            Err(ProveError::Library)
        );
    }
}
