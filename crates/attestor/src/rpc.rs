use crate::error::AttestorError;
use crate::ledger::{RawInstruction, RawTx, TokenBalance};
use crate::token::is_token_program;
use serde_json::{json, Value};
use snapshot::{decode_address, AddressBytes};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_request::RpcRequest;
use solana_commitment_config::CommitmentConfig;
use solana_pubkey::Pubkey;
use std::collections::BTreeSet;

pub struct RpcLedger {
    client: RpcClient,
}

impl RpcLedger {
    pub fn new(rpc_url: &str) -> Self {
        Self {
            client: RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::finalized()),
        }
    }

    pub fn fetch_mint_account(&self, mint: &AddressBytes) -> Result<Vec<u8>, AttestorError> {
        let pk = Pubkey::new_from_array(*mint);
        let acct = self
            .client
            .get_account(&pk)
            .map_err(|e| AttestorError::Rpc(e.to_string()))?;
        Ok(acct.data)
    }

    pub fn fetch_history(
        &self,
        mint: &AddressBytes,
        record_slot: u64,
    ) -> Result<Vec<RawTx>, AttestorError> {
        let mint_pk = encode(mint);
        let mut sigs = self.signatures_for(&mint_pk, record_slot)?;
        let mut txs = Vec::new();
        let mut discovered: BTreeSet<AddressBytes> = BTreeSet::new();

        for sig in &sigs {
            if let Some(raw) = self.transaction(sig)? {
                for op in raw.ops() {
                    for acc in crate::token::discovery_accounts(&op, mint) {
                        discovered.insert(acc);
                    }
                }
                txs.push(raw);
            }
        }

        for acc in &discovered {
            let extra = self.signatures_for(&encode(acc), record_slot)?;
            for sig in extra {
                if !sigs.iter().any(|s| s == &sig) {
                    sigs.push(sig.clone());
                    if let Some(raw) = self.transaction(&sig)? {
                        txs.push(raw);
                    }
                }
            }
        }
        Ok(txs)
    }

    fn signatures_for(&self, address: &str, record_slot: u64) -> Result<Vec<String>, AttestorError> {
        let mut out = Vec::new();
        let mut before: Option<String> = None;
        loop {
            let mut cfg = json!({
                "commitment": "finalized",
                "limit": 1000,
            });
            if let Some(b) = &before {
                cfg["before"] = json!(b);
            }
            let val: Value = self
                .client
                .send(
                    RpcRequest::GetSignaturesForAddress,
                    json!([address, cfg]),
                )
                .map_err(|e| AttestorError::Rpc(e.to_string()))?;
            let list = val.as_array().cloned().unwrap_or_default();
            if list.is_empty() {
                break;
            }
            let last_sig = list
                .last()
                .and_then(|x| x.get("signature"))
                .and_then(|s| s.as_str())
                .map(|s| s.to_string());
            let mut all_older = true;
            for item in &list {
                let slot = item.get("slot").and_then(|s| s.as_u64()).unwrap_or(0);
                let sig = item
                    .get("signature")
                    .and_then(|s| s.as_str())
                    .unwrap_or_default()
                    .to_string();
                if slot <= record_slot {
                    out.push(sig);
                    all_older = false;
                } else {
                    all_older = false;
                }
            }
            let Some(last) = last_sig else { break };
            before = Some(last);
            if list.len() < 1000 && all_older {
                break;
            }
            if list.len() < 1000 {
                break;
            }
        }
        Ok(out)
    }

    fn transaction(&self, signature: &str) -> Result<Option<RawTx>, AttestorError> {
        let val: Value = match self.client.send(
            RpcRequest::GetTransaction,
            json!([
                signature,
                {
                    "encoding": "json",
                    "commitment": "finalized",
                    "maxSupportedTransactionVersion": 0
                }
            ]),
        ) {
            Ok(v) => v,
            Err(e) => return Err(AttestorError::Rpc(e.to_string())),
        };
        if val.is_null() {
            return Ok(None);
        }
        Ok(Some(parse_rpc_tx(&val)?))
    }
}

fn encode(a: &AddressBytes) -> String {
    snapshot::encode_address(a)
}

