use crate::error::AttestorError;
use crate::ledger::{RawInstruction, RawTx, TokenBalance};
use crate::token::is_token_program;
use serde_json::{json, Value};
use snapshot::{decode_address, AddressBytes};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_request::RpcRequest;
use solana_commitment_config::CommitmentConfig;
use solana_pubkey::Pubkey;
use std::collections::{BTreeMap, BTreeSet};

pub struct RpcLedger {
    client: RpcClient,
}

impl RpcLedger {
    pub fn new(rpc_url: &str) -> Self {
        Self {
            client: RpcClient::new_with_commitment(
                rpc_url.to_string(),
                CommitmentConfig::finalized(),
            ),
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
        let mut parsed_index: BTreeMap<String, Option<u32>> = BTreeMap::new();

        for sig in &sigs {
            if let Some((raw, idx)) = self.transaction(sig)? {
                for op in raw.ops() {
                    for acc in crate::token::discovery_accounts(&op, mint) {
                        discovered.insert(acc);
                    }
                }
                parsed_index.insert(raw.signature.clone(), idx);
                txs.push(raw);
            }
        }

        for acc in &discovered {
            let extra = self.signatures_for(&encode(acc), record_slot)?;
            for sig in extra {
                if !sigs.iter().any(|s| s == &sig) {
                    sigs.push(sig.clone());
                    if let Some((raw, idx)) = self.transaction(&sig)? {
                        parsed_index.insert(raw.signature.clone(), idx);
                        txs.push(raw);
                    }
                }
            }
        }

        self.assign_block_indices(&mut txs, &parsed_index)?;
        Ok(txs)
    }

    fn signatures_for(
        &self,
        address: &str,
        record_slot: u64,
    ) -> Result<Vec<String>, AttestorError> {
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
                .send(RpcRequest::GetSignaturesForAddress, json!([address, cfg]))
                .map_err(|e| AttestorError::Rpc(e.to_string()))?;
            let list = val.as_array().cloned().ok_or_else(|| {
                AttestorError::Rpc("getSignaturesForAddress did not return an array".into())
            })?;
            if list.is_empty() {
                break;
            }
            let last_sig = list
                .last()
                .and_then(|x| x.get("signature"))
                .and_then(|s| s.as_str())
                .ok_or_else(|| {
                    AttestorError::Rpc("signature list page missing last signature".into())
                })?
                .to_string();
            for item in &list {
                let (slot, sig) = parse_signature_row(item)?;
                if slot <= record_slot {
                    out.push(sig);
                }
            }
            before = Some(last_sig);
            if list.len() < 1000 {
                break;
            }
        }
        Ok(out)
    }

    fn transaction(&self, signature: &str) -> Result<Option<(RawTx, Option<u32>)>, AttestorError> {
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

    fn assign_block_indices(
        &self,
        txs: &mut [RawTx],
        parsed_index: &BTreeMap<String, Option<u32>>,
    ) -> Result<(), AttestorError> {
        let mut block_order: BTreeMap<u64, Vec<String>> = BTreeMap::new();
        let slots: BTreeSet<u64> = txs
            .iter()
            .filter(|tx| parsed_index.get(&tx.signature).copied().flatten().is_none())
            .map(|tx| tx.slot)
            .collect();
        for slot in slots {
            block_order.insert(slot, self.block_signatures(slot)?);
        }
        fill_tx_indices(txs, parsed_index, &block_order)
    }

    fn block_signatures(&self, slot: u64) -> Result<Vec<String>, AttestorError> {
        let val: Value = self
            .client
            .send(
                RpcRequest::GetBlock,
                json!([
                    slot,
                    {
                        "encoding": "json",
                        "transactionDetails": "signatures",
                        "rewards": false,
                        "maxSupportedTransactionVersion": 0,
                        "commitment": "finalized"
                    }
                ]),
            )
            .map_err(|e| AttestorError::Rpc(e.to_string()))?;
        if val.is_null() {
            return Err(AttestorError::Rpc(format!(
                "getBlock({slot}) returned null"
            )));
        }
        parse_block_signatures(&val)
    }
}

fn encode(a: &AddressBytes) -> String {
    snapshot::encode_address(a)
}

/// Apply `transactionIndex` when present; otherwise the signature's position in
/// `getBlock(slot).transactions`. Errors if a signature is missing from that block.
pub fn fill_tx_indices(
    txs: &mut [RawTx],
    parsed_index: &BTreeMap<String, Option<u32>>,
    block_order: &BTreeMap<u64, Vec<String>>,
) -> Result<(), AttestorError> {
    for tx in txs.iter_mut() {
        if let Some(i) = parsed_index.get(&tx.signature).copied().flatten() {
            tx.index = i;
            continue;
        }
        let sigs = block_order.get(&tx.slot).ok_or_else(|| {
            AttestorError::Rpc(format!("missing block signatures for slot {}", tx.slot))
        })?;
        let pos = sigs
            .iter()
            .position(|s| s == &tx.signature)
            .ok_or_else(|| {
                AttestorError::Rpc(format!(
                    "signature {} not found in block {}",
                    tx.signature, tx.slot
                ))
            })?;
        tx.index = pos as u32;
    }
    Ok(())
}

pub fn parse_signature_row(item: &Value) -> Result<(u64, String), AttestorError> {
    let slot = required_u64(item, "slot")?;
    let sig = required_str(item, "signature")?.to_string();
    Ok((slot, sig))
}

fn required_str<'a>(obj: &'a Value, key: &str) -> Result<&'a str, AttestorError> {
    obj.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| AttestorError::Rpc(format!("missing or invalid {key}")))
}

