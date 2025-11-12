#!/usr/bin/env bash
set -euo pipefail

export GOTOOLCHAIN="${GOTOOLCHAIN:-auto}"

echo "[protocgen] Formatting protobuf files"
find ./ -name "*.proto" -exec clang-format -i {} \;

echo "[protocgen] Generating gogo proto code (gocosmos, full module)"
(
  cd proto
  # Generate all protos in module using gocosmos template at repo root
  buf generate --template ../buf.gen.gogo.yaml
)

# Sync nested module path into the repo tree used by the app and ensure fresh files
GEN_DIR=github.com/binary-builders/astro/simapp/x/astro/types
DST_DIR=x/astro/types
if [ -d "$GEN_DIR" ]; then
  echo "[protocgen] cleaning $DST_DIR/*.pb.go"
  rm -f $DST_DIR/*_grpc.pb.go $DST_DIR/*.pb.go || true
  echo "[protocgen] syncing $GEN_DIR -> $DST_DIR"
  mkdir -p "$DST_DIR"
  cp -a "$GEN_DIR/." "$DST_DIR/"
fi

rm -rf github.com cosmossdk.io || true

echo "[protocgen] done"
