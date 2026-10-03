use std::process::Command;

use crate::CliError;

pub fn run() -> Result<(), CliError> {
    run_with_api_url(std::env::var("NUVEX_API_URL").ok())
}

pub fn run_with_api_url(api_url: Option<String>) -> Result<(), CliError> {
    let base = api_url
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or(CliError::ApiUnset)?;
    let endpoint = format!("{}/v1/network", base.trim_end_matches('/'));
    let output = Command::new("curl")
        .args(["-fsS", "--max-time", "10", &endpoint])
        .output()
        .map_err(|error| CliError::ApiFailed(error.to_string()))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr);
        return Err(CliError::ApiFailed(if detail.trim().is_empty() {
            format!("curl exited {}", output.status)
        } else {
            detail.trim().to_string()
        }));
    }
    let body = String::from_utf8_lossy(&output.stdout);
    println!("{body}");
    Ok(())
}