fn required_u64(obj: &Value, key: &str) -> Result<u64, AttestorError> {
    obj.get(key)
        .and_then(|v| v.as_u64())
        .ok_or_else(|| AttestorError::Rpc(format!("missing or invalid {key}")))
}

pub fn parse_block_signatures(val: &Value) -> Result<Vec<String>, AttestorError> {
    let txs = val
        .get("transactions")
        .and_then(|v| v.as_array())
        .ok_or_else(|| AttestorError::Rpc("getBlock missing transactions".into()))?;
    let mut out = Vec::with_capacity(txs.len());
    for (i, tx) in txs.iter().enumerate() {
        let sig = if let Some(s) = tx.as_str() {
            s.to_string()
        } else if let Some(arr) = tx.as_array() {
            arr.first()
                .and_then(|s| s.as_str())
                .ok_or_else(|| AttestorError::Rpc(format!("block tx {i} missing signature")))?
                .to_string()
        } else {
            tx.pointer("/transaction/signatures/0")
                .and_then(|s| s.as_str())
                .or_else(|| {
                    tx.get("signatures")
                        .and_then(|s| s.as_array())
                        .and_then(|a| a.first())
                        .and_then(|s| s.as_str())
                })
                .ok_or_else(|| AttestorError::Rpc(format!("block tx {i} missing signature")))?
                .to_string()
        };
        out.push(sig);
    }
    Ok(out)
}

