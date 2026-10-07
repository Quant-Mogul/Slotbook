# slotbook-demo

Runs the Slotbook demo, steps 1 to 8, against a live cluster and writes every
transaction link to `<out>/DEMO_RUN.md` and every address to `<out>/addresses.json`.
Step 9 (Superbank design issue, issuer write-ups) is not on-chain.

Each run is independent: it creates its own permissioned mint, test USDC mints,
three holders, two attestors, a resolver and two challengers, all funded by the issuer.

| Step | What happens |
|---|---|
| 1 | Token-2022 mint (DefaultAccountState = Frozen) handed to Token ACL; transfer history; `init_issuer`, two `register_attestor`, `declare_distribution`, more transfers before the record slot |
| 2 | After record slot + finality margin, both attestors `commit_root` the same root (it deliberately omits holder C) |
| 3 | `scripts/reproduce_root.py` recomputes the root from the register with the independent Python spec implementation and must match on-chain |
| 4 | A wrong-balance challenge on A is `Rejected` |
| 5 | A challenge naming the omitted owner C is `Upheld`: both attestors slashed, withdraw, re-register, commit the corrected root |
| 6 | Window elapses, `finalize`, A (thawed) claims and is paid |
| 7 | B is frozen, claims into Pending, is thawed, `release_pending` pays |
| 8 | C is frozen, claims into Pending, hold window elapses, `sweep_pending`; after claim expiry `close_distribution`, then `release_attestor` and `withdraw_bond` for both |

## Prerequisites

- The program deployed at `3ZaUF1Q6FH9HaD7kjMNVztaFtJxQY9U3Zk7CuR3wAqJp` (`anchor build` first).
- Token ACL (`TACLkU6CiCdkQN2MjoyDkVg2yAH9zkxiHDsiztQ52TP`): on devnet already; on a local validator, clone it.
- An issuer keypair with at least 2 SOL. It pays for everything.
- `python3` (step 3).

## Local validator

    solana-test-validator --reset \
      --bpf-program 3ZaUF1Q6FH9HaD7kjMNVztaFtJxQY9U3Zk7CuR3wAqJp target/deploy/slotbook.so \
      --clone-upgradeable-program TACLkU6CiCdkQN2MjoyDkVg2yAH9zkxiHDsiztQ52TP --url https://api.devnet.solana.com

    cargo run -p demo -- --rpc-url http://127.0.0.1:8899 --out demo-out \
      --notice-slots 40 --challenge-window-slots 60 --resolve-timeout-slots 60 \
      --hold-window-slots 40 --claim-expiry-slots 120

## Devnet

    cargo run -p demo -- --rpc-url https://api.devnet.solana.com --out demo-out-devnet

Defaults: notice 150 slots, finality margin 32, window 150, resolve timeout 150,
hold 150, claim expiry 400. A full devnet run takes about 10 minutes.

## What is simplified, said out loud

- Payment and bonds use test USDC mints the run creates (6 decimals), not Circle's devnet USDC.
- Freezing and thawing use Token ACL's authority `freeze` / `thaw`. In production the
  ABL gate's permissionless thaw does the thawing.
- The register comes from the balances this run sets up. The independent replay is
  the attestor crate (`attestor replay --mint <mint> --slot <record_slot>`).
- Both attestors and the resolver are run by the same operator, as the brief says.
