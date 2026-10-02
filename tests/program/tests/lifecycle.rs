//! LiteSVM coverage for the Milestone 1 instructions.
//!
//! The SBF binaries come from `cargo build-sbf`. This crate does not embed them,
//! so a host-only `cargo test` before that build fails with the path below.

use std::path::PathBuf;

use anchor_lang::AccountDeserialize;
use anchor_lang::InstructionData;
use anchor_lang::ToAccountMetas;
use litesvm::LiteSVM;
use nuvex_common::{
    JOB_CONFIG_SEED, NODE_SEED, PROTOCOL_SEED, REGISTRY_SEED, REQUEST_SEED, VERIFICATION_SEED,
};
use nuvex_job_types::JobType;
use nuvex_oracle_core::state::{JobConfig, OracleRequest, ProtocolConfig};
use nuvex_oracle_registry::state::{NodeAccount, NodeRegistry};
use nuvex_protocol_types::RequestStatus;
use nuvex_verification::state::VrfResult;
use nuvex_vrf::{prove, public_key, write_vrf_alpha, MAX_VRF_ALPHA_LEN};
use solana_address::Address;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_signer::Signer;
use solana_transaction::Transaction;

const AIRDROP: u64 = 10_000_000_000;
const STORED_MAX_FEE: u64 = 100_000_000_000;

fn deploy(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/deploy")
        .join(name)
}

fn address_from(bytes: [u8; 32]) -> Address {
    Address::new_from_array(bytes)
}

fn core_id() -> Address {
    address_from(nuvex_oracle_core::ID.to_bytes())
}

fn registry_id() -> Address {
    address_from(nuvex_oracle_registry::ID.to_bytes())
}

fn verification_id() -> Address {
    address_from(nuvex_verification::ID.to_bytes())
}

fn callback_id() -> Address {
    address_from(nuvex_callback_consumer::ID.to_bytes())
}

fn pk(address: &Address) -> anchor_lang::prelude::Pubkey {
    anchor_lang::prelude::Pubkey::new_from_array(address.to_bytes())
}

fn system_program() -> Address {
    solana_system_interface::program::ID
}

fn load_svm() -> LiteSVM {
    // LiteSVM builds the syscall table when builtins are installed. `new()`
    // uses the mainnet set, which does not include `sol_sha512`, and
    // `with_feature_set` does not rebuild that table.
    let mut features = LiteSVM::mainnet_feature_set();
    features.activate(&agave_feature_set::enable_sha512_syscall::id(), 0);
    let mut svm = LiteSVM::new().with_feature_set(features).with_builtins();
    let core = deploy("nuvex_oracle_core.so");
    let registry = deploy("nuvex_oracle_registry.so");
    let verification = deploy("nuvex_verification.so");
    let callback = deploy("nuvex_callback_consumer.so");
    svm.add_program_from_file(core_id(), &core)
        .unwrap_or_else(|err| panic!("load {}: {err}. Run cargo build-sbf first", core.display()));
    svm.add_program_from_file(registry_id(), &registry)
        .unwrap_or_else(|err| {
            panic!(
                "load {}: {err}. Run cargo build-sbf first",
                registry.display()
            )
        });
    svm.add_program_from_file(verification_id(), &verification)
        .unwrap_or_else(|err| {
            panic!(
                "load {}: {err}. Run cargo build-sbf first",
                verification.display()
            )
        });
    svm.add_program_from_file(callback_id(), &callback)
        .unwrap_or_else(|err| {
            panic!(
                "load {}: {err}. Run cargo build-sbf first",
                callback.display()
            )
        });
    svm
}

fn payer(svm: &mut LiteSVM) -> Keypair {
    let keypair = Keypair::new();
    svm.airdrop(&keypair.pubkey(), AIRDROP)
        .unwrap_or_else(|err| panic!("airdrop: {err:?}"));
    keypair
}

fn to_ix(
    program: Address,
    data: Vec<u8>,
    metas: Vec<anchor_lang::prelude::AccountMeta>,
) -> Instruction {
    Instruction {
        program_id: program,
        accounts: metas
            .into_iter()
            .map(|meta| solana_instruction::AccountMeta {
                pubkey: address_from(meta.pubkey.to_bytes()),
                is_signer: meta.is_signer,
                is_writable: meta.is_writable,
            })
            .collect(),
        data,
    }
}

#[allow(clippy::result_large_err)]
fn send(svm: &mut LiteSVM, signer: &Keypair, ix: Instruction) -> litesvm::types::TransactionResult {
    svm.expire_blockhash();
    let tx = Transaction::new(
        &[signer],
        Message::new(&[ix], Some(&signer.pubkey())),
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx)
}

fn send_ok(svm: &mut LiteSVM, signer: &Keypair, ix: Instruction) {
    match send(svm, signer, ix) {
        Ok(_) => {}
        Err(err) => panic!("{}\n{}", err.err, err.meta.pretty_logs()),
    }
}

fn assert_logs(result: litesvm::types::TransactionResult, needle: &str) {
    let err = match result {
        Err(err) => err,
        Ok(meta) => panic!("expected {needle}, logs:\n{:?}", meta.logs),
    };
    let logs = err.meta.logs.join("\n");
    let rendered = format!("{:?}\n{logs}", err.err);
    assert!(
        rendered.contains(needle),
        "expected {needle} in\n{rendered}"
    );
}

fn protocol_pda() -> Address {
    let (pda, _) = anchor_lang::prelude::Pubkey::find_program_address(
        &[PROTOCOL_SEED],
        &nuvex_oracle_core::ID,
    );
    address_from(pda.to_bytes())
}

fn job_pda(job: u8) -> Address {
    let (pda, _) = anchor_lang::prelude::Pubkey::find_program_address(
        &[JOB_CONFIG_SEED, &[job]],
        &nuvex_oracle_core::ID,
    );
    address_from(pda.to_bytes())
}

fn request_pda(requester: &Address, request_id: &[u8; 32]) -> Address {
    let (pda, _) = anchor_lang::prelude::Pubkey::find_program_address(
        &[REQUEST_SEED, requester.as_ref(), request_id],
        &nuvex_oracle_core::ID,
    );
    address_from(pda.to_bytes())
}