/// Returns the parsed tx and an optional index from the RPC payload.
pub fn parse_rpc_tx(val: &Value) -> Result<(RawTx, Option<u32>), AttestorError> {
    let slot = val
        .get("slot")
        .and_then(|s| s.as_u64())
        .ok_or_else(|| AttestorError::Rpc("transaction missing slot".into()))?;
    let index = val
        .get("transactionIndex")
        .or_else(|| val.get("index"))
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);
    let meta = val.get("meta");
    let failed = match meta.and_then(|m| m.get("err")) {
        None => false,
        Some(e) => !e.is_null(),
    };
    let tx = val
        .get("transaction")
        .ok_or_else(|| AttestorError::Rpc("missing transaction".into()))?;
    let signature = tx
        .get("signatures")
        .and_then(|s| s.as_array())
        .and_then(|a| a.first())
        .and_then(|s| s.as_str())
        .ok_or_else(|| AttestorError::Rpc("transaction missing signature".into()))?
        .to_string();
    let message = tx
        .get("message")
        .ok_or_else(|| AttestorError::Rpc("missing message".into()))?;
    let keys = parse_account_keys(message)?;
    let mut instructions = parse_compiled_ixs(message.get("instructions"), &keys)?;
    if let Some(inner) = meta
        .and_then(|m| m.get("innerInstructions"))
        .and_then(|v| v.as_array())
    {
        for group in inner {
            let inner_ixs = group.get("instructions");
            instructions.extend(parse_compiled_ixs(inner_ixs, &keys)?);
        }
    }
    let pre = parse_balances(meta.and_then(|m| m.get("preTokenBalances")), &keys)?;
    let post = parse_balances(meta.and_then(|m| m.get("postTokenBalances")), &keys)?;
    Ok((
        RawTx {
            slot,
            index: index.unwrap_or(0),
            signature,
            failed,
            instructions,
            pre_token_balances: pre,
            post_token_balances: post,
        },
        index,
    ))
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
            let program_id =
                decode_address(pid).map_err(|e| AttestorError::Address(e.to_string()))?;
            let accounts = parse_pubkey_account_list(ix.get("accounts"))?;
            let data = parse_ix_data(ix)?;
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
            .ok_or_else(|| AttestorError::Rpc("instruction missing programIdIndex".into()))?
            as usize;
        let program_id = *keys
            .get(program_id_index)
            .ok_or_else(|| AttestorError::Rpc("programIdIndex out of range".into()))?;
        let accounts = parse_index_account_list(ix.get("accounts"), keys)?;
        let data = parse_ix_data(ix)?;
        out.push(RawInstruction {
            program_id,
            accounts,
            data,
        });
    }
    Ok(out)
}

fn parse_pubkey_account_list(val: Option<&Value>) -> Result<Vec<AddressBytes>, AttestorError> {
    let Some(arr) = val.and_then(|v| v.as_array()) else {
        return Ok(vec![]);
    };
    arr.iter()
        .map(|x| {
            let s = x.as_str().ok_or_else(|| {
                AttestorError::Rpc("instruction account is not a pubkey string".into())
            })?;
            decode_address(s).map_err(|e| AttestorError::Address(e.to_string()))
        })
        .collect()
}

fn parse_index_account_list(
    val: Option<&Value>,
    keys: &[AddressBytes],
) -> Result<Vec<AddressBytes>, AttestorError> {
    let Some(arr) = val.and_then(|v| v.as_array()) else {
        return Ok(vec![]);
    };
    arr.iter()
        .map(|x| {
            let i = x
                .as_u64()
                .ok_or_else(|| AttestorError::Rpc("instruction account is not an index".into()))?
                as usize;
            keys.get(i)
                .copied()
                .ok_or_else(|| AttestorError::Rpc(format!("account index {i} out of range")))
        })
        .collect()
}

