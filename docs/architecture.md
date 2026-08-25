# StellarAid Contract Architecture

## Diagram

    +---------------------------+
    | CommissionAgreement       |
    |---------------------------|
    | create_agreement()        |
    | sign_agreement()          |
    | release_commission()      |
    +---------------------------+
               |
               v
    +---------------------------+
    | DisputeArbiter            |
    |---------------------------|
    | raise_dispute()           |
    | resolve_dispute()         |
    +---------------------------+
               |
               v
    +---------------------------+
    | Event Emitter             |
    |---------------------------|
    | AgreementCreated          |
    | DisputeResolved           |
    +---------------------------+