fn registry_pda() -> Address {
    let (pda, _) = anchor_lang::prelude::Pubkey::find_program_address(
        &[REGISTRY_SEED],
        &nuvex_oracle_registry::ID,
    );
    address_from(pda.to_bytes())
}

fn vrf_result_pda(request: &Address) -> Address {
    let (pda, _) = anchor_lang::prelude::Pubkey::find_program_address(
        &[VERIFICATION_SEED, request.as_ref()],
        &nuvex_verification::ID,
    );
    address_from(pda.to_bytes())
}

fn node_pda(authority: &Address) -> Address {
    let (pda, _) = anchor_lang::prelude::Pubkey::find_program_address(
        &[NODE_SEED, authority.as_ref()],
        &nuvex_oracle_registry::ID,
    );
    address_from(pda.to_bytes())
}

fn account_data<T: AccountDeserialize>(svm: &LiteSVM, address: &Address) -> T {
    let account = svm
        .get_account(address)
        .unwrap_or_else(|| panic!("missing account {address}"));
    T::try_deserialize(&mut account.data.as_slice())
        .unwrap_or_else(|err| panic!("deserialize {address}: {err}"))
}

fn initialize(svm: &mut LiteSVM, authority: &Keypair) {
    let ix = to_ix(
        core_id(),
        nuvex_oracle_core::instruction::Initialize.data(),
        nuvex_oracle_core::accounts::Initialize {
            authority: pk(&authority.pubkey()),
            protocol: pk(&protocol_pda()),
            system_program: pk(&system_program()),
        }
        .to_account_metas(None),
    );
    send_ok(svm, authority, ix);
}

fn configure(svm: &mut LiteSVM, authority: &Keypair, job: u8, timeout: u64, enabled: bool) {
    let ix = to_ix(
        core_id(),
        nuvex_oracle_core::instruction::ConfigureJob {
            job_type: job,
            timeout_slots: timeout,
            requests_enabled: enabled,
        }
        .data(),
        nuvex_oracle_core::accounts::ConfigureJob {
            authority: pk(&authority.pubkey()),
            protocol: pk(&protocol_pda()),
            job_config: pk(&job_pda(job)),
            system_program: pk(&system_program()),
        }
        .to_account_metas(None),
    );
    send_ok(svm, authority, ix);
}

fn open_request(
    svm: &mut LiteSVM,
    requester: &Keypair,
    request_id: [u8; 32],
    job: u8,
    max_fee: u64,
) -> Address {
    let request = request_pda(&requester.pubkey(), &request_id);
    let ix = to_ix(
        core_id(),
        nuvex_oracle_core::instruction::Request {
            request_id,
            job_type: job,
            input: vec![7, 8, 9],
            max_fee,
            callback_program: anchor_lang::prelude::Pubkey::default(),
            callback_data: vec![],
        }
        .data(),
        nuvex_oracle_core::accounts::CreateRequest {
            requester: pk(&requester.pubkey()),
            protocol: pk(&protocol_pda()),
            job_config: pk(&job_pda(job)),
            request: pk(&request),
            system_program: pk(&system_program()),
        }
        .to_account_metas(None),
    );
    send_ok(svm, requester, ix);
    request
}

