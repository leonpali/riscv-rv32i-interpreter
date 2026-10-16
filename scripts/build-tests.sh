#!/usr/bin/env bash
# Assembles tests/programs/asm/*.S into flat binaries in tests/programs/bin/.
# Each binary is linked at address 0 with _start first, which is where the
# loader puts it and starts execution.
#
# --no-relax: without it the linker rewrites `la` into gp-relative
# addressing, but nothing sets up gp (x3) in these programs.
#
# Set CROSS to use a different toolchain prefix, e.g.
#   CROSS=riscv32-unknown-elf- scripts/build-tests.sh
set -euo pipefail

CROSS=${CROSS:-riscv64-unknown-elf-}
ROOT=$(cd "$(dirname "$0")/.." && pwd)
SRC=$ROOT/tests/programs/asm
OUT=$ROOT/tests/programs/bin
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

mkdir -p "$OUT"

for src in "$SRC"/*.S; do
    name=$(basename "$src" .S)
    "${CROSS}as" -march=rv32i -mabi=ilp32 -I "$SRC" -o "$TMP/$name.o" "$src"
    "${CROSS}ld" -m elf32lriscv --no-relax -nostdlib -Ttext=0 -e _start -o "$TMP/$name.elf" "$TMP/$name.o"
    "${CROSS}objcopy" -O binary "$TMP/$name.elf" "$OUT/$name.bin"
    echo "built $name.bin"
done
