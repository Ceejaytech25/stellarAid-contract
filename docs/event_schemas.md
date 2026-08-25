# StellarAid Event Schemas and Contract Signals

## Overview

This document describes the events emitted by StellarAid smart contracts.

---

## AgreementCreated

Emitted when a new commission agreement is created.

| Field         | Type   | Description                        |
|---------------|--------|------------------------------------|
| agreement_id  | u64    | Unique identifier for the agreement|
| creator       | String | Address of the agreement creator   |
| recipient     | String | Address of the commission recipient|
| amount        | u64    | Commission amount                  |

---

## AgreementSigned

Emitted when a party signs an agreement.

| Field        | Type   | Description              |
|--------------|--------|--------------------------|
| agreement_id | u64    | ID of the signed agreement|
| signer       | String | Address of the signer    |

---

## CommissionReleased

Emitted when commission is released to the recipient.

| Field        | Type   | Description                    |
|--------------|--------|--------------------------------|
| agreement_id | u64    | ID of the settled agreement    |
| recipient    | String | Address that received payment  |
| amount       | u64    | Amount released                |

---

## DisputeRaised

Emitted when a dispute is raised on an agreement.

| Field        | Type   | Description                  |
|--------------|--------|------------------------------|
| dispute_id   | u64    | Unique identifier of dispute |
| agreement_id | u64    | Associated agreement ID      |
| reason       | String | Reason for the dispute       |

---

## DisputeResolved

Emitted when a dispute is resolved.

| Field      | Type   | Description                   |
|------------|--------|-------------------------------|
| dispute_id | u64    | ID of the resolved dispute    |
| winner     | String | Address of the winning party  |