fn parse_ix_data(ix: &Value) -> Result<Vec<u8>, AttestorError> {
    let Some(s) = ix.get("data").and_then(|d| d.as_str()) else {
        return Ok(vec![]);
    };
    bs58::decode(s)
        .into_vec()
        .map_err(|e| AttestorError::Rpc(format!("invalid instruction data: {e}")))
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
        let idx = b
            .get("accountIndex")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| AttestorError::Rpc("token balance missing accountIndex".into()))?
            as usize;
        let account = *keys.get(idx).ok_or_else(|| {
            AttestorError::Rpc(format!("token balance accountIndex {idx} out of range"))
        })?;
        let mint_s = b
            .get("mint")
            .and_then(|s| s.as_str())
            .ok_or_else(|| AttestorError::Rpc("token balance missing mint".into()))?;
        let owner_s = b
            .get("owner")
            .and_then(|s| s.as_str())
            .ok_or_else(|| AttestorError::Rpc("token balance missing owner".into()))?;
        if owner_s.is_empty() {
            return Err(AttestorError::Rpc("token balance owner is empty".into()));
        }
        let amount_s = b
            .pointer("/uiTokenAmount/amount")
            .and_then(|s| s.as_str())
            .ok_or_else(|| AttestorError::Rpc("token balance missing amount".into()))?;
        let mint = decode_address(mint_s).map_err(|e| AttestorError::Address(e.to_string()))?;
        let owner = decode_address(owner_s).map_err(|e| AttestorError::Address(e.to_string()))?;
        let amount: u64 = amount_s.parse().map_err(|_| {
            AttestorError::Rpc(format!("token balance amount is not u64: {amount_s}"))
        })?;
        out.push(TokenBalance {
            account,
            mint,
            owner,
            amount,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_keys() -> (String, String, String) {
        let a = snapshot::encode_address(&[1u8; 32]);
        let b = snapshot::encode_address(&[2u8; 32]);
        let c = snapshot::encode_address(&[3u8; 32]);
        (a, b, c)
    }

    #[test]
    fn parse_rpc_tx_reads_transaction_index() {
        let (k0, k1, k2) = sample_keys();
        let val = json!({
            "slot": 42,
            "transactionIndex": 7,
            "transaction": {
                "signatures": ["sigA"],
                "message": {
                    "accountKeys": [k0, k1, k2],
                    "instructions": [{
                        "programIdIndex": 0,
                        "accounts": [1, 2],
                        "data": ""
                    }]
                }
            },
            "meta": { "err": null, "preTokenBalances": [], "postTokenBalances": [] }
        });
        let (tx, idx) = parse_rpc_tx(&val).unwrap();
        assert_eq!(tx.slot, 42);
        assert_eq!(idx, Some(7));
        assert_eq!(tx.signature, "sigA");
    }

    #[test]
    fn parse_rpc_tx_requires_slot() {
        let (k0, _, _) = sample_keys();
        let val = json!({
            "transaction": {
                "signatures": ["sigA"],
                "message": { "accountKeys": [k0], "instructions": [] }
            }
        });
        assert!(parse_rpc_tx(&val).is_err());
    }

    #[test]
    fn drops_bad_account_key_is_error() {
        let val = json!({
            "slot": 1,
            "transaction": {
                "signatures": ["sigA"],
                "message": {
                    "accountKeys": ["not-a-pubkey!!!"],
                    "instructions": []
                }
            },
            "meta": { "err": null }
        });
        assert!(parse_rpc_tx(&val).is_err());
    }

    #[test]
    fn empty_owner_is_error() {
        let (k0, k1, k2) = sample_keys();
        let mint = snapshot::encode_address(&[9u8; 32]);
        let val = json!({
            "slot": 1,
            "transaction": {
                "signatures": ["sigA"],
                "message": {
                    "accountKeys": [k0, k1, k2],
                    "instructions": []
                }
            },
            "meta": {
                "err": null,
                "postTokenBalances": [{
                    "accountIndex": 1,
                    "mint": mint,
                    "owner": "",
                    "uiTokenAmount": { "amount": "10" }
                }]
            }
        });
        assert!(parse_rpc_tx(&val).is_err());
    }

    #[test]
    fn invalid_ix_data_is_error() {
        let (k0, k1, k2) = sample_keys();
        let val = json!({
            "slot": 1,
            "transaction": {
                "signatures": ["sigA"],
                "message": {
                    "accountKeys": [k0, k1, k2],
                    "instructions": [{
                        "programIdIndex": 0,
                        "accounts": [1, 2],
                        "data": "!!not-bs58!!"
                    }]
                }
            },
            "meta": { "err": null }
        });
        assert!(parse_rpc_tx(&val).is_err());
    }

    #[test]
    fn missing_owner_is_error() {
        let (k0, k1, k2) = sample_keys();
        let mint = snapshot::encode_address(&[9u8; 32]);
        let val = json!({
            "slot": 1,
            "transaction": {
                "signatures": ["sigA"],
                "message": {
                    "accountKeys": [k0, k1, k2],
                    "instructions": []
                }
            },
            "meta": {
                "err": null,
                "postTokenBalances": [{
                    "accountIndex": 1,
                    "mint": mint,
                    "uiTokenAmount": { "amount": "10" }
                }]
            }
        });
        assert!(parse_rpc_tx(&val).is_err());
    }

    #[test]
    fn unparsable_amount_is_error() {
        let (k0, k1, k2) = sample_keys();
        let mint = snapshot::encode_address(&[9u8; 32]);
        let val = json!({
            "slot": 1,
            "transaction": {
                "signatures": ["sigA"],
                "message": {
                    "accountKeys": [k0, k1, k2],
                    "instructions": []
                }
            },
            "meta": {
                "err": null,
                "postTokenBalances": [{
                    "accountIndex": 1,
                    "mint": mint,
                    "owner": k2,
                    "uiTokenAmount": { "amount": "nope" }
                }]
            }
        });
        assert!(parse_rpc_tx(&val).is_err());
    }

    #[test]
    fn oob_account_index_is_error() {
        let (k0, k1, k2) = sample_keys();
        let mint = snapshot::encode_address(&[9u8; 32]);
        let val = json!({
            "slot": 1,
            "transaction": {
                "signatures": ["sigA"],
                "message": {
                    "accountKeys": [k0, k1, k2],
                    "instructions": []
                }
            },
            "meta": {
                "err": null,
                "postTokenBalances": [{
                    "accountIndex": 99,
                    "mint": mint,
                    "owner": k2,
                    "uiTokenAmount": { "amount": "1" }
                }]
            }
        });
        assert!(parse_rpc_tx(&val).is_err());
    }

    #[test]
    fn compiled_ix_bad_account_key_does_not_shift_indexes() {
        let (k0, k1, _) = sample_keys();
        let val = json!({
            "slot": 1,
            "transaction": {
                "signatures": ["sigA"],
                "message": {
                    "accountKeys": [k0, k1],
                    "instructions": [{
                        "programId": k0,
                        "accounts": ["not-valid", k1],
                        "data": ""
                    }]
                }
            },
            "meta": { "err": null }
        });
        assert!(parse_rpc_tx(&val).is_err());
    }

    #[test]
    fn block_signatures_from_signature_lists() {
        let val = json!({
            "transactions": [
                ["sig0", "sig0b"],
                ["sig1"]
            ]
        });
        assert_eq!(parse_block_signatures(&val).unwrap(), vec!["sig0", "sig1"]);
    }

    #[test]
    fn fill_tx_indices_uses_block_position_when_rpc_omits_index() {
        let mut txs = vec![
            RawTx {
                slot: 9,
                index: 0,
                signature: "sigB".into(),
                failed: false,
                instructions: vec![],
                pre_token_balances: vec![],
                post_token_balances: vec![],
            },
            RawTx {
                slot: 9,
                index: 0,
                signature: "sigA".into(),
                failed: false,
                instructions: vec![],
                pre_token_balances: vec![],
                post_token_balances: vec![],
            },
        ];
        let parsed = BTreeMap::from([("sigA".into(), None), ("sigB".into(), None)]);
        let blocks = BTreeMap::from([(9u64, vec!["sigA".into(), "sigB".into()])]);
        fill_tx_indices(&mut txs, &parsed, &blocks).unwrap();
        let a = txs.iter().find(|t| t.signature == "sigA").unwrap();
        let b = txs.iter().find(|t| t.signature == "sigB").unwrap();
        assert_eq!(a.index, 0);
        assert_eq!(b.index, 1);
    }

    #[test]
    fn fill_tx_indices_errors_if_signature_not_in_block() {
        let mut txs = vec![RawTx {
            slot: 9,
            index: 0,
            signature: "missing".into(),
            failed: false,
            instructions: vec![],
            pre_token_balances: vec![],
            post_token_balances: vec![],
        }];
        let parsed = BTreeMap::from([("missing".into(), None)]);
        let blocks = BTreeMap::from([(9u64, vec!["other".into()])]);
        assert!(fill_tx_indices(&mut txs, &parsed, &blocks).is_err());
    }

    #[test]
    fn missing_program_id_index_is_error() {
        let (k0, k1, k2) = sample_keys();
        let val = json!({
            "slot": 1,
            "transaction": {
                "signatures": ["sigA"],
                "message": {
                    "accountKeys": [k0, k1, k2],
                    "instructions": [{
                        "accounts": [1, 2],
                        "data": ""
                    }]
                }
            },
            "meta": { "err": null }
        });
        assert!(parse_rpc_tx(&val).is_err());
    }

    #[test]
    fn signature_row_requires_signature_and_slot() {
        assert!(parse_signature_row(&json!({"slot": 1})).is_err());
        assert!(parse_signature_row(&json!({"signature": "abc"})).is_err());
        let (slot, sig) = parse_signature_row(&json!({"slot": 3, "signature": "abc"})).unwrap();
        assert_eq!(slot, 3);
        assert_eq!(sig, "abc");
    }
}
