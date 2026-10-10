# Surfpool runbooks

These shell runbooks exercise the issuer script against a local Surfpool fork of devnet. Surfpool provides the local RPC; the Token ACL and ABL Gate programs are fetched from devnet when the first ACL transaction needs them.

The start script uses an in-memory ledger. If Surfpool is restarted, rerun setup before using an existing transfer state file.

Start Surfpool in one terminal:

```bash
scripts/surfpool/start.sh
```

On macOS, keep this process in the foreground. To use different ports:

```bash
RPC_PORT=8891 WS_PORT=8901 scripts/surfpool/start.sh
```

In a second terminal, run the issuer setup:

```bash
scripts/surfpool/issuer-setup.sh
```

Then run the initial history:

```bash
scripts/surfpool/issuer-transfers.sh initial
```

After declaring the distribution, run the additional pre-record history:

```bash
scripts/surfpool/issuer-transfers.sh pre-record
```

The deterministic demo requires exactly five holders. Both phases record completion in the state file and refuse to run twice. The pre-record phase also checks that the initial balances are untouched before submitting its transfers. Set `RPC_URL`, `PAYER`, or `STATE` to override the defaults.
