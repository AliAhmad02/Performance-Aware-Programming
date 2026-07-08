#!/usr/bin/env bash

set -o pipefail

usage() {
    echo "Usage: $0 <asm_generator_binary> <reference_binary>" >&2
    exit 1
}

cleanup() {
    [[ -n "${TMPDIR_CREATED:-}" && -d "$TMPDIR_CREATED" ]] && rm -rf "$TMPDIR_CREATED"
}

error() {
    echo "Error: $*" >&2
    exit 1
}

[[ $# -eq 2 ]] || usage

GENERATOR="$1"
REFERENCE="$2"

[[ -e "$GENERATOR" ]] || error "Generator binary does not exist: $GENERATOR"
[[ -f "$GENERATOR" ]] || error "Generator path is not a regular file: $GENERATOR"
[[ -x "$GENERATOR" ]] || error "Generator binary is not executable: $GENERATOR"

[[ -e "$REFERENCE" ]] || error "Reference binary does not exist: $REFERENCE"
[[ -f "$REFERENCE" ]] || error "Reference path is not a regular file: $REFERENCE"

command -v nasm >/dev/null 2>&1 || error "nasm is not installed or not in PATH"

TMPDIR_CREATED="$(mktemp -d)"
trap cleanup EXIT

ASM_FILE="$TMPDIR_CREATED/out.asm"
BIN_FILE="$TMPDIR_CREATED/out.bin"

# Generate assembly.
if ! "$GENERATOR" >"$ASM_FILE"; then
    error "Generator binary failed."
fi

# Assemble.
if ! nasm "$ASM_FILE" -o "$BIN_FILE"; then
    error "Assembly failed."
fi

# Compare.
if cmp -s "$BIN_FILE" "$REFERENCE"; then
    echo "Identical"
else
    echo "Not identical"
fi
