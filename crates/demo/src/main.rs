//! slotbook-demo: runs the Slotbook demo, steps 1 to 8, against a live cluster and
//! writes every transaction link to `<out>/DEMO_RUN.md`.
//!
//! It creates everything it needs, so each run is independent:
//! - a Token-2022 permissioned mint with DefaultAccountState = Frozen, handed to
//!   Token ACL (create_config), so the program's init_issuer checks pass (D15, D22);
//! - two legacy SPL test mints standing in for USDC (payment and bonds);
//! - three holders, two attestors, a resolver and two challengers, funded by the issuer.
//!
//! Freezing and thawing use Token ACL's authority `freeze` / `thaw`; in production
//! the gate's permissionless thaw does the thawing (demo step 7 says "the gate thaws").
//! The register is built from the balances this run sets up; the attestor crate is the
//! independent replay. Step 3 recomputes the root with scripts/reproduce_root.py.
//!
//! Prerequisites: the Slotbook program deployed at its declared id, Token ACL deployed
//! (it is on devnet; on a local validator clone it), and a funded issuer keypair.

use std::{path::PathBuf, process::Command, thread::sleep, time::Duration};

use anchor_lang::{
    prelude::Pubkey,
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program_pack::Pack,
        system_instruction, system_program,
    },
    AccountDeserialize, InstructionData, ToAccountMetas,
};
use anchor_spl::{
    associated_token::{
        self, get_associated_token_address_with_program_id,
        spl_associated_token_account::instruction::create_associated_token_account,
    },
    token::{self, spl_token},
    token_2022::{
        self,
        spl_token_2022::{
            extension::{default_account_state, ExtensionType},
            state::{AccountState, Mint as T22Mint},
        },
    },
};
use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use merkle::MerkleTree;
use slotbook::{
    state::{Backend, CommitArgs, Distribution, IssuerParams, Pending, Ruling},
    SPEC_VERSION, TOKEN_ACL_MINT_CONFIG_SEED, TOKEN_ACL_PROGRAM_ID,
};
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_keypair::{read_keypair_file, Keypair};
use solana_signer::Signer;
use solana_transaction::Transaction;

// Payment and bond tokens: 6 decimals, like USDC.
const ONE: u64 = 1_000_000;
const TOTAL: u64 = 1_000 * ONE;
const ATTESTOR_BOND: u64 = 10 * ONE;
const CHALLENGER_BOND: u64 = 10 * ONE;
const RESOLVER_FEE: u64 = ONE;
const ACTOR_SOL: u64 = 50_000_000; // 0.05 SOL each for rent and fees

#[derive(Parser)]
#[command(about = "Run the Slotbook demo (steps 1 to 8) against a cluster")]
struct Args {
    /// RPC endpoint.
    #[arg(long, default_value = "http://127.0.0.1:8899")]
    rpc_url: String,
    /// Issuer keypair; it also pays for everything. Default: ~/.config/solana/id.json.
    #[arg(long)]
    keypair: Option<PathBuf>,
    /// Folder for DEMO_RUN.md and the register JSON.
    #[arg(long, default_value = "demo-out")]
    out: PathBuf,
    /// Slots between declare and the record slot (D14).
    #[arg(long, default_value_t = 150)]
    notice_slots: u64,
    #[arg(long, default_value_t = 32)]
    finality_margin_slots: u64,
    #[arg(long, default_value_t = 150)]
    challenge_window_slots: u64,
    #[arg(long, default_value_t = 150)]
    resolve_timeout_slots: u64,
    #[arg(long, default_value_t = 150)]
    hold_window_slots: u64,
    #[arg(long, default_value_t = 400)]
    claim_expiry_slots: u64,
}

struct Holder {
    name: &'static str,
    kp: Keypair,
    mint_ata: Pubkey,
    pay_ata: Pubkey,
}

struct Attestor {
    kp: Keypair,
    bond_ata: Pubkey,
    account: Pubkey,
}

struct Run {
    rpc: RpcClient,
    url: String,
    issuer: Keypair,
    log: Vec<String>,
}

impl Run {
    fn explorer(&self, sig: &str) -> String {
        if self.url.contains("devnet") {
            format!("https://explorer.solana.com/tx/{sig}?cluster=devnet")
        } else {
            let enc = self.url.replace(':', "%3A").replace('/', "%2F");
            format!("https://explorer.solana.com/tx/{sig}?cluster=custom&customUrl={enc}")
        }
    }

    fn note(&mut self, line: impl Into<String>) {
        let line = line.into();
        println!("{line}");
        self.log.push(line);
    }

