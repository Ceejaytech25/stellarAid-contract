# Contract Upgrade and Rollback Procedure

This document describes the safe process for upgrading Soroban contracts and rolling back if issues are detected after deployment.

## Prerequisites

- Admin secret key for the contract (admin must be set during initialization).
- Soroban CLI configured for the target network.
- New WASM binary compiled and tested.
- Current contract ID.

## Pre-Upgrade Validation

1. **Verify the new WASM**:
   ```bash
   soroban contract inspect --wasm target/wasm32-unknown-unknown/release/new_contract.wasm
   ```

2. **Compare storage keys**: Ensure the new contract version does not change existing storage key formats unless a migration is explicitly written.

3. **Review events**: Confirm no existing events are removed or have their payloads changed.

4. **Run the full test suite**:
   ```bash
   cargo test --workspace
   ```

5. **Check WASM size**:
   ```bash
   ls -lh target/wasm32-unknown-unknown/release/new_contract.wasm
   ```

## Upgrade Procedure

1. Pause the contract (see [PAUSE_AND_EMERGENCY.md](./PAUSE_AND_EMERGENCY.md)):
   ```bash
   soroban contract invoke --id <CONTRACT_ID> --network <NETWORK> --source <ADMIN> -- pause
   ```

2. Deploy the new WASM to a fresh contract ID:
   ```bash
   NEW_ID=$(soroban contract deploy \
     --wasm target/wasm32-unknown-unknown/release/new_contract.wasm \
     --network <NETWORK> --source <ADMIN>)
   ```

3. Migrate storage (if schema changed):
   - Read existing records from the old contract.
   - Transform and write to the new contract.
   - Verify record counts match.

4. Reinitialize the new contract with the existing admin and configuration.

5. Run smoke tests against the new contract:
   ```bash
   soroban contract invoke --id $NEW_ID --network <NETWORK> --source <ADMIN> -- ping
   ```

6. Redirect traffic to the new contract ID.

7. Unpause the new contract (if applicable) and verify normal operations.

## Rollback Criteria

Rollback is triggered if any of the following are detected within the monitoring window:

- Token transfers fail or produce incorrect amounts.
- Event payloads are malformed or missing.
- Contract panics with unexpected errors.
- Storage contains corrupted or missing data.

## Rollback Procedure

1. **Immediate**: Call `pause` on the new contract to halt operations.
2. **Restore**: Point all traffic back to the old contract ID.
3. **Verify**: Run the verification steps against the old contract.
4. **Investigate**: Fix the issue in the new WASM and repeat the upgrade process.

## Post-Upgrade Monitoring

Monitor the following for at least 24 hours after upgrade:

- Transaction success rate (target: >99%).
- Event emission completeness.
- Storage record consistency (use view functions).
- Error rate per operation type.

## Rollback Safety Considerations

- Upgrades are one-directional on Soroban: the old WASM is replaced in-place.
- Always deploy to a new contract ID first and migrate traffic, rather than upgrading in place.
- Keep the old contract ID and deployment artifacts for at least 30 days.
- Test the rollback procedure on testnet before using it on mainnet.
