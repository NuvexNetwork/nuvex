use nuvex_common::{
    seed_is_valid, CHALLENGE_SEED, JOB_CONFIG_SEED, MAX_SEED_LEN, NODE_SEED, PROTOCOL_SEED,
    REGISTRY_SEED, REQUEST_SEED, REWARD_VAULT_SEED, VERIFICATION_SEED,
};

const FIXTURE: &str = include_str!("../../../tests/fixtures/pda-seeds.json");

#[test]
fn seeds_match_the_shared_fixture() {
    let parsed: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(FIXTURE).expect("pda seed fixture is json");
    let expected = [
        ("protocol", PROTOCOL_SEED),
        ("registry", REGISTRY_SEED),
        ("node", NODE_SEED),
        ("request", REQUEST_SEED),
        ("verification", VERIFICATION_SEED),
        ("challenge", CHALLENGE_SEED),
        ("reward_vault", REWARD_VAULT_SEED),
        ("job_config", JOB_CONFIG_SEED),
    ];
    assert_eq!(parsed.len(), expected.len());
    for (name, seed) in expected {
        let value = parsed
            .get(name)
            .and_then(|value| value.as_str())
            .expect("fixture entry");
        assert_eq!(value.as_bytes(), seed);
        assert!(seed_is_valid(seed));
        assert!(seed.len() <= MAX_SEED_LEN);
    }
}
