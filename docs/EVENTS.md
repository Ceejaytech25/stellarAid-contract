# Contract Events

All Soroban contracts emit structured events for off-chain indexing and analytics. Events are published using `env.events().publish()` with a topic tuple and payload.

## Event Format

Each event follows this general structure:

```rust
env.events().publish(
    (symbol!("<contract>"), symbol!("<action>")),
    (field1, field2, ...),
);
```

The first element of the topic tuple identifies the contract domain; the second identifies the action. The body contains the payload fields.

## Escrow Contract

### `escrow` / `created`

Emitted when a new escrow is created and USDC is locked.

| Field         | Type    | Description                             |
|---------------|---------|-----------------------------------------|
| commission_id | Bytes   | Unique commission identifier            |
| amount        | i128    | Amount locked in stroops                |

**Consumer example:**

```typescript
// Subscribe to escrow.created events and index them
eventListener.onEvent("escrow", "created", (event) => {
  const { commission_id, amount } = event.payload;
  db.insertEscrow({ commissionId: commission_id, amount });
});
```

### `escrow` / `released`

Emitted when payment is released to the artist.

| Field         | Type    | Description                     |
|---------------|---------|---------------------------------|
| commission_id | Bytes   | Unique commission identifier    |
| payout        | i128    | Amount paid to artist (net fee) |
| fee           | i128    | Platform fee deducted           |

### `escrow` / `refunded`

Emitted when funds are refunded to the client.

| Field         | Type    | Description                  |
|---------------|---------|------------------------------|
| commission_id | Bytes   | Unique commission identifier |
| client        | Address | Client receiving refund      |
| amount        | i128    | Refund amount                |

### `escrow` / `expired`

Emitted when an escrow expires without release.

| Field         | Type    | Description                  |
|---------------|---------|------------------------------|
| commission_id | Bytes   | Unique commission identifier |
| expiry_ledger | u32     | Ledger at which it expired   |

### `escrow` / `disputed`

Emitted when a dispute is opened on an escrow.

| Field         | Type    | Description                  |
|---------------|---------|------------------------------|
| commission_id | Bytes   | Unique commission identifier |
| initiator     | Address | Address that opened dispute  |

## Platform Config Contract

### `config_initialized`

Emitted when the platform configuration is initialized.

| Field      | Type    | Description          |
|------------|---------|----------------------|
| admin      | Address | Admin address        |
| usdc_token | Address | USDC token address   |
| fee_bps    | u32     | Platform fee in bps  |

### `fee_bps_updated`

Emitted when the platform fee is changed.

| Field        | Type    | Description                |
|--------------|---------|----------------------------|
| old_fee_bps  | u32     | Previous fee in bps        |
| new_fee_bps  | u32     | New fee in bps             |

### `admin_transfer_initiated`

Emitted when an admin transfer is requested.

| Field        | Type    | Description                  |
|--------------|---------|------------------------------|
| current_admin| Address | Current admin address        |
| pending_admin| Address | Proposed new admin address   |

### `admin_transfer_completed`

Emitted when the pending admin accepts the transfer.

| Field        | Type    | Description                 |
|--------------|---------|-----------------------------|
| old_admin    | Address | Previous admin address      |
| new_admin    | Address | New admin address           |

## Campaign Contract

### `campaign_registered`

Emitted when a new campaign is created.

| Field       | Type      | Description                     |
|-------------|-----------|---------------------------------|
| campaign_id | u64       | Unique campaign identifier      |
| owner       | Address   | Campaign creator address        |
| goal        | i128      | Fundraising target amount       |
| deadline    | u64       | Campaign expiration timestamp   |

### `campaign_status_changed`

Emitted when a campaign's status is updated.

| Field       | Type            | Description               |
|-------------|-----------------|---------------------------|
| campaign_id | u64             | Campaign identifier       |
| old_status  | CampaignStatus  | Previous status           |
| new_status  | CampaignStatus  | New status                |

### `campaign_archived`

Emitted when a completed or expired campaign is archived.

| Field       | Type    | Description               |
|-------------|---------|---------------------------|
| campaign_id | u64     | Campaign identifier       |

## Donation Contract

### `donation_made`

Emitted when a donation is made.

