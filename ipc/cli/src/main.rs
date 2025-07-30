// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

pub mod comet_runner;

#[tokio::main]
async fn main() {
    if let Err(e) = ipc_cli::cli().await {
        print_user_friendly_error(&e);
        std::process::exit(1);
    }
}

fn print_user_friendly_error(error: &anyhow::Error) {
    // Extract the most meaningful error message
    let error_msg = extract_meaningful_error(error);

    // Print a clean, user-friendly error message
    eprintln!("\n❌ Error: {}", error_msg);

    // Provide helpful suggestions based on the error type
    if let Some(suggestion) = get_error_suggestion(&error_msg) {
        eprintln!("\n💡 Suggestion: {}", suggestion);
    }

    // For debugging, show the full error chain if RUST_BACKTRACE is set
    if std::env::var("RUST_BACKTRACE").is_ok() {
        eprintln!("\n🔍 Full error details:");
        eprintln!("{}", error);
    }

    eprintln!(); // Add spacing for better readability
}

fn extract_meaningful_error(error: &anyhow::Error) -> String {
    // Get the root cause of the error chain
    let current = error;
    let mut root_cause = error.to_string();

    while let Some(source) = current.source() {
        root_cause = source.to_string();
        // We can't directly assign source to current since they have different types
        // Instead, we'll just keep track of the root cause
        break; // For now, just get the first source to avoid the type issue
    }

    // Clean up common error patterns
    let cleaned = root_cause
        .replace("error processing command Some(", "")
        .replace("main process failed: ", "")
        .replace(": ", ": ")
        .trim()
        .to_string();

    // Special handling for contract revert errors
    if cleaned.contains("Contract call reverted with data:") {
        // Provide a generic but helpful message
        return "Contract operation failed. The transaction was reverted by the smart contract."
            .to_string();
    }

    // If the cleaned message is significantly shorter, use it
    if cleaned.len() < root_cause.len() * 2 / 3 {
        cleaned
    } else {
        root_cause
    }
}

fn get_error_suggestion(error_msg: &str) -> Option<&'static str> {
    let error_lower = error_msg.to_lowercase();

    if error_lower.contains("no default evm account") {
        Some("Use the --from flag to specify an account address, or configure a default account in your wallet.")
    } else if error_lower.contains("not owner of public key") {
        Some("Make sure you're using the correct account that owns the validator public key.")
    } else if error_lower.contains("insufficient funds") {
        Some("Check your account balance and ensure you have enough tokens for the transaction.")
    } else if error_lower.contains("invalid subnet") {
        Some("Verify the subnet ID format and ensure the subnet exists.")
    } else if error_lower.contains("connection") || error_lower.contains("timeout") {
        Some("Check your network connection and ensure the RPC endpoint is accessible.")
    } else if error_lower.contains("permission") || error_lower.contains("unauthorized") {
        Some("Verify you have the necessary permissions for this operation.")
    } else if error_lower.contains("contract operation failed") {
        Some("The smart contract rejected the transaction. Check the contract requirements and your input parameters.")
    } else if error_lower.contains("contract") && error_lower.contains("reverted") {
        Some("The contract operation failed. Check the error details above for more information.")
    } else {
        None
    }
}
