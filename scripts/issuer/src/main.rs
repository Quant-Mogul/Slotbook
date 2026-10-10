use anyhow::{bail, ensure, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_instruction::Instruction;
use solana_keypair::{read_keypair_file, write_keypair_file, Keypair};
use solana_program_option::COption;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_system_interface::instruction::create_account;
use solana_transaction::Transaction;
use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;
use spl_token_2022_interface::{
    extension::{
        default_account_state::{instruction as default_state, DefaultAccountState},
        metadata_pointer::instruction as metadata_pointer,
        BaseStateWithExtensions, ExtensionType, StateWithExtensions,
    },
    instruction as token_instruction,
    state::{Account as TokenAccount, AccountState, Mint},
};
use spl_token_metadata_interface::{
    instruction as metadata_instruction,
    state::{Field, TokenMetadata},
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

const TOKEN_2022_PROGRAM: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
const TOKEN_ACL_GATE: &str = "GATEzzqxhJnsWF6vHRsgtixxSB8PaQdcqGEVTEHWiULz";
const TOKEN_ACL_CLI: &str = "token-acl-cli";
const TOKEN_ACL_GATE_CLI: &str = "token-acl-gate-cli";
const DECIMALS: u8 = 6;
const DEFAULT_RPC_URL: &str = "https://api.devnet.solana.com";

#[derive(Parser)]
#[command(
    name = "issuer-scripts",
    about = "Slotbook issuer-side devnet setup and history scripts"
)]
struct Cli {
    #[arg(long)]
    rpc_url: Option<String>,
    #[arg(long, default_value = "~/.config/solana/id.json")]
    payer: String,
    #[command(subcommand)]
    command: CommandKind,
}

