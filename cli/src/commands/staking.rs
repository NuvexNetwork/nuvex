use crate::CliError;

pub fn run() -> Result<(), CliError> {
    Err(CliError::NotImplemented {
        command: "staking",
        milestone: 3,
    })
}
