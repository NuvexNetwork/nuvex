use nuvex_cli::{execute, CliError};

#[test]
fn operational_commands_are_not_implemented() {
    let cases = [
        ("node", 2_u8),
        ("request", 1),
        ("registry", 1),
        ("staking", 3),
        ("network", 4),
    ];
    for (command, milestone) in cases {
        let error = execute(["nuvex", command]).expect_err(command);
        assert_eq!(error, CliError::NotImplemented { command, milestone });
    }
}
