# Astro Implementation (Rust)

*Status: Draft v0.3*

This file covers the **Astro/Commonware daemon** side: repo shape (matching Alto), internal types, encoding, ABCI v2 adapter behavior, storage, and testing.

---

## 1. Repo & Module Layout (match Alto)

```
astro/
  chain/
    src/
      abci/           # ABCI v2 adapter (server/client)
      adapter.rs      # glue: consensus ↔ ABCI calls
      attest/         # helpers for recognizing AttestTx bytes (no policy)
      mempool/        # reused from Alto (no special lanes)
      net/            # reused from Alto
      store/          # reused from Alto
      ...             # other Alto modules unchanged
  types/
```

---

## 2. Internal Block Types (no protobuf)

```rust
use bytes::Bytes;

#[derive(Clone, Debug)]
pub struct BlockHeaderV1 {
  pub height: u64,
  pub parent_id: [u8; 32],
  pub time_unix_ns: u64,
  pub proposer_address: [u8; 20],
  pub view: u32,
  pub txs_root: [u8; 32],
  pub app_version: u32,
  pub parent_results_root: Option<[u8; 32]>,
}

#[derive(Clone, Debug)]
pub struct BlockBodyV1 { pub txs: Vec<Bytes> }

#[derive(Clone, Debug)]
pub struct BlockV1 { pub header: BlockHeaderV1, pub body: BlockBodyV1, pub qc: Option<Bytes> }
```

### 2.1 Astro Canonical Encoding (ACE)

* `u32/u64` → LEB128 varint; fixed arrays raw; `Option<T>` → 1‑byte tag; `Vec<T>` → length (LEB128) + items.
* `block_id = sha256(b"astro.header.v1" || encode(header))`
* `txs_root = Tendermint SimpleHashFromByteSlices(body.txs)`

---

## 3. ABCI v2 Adapter Behavior

* **PrepareProposal(h)**: read mempool (default ordering), send txs → app; receive filtered list.
* **ProcessProposal(h)**: relay decision to consensus (abort view on reject).
* **FinalizeBlock(h)**: after decision, execute on app; collect tx results; no attestation caching in Astro.
* **Commit()**: fetch `AppHash(h)`; persist `Block(h)` only; SDK owns header/commit persistence & serving.
* **CheckTx**: forward to app; no Astro‑side priority or fees.

---

## 4. Storage & Sync

* Append‑only block files with mmap; index `{height → (file, offset)}`.
* Syncer exposes: get block by height/id and ranges for catch-up (headers/commits are served by the SDK).

---

## 5. Keys & CLI

* **BLS share** for consensus (internal only). `astrod keys bls`.
* **Ed25519** remains in the validator’s SDK keyring (used by `AttestTx`).
* Config: `abci.socket`, consensus timeouts, view backoff. No attestation lanes here.

---

## 6. Testing

* Port Alto consensus tests.
* Adapter integration test: simulate app responses across the ABCI v2 flow.
* Storage round‑trip + `txs_root` invariants.
