# Slotbook issuer demo

The demo shows the issuer side of a permissioned Token-2022 distribution. It deliberately uses generated wallets and a mint created by the issuer so the attestor can replay every relevant transfer.

## What the audience should see

The issuer wallet creates a Token-2022 mint with frozen default accounts and writes `token_acl=gate` metadata. Token ACL owns the mint's freeze authority, while the issuer keeps mint authority. Five holder wallets are added to the allow list. Three receive tokens, ten known transfers build the history, and one holder is frozen.

The Slotbook program is not needed for this preparation. Its later responsibility is to verify the issuer's mint authority, commit the attested balance root, and pay claims from a USDC vault. It never issues the permissioned token.

## Preparation

Run the issuer setup and history scripts from the repository root:

```bash
cargo run -p issuer-scripts -- setup --output scripts/state/issuer.json --holders 5
cargo run -p issuer-scripts -- transfers --state scripts/state/issuer.json
```

For Surfpool, add `--rpc-url http://127.0.0.1:8899`. For devnet, ensure the payer has SOL and acquire devnet USDC separately for the later vault and claim demonstration.

The output includes:

- the mint address;
- the generated state file path;
- the frozen holder;
- a record-slot candidate.

The state file is local demo state and contains paths to private keys. Do not commit it or paste its contents into the demo notes.

## Storyboard

1. **Create** — show the issuer mint and Token ACL config.
2. **Allow** — show five holder owners on the allow list and their Token-2022 accounts.
3. **Distribute** — show initial balances for three holders.
4. **Trade** — show the deterministic ten-transfer history.
5. **Freeze** — freeze holder four and explain that a frozen holder remains pending.
6. **Declare** — create a Slotbook distribution with a future record slot.
7. **Trade before record** — run `transfers --phase pre-record`, then stop changing the history and choose the record slot.
8. **Attest** — let the attestor replay the known transaction history and publish the Merkle root.
9. **Settle** — after the challenge window, finalize and show eligible holders claiming devnet USDC.

The final claim page and Codama client depend on the program interface stubs and IDL. They should be added after those land; this document covers the issuer preparation that can be built independently.

## Replay handoff

Before the final run, agree with the attestor owner on:

- the exact mint address and RPC endpoint;
- the transfer schedule and any post-declaration transfers;
- the record slot and confirmation policy;
- the frozen holder's expected pending status;
- the snapshot and proof manifest locations.

The attestor must replay the same mint history from RPC and record transaction ordering within slots. Do not pick a record slot until the last intended pre-record transaction is confirmed.
