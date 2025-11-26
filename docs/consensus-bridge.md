# CommonWare ↔ Cosmos Bridge

This doc shows how the Rust consensus side (CommonWare/Astro) links into the Go application side (Cosmos SDK Simapp + x/astro).

## Diagram

```
CommonWare Consensus (Rust)
          |
          | Astro blocks + validator metadata
          v
    AbciExecutor (Rust)
          |
          | ABCI v1beta3 gRPC
          v
      Simapp (Go)
          |
          | x/astro keeper/module
          v
    Astro Module (Go)
```

## Component roles

- CommonWare consensus: builds and decides Astro blocks; expects deterministic app responses and validator metadata.
- `AbciExecutor` (Rust, `crates/application/src/abci_executor.rs`): converts between Astro block/validator types and ABCI requests/responses; drives `InitChain` and `FinalizeBlock` over gRPC.
- Simapp (Go, `simapp/`): stock Cosmos SDK wiring that hosts x/astro; provides ABCI server CometBFT expects.
- x/astro module (Go, `simapp/x/astro/`): owns genesis layout, params, and the validator set record; keeps the SDK view aligned with the consensus-layer identity.

## Genesis path

1. Operator runs `astrod astro-bls-validator <moniker>` (from `simapp/x/astro/client/cli/genesis.go`). It creates a BLS12-381 MinSig keypair, writes `identity` plus a single validator entry into `genesis.json` under the x/astro module state.
2. During `InitGenesis`, the module stores params and validators via the keeper (`simapp/x/astro/keeper/keeper.go`) and returns no validator updates; consensus is PoA/static for now.
3. The Rust side calls `AbciExecutor::do_genesis`, sending the app state bytes in `RequestInitChain`; it records the returned `app_hash` as the application commitment for height 0/initial height.

## Block execution path

1. Consensus builds an `AstroBlock` and hands it to `AbciExecutor::finalize_block`, which will map it to `RequestFinalizeBlock` (conversion TODO) and send it to Simapp.
2. Simapp runs the usual Cosmos SDK pipeline; module logic can emit events/tx results, which ABCI returns in `ResponseFinalizeBlock`.
3. `AbciExecutor` converts the response into `BlockResult` (app hash, tx results, events) for the consensus engine to seal the block and advance state.

## Validator set and identity

- x/astro keeps validators keyed by moniker with an opaque `Identity` byte string (intended to hold the BLS MinSig public key used by the CommonWare side).
- No staking/updates exist yet; `ApplyAndReturnValidatorSetUpdates` simply replays the stored validators as ABCI updates with constant voting power, matching the current PoA/static model.
- Future dynamic sets can add messages that mutate the keeper’s `Validators` map; until then, the SDK view remains fixed at what was written in genesis.

## ABCI type translation (AbciExecutor)

- Converts Unix millis to `Timestamp` for ABCI requests; builds `RequestInitChain` and `RequestFinalizeBlock` stubs.
- Maps `AstroValidator` → `CometValidator` (`astro_validator_into_comet_validator`), encoding the public key bytes the SDK/Comet side can consume.
- Converts `ResponseFinalizeBlock` into the internal `BlockResult`, validating sizes such as `app_hash`.
