# Maintenance Window Procedures

> Closes **#681** — scheduled windows, pause runbook, state backup, upgrade steps, and communication templates for every Lumora / StellarAid contract.

Related: [PAUSE_AND_EMERGENCY.md](./PAUSE_AND_EMERGENCY.md), [UPGRADE_AND_ROLLBACK.md](./UPGRADE_AND_ROLLBACK.md), [VERSIONING.md](./VERSIONING.md), [OPERATIONAL_RUNBOOK.md](./OPERATIONAL_RUNBOOK.md).

## 1. Maintenance windows

### Scheduled windows

| Window | UTC | Purpose |
|--------|-----|---------|
| **Primary** | Tuesday 02:00–04:00 | Planned upgrades, schema migrations, config changes |
| **Secondary** | Thursday 02:00–04:00 | Overflow / follow-up if Tuesday overruns or needs a second cut |
| **Emergency** | Any time | Confirmed exploit, fund-loss bug, or chain halt |

Keep scheduled work **inside** a window. If a change cannot finish 15 minutes before the window ends, **stop, unpause (if safe), and continue in the next window**. Do not start a MAJOR upgrade in the last 45 minutes of a window.

### Notice and freeze

| Change class | Public notice | Code freeze | Who approves |
|--------------|---------------|-------------|--------------|
| PATCH (no pause, or pause < 10 min) | 24 hours | 6 hours | On-call engineer |
| MINOR / config | 48 hours | 12 hours | On-call + a second engineer |
| MAJOR / storage-schema / new contract ID | 72 hours | 24 hours | Two engineers + incident lead |
| Emergency pause | Immediate | n/a | Any admin; notify within 15 minutes |

Announce the window timezone as **UTC**. Include the estimated pause duration and which contract IDs are in scope.

### What is allowed inside a window

- Pause / unpause
- WASM upgrade or new-ID deploy
- `migrate_vN_to_vM`
- Platform fee / wallet / token metadata updates
- Worker restart and Horizon/RPC endpoint failover
- State backup and post-change verification

### What is not allowed

- Mainnet in-place WASM replace for a MAJOR bump (always new contract ID)
- Deleting old contract IDs
- Changing `DataKey` discriminants
- Running untested scripts against production

---

## 2. Pause procedures

Full command reference: [PAUSE_AND_EMERGENCY.md](./PAUSE_AND_EMERGENCY.md). Use this section for **ordered, multi-contract** maintenance pauses.

### Pre-pause checklist

- [ ] Admin key is available and funded for fees
- [ ] `get_version` / `get_version_metadata` recorded for every target contract
- [ ] State backup completed (section 3)
- [ ] Status page / Discord / email templates sent (section 5)
- [ ] Worker donation verification can tolerate the pause (or is drained)
- [ ] No in-flight dispute with a ledger deadline inside the pause window — if there is, extend TTL or postpone

### Pause order (inbound funds first)

Pause **dependents before dependencies** so a paused callee cannot be invoked by a still-live caller:

1. `donation`, `withdrawal`
2. `campaign`
3. `escrow`, `commission_agreement`, `subscription`, `revenue_sharing`, `creator_fund`
4. `dispute_arbiter`, `messaging`, `competitions`, `verification`, `recruitment`
5. `platform_config` (last — other contracts may still need to read fees during drain)

```bash
# Repeat per contract. Some crates take `--admin <ADDRESS>`, others take no args.
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --network "$NETWORK" \
  --source "$ADMIN_SECRET" \
  -- \
  pause --admin "$ADMIN_ADDRESS"
```

### Verify pause

1. A non-admin state-changing call fails with `"contract is paused"` / `ContractPaused`.
2. `contract_paused` (or contract-specific `paused`) event is in the event stream.
3. Token trustlines still exist — pause does **not** freeze classic account balances.

### Unpause order (reverse)

Unpause `platform_config` first, then marketplace contracts, then campaign, then donation/withdrawal. After each unpause, run one smoke tx (section 4) before continuing.

Emergency stop (multi-sig / timelock) remains as specified in [PAUSE_AND_EMERGENCY.md](./PAUSE_AND_EMERGENCY.md). An emergency pause **skips** the notice period but still requires a backup snapshot if ledgers are still reachable.