#[derive(Subcommand)]
enum CommandKind {
    /// Create the Token-2022 mint, ACL config, allow list, and five holder accounts.
    Setup {
        #[arg(long, default_value = "scripts/state/issuer.json")]
        output: PathBuf,
        #[arg(long, default_value_t = 5)]
        holders: usize,
    },
    /// Mint and transfer a deterministic history, then freeze one holder.
    Transfers {
        #[arg(long, default_value = "scripts/state/issuer.json")]
        state: PathBuf,
        /// Select the initial history or the post-declaration pre-record batch.
        #[arg(long, value_enum, default_value_t = TransferPhase::Initial)]
        phase: TransferPhase,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum TransferPhase {
    Initial,
    PreRecord,
}

#[derive(Debug, Serialize, Deserialize)]
struct IssuerState {
    rpc_url: String,
    mint: String,
    decimals: u8,
    mint_keypair: String,
    list_config: String,
    holders: Vec<HolderState>,
    #[serde(default)]
    initial_phase_completed: bool,
    #[serde(default)]
    pre_record_phase_completed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct HolderState {
    name: String,
    owner: String,
    keypair: String,
    token_account: String,
}

fn main() -> Result<()> {
    let Cli {
        rpc_url,
        payer,
        command,
    } = Cli::parse();
    match command {
        CommandKind::Setup { output, holders } => setup(
            rpc_url.as_deref().unwrap_or(DEFAULT_RPC_URL),
            &payer,
            &output,
            holders,
        ),
        CommandKind::Transfers { state, phase } => {
            transfers(rpc_url.as_deref(), &payer, &state, phase)
        }
    }
}

fn setup(rpc_url: &str, payer_path: &str, output: &Path, holder_count: usize) -> Result<()> {
    if holder_count != 5 {
        bail!("the deterministic demo requires exactly five holders");
    }
    let payer = load_keypair(payer_path)?;
    let rpc = RpcClient::new(rpc_url.to_string());
    let root = output
        .parent()
        .unwrap_or_else(|| Path::new("scripts/state"));
    fs::create_dir_all(root)?;

    let mint = Keypair::new();
    let mint_address = create_mint(&rpc, &payer, &mint)?;
    let mint_path = root.join("mint.json");
    write_keypair_file(&mint, &mint_path)
        .map_err(|e| anyhow::anyhow!("could not write mint keypair: {e}"))?;

    let acl_output = run_external(
        TOKEN_ACL_CLI,
        &[
            "create-config",
            &mint_address.to_string(),
            "--gating-program",
            TOKEN_ACL_GATE,
        ],
        rpc_url,
        payer_path,
    )?;
    let _ = run_external(
        TOKEN_ACL_CLI,
        &[
            "set-instructions",
            "--enable-thaw",
            "--disable-freeze",
            &mint_address.to_string(),
        ],
        rpc_url,
        payer_path,
    )?;

    let list_output = run_external(
        TOKEN_ACL_GATE_CLI,
        &["create-list", "--mode", "allow"],
        rpc_url,
        payer_path,
    )?;
    let list_config = parse_field(&list_output, "list_config:")
        .or_else(|| parse_field(&list_output, "List config:"))
        .context("token-acl-gate-cli did not print a list_config address")?;
    let _ = run_external(
        TOKEN_ACL_GATE_CLI,
        &[
            "apply-lists-to-mint",
            &mint_address.to_string(),
            &list_config,
        ],
        rpc_url,
        payer_path,
    )?;

    let mut holder_states = Vec::with_capacity(holder_count);
    for index in 0..holder_count {
        let holder = Keypair::new();
        let holder_path = root.join(format!("holder-{index}.json"));
        write_keypair_file(&holder, &holder_path)
            .map_err(|e| anyhow::anyhow!("could not write holder keypair: {e}"))?;
        let owner = holder.pubkey();
        let _ = run_external(
            TOKEN_ACL_GATE_CLI,
            &["add-wallet", &list_config, &owner.to_string()],
            rpc_url,
            payer_path,
        )?;
        let _ = run_external(
            TOKEN_ACL_CLI,
            &[
                "create-ata-and-thaw-permissionless",
                "--mint",
                &mint_address.to_string(),
                "--owner",
                &owner.to_string(),
            ],
            rpc_url,
            payer_path,
        )?;
        let token_account =
            get_associated_token_address_with_program_id(&owner, &mint_address, &token_program());
        holder_states.push(HolderState {
            name: format!("holder-{index}"),
            owner: owner.to_string(),
            keypair: holder_path.display().to_string(),
            token_account: token_account.to_string(),
        });
    }

    let holder_accounts: Vec<_> = holder_states
        .iter()
        .map(|holder| {
            Ok((
                Pubkey::from_str(&holder.owner)?,
                Pubkey::from_str(&holder.token_account)?,
            ))
        })
        .collect::<Result<_>>()?;
    assert_mint_and_holders(
        &rpc,
        &mint_address,
        &payer.pubkey(),
        DECIMALS,
        &holder_accounts,
    )?;

    let state = IssuerState {
        rpc_url: rpc_url.to_string(),
        mint: mint_address.to_string(),
        decimals: DECIMALS,
        mint_keypair: mint_path.display().to_string(),
        list_config,
        holders: holder_states,
        initial_phase_completed: false,
        pre_record_phase_completed: false,
    };
    fs::write(output, serde_json::to_vec_pretty(&state)?)?;
    println!("mint: {}", state.mint);
    println!("state: {}", output.display());
    println!(
        "ACL config created: {}",
        acl_output.lines().last().unwrap_or("see CLI output")
    );
    Ok(())
}

fn transfers(
    rpc_url_override: Option<&str>,
    payer_path: &str,
    state_path: &Path,
    phase: TransferPhase,
) -> Result<()> {
    let mut state: IssuerState = serde_json::from_slice(&fs::read(state_path)?)?;
    match phase {
        TransferPhase::Initial if state.initial_phase_completed => {
            bail!("initial phase already completed; use --phase pre-record to add known transfers")
        }
        TransferPhase::PreRecord if state.pre_record_phase_completed => {
            bail!("pre-record phase already completed; stop before choosing the record slot")
        }
        _ => {}
    }
    let rpc_url =
        rpc_url_override
            .filter(|url| !url.is_empty())
            .unwrap_or(if state.rpc_url.is_empty() {
                DEFAULT_RPC_URL
            } else {
                &state.rpc_url
            });
    let rpc = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::processed());
    let payer = load_keypair(payer_path)?;
    let mint = Pubkey::from_str(&state.mint)?;
    let mint_authority = payer.pubkey();
    let holders: Vec<(Keypair, Pubkey, Pubkey)> = state
        .holders
        .iter()
        .map(|h| {
            Ok((
                load_keypair(&h.keypair)?,
                Pubkey::from_str(&h.owner)?,
                Pubkey::from_str(&h.token_account)?,
            ))
        })
        .collect::<Result<_>>()?;

    let initial_balances = [845, 771, 546, 18, 20];
    let pre_record_balances = [828, 777, 549, 26, 20];
    if matches!(phase, TransferPhase::PreRecord) {
        let balances = read_balances(&rpc, &holders)?;
        ensure!(
            balances == initial_balances,
            "pre-record requires the untouched initial balances; got {balances:?}"
        );
    }

    if matches!(phase, TransferPhase::Initial) {
        let initial = [(0usize, 1_000u64), (1usize, 700u64), (2usize, 500u64)];
        for (index, amount) in initial {
            send(
                &rpc,
                &[token_instruction::mint_to_checked(
                    &token_program(),
                    &mint,
                    &holders[index].2,
                    &mint_authority,
                    &[],
                    amount,
                    state.decimals,
                )?],
                &[&payer as &dyn Signer],
            )?;
        }
    }

    let transfers = match phase {
        TransferPhase::Initial => vec![
            (0, 1, 110),
            (1, 3, 25),
            (0, 2, 50),
            (2, 4, 15),
            (1, 2, 30),
            (2, 0, 10),
            (3, 4, 12),
            (4, 1, 7),
            (0, 3, 5),
            (2, 1, 9),
        ],
        TransferPhase::PreRecord => vec![(0, 1, 17), (1, 2, 11), (2, 3, 8)],
    };
    for (from, to, amount) in transfers {
        let source = &holders[from];
        let destination = &holders[to];
        send(
            &rpc,
            &[token_instruction::transfer_checked(
                &token_program(),
                &source.2,
                &mint,
                &destination.2,
                &source.1,
                &[],
                amount,
                state.decimals,
            )?],
            &[&payer as &dyn Signer, &source.0 as &dyn Signer],
        )?;
    }

    if matches!(phase, TransferPhase::Initial) {
        let frozen = &state.holders[4];
        let _ = run_external(
            TOKEN_ACL_CLI,
            &["freeze", &frozen.token_account],
            rpc_url,
            payer_path,
        )?;
    }
    let expected_balances = match phase {
        TransferPhase::Initial => initial_balances,
        TransferPhase::PreRecord => pre_record_balances,
    };
    assert_balances(&rpc, &holders, &expected_balances)?;
    if matches!(phase, TransferPhase::Initial) {
        assert_account_state(&rpc, &holders[4].2, AccountState::Frozen)?;
    }
    let last_slot = rpc.get_slot()?;
    match phase {
        TransferPhase::Initial => state.initial_phase_completed = true,
        TransferPhase::PreRecord => state.pre_record_phase_completed = true,
    }
    fs::write(state_path, serde_json::to_vec_pretty(&state)?)?;
    println!("mint: {}", state.mint);
    println!("phase: {:?}", phase);
    if matches!(phase, TransferPhase::Initial) {
        println!("frozen holder: {}", state.holders[4].name);
    }
    println!("latest slot: {}", last_slot);
    if matches!(phase, TransferPhase::Initial) {
        println!("initial history complete; use --phase pre-record after declaring a distribution");
    } else {
        println!("pre-record history complete; stop transfers before choosing the record slot");
    }
    Ok(())
}

fn create_mint(rpc: &RpcClient, payer: &Keypair, mint: &Keypair) -> Result<Pubkey> {
    let token_program = token_program();
    let metadata_name = "Slotbook Demo Asset".to_string();
    let metadata_symbol = "SLOT".to_string();
    let metadata_uri = "".to_string();
    let metadata_for_sizing = TokenMetadata {
        mint: mint.pubkey(),
        name: metadata_name.clone(),
        symbol: metadata_symbol.clone(),
        uri: metadata_uri.clone(),
        additional_metadata: vec![("token_acl".to_string(), TOKEN_ACL_GATE.to_string())],
        ..TokenMetadata::default()
    };
    let mint_space = ExtensionType::try_calculate_account_len::<Mint>(&[
        ExtensionType::DefaultAccountState,
        ExtensionType::MetadataPointer,
    ])?;
    let rent_space = mint_space
        .checked_add(metadata_for_sizing.tlv_size_of()?)
        .context("Token-2022 mint rent size overflow")?;
    let rent = rpc.get_minimum_balance_for_rent_exemption(rent_space)?;
    let instructions = vec![
        create_account(
            &payer.pubkey(),
            &mint.pubkey(),
            rent,
            mint_space as u64,
            &token_program,
        ),
        default_state::initialize_default_account_state(
            &token_program,
            &mint.pubkey(),
            &AccountState::Frozen,
        )?,
        metadata_pointer::initialize(
            &token_program,
            &mint.pubkey(),
            Some(payer.pubkey()),
            Some(mint.pubkey()),
        )?,
        token_instruction::initialize_mint2(
            &token_program,
            &mint.pubkey(),
            &payer.pubkey(),
            Some(&payer.pubkey()),
            DECIMALS,
        )?,
        metadata_instruction::initialize(
            &token_program,
            &mint.pubkey(),
            &payer.pubkey(),
            &mint.pubkey(),
            &payer.pubkey(),
            metadata_name,
            metadata_symbol,
            metadata_uri,
        ),
        metadata_instruction::update_field(
            &token_program,
            &mint.pubkey(),
            &payer.pubkey(),
            Field::Key("token_acl".to_string()),
            TOKEN_ACL_GATE.to_string(),
        ),
    ];
    send(
        rpc,
        &instructions,
        &[payer as &dyn Signer, mint as &dyn Signer],
    )?;
    Ok(mint.pubkey())
}

fn assert_mint_and_holders(
    rpc: &RpcClient,
    mint: &Pubkey,
    payer: &Pubkey,
    decimals: u8,
    holders: &[(Pubkey, Pubkey)],
) -> Result<()> {
    let mint_account = rpc.get_account(mint)?;
    ensure!(
        mint_account.owner == token_program(),
        "mint is not Token-2022 owned"
    );
    let mint_state = StateWithExtensions::<Mint>::unpack(&mint_account.data)
        .map_err(|error| anyhow::anyhow!("could not unpack mint state: {error}"))?;
    ensure!(mint_state.base.is_initialized, "mint is not initialized");
    ensure!(
        mint_state.base.decimals == decimals,
        "mint decimals do not match"
    );
    ensure!(
        mint_state.base.mint_authority == COption::Some(*payer),
        "issuer payer is not the mint authority"
    );
    ensure!(
        mint_state.base.freeze_authority.is_some(),
        "mint freeze authority is missing"
    );
    let mint_extensions = mint_state.get_extension_types()?;
    ensure!(
        mint_extensions.contains(&ExtensionType::DefaultAccountState),
        "mint is missing DefaultAccountState"
    );
    let default_account_state = mint_state.get_extension::<DefaultAccountState>()?;
    ensure!(
        default_account_state.state == AccountState::Frozen as u8,
        "mint default account state is not Frozen"
    );
    ensure!(
        mint_extensions.contains(&ExtensionType::MetadataPointer),
        "mint is missing MetadataPointer"
    );

    for (owner, token_account) in holders {
        let account = rpc.get_account(token_account)?;
        ensure!(
            account.owner == token_program(),
            "holder token account is not Token-2022 owned"
        );
        let token_state = StateWithExtensions::<TokenAccount>::unpack(&account.data)
            .map_err(|error| anyhow::anyhow!("could not unpack holder account: {error}"))?;
        ensure!(
            token_state.base.mint == *mint,
            "holder account has the wrong mint"
        );
        ensure!(
            token_state.base.owner == *owner,
            "holder account has the wrong owner"
        );
        ensure!(
            token_state
                .get_extension_types()?
                .contains(&ExtensionType::ImmutableOwner),
            "holder account is missing ImmutableOwner"
        );
    }
    Ok(())
}

fn assert_balances(
    rpc: &RpcClient,
    holders: &[(Keypair, Pubkey, Pubkey)],
    expected: &[u64; 5],
) -> Result<()> {
    let actual = read_balances(rpc, holders)?;
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        ensure!(
            actual == expected,
            "holder-{index} balance mismatch: expected {}, got {actual}",
            expected
        );
    }
    Ok(())
}

fn read_balances(rpc: &RpcClient, holders: &[(Keypair, Pubkey, Pubkey)]) -> Result<[u64; 5]> {
    ensure!(
        holders.len() == 5,
        "the deterministic history requires exactly five holders"
    );
    let mut balances = [0; 5];
    for (index, (_, _, token_account)) in holders.iter().enumerate() {
        let amount = rpc.get_token_account_balance(token_account)?.amount;
        balances[index] = amount
            .parse::<u64>()
            .with_context(|| format!("invalid balance for holder-{index}: {amount}"))?;
    }
    Ok(balances)
}

fn assert_account_state(
    rpc: &RpcClient,
    token_account: &Pubkey,
    expected: AccountState,
) -> Result<()> {
    let account = rpc.get_account(token_account)?;
    let token_state = StateWithExtensions::<TokenAccount>::unpack(&account.data)
        .map_err(|error| anyhow::anyhow!("could not unpack token account: {error}"))?;
    ensure!(
        token_state.base.state == expected,
        "token account state mismatch: expected {expected:?}, got {:?}",
        token_state.base.state
    );
    Ok(())
}

fn send(rpc: &RpcClient, instructions: &[Instruction], signers: &[&dyn Signer]) -> Result<()> {
    let blockhash = rpc.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        instructions,
        Some(&signers[0].pubkey()),
        signers,
        blockhash,
    );
    rpc.send_and_confirm_transaction(&tx)
        .context("transaction failed")?;
    Ok(())
}

