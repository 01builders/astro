# Astro Spec — Overview

*Status: Draft v0.3 (ABCI v2)*

This file summarizes the end‑to‑end design so Cosmos SDK apps can swap Comet/CometBFT for **Astro (Commonware‑based)** while keeping **IBC light‑clients** fully compatible.

---

## 1. Goals

* **IBC Compatibility**: Preserve Comet‑shaped `Header` + `Commit` bytes so ICS‑07 Tendermint clients verify unmodified.
* **Modular Consensus**: Run the Commonware (Rust) daemon out‑of‑process via **ABCI v2 (gRPC)**.
* **Validator Attestations**: Collect validator precommit votes as normal txs (`AttestTx`) signed with **Ed25519**.
* **Low Migration Friction**: No changes to existing module logic; only an SDK adapter (`x/attest` + header builder).

---

## 2. Constraints & Invariants

* **Header shape must match Comet**: fields and encoding for `Header` and `Commit` are byte‑identical.
* **Ed25519 outward, BLS inward**: BLS is used internally by Commonware; exposed commits remain Ed25519.
* **ABCI v2 only**: `PrepareProposal → ProcessProposal → FinalizeBlock → Commit` (+ concurrent `CheckTx`).

---

## 3. Architecture

```
Cosmos‑SDK App (Go)  ⇆  ABCI v2 gRPC  ⇆  Astro / Commonware (Rust)
   • x/attest module                       • STC consensus core
   • Header/Commit builder                 • P2P + mempool + syncer
```

**Startup**: SDK spawns Astro → ABCI handshake (`Info`, `InitChain`) → block production.

---

## 4. Validator Attestations (Ed25519)

1. At height **h**, each validator signs **block h‑1** with Ed25519 (Comet vote sign‑bytes, type=Precommit).
2. The vote is carried as **`AttestTx`** and included in **block h**.
3. Proposals without ≥ **2/3 power** of `AttestTx(h‑1)` are **rejected** in `ProcessProposal`.
4. The SDK builds `(Header h‑1, Commit h‑1)` from the `AttestTx` set and exposes it to IBC.

---

## 5. ABCI v2 Mapping

* **PrepareProposal(h)**: app filters/returns txs; should prefer `AttestTx(h‑1)` until ≥2/3 power, then fill with user txs.
* **ProcessProposal(h)**: all validators deterministically accept/reject the proposed tx list.
* **FinalizeBlock(h)**: execute decided block; stage commit parts for **h‑1**.
* **Commit()**: persist state; return `AppHash(h)`; persist `(Header h‑1, Commit h‑1)`.

---

## 6. Header Construction (SDK side)

* `DataHash = SimpleHash(tx_bytes of block h‑1)`
* `Time = block h .time`
* `Commit(h‑1)` = ordered Ed25519 precommit vector from `AttestTx(h‑1)`
* All other header fields follow Comet semantics (validators, params, results, evidence, proposer).

---

## 7. Migration Plan (high‑level)

1. **Dual‑run** Comet & Astro and diff `(Header, Commit)`.
2. **Public testnet** with relayers attached.
3. **Mainnet upgrade** flips to Astro; validators run Rust daemon alongside the node.

---

## 8. Risks (capsule)

* Missing ≥⅓ `AttestTx` → delayed finality (mitigate with proposer retries / inclusion rules).
* AttestTx spam → fee floor & per‑validator quota (SDK‑side policy).
* Header divergence → dual‑run diffs + property tests.

---

## 9. Open Questions

* Allow same‑height attestations vs. `h‑1` only?
* Parameterizing gas/priority for `AttestTx` without abuse.
* Optional use of vote extensions for future features.
