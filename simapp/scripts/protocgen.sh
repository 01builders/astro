#!/usr/bin/env bash
set -euo pipefail

export GOTOOLCHAIN="${GOTOOLCHAIN:-auto}"

echo "[protocgen] Formatting protobuf files"
find ./ -name "*.proto" -exec clang-format -i {} \;

echo "[protocgen] Generating gogo proto code (gocosmos)"
cd proto

# Generate per-file like Cosmos SDK script (skip cosmossdk.io/api packages)
proto_dirs=$(find ./astro -path -prune -o -name '*.proto' -print0 | xargs -0 -n1 dirname | sort | uniq)
for dir in $proto_dirs; do
  for file in $(find "${dir}" -maxdepth 1 -name '*.proto'); do
    if grep -q "option go_package" "$file" && grep -H -o -c 'option go_package.*cosmossdk.io/api' "$file" | grep -q ':0$'; then
      buf generate --template ../buf.gen.gogo.yaml "$file"
    fi
  done
done

cd ..

# Sync nested module path into the repo tree used by the app
if [ -d github.com/binary-builders/astro/simapp/x/astro/types ]; then
  echo "[protocgen] syncing -> x/astro/types"
  rsync -a github.com/binary-builders/astro/simapp/x/astro/types/ x/astro/types/
fi

rm -rf github.com cosmossdk.io || true

echo "[protocgen] done"
