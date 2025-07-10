use crate::types::{ExecutionResult, Transaction, TransactionStatus};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use starknet::{
    core::types::Felt,
    providers::{
        Provider, Url,
        jsonrpc::{HttpTransport, JsonRpcClient},
    },
};

#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: u64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: u64,
    #[serde(flatten)]
    result: JsonRpcResult,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum JsonRpcResult {
    Success { result: TransactionStatusResult },
    Error { error: JsonRpcError },
}

#[derive(Debug, Deserialize)]
struct TransactionStatusResult {
    #[serde(default)]
    execution_status: Option<String>,
    finality_status: String,
    #[serde(default)]
    failure_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

pub struct QueryClient {
    provider: JsonRpcClient<HttpTransport>,
    rpc_url: String,
    client: reqwest::Client,
}

impl QueryClient {
    pub fn new(url: &str) -> Self {
        let provider =
            JsonRpcClient::new(HttpTransport::new(Url::parse(url).expect("Invalid URL")));
        let client = reqwest::Client::new();
        Self {
            provider,
            rpc_url: url.to_string(),
            client,
        }
    }

    pub async fn get_transaction_receipt(&self, tx_hash: &str) -> anyhow::Result<ExecutionResult> {
        let tx_hash = Felt::from_hex(tx_hash)?;
        let exe_res = self
            .provider
            .get_transaction_receipt(tx_hash)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get transaction receipt with error: {:?}", e))?
            .receipt
            .execution_result()
            .clone();
        Ok(exe_res)
    }

    pub async fn get_transaction(&self, tx_hash: &str) -> anyhow::Result<Transaction> {
        let tx_hash = Felt::from_hex(tx_hash)?;
        let tx = self.provider.get_transaction_by_hash(tx_hash).await?;
        Ok(tx)
    }

    pub async fn get_transaction_status(&self, tx_hash: &str) -> anyhow::Result<TransactionStatus> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "starknet_getTransactionStatus".to_string(),
            params: serde_json::json!({
                "transaction_hash": tx_hash
            }),
            id: 1,
        };

        let response = self
            .client
            .post(&self.rpc_url)
            .json(&request)
            .send()
            .await?;

        let json_response: JsonRpcResponse = response.json().await?;

        match json_response.result {
            JsonRpcResult::Success { result } => {
                // Map the JSON response to Starknet's TransactionStatus

                // Handle different response formats based on finality_status
                match result.finality_status.as_str() {
                    "REJECTED" => {
                        let reason = result
                            .failure_reason
                            .unwrap_or_else(|| "Transaction rejected".to_string());
                        Ok(TransactionStatus::Rejected { reason })
                    }
                    "ACCEPTED_ON_L1" | "ACCEPTED_ON_L2" => {
                        if let Some(exec_status) = result.execution_status {
                            let execution_result = match exec_status.as_str() {
                                "SUCCEEDED" => ExecutionResult::Succeeded,
                                "REVERTED" => {
                                    // For reverted transactions, include the failure reason if available
                                    let reason = result
                                        .failure_reason
                                        .unwrap_or_else(|| "Transaction reverted".to_string());
                                    ExecutionResult::Reverted { reason }
                                }
                                _ => {
                                    return Err(anyhow::anyhow!(
                                        "Unknown execution status: {}",
                                        exec_status
                                    ));
                                }
                            };
                            Ok(TransactionStatus::AcceptedOnL2(execution_result))
                        } else {
                            Ok(TransactionStatus::Received)
                        }
                    }
                    _ => {
                        // For other statuses, default to Received
                        Ok(TransactionStatus::Received)
                    }
                }
            }
            JsonRpcResult::Error { error } => {
                if error.code == 29 {
                    Err(anyhow::anyhow!(
                        "Transaction hash not found: {}",
                        error.message
                    ))
                } else {
                    Err(anyhow::anyhow!(
                        "RPC error {}: {}",
                        error.code,
                        error.message
                    ))
                }
            }
        }
    }
}
