use crate::CliError;

pub fn run() -> Result<(), CliError> {
    Err(CliError::NotImplemented {
        command: "registry",
        milestone: 1,
    })
}