#[test]
fn protocol_request_and_registry_lifecycle() {
    let mut svm = load_svm();
    let authority = payer(&mut svm);
    let requester = payer(&mut svm);
    let other = payer(&mut svm);
    let crank = payer(&mut svm);
    let node_authority = payer(&mut svm);

    initialize(&mut svm, &authority);
    assert_logs(
        send(
            &mut svm,
            &authority,
            to_ix(
                core_id(),
                nuvex_oracle_core::instruction::Initialize.data(),
                nuvex_oracle_core::accounts::Initialize {
                    authority: pk(&authority.pubkey()),
                    protocol: pk(&protocol_pda()),
                    system_program: pk(&system_program()),
                }
                .to_account_metas(None),
            ),
        ),
        "already in use",
    );

    let protocol_lamports = svm.get_account(&protocol_pda()).expect("protocol").lamports;
    configure(&mut svm, &authority, JobType::Vrf.as_u8(), 10, true);
    assert_logs(
        send(
            &mut svm,
            &authority,
            to_ix(
                core_id(),
                nuvex_oracle_core::instruction::ConfigureJob {
                    job_type: JobType::Price.as_u8(),
                    timeout_slots: 10,
                    requests_enabled: true,
                }
                .data(),
                nuvex_oracle_core::accounts::ConfigureJob {
                    authority: pk(&authority.pubkey()),
                    protocol: pk(&protocol_pda()),
                    job_config: pk(&job_pda(JobType::Price.as_u8())),
                    system_program: pk(&system_program()),
                }
                .to_account_metas(None),
            ),
        ),
        "This job type cannot be requested",
    );
    assert_logs(
        send(
            &mut svm,
            &authority,
            to_ix(
                core_id(),
                nuvex_oracle_core::instruction::UpdateJob {
                    job_type: JobType::Vrf.as_u8(),
                    timeout_slots: 0,
                    requests_enabled: true,
                }
                .data(),
                nuvex_oracle_core::accounts::UpdateJob {
                    authority: pk(&authority.pubkey()),
                    protocol: pk(&protocol_pda()),
                    job_config: pk(&job_pda(JobType::Vrf.as_u8())),
                }
                .to_account_metas(None),
            ),
        ),
        "Timeout is outside the allowed range",
    );

    let request_id = [4_u8; 32];
    let request = open_request(
        &mut svm,
        &requester,
        request_id,
        JobType::Vrf.as_u8(),
        STORED_MAX_FEE,
    );
    let stored: OracleRequest = account_data(&svm, &request);
    assert_eq!(stored.status, RequestStatus::Pending.as_u8());
    assert_eq!(stored.max_fee, STORED_MAX_FEE);
    assert_eq!(stored.input_len, 3);
    assert_eq!(stored.expires_slot, stored.created_slot + 10);
    assert_eq!(
        svm.get_account(&protocol_pda()).expect("protocol").lamports,
        protocol_lamports,
        "request creation must not pay the protocol"
    );
    let _: ProtocolConfig = account_data(&svm, &protocol_pda());
    let job: JobConfig = account_data(&svm, &job_pda(JobType::Vrf.as_u8()));
    assert!(job.requests_enabled);

    let same_id = request_pda(&other.pubkey(), &request_id);
    assert_ne!(same_id, request);
    open_request(
        &mut svm,
        &other,
        request_id,
        JobType::Vrf.as_u8(),
        STORED_MAX_FEE,
    );

    assert_logs(
        send(
            &mut svm,
            &other,
            to_ix(
                core_id(),
                nuvex_oracle_core::instruction::Cancel.data(),
                nuvex_oracle_core::accounts::Cancel {
                    requester: pk(&other.pubkey()),
                    request: pk(&request),
                }
                .to_account_metas(None),
            ),
        ),
        "A has one constraint was violated",
    );

    svm.warp_to_slot(stored.expires_slot.saturating_sub(1));
    assert_logs(
        send(
            &mut svm,
            &crank,
            to_ix(
                core_id(),
                nuvex_oracle_core::instruction::Expire.data(),
                nuvex_oracle_core::accounts::Expire {
                    crank: pk(&crank.pubkey()),
                    request: pk(&request),
                }
                .to_account_metas(None),
            ),
        ),
        "The request has not expired",
    );

    send_ok(
        &mut svm,
        &requester,
        to_ix(
            core_id(),
            nuvex_oracle_core::instruction::Cancel.data(),
            nuvex_oracle_core::accounts::Cancel {
                requester: pk(&requester.pubkey()),
                request: pk(&request),
            }
            .to_account_metas(None),
        ),
    );
    let cancelled: OracleRequest = account_data(&svm, &request);
    assert_eq!(cancelled.status, RequestStatus::Cancelled.as_u8());

    let expired_id = [5_u8; 32];
    let expiring = open_request(&mut svm, &requester, expired_id, JobType::Vrf.as_u8(), 0);
    let expiring_account: OracleRequest = account_data(&svm, &expiring);
    svm.warp_to_slot(expiring_account.expires_slot);
    assert_logs(
        send(
            &mut svm,
            &requester,
            to_ix(
                core_id(),
                nuvex_oracle_core::instruction::Cancel.data(),
                nuvex_oracle_core::accounts::Cancel {
                    requester: pk(&requester.pubkey()),
                    request: pk(&expiring),
                }
                .to_account_metas(None),
            ),
        ),
        "The request has expired",
    );
    send_ok(
        &mut svm,
        &crank,
        to_ix(
            core_id(),
            nuvex_oracle_core::instruction::Expire.data(),
            nuvex_oracle_core::accounts::Expire {
                crank: pk(&crank.pubkey()),
                request: pk(&expiring),
            }
            .to_account_metas(None),
        ),
    );
    let expired: OracleRequest = account_data(&svm, &expiring);
    assert_eq!(expired.status, RequestStatus::Expired.as_u8());

    let pending_id = [6_u8; 32];
    let pending = open_request(&mut svm, &requester, pending_id, JobType::Vrf.as_u8(), 0);
    assert_logs(
        send(
            &mut svm,
            &crank,
            to_ix(
                core_id(),
                nuvex_oracle_core::instruction::CloseRequest.data(),
                nuvex_oracle_core::accounts::CloseRequest {
                    crank: pk(&crank.pubkey()),
                    request: pk(&pending),
                    requester: pk(&requester.pubkey()),
                }
                .to_account_metas(None),
            ),
        ),
        "The request is not in a terminal state",
    );

    let rent = svm
        .get_account(&request)
        .expect("cancelled request")
        .lamports;
    let before = svm
        .get_account(&requester.pubkey())
        .expect("requester")
        .lamports;
    send_ok(
        &mut svm,
        &crank,
        to_ix(
            core_id(),
            nuvex_oracle_core::instruction::CloseRequest.data(),
            nuvex_oracle_core::accounts::CloseRequest {
                crank: pk(&crank.pubkey()),
                request: pk(&request),
                requester: pk(&requester.pubkey()),
            }
            .to_account_metas(None),
        ),
    );
    assert!(svm.get_account(&request).is_none());
    let after = svm
        .get_account(&requester.pubkey())
        .expect("requester")
        .lamports;
    assert_eq!(after, before + rent);

    send_ok(
        &mut svm,
        &authority,
        to_ix(
            core_id(),
            nuvex_oracle_core::instruction::SetPaused { paused: true }.data(),
            nuvex_oracle_core::accounts::SetPaused {
                authority: pk(&authority.pubkey()),
                protocol: pk(&protocol_pda()),
            }
            .to_account_metas(None),
        ),
    );
    let paused_id = [7_u8; 32];
    assert_logs(
        send(
            &mut svm,
            &requester,
            to_ix(
                core_id(),
                nuvex_oracle_core::instruction::Request {
                    request_id: paused_id,
                    job_type: JobType::Vrf.as_u8(),
                    input: vec![],
                    max_fee: 0,
                    callback_program: anchor_lang::prelude::Pubkey::default(),
                    callback_data: vec![],
                }
                .data(),
                nuvex_oracle_core::accounts::CreateRequest {
                    requester: pk(&requester.pubkey()),
                    protocol: pk(&protocol_pda()),
                    job_config: pk(&job_pda(JobType::Vrf.as_u8())),
                    request: pk(&request_pda(&requester.pubkey(), &paused_id)),
                    system_program: pk(&system_program()),
                }
                .to_account_metas(None),
            ),
        ),
        "The protocol is paused",
    );
    send_ok(
        &mut svm,
        &requester,
        to_ix(
            core_id(),
            nuvex_oracle_core::instruction::Cancel.data(),
            nuvex_oracle_core::accounts::Cancel {
                requester: pk(&requester.pubkey()),
                request: pk(&pending),
            }
            .to_account_metas(None),
        ),
    );

    send_ok(
        &mut svm,
        &authority,
        to_ix(
            registry_id(),
            nuvex_oracle_registry::instruction::Initialize.data(),
            nuvex_oracle_registry::accounts::Initialize {
                authority: pk(&authority.pubkey()),
                registry: pk(&registry_pda()),
                system_program: pk(&system_program()),
            }
            .to_account_metas(None),
        ),
    );
    let operator = Keypair::new().pubkey();
    send_ok(
        &mut svm,
        &node_authority,
        to_ix(
            registry_id(),
            nuvex_oracle_registry::instruction::RegisterNode {
                operator: pk(&operator),
                vrf_pubkey: [0u8; 32],
                supported_job_mask: JobType::Vrf.capability_bit(),
            }
            .data(),
            nuvex_oracle_registry::accounts::RegisterNode {
                node_authority: pk(&node_authority.pubkey()),
                registry: pk(&registry_pda()),
                node: pk(&node_pda(&node_authority.pubkey())),
                system_program: pk(&system_program()),
            }
            .to_account_metas(None),
        ),
    );
    let node: NodeAccount = account_data(&svm, &node_pda(&node_authority.pubkey()));
    assert_eq!(node.vrf_pubkey, [0u8; 32]);
    assert_eq!(node.stake, 0);
    assert_eq!(node.reputation, 0);
    assert_eq!(node.last_heartbeat, 0);
    assert_eq!(node.status, 1);
    let registry: NodeRegistry = account_data(&svm, &registry_pda());
    assert_eq!(registry.node_count, 1);

    assert_logs(
        send(
            &mut svm,
            &node_authority,
            to_ix(
                registry_id(),
                nuvex_oracle_registry::instruction::RegisterNode {
                    operator: pk(&operator),
                    vrf_pubkey: [0u8; 32],
                    supported_job_mask: 0,
                }
                .data(),
                nuvex_oracle_registry::accounts::RegisterNode {
                    node_authority: pk(&node_authority.pubkey()),
                    registry: pk(&registry_pda()),
                    node: pk(&node_pda(&node_authority.pubkey())),
                    system_program: pk(&system_program()),
                }
                .to_account_metas(None),
            ),
        ),
        "already in use",
    );
    assert_logs(
        send(
            &mut svm,
            &other,
            to_ix(
                registry_id(),
                nuvex_oracle_registry::instruction::UpdateNode {
                    operator: pk(&operator),
                    supported_job_mask: 0,
                }
                .data(),
                nuvex_oracle_registry::accounts::UpdateNode {
                    node_authority: pk(&other.pubkey()),
                    node: pk(&node_pda(&node_authority.pubkey())),
                }
                .to_account_metas(None),
            ),
        ),
        "A seeds constraint was violated",
    );
    assert_logs(
        send(
            &mut svm,
            &other,
            to_ix(
                registry_id(),
                nuvex_oracle_registry::instruction::RegisterNode {
                    operator: anchor_lang::prelude::Pubkey::default(),
                    vrf_pubkey: [0u8; 32],
                    supported_job_mask: 0,
                }
                .data(),
                nuvex_oracle_registry::accounts::RegisterNode {
                    node_authority: pk(&other.pubkey()),
                    registry: pk(&registry_pda()),
                    node: pk(&node_pda(&other.pubkey())),
                    system_program: pk(&system_program()),
                }
                .to_account_metas(None),
            ),
        ),
        "The operator pubkey is the default pubkey",
    );
    assert_logs(
        send(
            &mut svm,
            &other,
            to_ix(
                registry_id(),
                nuvex_oracle_registry::instruction::RegisterNode {
                    operator: pk(&operator),
                    vrf_pubkey: [0u8; 32],
                    supported_job_mask: 1,
                }
                .data(),
                nuvex_oracle_registry::accounts::RegisterNode {
                    node_authority: pk(&other.pubkey()),
                    registry: pk(&registry_pda()),
                    node: pk(&node_pda(&other.pubkey())),
                    system_program: pk(&system_program()),
                }
                .to_account_metas(None),
            ),
        ),
        "The job mask sets a bit this program does not know",
    );
    assert_logs(
        send(
            &mut svm,
            &other,
            to_ix(
                registry_id(),
                nuvex_oracle_registry::instruction::RegisterNode {
                    operator: pk(&operator),
                    vrf_pubkey: {
                        let mut small_order = [0u8; 32];
                        small_order[0] = 1;
                        small_order
                    },
                    supported_job_mask: 0,
                }
                .data(),
                nuvex_oracle_registry::accounts::RegisterNode {
                    node_authority: pk(&other.pubkey()),
                    registry: pk(&registry_pda()),
                    node: pk(&node_pda(&other.pubkey())),
                    system_program: pk(&system_program()),
                }
                .to_account_metas(None),
            ),
        ),
        "The VRF public key is not a valid Ed25519 point",
    );
}

