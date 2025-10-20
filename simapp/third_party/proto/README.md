Place third-party `.proto` dependencies here if vendoring.

Common examples:
- cosmos/gogo-proto
- cosmos/cosmos-proto
- googleapis/google/api

If you use the `ghcr.io/cosmos/proto-builder` image with `buf`, you typically don't need to vendor these; `buf` fetches them from the Buf Schema Registry per `buf.yaml` deps.