---

## 3. State backup procedures

Take a backup **before pause** (or immediately after emergency pause, before any WASM change). Retention: **30 days** for mainnet, **7 days** for testnet.

### 3.1 Inventory (always)

Write a dated directory, e.g. `backups/2026-08-27T0200Z/`:

| File | How to produce |
|------|----------------|
| `inventory.json` | Contract ID, network, WASM hash, `get_version`, `get_version_metadata` |
| `wasm/` | Copy of each `target/wasm32-unknown-unknown/release/*.wasm` that is live |
| `config/` | `config/testnet_contracts.json` (or mainnet equivalent) and `.env` **without secrets** |

```bash
OUT="backups/$(date -u +%Y-%m-%dT%H%MZ)"
mkdir -p "$OUT/wasm" "$OUT/config"

soroban contract invoke --id "$CONTRACT_ID" --network "$NETWORK" -- \
  get_version_metadata > "$OUT/${NAME}_version.json"

# WASM hash of the installed bytecode
soroban contract inspect --wasm "target/wasm32-unknown-unknown/release/${NAME}.wasm" \
  > "$OUT/wasm/${NAME}.inspect.txt"
cp "target/wasm32-unknown-unknown/release/${NAME}.wasm" "$OUT/wasm/"
```

### 3.2 On-chain view snapshot

Dump every stable view function for in-scope contracts. Examples:

```bash
soroban contract invoke --id "$PLATFORM_CONFIG_ID" --network "$NETWORK" -- get_config \
  > "$OUT/platform_config_get_config.json"

soroban contract invoke --id "$CAMPAIGN_ID" --network "$NETWORK" -- get_campaign_count \
  > "$OUT/campaign_count.json"

soroban contract invoke --id "$ESCROW_ID" --network "$NETWORK" -- \
  get_escrow --commission_id "$COMMISSION_ID" \
  > "$OUT/escrow_${COMMISSION_ID}.json"
```

For contracts with many records, page through known IDs from the worker DB or an indexer. Do not rely on a single `get` of the entire map — Soroban has no built-in "export all persistent keys" on-chain. Record **counts** (campaigns, escrows, agreements) so post-upgrade `export_storage_keys` / view totals can be compared.

### 3.3 Worker / off-chain DB

If the worker is in scope:

```bash
# PostgreSQL (adjust DSN). Never commit the dump.
pg_dump "$DATABASE_URL" --format=custom --file="$OUT/worker.dump"

# Redis (if used for webhook retries)
redis-cli --rdb "$OUT/redis.rdb"
```

Encrypt dumps at rest (`age` / `gpg`) and store outside the git repo.

### 3.4 Restore drill

A backup is not valid until it has been restored on **testnet or a local sandbox** once in the last 90 days:

1. Deploy the saved WASM to a fresh testnet ID.
2. Replay `initialize` with the backed-up admin / fee / token addresses.
3. Compare `get_version_metadata` and sampled view results to the snapshot.
4. File the drill date in the incident log.

---

## 4. Upgrade procedures (inside a window)

Detailed ABI/storage rules: [UPGRADE_AND_ROLLBACK.md](./UPGRADE_AND_ROLLBACK.md) and [VERSIONING.md](./VERSIONING.md).

### Timeline (90-minute template)

| T+ | Action |
|----|--------|
| 0 min | Announce "maintenance started"; freeze inbound traffic at the API |
| 5 min | Finish backup if not already done; confirm pause order complete |
| 10 min | `cargo test --workspace` already green from freeze; rebuild WASM |
| 15 min | PATCH/MINOR: `upgrade` entry point **or** deploy new ID (MAJOR) |
| 25 min | `migrate_vN_to_vM` if schema changed |
| 35 min | `get_version` matches CHANGELOG; sampled views match backup counts |
| 45 min | Smoke: one donate / one escrow happy path on testnet clone, then mainnet read-only |
| 55 min | Redirect clients / update `testnet_contracts.json` or mainnet registry |
| 65 min | Unpause in reverse order; watch error rate 10 minutes |
| 80 min | Send "complete" template; start 24-hour hypercare |

### Per-contract upgrade commands