fn run_external(program: &str, args: &[&str], rpc_url: &str, payer_path: &str) -> Result<String> {
    let output = Command::new(program)
        .env("NO_DNA", "1")
        .args(args)
        .args(["--url", rpc_url, "--payer", payer_path])
        .output()
        .with_context(|| format!("{program} is not installed"))?;
    if !output.status.success() {
        bail!(
            "{program} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let stdout = String::from_utf8(output.stdout)?;
    print!("{}", stdout);
    Ok(stdout)
}

fn parse_field(output: &str, label: &str) -> Option<String> {
    output.lines().find_map(|line| {
        line.trim()
            .strip_prefix(label)
            .map(|v| v.trim().to_string())
    })
}

fn load_keypair(path: &str) -> Result<Keypair> {
    let expanded = if path == "~/.config/solana/id.json" {
        std::env::var_os("HOME")
            .map(|home| PathBuf::from(home).join(".config/solana/id.json"))
            .unwrap_or_else(|| PathBuf::from(path))
    } else {
        PathBuf::from(path)
    };
    read_keypair_file(expanded).map_err(|e| anyhow::anyhow!("could not read keypair: {e}"))
}

fn token_program() -> Pubkey {
    Pubkey::from_str(TOKEN_2022_PROGRAM).expect("Token-2022 program id is valid")
}
