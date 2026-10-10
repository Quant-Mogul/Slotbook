# Issuer runbook

This runbook creates the controlled Token-2022 asset used by the Slotbook demo. The issuer wallet keeps mint authority. Token ACL receives control of the mint's freeze authority through its per-mint config. Slotbook later verifies the issuer during `init_issuer`, then records and settles balances; it does not mint this asset.

The Token ACL flow uses Token-2022's frozen default account state, a MintConfig PDA, and a gate reference in token metadata. See the [official Token ACL documentation](https://solana.com/docs/tokenization/token-acl) for the program roles and CLI syntax.

## Prerequisites

- Rust and a funded devnet payer at `~/.config/solana/id.json`.
- A Solana RPC endpoint. The default is `https://api.devnet.solana.com`.
- `token-acl-cli` version `0.3.1` and `token-acl-gate-cli` version `0.3.0` on `PATH`.
- A Surfpool environment with the Token ACL and ABL Gate programs available when testing locally.
- Devnet USDC for the later Slotbook vault and claim flow. The issuer setup does not mint or distribute USDC.

Generated holder and mint keypairs are written below `scripts/state/`, which is ignored by git. Treat that directory as devnet secret material.

## Build and inspect

```bash
cargo fmt -p issuer-scripts
cargo check -p issuer-scripts
cargo run -p issuer-scripts -- --help
```

The package is `issuer-scripts` at `scripts/issuer`. Its commands only send transactions when explicitly run.

## Surfpool setup

Run the local validator with the Token ACL and ABL Gate programs loaded, then point the script at that RPC:

```bash
cargo run -p issuer-scripts -- \
  --rpc-url http://127.0.0.1:8899 \
  setup --output scripts/state/issuer.json --holders 5
cargo run -p issuer-scripts -- \
  --rpc-url http://127.0.0.1:8899 \
  transfers --phase initial --state scripts/state/issuer.json
```

The setup command creates the mint, initializes frozen default accounts and Token Metadata, configures the gate, creates an allow list, and creates one Token-2022 associated account per holder. Token-2022 associated accounts use the immutable-owner extension path required by the claim design.

The initial phase mints to the first three holders, performs ten deterministic transfers, freezes holder four through Token ACL, and prints the latest observed slot as a record-slot candidate. It records completion in the state file and refuses to run a second time.

After declaring a distribution, use the separate pre-record batch before selecting the final record slot:

```bash
cargo run -p issuer-scripts -- \
  --rpc-url http://127.0.0.1:8899 \
  transfers --phase pre-record --state scripts/state/issuer.json
```

The `pre-record` phase performs three additional known transfers and does not mint or repeat the freeze operation.

## Devnet run

After the Surfpool run is understood, use the same commands without `--rpc-url`, or provide the chosen devnet RPC explicitly:

```bash
cargo run -p issuer-scripts -- \
  --rpc-url https://api.devnet.solana.com \
  setup --output scripts/state/issuer.json --holders 5
cargo run -p issuer-scripts -- \
  --rpc-url https://api.devnet.solana.com \
  transfers --phase initial --state scripts/state/issuer.json
```

Save the printed mint, holder owners, token accounts, and slot in the demo notes. Share the mint and transfer schedule with the attestor owner before choosing the final record slot so replay uses a history that the issuer controls and understands.

## Operational sequence

1. Create the mint and Token ACL configuration.
2. Add the five generated holders to the allow list and create their accounts.
3. Mint to three holders and run the deterministic transfer history.
4. Declare the Slotbook distribution with a future record slot.
5. Run the `pre-record` transfer phase for planned transfers before that slot, then stop changing the history.
6. Have the attestor replay RPC history and publish the Merkle root.
7. Keep the frozen holder pending until the gate thaws the account; do not treat a frozen balance as claimable.

Never use the generated demo keys on mainnet. Book issuer conversations after the demo is reproducible end to end.