    /// Sends one transaction; the first signer pays. Logs the explorer link.
    fn send(&mut self, label: &str, ixs: &[Instruction], signers: &[&Keypair]) -> Result<()> {
        let blockhash = self.rpc.get_latest_blockhash()?;
        let tx = Transaction::new_signed_with_payer(ixs, Some(&signers[0].pubkey()), signers, blockhash);
        let sig = self
            .rpc
            .send_and_confirm_transaction(&tx)
            .map_err(|e| anyhow!("{label} failed: {e}"))?;
        let link = self.explorer(&sig.to_string());
        self.note(format!("- {label}: [tx]({link})"));
        Ok(())
    }

    fn slot(&self) -> Result<u64> {
        Ok(self.rpc.get_slot()?)
    }

    fn wait_for_slot(&mut self, target: u64, why: &str) -> Result<()> {
        let now = self.slot()?;
        if now < target {
            println!("  waiting {} slots ({why}) ...", target - now);
        }
        while self.slot()? < target {
            sleep(Duration::from_millis(400));
        }
        Ok(())
    }

    fn read<T: AccountDeserialize>(&self, address: &Pubkey) -> Result<T> {
        let data = self.rpc.get_account_data(address)?;
        T::try_deserialize(&mut data.as_slice()).map_err(|e| anyhow!("decode {address}: {e}"))
    }

    fn token_amount(&self, address: &Pubkey) -> Result<u64> {
        let data = self.rpc.get_account_data(address)?;
        Ok(spl_token::state::Account::unpack(&data[..165])?.amount)
    }
}

fn slotbook_ix<A: ToAccountMetas, D: InstructionData>(accounts: A, data: D) -> Instruction {
    Instruction { program_id: slotbook::id(), accounts: accounts.to_account_metas(None), data: data.data() }
}

fn pda(seeds: &[&[u8]]) -> Pubkey {
    Pubkey::find_program_address(seeds, &slotbook::id()).0
}

/// Token ACL authority instructions (tags from solana-foundation/token-acl clients/js).
fn acl_create_config(payer: &Pubkey, authority: &Pubkey, mint: &Pubkey, mint_config: &Pubkey) -> Instruction {
    let mut data = vec![0u8];
    data.extend_from_slice(Pubkey::default().as_ref()); // no gating program
    Instruction {
        program_id: TOKEN_ACL_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new_readonly(*authority, true),
            AccountMeta::new(*mint, false),
            AccountMeta::new(*mint_config, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token_2022::ID, false),
        ],
        data,
    }
}

fn acl_set_frozen(frozen: bool, authority: &Pubkey, mint: &Pubkey, account: &Pubkey, mint_config: &Pubkey) -> Instruction {
    Instruction {
        program_id: TOKEN_ACL_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(*authority, true),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new(*account, false),
            AccountMeta::new_readonly(*mint_config, false),
            AccountMeta::new_readonly(token_2022::ID, false),
        ],
        data: vec![if frozen { 5 } else { 4 }],
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let keypair_path = args
        .keypair
        .clone()
        .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap()).join(".config/solana/id.json"));
    let issuer = read_keypair_file(&keypair_path).map_err(|e| anyhow!("read {keypair_path:?}: {e}"))?;
    let mut run = Run {
        rpc: RpcClient::new_with_commitment(args.rpc_url.clone(), CommitmentConfig::confirmed()),
        url: args.rpc_url.clone(),
        issuer,
        log: Vec::new(),
    };
    std::fs::create_dir_all(&args.out)?;
    let result = demo(&mut run, &args);
    let mut md = String::from("# Slotbook demo run\n\n");
    md.push_str(&format!("Cluster: {}\n\n", args.rpc_url));
    md.push_str(&run.log.join("\n"));
    md.push('\n');
    if let Err(e) = &result {
        md.push_str(&format!("\n**Stopped with an error:** {e:#}\n"));
    }
    std::fs::write(args.out.join("DEMO_RUN.md"), md)?;
    println!("\nWrote {}", args.out.join("DEMO_RUN.md").display());
    result
}