fn prove_for(secret: &[u8; 32], request: &Address, input: &[u8]) -> ([u8; 64], [u8; 80]) {
    let mut alpha = [0u8; MAX_VRF_ALPHA_LEN];
    let len = write_vrf_alpha(&mut alpha, &request.to_bytes(), JobType::Vrf.as_u8(), input)
        .expect("alpha");
    prove(secret, &alpha[..len]).expect("prove")
}

fn register(
    svm: &mut LiteSVM,
    authority: &Keypair,
    operator: &Address,
    vrf_pubkey: [u8; 32],
    mask: u32,
) {
    send_ok(
        svm,
        authority,
        to_ix(
            registry_id(),
            nuvex_oracle_registry::instruction::RegisterNode {
                operator: pk(operator),
                vrf_pubkey,
                supported_job_mask: mask,
            }
            .data(),
            nuvex_oracle_registry::accounts::RegisterNode {
                node_authority: pk(&authority.pubkey()),
                registry: pk(&registry_pda()),
                node: pk(&node_pda(&authority.pubkey())),
                system_program: pk(&system_program()),
            }
            .to_account_metas(None),
        ),
    );
}

fn fulfill_ix(
    authority: &Address,
    request: &Address,
    proof: [u8; 80],
    callback: Option<anchor_lang::prelude::Pubkey>,
) -> Instruction {
    to_ix(
        core_id(),
        nuvex_oracle_core::instruction::Fulfill { proof }.data(),
        nuvex_oracle_core::accounts::Fulfill {
            node_authority: pk(authority),
            node: pk(&node_pda(authority)),
            registry: pk(&registry_pda()),
            request: pk(request),
            protocol: pk(&protocol_pda()),
            verification_program: pk(&verification_id()),
            vrf_result: pk(&vrf_result_pda(request)),
            system_program: pk(&system_program()),
            registry_program: pk(&registry_id()),
            callback_program: callback,
        }
        .to_account_metas(None),
    )
}