| Field       | Type    | Description            |
|-------------|---------|------------------------|
| donor       | Address | Donor address          |
| campaign_id | u64     | Target campaign        |
| amount      | i128    | Donation amount        |

### `refund_recorded`

Emitted when a refund is processed.

| Field       | Type    | Description                |
|-------------|---------|----------------------------|
| campaign_id | u64     | Campaign identifier        |
| donor       | Address | Original donor address     |
| amount      | i128    | Refund amount              |
| caller      | Address | Address authorizing refund |

## Withdrawal Contract

### `withdrawal_requested`

Emitted when a withdrawal is requested.

| Field         | Type    | Description              |
|---------------|---------|--------------------------|
| withdrawal_id | u64     | Unique withdrawal ID     |
| campaign_id   | u64     | Campaign identifier      |
| recipient     | Address | Funds recipient address  |
| amount        | i128    | Withdrawal amount        |

### `withdrawal_approved`

Emitted when a withdrawal is approved.

| Field         | Type      | Description                         |
|---------------|-----------|-------------------------------------|
| withdrawal_id | u64       | Unique withdrawal ID                |
| tx_hash       | BytesN<32>| Transaction hash (0 if unavailable) |

### `withdrawal_rejected`

Emitted when a withdrawal is rejected.

| Field         | Type    | Description              |
|---------------|---------|--------------------------|
| withdrawal_id | u64     | Unique withdrawal ID     |
| reason        | String  | Rejection reason         |

## Commission Agreement Contract

### `agr_created`

Emitted when a commission agreement is created.

| Field         | Type    | Description              |
|---------------|---------|--------------------------|
| commission_id | Bytes   | Unique commission ID     |
| client        | Address | Client address           |
| artist        | Address | Artist address           |
| budget_usdc   | i128    | Total budget in stroops  |

### `agr_accepted`

Emitted when an agreement is accepted by the artist.

| Field         | Type    | Description              |
|---------------|---------|--------------------------|
| commission_id | Bytes   | Unique commission ID     |

### `agr_rejected`

Emitted when an agreement is rejected by the artist.

| Field         | Type    | Description              |
|---------------|---------|--------------------------|
| commission_id | Bytes   | Unique commission ID     |
| reason        | String  | Rejection reason         |

### `ms_proposed`

Emitted when a milestone is proposed on an active agreement.

| Field         | Type    | Description              |
|---------------|---------|--------------------------|
| commission_id | Bytes   | Unique commission ID     |
| milestone_id  | Bytes   | Unique milestone ID      |
| amount_usdc   | i128    | Milestone amount         |

### `ms_approved`

Emitted when a milestone is approved by the client.

| Field         | Type    | Description              |
|---------------|---------|--------------------------|
| commission_id | Bytes   | Unique commission ID     |
| milestone_id  | Bytes   | Unique milestone ID      |

## Shared / Pause Events

### `contract_paused`

Emitted when the contract is paused.

| Field | Type    | Description            |
|-------|---------|------------------------|
| admin | Address | Pausing admin address  |

### `contract_unpaused`

Emitted when the contract is unpaused.

| Field | Type    | Description              |
|-------|---------|--------------------------|
| admin | Address | Unpausing admin address  |

## Indexing Guide

To index events for analytics:

1. Use the Soroban event RPC endpoint to subscribe to contract events.
2. Filter by topic to select the contract and action of interest.
3. The payload is a tuple of XDR-serialized fields matching the table above.
4. Deserialize using the matching Soroban SDK types.

### Sample Consumer Flow (TypeScript)

```typescript
import { Server, Contract } from '@stellar/stellar-sdk';

const server = new Server('https://rpc.testnet.stellar.org');

async function* pollEvents(contractId: string) {
  let cursor = 'now';
  while (true) {
    const response = await server.getEvents({
      startLedger: cursor === 'now' ? await server.getLatestLedger() : undefined,
      filters: [{ contractId, topics: [['escrow', 'created']] }],
      pagination: { limit: 50 },
    });
    for (const event of response.events) {
      yield event;
    }
    cursor = response.pagination?.cursor ?? cursor;
    await new Promise(r => setTimeout(r, 5000));
  }
}
```

This event format is stable. New events follow the same two-topic convention.
