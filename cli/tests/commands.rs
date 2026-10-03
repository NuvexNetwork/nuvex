use nuvex_cli::commands::network;
use nuvex_cli::{execute, CliError};

#[test]
fn operational_commands_are_not_implemented() {
    let cases = [
        ("node", 2_u8),
        ("request", 1),
        ("registry", 1),
        ("staking", 3),
    ];
    for (command, milestone) in cases {
        let error = execute(["nuvex", command]).expect_err(command);
        assert_eq!(error, CliError::NotImplemented { command, milestone });
    }
}

#[test]
fn network_refuses_to_invent_status_without_an_api_url() {
    assert_eq!(
        network::run_with_api_url(None).expect_err("unset"),
        CliError::ApiUnset
    );
    assert_eq!(
        network::run_with_api_url(Some("   ".into())).expect_err("blank"),
        CliError::ApiUnset
    );
}
