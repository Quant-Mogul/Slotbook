use attestor::{replay, resolve_owner, RpcLedger, ReplayOpts};
use clap::{Parser, Subcommand};
use snapshot::{
    decode_address, proof_for_owner, AddressBytes, HolderRegister,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "attestor", about = "Slotbook ledger-replay attestor")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Discover token accounts and replay balances to a record slot over RPC.
    Replay {
        #[arg(long)]
        mint: String,
        #[arg(long)]
        slot: u64,
        #[arg(long)]
        rpc_url: String,
    },
    /// Recompute one owner's balance (resolver path).
    Resolve {
        #[arg(long)]
        mint: String,
        #[arg(long)]
        slot: u64,
        #[arg(long)]
        owner: String,
        #[arg(long)]
        rpc_url: String,
        #[arg(long)]
        claimed: Option<u64>,
    },
    /// Build a Merkle proof from a local register JSON (brief D12–D13).
    Proof {
        #[arg(long)]
        register: PathBuf,
        #[arg(long)]
        owner: String,
        #[arg(long)]
        distribution: String,
        #[arg(long)]
        salt_seed: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Replay {
            mint,
            slot,
            rpc_url,
        } => {
            let reg = load_register(&mint, slot, &rpc_url)?;
            println!("{}", serde_json::to_string_pretty(&reg)?);
        }
        Cmd::Resolve {
            mint,
            slot,
            owner,
            rpc_url,
            claimed,
        } => {
            let reg = load_register(&mint, slot, &rpc_url)?;
            let owner = decode_address(&owner)?;
            let balance = resolve_owner(&reg, &owner, claimed)?;
            println!("{balance}");
        }
        Cmd::Proof {
            register,
            owner,
            distribution,
            salt_seed,
        } => {
            let reg: HolderRegister = serde_json::from_str(&std::fs::read_to_string(register)?)?;
            let owner = decode_address(&owner)?;
            let distribution = decode_address(&distribution)?;
            let salt_seed = parse_seed(&salt_seed)?;
            let first = reg.first_covered_slot.unwrap_or(0);
            let bundle = proof_for_owner(
                &reg,
                &distribution,
                &salt_seed,
                &owner,
                first,
                reg.record_slot,
            )?;
            println!("{}", serde_json::to_string_pretty(&bundle)?);
        }
    }
    Ok(())
}

fn load_register(mint: &str, slot: u64, rpc_url: &str) -> anyhow::Result<HolderRegister> {
    let mint = decode_address(mint)?;
    let ledger = RpcLedger::new(rpc_url);
    let txs = ledger.fetch_history(&mint, slot)?;
    Ok(replay(
        &ReplayOpts {
            mint,
            record_slot: slot,
            require_frozen_default: true,
        },
        &txs,
    )?)
}

fn parse_seed(s: &str) -> anyhow::Result<AddressBytes> {
    decode_address(s).map_err(|e| anyhow::anyhow!(e))
}
