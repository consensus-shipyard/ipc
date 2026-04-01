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
    // Print the full error chain so nothing is hidden from the user.
    eprintln!("\n❌ Error: {:#}", error);

    // Provide helpful suggestions based on the top-level message.
    let top = error.to_string();
    if let Some(suggestion) = get_error_suggestion(&top) {
        eprintln!("\n💡 Suggestion: {}", suggestion);
    }

    // Suggest documentation for contract-related errors.
    if is_contract_related_error(&top) {
        eprintln!("\n📖 For detailed information about contract errors, see:");
        eprintln!(
            "   https://github.com/consensus-shipyard/ipc/blob/main/docs/ipc/contract-errors.md"
        );
        eprintln!("   or run: ipc-cli --help");
    }

    eprintln!();
}

fn is_contract_related_error(error_msg: &str) -> bool {
    let error_lower = error_msg.to_lowercase();

    // Check for common contract error patterns
    error_lower.contains("contract")
        || error_lower.contains("revert")
        || error_lower.contains("not owner")
        || error_lower.contains("not authorized")
        || error_lower.contains("insufficient")
        || error_lower.contains("invalid")
        || error_lower.contains("already exists")
        || error_lower.contains("not found")
        || error_lower.contains("permission")
        || error_lower.contains("unauthorized")
        || error_lower.contains("subnet")
        || error_lower.contains("validator")
        || error_lower.contains("checkpoint")
        || error_lower.contains("batch")
        || error_lower.contains("signature")
        || error_lower.contains("collateral")
        || error_lower.contains("balance")
        || error_lower.contains("funds")
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
