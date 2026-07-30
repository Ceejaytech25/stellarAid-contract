# Security Review Checklist

This document describes the threat model and security controls for the
StellarAid smart contracts and worker service.

## Threat Model

| Threat | Impact | Mitigation |
|--------|--------|------------|
| Re-entrancy | Attacker drains escrow via callback | CEI pattern, re-entrancy lock (#484) |
| Unauthorized admin access | Contract takeover | admin.require_auth() on all admin functions (#530) |
| Donation replay | Double-count donations | Nonce guard in donate_with_nonce (#536) |
| Overflow/underflow | Balance corruption | checked_mul/checked_add in arithmetic |
| Front-running | Campaign manipulation | Commit-reveal not required (donations are public) |
| Webhook replay | Duplicate notifications | Dedup by event:campaign:tx_hash:amount (#536) |

## Smart Contract Checklist

- [ ] All state-changing functions use `require_auth()` for the caller
- [ ] Admin functions verify the caller matches the stored admin address
- [ ] CEI (Checks-Effects-Interactions) ordering is followed
- [ ] Re-entrancy lock is acquired before external calls (#484)
- [ ] Arithmetic uses checked operations (overflow-safe)
- [ ] Events are emitted for all state changes
- [ ] Storage keys use unique prefixes to avoid collisions
- [ ] TTL is extended for persistent entries
- [ ] Pause/unpause controls block state mutations during incidents
- [ ] Withdrawal amounts are bounded by `raised - withdrawn`

## Worker Checklist

- [ ] Secrets (admin keys) are loaded from env, never hardcoded
- [ ] Webhook secrets are validated on delivery
- [ ] Idempotency keys prevent duplicate processing
- [ ] Health and readiness endpoints exposed
- [ ] Rate limiting on donation/withdrawal endpoints
- [ ] Logging does not expose secrets or PII

## Deployment Checklist

- [ ] Separate testnet and mainnet admin keys
- [ ] Dry-run before actual deployment
- [ ] Verify contract IDs after deployment
- [ ] Initialize contracts immediately after deploy
- [ ] Test pause/unpause on each contract
- [ ] Verify event emission matches documentation