```bash
# 1. Confirm stored version
soroban contract invoke --id "$CONTRACT_ID" --network "$NETWORK" -- get_version

# 2. Pause (section 2)

# 3a. PATCH / compatible MINOR — in-place WASM replace
soroban contract invoke \
  --id "$CONTRACT_ID" --network "$NETWORK" --source "$ADMIN_SECRET" -- \
  upgrade --admin "$ADMIN_ADDRESS" --new_wasm_hash "$NEW_WASM_HASH"

# 3b. MAJOR / schema change — new ID
NEW_ID=$(soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/${NAME}.wasm \
  --network "$NETWORK" --source "$ADMIN_SECRET")

# 4. Record version (if the new WASM's initialize/upgrade path did not)
#    shared::upgrade::record_upgrade is called from the contract upgrade guard.

# 5. Unpause (section 2)
```

### Abort / rollback

If smoke tests fail: **pause the new deployment**, point traffic at the previous contract ID, unpause the previous ID, send the rollback template. Do not delete the failed WASM or ID. Criteria and steps: [UPGRADE_AND_ROLLBACK.md](./UPGRADE_AND_ROLLBACK.md#rollback-criteria).

---

## 5. Communication templates

Replace bracketed fields. Post to status page, Discord `#status`, and email `ops@` / `users@` as applicable. Do **not** include secret keys, WASM hashes of unreleased builds that embed secrets, or individual user balances.

### 5.1 Scheduled maintenance (T–72h / T–48h / T–24h)

**Subject:** Lumora scheduled maintenance — [DATE] [START]–[END] UTC

```
We will perform planned maintenance on Stellar [testnet|mainnet] contracts.

When: [DAY], [DATE] [START]–[END] UTC ([local conversion])
Contracts: [list names + IDs]
Impact: [donations / escrow create / withdrawals] paused for up to [N] minutes
What you should do: avoid submitting [donate|create_escrow|…] during the window
Version: upgrading [name] from [old semver] to [new semver] (see CHANGELOG)
Status: https://[status-page]
Contact: [on-call rotation / Discord]
```

### 5.2 Window start

**Subject:** Lumora maintenance started — [DATE]

```
Maintenance has started at [HH:MM] UTC.
Contracts [list] are paused. Do not submit state-changing transactions.
We will post again when operations resume, or at [HH:MM] UTC if the window is extended.
```

### 5.3 Emergency pause

**Subject:** [EMERGENCY] Lumora contracts paused

```
We paused [contract list] at [HH:MM] UTC after detecting [one-line symptom, no exploit details].
Funds in existing escrows/campaigns remain in contract storage; token accounts are not frozen.
Do not send further [donations|escrow deposits] until we unpause.
Next update by [HH:MM] UTC ([max 30 min]).
Incident lead: [name]
```

### 5.4 Upgrade complete

**Subject:** Lumora maintenance complete — [contract] [new semver]

```
Maintenance finished at [HH:MM] UTC.
[contract] is live at version [new semver] (storage schema [n]).
Contract ID: [unchanged | new ID …]
Please update SDKs / config if the ID changed.
Hypercare: we will watch success rate for 24 hours. Report issues to [channel].
```

### 5.5 Rollback

**Subject:** Lumora rollback — traffic restored to [old semver]

```
The [new semver] deployment did not pass smoke checks. Traffic is back on
[old contract ID] at [old semver]. The new ID is paused and will not receive funds.
User action: none if you use our hosted API; self-hosted indexers should pin [old ID].
We will share a follow-up after the post-incident review.
```

### 5.6 Window cancelled / postponed

```
The [DATE] UTC maintenance window for [contracts] is cancelled / moved to [new DATE].
Reason: [weathered incident / failed testnet rehearsal / …].
No pause will occur on the original date.
```

### 5.7 Internal war-room checklist (paste into incident ticket)

```
- [ ] Window type: scheduled / emergency
- [ ] Backup path: backups/[timestamp]
- [ ] Pause order completed at:
- [ ] Versions recorded (get_version_metadata)
- [ ] Upgrade / migrate tx hashes:
- [ ] Unpause order completed at:
- [ ] Templates sent: T-72 / start / complete / rollback
- [ ] Hypercare owner (24h):
```