fn configure_stake(
    svm: &mut LiteSVM,
    authority: &Keypair,
    min_stake: u64,
    cooldown: u64,
    heartbeat: u64,
    slash_authority: anchor_lang::prelude::Pubkey,
    slash_destination: anchor_lang::prelude::Pubkey,
) {
    send_ok(
        svm,
        authority,
        to_ix(
            registry_id(),
            nuvex_oracle_registry::instruction::ConfigureStake {
                min_stake,
                unstake_cooldown_slots: cooldown,
                heartbeat_timeout_slots: heartbeat,
                slash_authority,
                slash_destination,
            }
            .data(),
            nuvex_oracle_registry::accounts::ConfigureStake {
                authority: pk(&authority.pubkey()),
                registry: pk(&registry_pda()),
            }
            .to_account_metas(None),
        ),
    );
}

fn stake_lamports(svm: &mut LiteSVM, authority: &Keypair, amount: u64) {
    send_ok(
        svm,
        authority,
        to_ix(
            registry_id(),
            nuvex_oracle_registry::instruction::Stake { amount }.data(),
            nuvex_oracle_registry::accounts::Stake {
                node_authority: pk(&authority.pubkey()),
                registry: pk(&registry_pda()),
                node: pk(&node_pda(&authority.pubkey())),
                system_program: pk(&system_program()),
            }
            .to_account_metas(None),
        ),
    );
}

fn heartbeat_node(svm: &mut LiteSVM, authority: &Keypair) {
    send_ok(
        svm,
        authority,
        to_ix(
            registry_id(),
            nuvex_oracle_registry::instruction::Heartbeat.data(),
            nuvex_oracle_registry::accounts::Heartbeat {
                node_authority: pk(&authority.pubkey()),
                node: pk(&node_pda(&authority.pubkey())),
            }
            .to_account_metas(None),
        ),
    );
}

