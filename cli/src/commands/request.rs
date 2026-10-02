use crate::CliError;

pub fn run() -> Result<(), CliError> {
    Err(CliError::NotImplemented {
        command: "request",
        milestone: 1,
    })
}