pub fn parse_rpc_tx(val: &Value) -> Result<RawTx, AttestorError> {
    let slot = val.get("slot").and_then(|s| s.as_u64()).unwrap_or(0);
    let meta = val.get("meta");
    let failed = meta
        .and_then(|m| m.get("err"))
        .map(|e| !e.is_null())
        .unwrap_or(false);
    let tx = val
        .get("transaction")
        .ok_or_else(|| AttestorError::Rpc("missing transaction".into()))?;
    let message = tx
        .get("message")
        .ok_or_else(|| AttestorError::Rpc("missing message".into()))?;
    let keys = parse_account_keys(message)?;
    let mut instructions = parse_compiled_ixs(message.get("instructions"), &keys)?;
    if let Some(inner) = meta.and_then(|m| m.get("innerInstructions")).and_then(|v| v.as_array())
    {
        for group in inner {
            let inner_ixs = group.get("instructions");
            instructions.extend(parse_compiled_ixs(inner_ixs, &keys)?);
        }
    }
    let pre = parse_balances(meta.and_then(|m| m.get("preTokenBalances")), &keys)?;
    let post = parse_balances(meta.and_then(|m| m.get("postTokenBalances")), &keys)?;
    Ok(RawTx {
        slot,
        index: 0,
        failed,
        instructions,
        pre_token_balances: pre,
        post_token_balances: post,
    })
}

fn parse_account_keys(message: &Value) -> Result<Vec<AddressBytes>, AttestorError> {
    let keys = message
        .get("accountKeys")
        .and_then(|v| v.as_array())
        .ok_or_else(|| AttestorError::Rpc("missing accountKeys".into()))?;
    keys.iter()
        .map(|k| {
            let s = if let Some(s) = k.as_str() {
                s
            } else {
                k.get("pubkey")
                    .and_then(|p| p.as_str())
                    .ok_or_else(|| AttestorError::Rpc("bad account key".into()))?
            };
            decode_address(s).map_err(|e| AttestorError::Address(e.to_string()))
        })
        .collect()
}

fn parse_compiled_ixs(
    ixs: Option<&Value>,
    keys: &[AddressBytes],
) -> Result<Vec<RawInstruction>, AttestorError> {
    let Some(arr) = ixs.and_then(|v| v.as_array()) else {
        return Ok(vec![]);
    };
    let mut out = Vec::new();
    for ix in arr {
        if let Some(pid) = ix.get("programId").and_then(|s| s.as_str()) {
            let program_id = decode_address(pid).map_err(|e| AttestorError::Address(e.to_string()))?;
            let accounts = ix
                .get("accounts")
                .and_then(|a| a.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str())
                        .filter_map(|s| decode_address(s).ok())
                        .collect()
                })
                .unwrap_or_default();
            let data = ix
                .get("data")
                .and_then(|d| d.as_str())
                .and_then(|s| bs58::decode(s).into_vec().ok())
                .unwrap_or_default();
            if is_token_program(&program_id) || !data.is_empty() {
                out.push(RawInstruction {
                    program_id,
                    accounts,
                    data,
                });
            }
            continue;
        }
        let program_id_index = ix
            .get("programIdIndex")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        let program_id = *keys
            .get(program_id_index)
            .ok_or_else(|| AttestorError::Rpc("programIdIndex oob".into()))?;
        let accounts = ix
            .get("accounts")
            .and_then(|a| a.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_u64())
                    .filter_map(|i| keys.get(i as usize).copied())
                    .collect()
            })
            .unwrap_or_default();
        let data = ix
            .get("data")
            .and_then(|d| d.as_str())
            .and_then(|s| bs58::decode(s).into_vec().ok())
            .unwrap_or_default();
        out.push(RawInstruction {
            program_id,
            accounts,
            data,
        });
    }
    Ok(out)
}

fn parse_balances(
    val: Option<&Value>,
    keys: &[AddressBytes],
) -> Result<Vec<TokenBalance>, AttestorError> {
    let Some(arr) = val.and_then(|v| v.as_array()) else {
        return Ok(vec![]);
    };
    let mut out = Vec::new();
    for b in arr {
        let idx = b.get("accountIndex").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let account = *keys
            .get(idx)
            .ok_or_else(|| AttestorError::Rpc("token balance accountIndex oob".into()))?;
        let mint_s = b
            .get("mint")
            .and_then(|s| s.as_str())
            .ok_or_else(|| AttestorError::Rpc("token balance missing mint".into()))?;
        let owner_s = b.get("owner").and_then(|s| s.as_str()).unwrap_or_default();
        let amount_s = b
            .pointer("/uiTokenAmount/amount")
            .and_then(|s| s.as_str())
            .unwrap_or("0");
        let mint = decode_address(mint_s).map_err(|e| AttestorError::Address(e.to_string()))?;
        let owner = if owner_s.is_empty() {
            [0u8; 32]
        } else {
            decode_address(owner_s).map_err(|e| AttestorError::Address(e.to_string()))?
        };
        let amount: u64 = amount_s.parse().unwrap_or(0);
        out.push(TokenBalance {
            account,
            mint,
            owner,
            amount,
        });
    }
    Ok(out)
}