#[test]
fn vrf_fulfillment_verifies_the_proof_and_runs_the_callback() {
    let mut svm = load_svm();
    let authority = payer(&mut svm);
    let requester = payer(&mut svm);
    let node_authority = payer(&mut svm);
    let late = payer(&mut svm);
    let incapable = payer(&mut svm);
    let operator = Keypair::new().pubkey();
    let secret = [11u8; 32];
    let input = vec![7u8, 8, 9];

    initialize(&mut svm, &authority);
    configure(&mut svm, &authority, JobType::Vrf.as_u8(), 20, true);
    send_ok(
        &mut svm,
        &authority,
        to_ix(
            registry_id(),
            nuvex_oracle_registry::instruction::Initialize.data(),
            nuvex_oracle_registry::accounts::Initialize {
                authority: pk(&authority.pubkey()),
                registry: pk(&registry_pda()),
                system_program: pk(&system_program()),
            }
            .to_account_metas(None),
        ),
    );
    register(
        &mut svm,
        &node_authority,
        &operator,
        public_key(&secret),
        JobType::Vrf.capability_bit(),
    );
    register(&mut svm, &incapable, &operator, public_key(&[12u8; 32]), 0);
    let protocol_lamports = svm.get_account(&protocol_pda()).expect("protocol").lamports;
    let registered: NodeAccount = account_data(&svm, &node_pda(&node_authority.pubkey()));
    svm.warp_to_slot(registered.created_slot.saturating_add(1));
    let slash_authority = payer(&mut svm);
    let slash_destination = payer(&mut svm);
    configure_stake(
        &mut svm,
        &authority,
        1_000_000,
        10,
        100_000,
        pk(&slash_authority.pubkey()),
        pk(&slash_destination.pubkey()),
    );
    stake_lamports(&mut svm, &node_authority, 1_000_000);
    heartbeat_node(&mut svm, &node_authority);
    let request_id = [3u8; 32];
    let request = request_pda(&requester.pubkey(), &request_id);
    send_ok(
        &mut svm,
        &requester,
        to_ix(
            core_id(),
            nuvex_oracle_core::instruction::Request {
                request_id,
                job_type: JobType::Vrf.as_u8(),
                input: input.clone(),
                max_fee: STORED_MAX_FEE,
                callback_program: anchor_lang::prelude::Pubkey::default(),
                callback_data: vec![],
            }
            .data(),
            nuvex_oracle_core::accounts::CreateRequest {
                requester: pk(&requester.pubkey()),
                protocol: pk(&protocol_pda()),
                job_config: pk(&job_pda(JobType::Vrf.as_u8())),
                request: pk(&request),
                system_program: pk(&system_program()),
            }
            .to_account_metas(None),
        ),
    );
    let (output, proof) = prove_for(&secret, &request, &input);

    assert_logs(
        send(
            &mut svm,
            &incapable,
            fulfill_ix(&incapable.pubkey(), &request, proof, None),
        ),
        "The node cannot fulfill this VRF request",
    );
    register(
        &mut svm,
        &late,
        &operator,
        public_key(&[13u8; 32]),
        JobType::Vrf.capability_bit(),
    );
    let late_proof = prove_for(&[13u8; 32], &request, &input).1;
    assert_logs(
        send(
            &mut svm,
            &late,
            fulfill_ix(&late.pubkey(), &request, late_proof, None),
        ),
        "The node registered its VRF key too late",
    );

    let mut forged = proof;
    forged[79] ^= 1;
    assert_logs(
        send(
            &mut svm,
            &node_authority,
            fulfill_ix(&node_authority.pubkey(), &request, forged, None),
        ),
        "The VRF proof does not verify",
    );
    let pending: OracleRequest = account_data(&svm, &request);
    assert_eq!(pending.status, RequestStatus::Pending.as_u8());
    assert!(svm.get_account(&vrf_result_pda(&request)).is_none());

    send_ok(
        &mut svm,
        &node_authority,
        fulfill_ix(&node_authority.pubkey(), &request, proof, None),
    );
    let stored: OracleRequest = account_data(&svm, &request);
    assert_eq!(stored.status, RequestStatus::CallbackExecuted.as_u8());
    assert_eq!(stored.max_fee, STORED_MAX_FEE);
    assert_eq!(
        stored.assigned_node,
        pk(&node_pda(&node_authority.pubkey()))
    );
    assert_eq!(stored.assigned_stake, 1_000_000);
    let fulfilled_node: NodeAccount = account_data(&svm, &node_pda(&node_authority.pubkey()));
    assert_eq!(stored.assigned_heartbeat, fulfilled_node.last_heartbeat);
    assert_eq!(fulfilled_node.reputation, 1);
    let result: VrfResult = account_data(&svm, &vrf_result_pda(&request));
    assert_eq!(result.output, output);
    assert_eq!(result.proof, proof);
    assert_eq!(result.vrf_pubkey, public_key(&secret));
    assert_eq!(
        svm.get_account(&protocol_pda()).expect("protocol").lamports,
        protocol_lamports,
        "fulfillment must not pay the protocol"
    );
    assert_logs(
        send(
            &mut svm,
            &node_authority,
            fulfill_ix(&node_authority.pubkey(), &request, proof, None),
        ),
        "The request status transition is not allowed",
    );

    let callback_id_bytes = [21u8; 32];
    let callback_request_id = callback_id_bytes;
    let callback_request = request_pda(&requester.pubkey(), &callback_request_id);
    send_ok(
        &mut svm,
        &requester,
        to_ix(
            core_id(),
            nuvex_oracle_core::instruction::Request {
                request_id: callback_request_id,
                job_type: JobType::Vrf.as_u8(),
                input: input.clone(),
                max_fee: 0,
                callback_program: nuvex_callback_consumer::ID,
                callback_data: vec![1, 2, 3],
            }
            .data(),
            nuvex_oracle_core::accounts::CreateRequest {
                requester: pk(&requester.pubkey()),
                protocol: pk(&protocol_pda()),
                job_config: pk(&job_pda(JobType::Vrf.as_u8())),
                request: pk(&callback_request),
                system_program: pk(&system_program()),
            }
            .to_account_metas(None),
        ),
    );
    let callback_proof = prove_for(&secret, &callback_request, &input).1;
    let callback_result = send(
        &mut svm,
        &node_authority,
        fulfill_ix(
            &node_authority.pubkey(),
            &callback_request,
            callback_proof,
            Some(nuvex_callback_consumer::ID),
        ),
    );
    let callback_logs = match callback_result {
        Ok(meta) => meta.logs.join("\n"),
        Err(err) => panic!("{}\n{}", err.err, err.meta.pretty_logs()),
    };
    assert!(callback_logs.contains("nuvex-callback"), "{callback_logs}");
    let called: OracleRequest = account_data(&svm, &callback_request);
    assert_eq!(called.status, RequestStatus::CallbackExecuted.as_u8());
    let after_callback: NodeAccount = account_data(&svm, &node_pda(&node_authority.pubkey()));
    assert_eq!(after_callback.reputation, 2);

    let rejected_id = [22u8; 32];
    let rejected = request_pda(&requester.pubkey(), &rejected_id);
    send_ok(
        &mut svm,
        &requester,
        to_ix(
            core_id(),
            nuvex_oracle_core::instruction::Request {
                request_id: rejected_id,
                job_type: JobType::Vrf.as_u8(),
                input: input.clone(),
                max_fee: 0,
                callback_program: nuvex_callback_consumer::ID,
                callback_data: vec![0xff],
            }
            .data(),
            nuvex_oracle_core::accounts::CreateRequest {
                requester: pk(&requester.pubkey()),
                protocol: pk(&protocol_pda()),
                job_config: pk(&job_pda(JobType::Vrf.as_u8())),
                request: pk(&rejected),
                system_program: pk(&system_program()),
            }
            .to_account_metas(None),
        ),
    );
    let rejected_proof = prove_for(&secret, &rejected, &input).1;
    assert_logs(
        send(
            &mut svm,
            &node_authority,
            fulfill_ix(
                &node_authority.pubkey(),
                &rejected,
                rejected_proof,
                Some(nuvex_callback_consumer::ID),
            ),
        ),
        "callback rejected",
    );
    let still_pending: OracleRequest = account_data(&svm, &rejected);
    assert_eq!(still_pending.status, RequestStatus::Pending.as_u8());
    assert!(svm.get_account(&vrf_result_pda(&rejected)).is_none());
    let after_reject: NodeAccount = account_data(&svm, &node_pda(&node_authority.pubkey()));
    assert_eq!(after_reject.reputation, 2);

    svm.warp_to_slot(still_pending.expires_slot);
    assert_logs(
        send(
            &mut svm,
            &node_authority,
            fulfill_ix(
                &node_authority.pubkey(),
                &rejected,
                rejected_proof,
                Some(nuvex_callback_consumer::ID),
            ),
        ),
        "The request has expired",
    );
}