fn demo(run: &mut Run, args: &Args) -> Result<()> {
    let issuer = run.issuer.insecure_clone();
    let ip = issuer.pubkey();

    // ---------------------------------------------------------------- preflight
    let program = run.rpc.get_account(&slotbook::id()).context("Slotbook program is not deployed")?;
    if !program.executable {
        bail!("{} is not an executable program", slotbook::id());
    }
    run.rpc
        .get_account(&TOKEN_ACL_PROGRAM_ID)
        .context("Token ACL is not deployed on this cluster (on a local validator, clone it from devnet)")?;
    let balance = run.rpc.get_balance(&ip)?;
    if balance < 2_000_000_000 {
        bail!("issuer {ip} has {} SOL; need at least 2", balance as f64 / 1e9);
    }
    run.note(format!("Issuer `{ip}`, program `{}`.\n", slotbook::id()));

    // ---------------------------------------------------------------- setup
    run.note("## Setup".to_string());
    let resolver = Keypair::new();
    let challengers = [Keypair::new(), Keypair::new()];
    let mut attestors: Vec<Attestor> = Vec::new();
    let names = ["A (thawed, paid)", "B (frozen, released after thaw)", "C (frozen, swept)"];
    let mut holders: Vec<Holder> = Vec::new();

    // Fund every actor with SOL for rent and fees.
    let att_kps = [Keypair::new(), Keypair::new()];
    let holder_kps = [Keypair::new(), Keypair::new(), Keypair::new()];
    let mut fund = Vec::new();
    for pk in [resolver.pubkey(), challengers[0].pubkey(), challengers[1].pubkey()]
        .into_iter()
        .chain(att_kps.iter().map(|k| k.pubkey()))
        .chain(holder_kps.iter().map(|k| k.pubkey()))
    {
        fund.push(system_instruction::transfer(&ip, &pk, ACTOR_SOL));
    }
    run.send("fund 8 actors with SOL", &fund, &[&issuer])?;

    // Test USDC: payment mint and bond mint (legacy SPL, 6 decimals).
    let legacy_mint = |run: &mut Run, label: &str| -> Result<Pubkey> {
        let kp = Keypair::new();
        let rent = run.rpc.get_minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN)?;
        run.send(
            label,
            &[
                system_instruction::create_account(&ip, &kp.pubkey(), rent, spl_token::state::Mint::LEN as u64, &token::ID),
                spl_token::instruction::initialize_mint2(&token::ID, &kp.pubkey(), &ip, None, 6)?,
            ],
            &[&issuer, &kp],
        )?;
        Ok(kp.pubkey())
    };
    let pay_mint = legacy_mint(run, "create payment mint (test USDC)")?;
    let bond_mint = legacy_mint(run, "create bond mint (test USDC)")?;

    // The permissioned mint: Token-2022, DefaultAccountState = Frozen, decimals 0.
    let mint_kp = Keypair::new();
    let mint = mint_kp.pubkey();
    let len = ExtensionType::try_calculate_account_len::<T22Mint>(&[ExtensionType::DefaultAccountState])?;
    let rent = run.rpc.get_minimum_balance_for_rent_exemption(len)?;
    run.send(
        "create permissioned Token-2022 mint (DefaultAccountState = Frozen)",
        &[
            system_instruction::create_account(&ip, &mint, rent, len as u64, &token_2022::ID),
            default_account_state::instruction::initialize_default_account_state(
                &token_2022::ID,
                &mint,
                &AccountState::Frozen,
            )?,
            token_2022::spl_token_2022::instruction::initialize_mint2(&token_2022::ID, &mint, &ip, Some(&ip), 0)?,
        ],
        &[&issuer, &mint_kp],
    )?;
    let (mint_config, _) =
        Pubkey::find_program_address(&[TOKEN_ACL_MINT_CONFIG_SEED, mint.as_ref()], &TOKEN_ACL_PROGRAM_ID);
    run.send(
        "Token ACL create_config (freeze authority -> MintConfig PDA)",
        &[acl_create_config(&ip, &ip, &mint, &mint_config)],
        &[&issuer],
    )?;

    // Holders: Token-2022 ATAs (ImmutableOwner, start Frozen) and payment ATAs.
    for (i, kp) in holder_kps.into_iter().enumerate() {
        let mint_ata = get_associated_token_address_with_program_id(&kp.pubkey(), &mint, &token_2022::ID);
        let pay_ata = get_associated_token_address_with_program_id(&kp.pubkey(), &pay_mint, &token::ID);
        run.send(
            &format!("holder {} accounts", names[i]),
            &[
                create_associated_token_account(&ip, &kp.pubkey(), &mint, &token_2022::ID),
                create_associated_token_account(&ip, &kp.pubkey(), &pay_mint, &token::ID),
                acl_set_frozen(false, &ip, &mint, &mint_ata, &mint_config),
            ],
            &[&issuer],
        )?;
        holders.push(Holder { name: names[i], kp, mint_ata, pay_ata });
    }

    // Transfer history: mint 700 to A and 300 to B; A sends 150 + 50 to C.
    let t22_mint_to = |to: &Pubkey, amount: u64| {
        token_2022::spl_token_2022::instruction::mint_to_checked(&token_2022::ID, &mint, to, &ip, &[], amount, 0)
    };
    run.send(
        "history: mint 700 to A, 300 to B",
        &[t22_mint_to(&holders[0].mint_ata, 700)?, t22_mint_to(&holders[1].mint_ata, 300)?],
        &[&issuer],
    )?;
    let t22_transfer = |from: &Holder, to: &Holder, amount: u64| {
        token_2022::spl_token_2022::instruction::transfer_checked(
            &token_2022::ID,
            &from.mint_ata,
            &mint,
            &to.mint_ata,
            &from.kp.pubkey(),
            &[],
            amount,
            0,
        )
    };
    let a_kp = holders[0].kp.insecure_clone();
    run.send("history: A -> C 150", &[t22_transfer(&holders[0], &holders[2], 150)?], &[&a_kp])?;
    run.send("history: A -> C 50", &[t22_transfer(&holders[0], &holders[2], 50)?], &[&a_kp])?;

    // Payment funds for the issuer, bond funds for attestors and challengers.
    let issuer_pay = get_associated_token_address_with_program_id(&ip, &pay_mint, &token::ID);
    let issuer_bond = get_associated_token_address_with_program_id(&ip, &bond_mint, &token::ID);
    let resolver_bond = get_associated_token_address_with_program_id(&resolver.pubkey(), &bond_mint, &token::ID);
    let issuer_config = pda(&[b"issuer", mint.as_ref()]);
    let mut ixs = vec![
        create_associated_token_account(&ip, &ip, &pay_mint, &token::ID),
        create_associated_token_account(&ip, &ip, &bond_mint, &token::ID),
        create_associated_token_account(&ip, &resolver.pubkey(), &bond_mint, &token::ID),
        spl_token::instruction::mint_to_checked(&token::ID, &pay_mint, &issuer_pay, &ip, &[], TOTAL, 6)?,
    ];
    for kp in att_kps {
        let bond_ata = get_associated_token_address_with_program_id(&kp.pubkey(), &bond_mint, &token::ID);
        ixs.push(create_associated_token_account(&ip, &kp.pubkey(), &bond_mint, &token::ID));
        ixs.push(spl_token::instruction::mint_to_checked(&token::ID, &bond_mint, &bond_ata, &ip, &[], 2 * ATTESTOR_BOND, 6)?);
        let account = pda(&[b"attestor", issuer_config.as_ref(), kp.pubkey().as_ref()]);
        attestors.push(Attestor { kp, bond_ata, account });
    }
    let mut challenger_bond = Vec::new();
    for c in &challengers {
        let ata = get_associated_token_address_with_program_id(&c.pubkey(), &bond_mint, &token::ID);
        ixs.push(create_associated_token_account(&ip, &c.pubkey(), &bond_mint, &token::ID));
        ixs.push(spl_token::instruction::mint_to_checked(&token::ID, &bond_mint, &ata, &ip, &[], CHALLENGER_BOND, 6)?);
        challenger_bond.push(ata);
    }
    run.send("fund test USDC: issuer total, attestor and challenger bonds", &ixs, &[&issuer])?;

    // ---------------------------------------------------------------- addresses
    let bond_vault = get_associated_token_address_with_program_id(&issuer_config, &bond_mint, &token::ID);
    let distribution = pda(&[b"distribution", issuer_config.as_ref(), &0u64.to_le_bytes()]);
    let vault = get_associated_token_address_with_program_id(&distribution, &pay_mint, &token::ID);
    let addresses = serde_json::json!({
        "program": slotbook::id().to_string(),
        "issuer": ip.to_string(),
        "permissioned_mint": mint.to_string(),
        "token_acl_mint_config": mint_config.to_string(),
        "payment_mint": pay_mint.to_string(),
        "bond_mint": bond_mint.to_string(),
        "issuer_config": issuer_config.to_string(),
        "distribution": distribution.to_string(),
        "distribution_vault": vault.to_string(),
        "bond_vault": bond_vault.to_string(),
        "resolver": resolver.pubkey().to_string(),
        "attestors": attestors.iter().map(|a| a.kp.pubkey().to_string()).collect::<Vec<_>>(),
        "holders": holders.iter().map(|h| serde_json::json!({"name": h.name, "owner": h.kp.pubkey().to_string(), "mint_account": h.mint_ata.to_string()})).collect::<Vec<_>>(),
    });
    std::fs::write(args.out.join("addresses.json"), serde_json::to_string_pretty(&addresses)?)?;
    run.note(format!(
        "\n## Addresses\n\n- permissioned mint `{mint}`\n- issuer config `{issuer_config}`\n- distribution `{distribution}`\n- distribution vault `{vault}`\n- bond vault `{bond_vault}`\n\n(Full list in addresses.json.)"
    ));

    // ---------------------------------------------------------------- step 1
    run.note("\n## Step 1: issuer registers the mint and declares a distribution".to_string());
    let params = IssuerParams {
        attestors: attestors.iter().map(|a| a.kp.pubkey()).collect(),
        quorum: 2,
        require_both_backends: false,
        resolver: resolver.pubkey(),
        notice_slots: args.notice_slots,
        finality_margin_slots: args.finality_margin_slots,
        challenge_window_slots: args.challenge_window_slots,
        resolve_timeout_slots: args.resolve_timeout_slots,
        hold_window_slots: args.hold_window_slots,
        claim_expiry_slots: args.claim_expiry_slots,
        attestor_bond: ATTESTOR_BOND,
        challenger_bond: CHALLENGER_BOND,
        resolver_fee: RESOLVER_FEE,
    };
    run.send(
        "init_issuer",
        &[slotbook_ix(
            slotbook::accounts::InitIssuer {
                authority: ip,
                mint,
                bond_mint,
                issuer_config,
                bond_vault,
                bond_token_program: token::ID,
                associated_token_program: associated_token::ID,
                system_program: system_program::ID,
            },
            slotbook::instruction::InitIssuer { params },
        )],
        &[&issuer],
    )?;
    let register = |run: &mut Run, a: &Attestor| -> Result<()> {
        let kp = a.kp.insecure_clone();
        run.send(
            &format!("register_attestor {}", kp.pubkey()),
            &[slotbook_ix(
                slotbook::accounts::RegisterAttestor {
                    attestor: kp.pubkey(),
                    issuer_config,
                    attestor_account: a.account,
                    bond_mint,
                    bond_vault,
                    attestor_bond_account: a.bond_ata,
                    bond_token_program: token::ID,
                    system_program: system_program::ID,
                },
                slotbook::instruction::RegisterAttestor { backend: Backend::LedgerReplay },
            )],
            &[&kp],
        )
    };
    for a in &attestors {
        register(run, a)?;
    }

    let salt_seed: [u8; 32] = Keypair::new().pubkey().to_bytes();
    let record_slot = run.slot()? + args.notice_slots + 20;
    run.send(
        &format!("declare_distribution (record slot {record_slot})"),
        &[slotbook_ix(
            slotbook::accounts::DeclareDistribution {
                authority: ip,
                issuer_config,
                payment_mint: pay_mint,
                distribution,
                vault,
                issuer_payment_account: issuer_pay,
                payment_token_program: token::ID,
                associated_token_program: associated_token::ID,
                system_program: system_program::ID,
            },
            slotbook::instruction::DeclareDistribution {
                total: TOTAL,
                record_slot,
                salt_seed_hash: merkle::keccak(&[&salt_seed]),
            },
        )],
        &[&issuer],
    )?;
    // History keeps moving after the announcement, before the record slot.
    let b_kp = holders[1].kp.insecure_clone();
    run.send("history after declare: A -> B 10", &[t22_transfer(&holders[0], &holders[1], 10)?], &[&a_kp])?;
    run.send("history after declare: B -> A 10", &[t22_transfer(&holders[1], &holders[0], 10)?], &[&b_kp])?;

    // ---------------------------------------------------------------- register
    let balances = [500u64, 300, 200];
    for (h, want) in holders.iter().zip(balances) {
        let data = run.rpc.get_account_data(&h.mint_ata)?;
        let got = spl_token::state::Account::unpack(&data[..165])?.amount;
        if got != want {
            bail!("holder {} has {got}, expected {want}", h.name);
        }
    }
    let rows_for = |included: &[usize]| -> Vec<(Pubkey, u64)> {
        let mut rows: Vec<(Pubkey, u64)> = included.iter().map(|&i| (holders[i].kp.pubkey(), balances[i])).collect();
        rows.sort_by_key(|(o, _)| o.to_bytes());
        rows
    };
    let salt_of = |o: &Pubkey| merkle::salt(&distribution.to_bytes(), &o.to_bytes(), &salt_seed);
    let tree_of = |rows: &[(Pubkey, u64)]| {
        let leaves = rows.iter().map(|(o, b)| merkle::leaf_hash(&o.to_bytes(), *b, &salt_of(o))).collect();
        MerkleTree::from_leaves(leaves).unwrap()
    };
    let write_register = |name: &str, rows: &[(Pubkey, u64)]| -> Result<PathBuf> {
        let json = serde_json::json!({
            "distribution": distribution.to_string(),
            "salt_seed": salt_seed.iter().map(|b| format!("{b:02x}")).collect::<String>(),
            "rows": rows.iter().map(|(o, b)| serde_json::json!({"owner": o.to_string(), "balance": b})).collect::<Vec<_>>(),
        });
        let path = args.out.join(name);
        std::fs::write(&path, serde_json::to_string_pretty(&json)?)?;
        Ok(path)
    };
    let wrong_rows = rows_for(&[0, 1]); // C deliberately omitted (demo step 5)
    let good_rows = rows_for(&[0, 1, 2]);

    // ---------------------------------------------------------------- step 2
    run.note("\n## Step 2: two attestors commit matching roots after record slot + margin".to_string());
    run.note("(The first register deliberately omits holder C, so step 5 can uphold a challenge.)".to_string());
    run.wait_for_slot(record_slot + args.finality_margin_slots, "record slot + finality margin")?;
    let commit = |run: &mut Run, a: &Attestor, rows: &[(Pubkey, u64)]| -> Result<()> {
        let kp = a.kp.insecure_clone();
        let commitment = pda(&[b"commitment", distribution.as_ref(), kp.pubkey().as_ref()]);
        run.send(
            &format!("commit_root by {} ({} rows)", kp.pubkey(), rows.len()),
            &[slotbook_ix(
                slotbook::accounts::CommitRoot {
                    attestor: kp.pubkey(),
                    issuer_config,
                    distribution,
                    attestor_account: a.account,
                    commitment,
                    system_program: system_program::ID,
                },
                slotbook::instruction::CommitRoot {
                    args: CommitArgs {
                        root: tree_of(rows).root(),
                        manifest_hash: [0; 32],
                        register_total: rows.iter().map(|r| r.1).sum(),
                        resolved_slot: record_slot,
                        first_covered_slot: 0,
                        rows: rows.len() as u32,
                        spec_version: SPEC_VERSION,
                    },
                },
            )],
            &[&kp],
        )
    };
    for a in &attestors {
        commit(run, a, &wrong_rows)?;
    }
    let d: Distribution = run.read(&distribution)?;
    run.note(format!("State: {:?}, window ends at slot {}.", d.state, d.window_end_slot));

    // ---------------------------------------------------------------- step 3
    run.note("\n## Step 3: the root is reproduced from the spec by an independent script".to_string());
    let reg_path = write_register("register_committed.json", &wrong_rows)?;
    let root_hex: String = d.final_root.iter().map(|b| format!("{b:02x}")).collect();
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../../scripts/reproduce_root.py");
    let out = Command::new("python3").arg(script).arg(&reg_path).arg(&root_hex).output()?;
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    run.note(format!("`python3 scripts/reproduce_root.py {}`\n\n```\n{text}\n```", reg_path.display()));
    if !out.status.success() {
        bail!("root not reproduced: {}", String::from_utf8_lossy(&out.stderr));
    }

    // ---------------------------------------------------------------- step 4
    run.note("\n## Step 4: a wrong-balance challenge is rejected".to_string());
    let challenge = |run: &mut Run, who: usize, index: u32, owner: Pubkey, bal: u64| -> Result<()> {
        let c = challengers[who].insecure_clone();
        run.send(
            &format!("challenge #{index}: owner {owner} balance {bal}"),
            &[slotbook_ix(
                slotbook::accounts::CreateChallenge {
                    challenger: c.pubkey(),
                    issuer_config,
                    distribution,
                    challenge: pda(&[b"challenge", distribution.as_ref(), &index.to_le_bytes()]),
                    bond_mint,
                    bond_vault,
                    challenger_bond_account: challenger_bond[who],
                    bond_token_program: token::ID,
                    system_program: system_program::ID,
                },
                slotbook::instruction::Challenge { owner, claimed_balance: bal },
            )],
            &[&c],
        )
    };
    let resolve = |run: &mut Run, who: usize, index: u32, ruling: Ruling| -> Result<()> {
        let c = &challengers[who];
        let mut ix = slotbook_ix(
            slotbook::accounts::ResolveChallenge {
                signer: resolver.pubkey(),
                issuer_config,
                distribution,
                challenge: Some(pda(&[b"challenge", distribution.as_ref(), &index.to_le_bytes()])),
                challenger: Some(c.pubkey()),
                bond_mint,
                bond_vault,
                resolver_bond_account: resolver_bond,
                challenger_bond_account: Some(challenger_bond[who]),
                issuer_bond_account: issuer_bond,
                bond_token_program: token::ID,
            },
            slotbook::instruction::ResolveChallenge { ruling },
        );
        for a in &attestors {
            ix.accounts.push(AccountMeta::new(pda(&[b"commitment", distribution.as_ref(), a.kp.pubkey().as_ref()]), false));
            ix.accounts.push(AccountMeta::new(a.account, false));
        }
        let r = resolver.insecure_clone();
        run.send(&format!("resolve_challenge #{index}: {ruling:?}"), &[ix], &[&r])
    };
    challenge(run, 0, 0, holders[0].kp.pubkey(), 999)?;
    resolve(run, 0, 0, Ruling::Rejected)?;
    run.note(format!(
        "Resolver received {} test USDC; challenger bond went to the fee and the attestors' bonds.",
        run.token_amount(&resolver_bond)? as f64 / ONE as f64
    ));

    // ---------------------------------------------------------------- step 5
    run.note("\n## Step 5: a challenge naming the omitted owner is upheld".to_string());
    challenge(run, 1, 1, holders[2].kp.pubkey(), balances[2])?;
    resolve(run, 1, 1, Ruling::Upheld)?;
    run.note(format!(
        "Both attestors slashed; challenger now holds {} test USDC (bond back + slashes minus fees).",
        run.token_amount(&challenger_bond[1])? as f64 / ONE as f64
    ));
    for a in &attestors {
        let kp = a.kp.insecure_clone();
        run.send(
            &format!("withdraw_bond {}", kp.pubkey()),
            &[slotbook_ix(
                slotbook::accounts::WithdrawBond {
                    attestor: kp.pubkey(),
                    issuer_config,
                    attestor_account: a.account,
                    bond_mint,
                    bond_vault,
                    attestor_bond_account: a.bond_ata,
                    bond_token_program: token::ID,
                },
                slotbook::instruction::WithdrawBond {},
            )],
            &[&kp],
        )?;
        register(run, a)?;
    }
    for a in &attestors {
        commit(run, a, &good_rows)?;
    }
    let d: Distribution = run.read(&distribution)?;
    let reg_path = write_register("register_corrected.json", &good_rows)?;
    let root_hex: String = d.final_root.iter().map(|b| format!("{b:02x}")).collect();
    let out = Command::new("python3").arg(script).arg(&reg_path).arg(&root_hex).output()?;
    if !out.status.success() {
        bail!("corrected root not reproduced");
    }
    run.note(format!("Corrected root `{root_hex}` committed and reproduced; state {:?}.", d.state));

    // ---------------------------------------------------------------- step 6
    run.note("\n## Step 6: window elapses, finalize, a thawed holder claims and is paid".to_string());
    run.wait_for_slot(d.window_end_slot, "challenge window")?;
    run.send(
        "finalize",
        &[slotbook_ix(slotbook::accounts::Finalize { distribution }, slotbook::instruction::Finalize {})],
        &[&issuer],
    )?;
    let claim = |run: &mut Run, i: usize| -> Result<()> {
        let h = &holders[i];
        let o = h.kp.pubkey();
        let idx = good_rows.iter().position(|(r, _)| *r == o).unwrap();
        let kp = h.kp.insecure_clone();
        run.send(
            &format!("claim by {}", h.name),
            &[slotbook_ix(
                slotbook::accounts::Claim {
                    holder: o,
                    issuer_config,
                    distribution,
                    claim_receipt: pda(&[b"claim", distribution.as_ref(), o.as_ref()]),
                    pending: pda(&[b"pending", distribution.as_ref(), o.as_ref()]),
                    payment_mint: pay_mint,
                    vault,
                    holder_payment_account: h.pay_ata,
                    holder_mint_account: h.mint_ata,
                    payment_token_program: token::ID,
                    system_program: system_program::ID,
                },
                slotbook::instruction::Claim {
                    balance: good_rows[idx].1,
                    salt: salt_of(&o),
                    proof: tree_of(&good_rows).proof(idx).unwrap().siblings,
                },
            )],
            &[&kp],
        )
    };
    claim(run, 0)?;
    run.note(format!("A received {} test USDC.", run.token_amount(&holders[0].pay_ata)? as f64 / ONE as f64));

    // ---------------------------------------------------------------- step 7
    run.note("\n## Step 7: a frozen holder lands in Pending, is thawed, and is released".to_string());
    run.send(
        "freeze B and C (Token ACL authority freeze)",
        &[
            acl_set_frozen(true, &ip, &mint, &holders[1].mint_ata, &mint_config),
            acl_set_frozen(true, &ip, &mint, &holders[2].mint_ata, &mint_config),
        ],
        &[&issuer],
    )?;
    claim(run, 1)?;
    run.send(
        "thaw B (stand-in for the gate's permissionless thaw)",
        &[acl_set_frozen(false, &ip, &mint, &holders[1].mint_ata, &mint_config)],
        &[&issuer],
    )?;
    let release = slotbook_ix(
        slotbook::accounts::ReleasePending {
            issuer_config,
            distribution,
            pending: pda(&[b"pending", distribution.as_ref(), holders[1].kp.pubkey().as_ref()]),
            payer: holders[1].kp.pubkey(),
            payment_mint: pay_mint,
            vault,
            holder_payment_account: holders[1].pay_ata,
            holder_mint_account: holders[1].mint_ata,
            payment_token_program: token::ID,
        },
        slotbook::instruction::ReleasePending {},
    );
    run.send("release_pending for B (crank)", &[release], &[&issuer])?;
    run.note(format!("B received {} test USDC.", run.token_amount(&holders[1].pay_ata)? as f64 / ONE as f64));

    // ---------------------------------------------------------------- step 8
    run.note("\n## Step 8: hold window elapses, issuer sweeps, distribution closes, attestors leave".to_string());
    claim(run, 2)?;
    let pending_c = pda(&[b"pending", distribution.as_ref(), holders[2].kp.pubkey().as_ref()]);
    let p: Pending = run.read(&pending_c)?;
    run.wait_for_slot(p.created_slot + args.hold_window_slots, "hold window")?;
    run.send(
        "sweep_pending for C",
        &[slotbook_ix(
            slotbook::accounts::SweepPending {
                authority: ip,
                issuer_config,
                distribution,
                pending: pending_c,
                payer: holders[2].kp.pubkey(),
                payment_mint: pay_mint,
                vault,
                issuer_payment_account: issuer_pay,
                payment_token_program: token::ID,
            },
            slotbook::instruction::SweepPending {},
        )],
        &[&issuer],
    )?;
    let d: Distribution = run.read(&distribution)?;
    run.wait_for_slot(d.open_slot + args.claim_expiry_slots, "claim expiry")?;
    run.send(
        "close_distribution",
        &[slotbook_ix(
            slotbook::accounts::CloseDistribution {
                authority: ip,
                issuer_config,
                distribution,
                payment_mint: pay_mint,
                vault,
                issuer_payment_account: issuer_pay,
                payment_token_program: token::ID,
            },
            slotbook::instruction::CloseDistribution {},
        )],
        &[&issuer],
    )?;
    for a in &attestors {
        let kp = a.kp.insecure_clone();
        run.send(
            &format!("release_attestor {}", kp.pubkey()),
            &[slotbook_ix(
                slotbook::accounts::ReleaseAttestor {
                    distribution,
                    attestor_account: a.account,
                    commitment: pda(&[b"commitment", distribution.as_ref(), kp.pubkey().as_ref()]),
                },
                slotbook::instruction::ReleaseAttestor {},
            )],
            &[&issuer],
        )?;
        run.send(
            &format!("withdraw_bond {}", kp.pubkey()),
            &[slotbook_ix(
                slotbook::accounts::WithdrawBond {
                    attestor: kp.pubkey(),
                    issuer_config,
                    attestor_account: a.account,
                    bond_mint,
                    bond_vault,
                    attestor_bond_account: a.bond_ata,
                    bond_token_program: token::ID,
                },
                slotbook::instruction::WithdrawBond {},
            )],
            &[&kp],
        )?;
    }
    let d: Distribution = run.read(&distribution)?;
    run.note(format!(
        "Final: state {:?}, claimed {}, pending {}, swept {} (test USDC base units), bond vault {}.",
        d.state,
        d.claimed_total,
        d.pending_total,
        d.swept_total,
        run.token_amount(&bond_vault)?
    ));
    run.note("\n## Step 9 (not on-chain)\n\nSuperbank design issue with SQL, and two issuer write-ups.".to_string());
    Ok(())
}
