# Error Codes Documentation

This document lists all error codes used across stellarAid contracts.

## Common Error Codes

| Code | Name | Description |
|------|------|-------------|
| E001 | NotInitialized | Contract has not been initialized |
| E002 | AlreadyInitialized | Contract is already initialized |
| E003 | Unauthorized | Caller is not authorized to perform this action |
| E004 | InvalidAmount | The provided amount is invalid (zero or negative) |
| E005 | InsufficientBalance | Insufficient balance for the operation |
| E006 | InvalidAddress | The provided address is invalid or empty |
| E007 | OperationFailed | Generic operation failure |
| E008 | Overflow | Arithmetic overflow detected |
| E009 | DeadlineExpired | Transaction deadline has expired |
| E010 | NotFound | Requested resource was not found |

## Usage

Error codes are returned as part of contract error responses. Each contract
maps these codes to specific ContractError enum variants.