#[test]
fn stake_heartbeat_and_slash_follow_the_cooldown() {
    let mut svm = load_svm();
    let authority = payer(&mut svm);
    let requester = payer(&mut svm);
    let node_authority = payer(&mut svm);
    let slash_authority = payer(&mut svm);
    let destination = payer(&mut svm);
    let operator = Keypair::new().pubkey();

    initialize(&mut svm, &authority);
    configure(&mut svm, &authority, JobType::Vrf.as_u8(), 50, true);
    send_ok(
        &mut svm,
        &authority,
        to_ix(
            registry_id(),
            nuvex_oracle_registry::instruction::Initialize.data(),
            nuvex_oracle_registry::accounts::Initialize {
                authority: pk(&authority.pubkey()),
                registry: pk(&registry_pda()),
                system_program: pk(&system_program()),
            }
            .to_account_metas(None),
        ),
    );
    register(
        &mut svm,
        &node_authority,
        &operator,
        public_key(&[9u8; 32]),
        JobType::Vrf.capability_bit(),
    );
    let node_pda = node_pda(&node_authority.pubkey());
    assert_logs(
        send(
            &mut svm,
            &node_authority,
            to_ix(
                registry_id(),
                nuvex_oracle_registry::instruction::Stake { amount: 1 }.data(),
                nuvex_oracle_registry::accounts::Stake {
                    node_authority: pk(&node_authority.pubkey()),
                    registry: pk(&registry_pda()),
                    node: pk(&node_pda),
                    system_program: pk(&system_program()),
                }
                .to_account_metas(None),
            ),
        ),
        "Stake parameters are not configured",
    );
    assert_logs(
        send(
            &mut svm,
            &authority,
            to_ix(
                registry_id(),
                nuvex_oracle_registry::instruction::ConfigureStake {
                    min_stake: 1_000,
                    unstake_cooldown_slots: 0,
                    heartbeat_timeout_slots: 3,
                    slash_authority: anchor_lang::prelude::Pubkey::default(),
                    slash_destination: anchor_lang::prelude::Pubkey::default(),
                }
                .data(),
                nuvex_oracle_registry::accounts::ConfigureStake {
                    authority: pk(&authority.pubkey()),
                    registry: pk(&registry_pda()),
                }
                .to_account_metas(None),
            ),
        ),
        "The cooldown or heartbeat window is outside the allowed range",
    );
    assert_logs(
        send(
            &mut svm,
            &authority,
            to_ix(
                registry_id(),
                nuvex_oracle_registry::instruction::ConfigureStake {
                    min_stake: 1_000,
                    unstake_cooldown_slots: 5,
                    heartbeat_timeout_slots: 3,
                    slash_authority: pk(&slash_authority.pubkey()),
                    slash_destination: anchor_lang::prelude::Pubkey::default(),
                }
                .data(),
                nuvex_oracle_registry::accounts::ConfigureStake {
                    authority: pk(&authority.pubkey()),
                    registry: pk(&registry_pda()),
                }
                .to_account_metas(None),
            ),
        ),
        "Slash authority and destination must both be set or both be empty",
    );
    configure_stake(
        &mut svm,
        &authority,
        1_000,
        5,
        3,
        pk(&slash_authority.pubkey()),
        pk(&destination.pubkey()),
    );

    let registered: NodeAccount = account_data(&svm, &node_pda);
    svm.warp_to_slot(registered.created_slot.saturating_add(1));
    let rent = svm.get_account(&node_pda).expect("node").lamports;
    stake_lamports(&mut svm, &node_authority, 400);
    let partial: NodeAccount = account_data(&svm, &node_pda);
    assert_eq!(
        partial.status,
        nuvex_oracle_registry::constants::NODE_STATUS_REGISTERED
    );
    assert_eq!(partial.stake, 400);
    assert_eq!(
        svm.get_account(&node_pda).expect("node").lamports,
        rent + 400
    );
    assert_logs(
        send(
            &mut svm,
            &node_authority,
            to_ix(
                registry_id(),
                nuvex_oracle_registry::instruction::Heartbeat.data(),
                nuvex_oracle_registry::accounts::Heartbeat {
                    node_authority: pk(&node_authority.pubkey()),
                    node: pk(&node_pda),
                }
                .to_account_metas(None),
            ),
        ),
        "Only an active node can heartbeat",
    );

    send_ok(
        &mut svm,
        &node_authority,
        to_ix(
            registry_id(),
            nuvex_oracle_registry::instruction::Unstake.data(),
            nuvex_oracle_registry::accounts::Unstake {
                node_authority: pk(&node_authority.pubkey()),
                registry: pk(&registry_pda()),
                node: pk(&node_pda),
            }
            .to_account_metas(None),
        ),
    );
    let unstaking: NodeAccount = account_data(&svm, &node_pda);
    assert_eq!(
        unstaking.status,
        nuvex_oracle_registry::constants::NODE_STATUS_UNSTAKING
    );
    assert_logs(
        send(
            &mut svm,
            &node_authority,
            to_ix(
                registry_id(),
                nuvex_oracle_registry::instruction::Withdraw.data(),
                nuvex_oracle_registry::accounts::Withdraw {
                    node_authority: pk(&node_authority.pubkey()),
                    node: pk(&node_pda),
                }
                .to_account_metas(None),
            ),
        ),
        "The unstake cooldown has not elapsed",
    );
    svm.warp_to_slot(unstaking.cooldown_end_slot);
    send_ok(
        &mut svm,
        &node_authority,
        to_ix(
            registry_id(),
            nuvex_oracle_registry::instruction::Withdraw.data(),
            nuvex_oracle_registry::accounts::Withdraw {
                node_authority: pk(&node_authority.pubkey()),
                node: pk(&node_pda),
            }
            .to_account_metas(None),
        ),
    );
    let withdrawn: NodeAccount = account_data(&svm, &node_pda);
    assert_eq!(withdrawn.stake, 0);
    assert_eq!(
        withdrawn.status,
        nuvex_oracle_registry::constants::NODE_STATUS_REGISTERED
    );
    assert_eq!(svm.get_account(&node_pda).expect("node").lamports, rent);

    stake_lamports(&mut svm, &node_authority, 1_000);
    let active: NodeAccount = account_data(&svm, &node_pda);
    assert_eq!(
        active.status,
        nuvex_oracle_registry::constants::NODE_STATUS_ACTIVE
    );
    let request_id = [9u8; 32];
    let request = request_pda(&requester.pubkey(), &request_id);
    send_ok(
        &mut svm,
        &requester,
        to_ix(
            core_id(),
            nuvex_oracle_core::instruction::Request {
                request_id,
                job_type: JobType::Vrf.as_u8(),
                input: vec![1],
                max_fee: 0,
                callback_program: anchor_lang::prelude::Pubkey::default(),
                callback_data: vec![],
            }
            .data(),
            nuvex_oracle_core::accounts::CreateRequest {
                requester: pk(&requester.pubkey()),
                protocol: pk(&protocol_pda()),
                job_config: pk(&job_pda(JobType::Vrf.as_u8())),
                request: pk(&request),
                system_program: pk(&system_program()),
            }
            .to_account_metas(None),
        ),
    );
    let blank = [0u8; 80];
    assert_logs(
        send(
            &mut svm,
            &node_authority,
            fulfill_ix(&node_authority.pubkey(), &request, blank, None),
        ),
        "The node has not sent a heartbeat",
    );
    heartbeat_node(&mut svm, &node_authority);
    configure_stake(
        &mut svm,
        &authority,
        5_000,
        5,
        3,
        pk(&slash_authority.pubkey()),
        pk(&destination.pubkey()),
    );
    assert_logs(
        send(
            &mut svm,
            &node_authority,
            fulfill_ix(&node_authority.pubkey(), &request, blank, None),
        ),
        "The node stake is below the configured minimum",
    );
    configure_stake(
        &mut svm,
        &authority,
        1_000,
        5,
        3,
        pk(&slash_authority.pubkey()),
        pk(&destination.pubkey()),
    );
    let beating: NodeAccount = account_data(&svm, &node_pda);
    svm.warp_to_slot(beating.last_heartbeat.saturating_add(3));
    assert_logs(
        send(
            &mut svm,
            &node_authority,
            fulfill_ix(&node_authority.pubkey(), &request, blank, None),
        ),
        "The node heartbeat is older than the configured window",
    );
    heartbeat_node(&mut svm, &node_authority);
    send_ok(
        &mut svm,
        &node_authority,
        to_ix(
            registry_id(),
            nuvex_oracle_registry::instruction::Unstake.data(),
            nuvex_oracle_registry::accounts::Unstake {
                node_authority: pk(&node_authority.pubkey()),
                registry: pk(&registry_pda()),
                node: pk(&node_pda),
            }
            .to_account_metas(None),
        ),
    );
    assert_logs(
        send(
            &mut svm,
            &node_authority,
            to_ix(
                registry_id(),
                nuvex_oracle_registry::instruction::Stake { amount: 1 }.data(),
                nuvex_oracle_registry::accounts::Stake {
                    node_authority: pk(&node_authority.pubkey()),
                    registry: pk(&registry_pda()),
                    node: pk(&node_pda),
                    system_program: pk(&system_program()),
                }
                .to_account_metas(None),
            ),
        ),
        "The node status does not allow this stake change",
    );
    assert_logs(
        send(
            &mut svm,
            &node_authority,
            fulfill_ix(&node_authority.pubkey(), &request, blank, None),
        ),
        "The node is not active",
    );

    let destination_before = svm
        .get_account(&destination.pubkey())
        .expect("destination")
        .lamports;
    assert_logs(
        send(
            &mut svm,
            &slash_authority,
            to_ix(
                registry_id(),
                nuvex_oracle_registry::instruction::Slash { amount: 1_001 }.data(),
                nuvex_oracle_registry::accounts::Slash {
                    slash_authority: pk(&slash_authority.pubkey()),
                    registry: pk(&registry_pda()),
                    node: pk(&node_pda),
                    destination: pk(&destination.pubkey()),
                }
                .to_account_metas(None),
            ),
        ),
        "The slash amount exceeds the locked stake",
    );
    send_ok(
        &mut svm,
        &slash_authority,
        to_ix(
            registry_id(),
            nuvex_oracle_registry::instruction::Slash { amount: 400 }.data(),
            nuvex_oracle_registry::accounts::Slash {
                slash_authority: pk(&slash_authority.pubkey()),
                registry: pk(&registry_pda()),
                node: pk(&node_pda),
                destination: pk(&destination.pubkey()),
            }
            .to_account_metas(None),
        ),
    );
    let slashed: NodeAccount = account_data(&svm, &node_pda);
    assert_eq!(slashed.stake, 600);
    assert_eq!(slashed.reputation, 0);
    assert_eq!(
        slashed.status,
        nuvex_oracle_registry::constants::NODE_STATUS_UNSTAKING
    );
    assert_eq!(
        svm.get_account(&destination.pubkey())
            .expect("destination")
            .lamports,
        destination_before + 400
    );
    svm.warp_to_slot(slashed.cooldown_end_slot);
    send_ok(
        &mut svm,
        &node_authority,
        to_ix(
            registry_id(),
            nuvex_oracle_registry::instruction::Withdraw.data(),
            nuvex_oracle_registry::accounts::Withdraw {
                node_authority: pk(&node_authority.pubkey()),
                node: pk(&node_pda),
            }
            .to_account_metas(None),
        ),
    );
    assert_eq!(svm.get_account(&node_pda).expect("node").lamports, rent);

    stake_lamports(&mut svm, &node_authority, 1_000);
    send_ok(
        &mut svm,
        &slash_authority,
        to_ix(
            registry_id(),
            nuvex_oracle_registry::instruction::Slash { amount: 1_000 }.data(),
            nuvex_oracle_registry::accounts::Slash {
                slash_authority: pk(&slash_authority.pubkey()),
                registry: pk(&registry_pda()),
                node: pk(&node_pda),
                destination: pk(&destination.pubkey()),
            }
            .to_account_metas(None),
        ),
    );
    let emptied: NodeAccount = account_data(&svm, &node_pda);
    assert_eq!(emptied.stake, 0);
    assert_eq!(
        emptied.status,
        nuvex_oracle_registry::constants::NODE_STATUS_REGISTERED
    );
    assert_logs(
        send(
            &mut svm,
            &node_authority,
            to_ix(
                registry_id(),
                nuvex_oracle_registry::instruction::Withdraw.data(),
                nuvex_oracle_registry::accounts::Withdraw {
                    node_authority: pk(&node_authority.pubkey()),
                    node: pk(&node_pda),
                }
                .to_account_metas(None),
            ),
        ),
        "The node is not unstaking",
    );
    let pending: OracleRequest = account_data(&svm, &request);
    assert_eq!(pending.status, RequestStatus::Pending.as_u8());
    assert_eq!(
        pending.assigned_node,
        anchor_lang::prelude::Pubkey::default()
    );
}
