#!/bin/bash
# Builds lib.rs as a static library with RUSTC, links it into host.c with clang, runs the host and
# checks what it prints. A panic that escapes an `extern "C"` function must abort cleanly.
# Usage: run.sh RUSTC OPT_LEVEL OUT_DIR [TARGET [CLANG_ARCH]]
set -euo pipefail

rustc="$1"
opt="$2"
out="$3"
target="${4:-arm64e-apple-darwin}"
arch="${5:-arm64e}"
here="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$out"
export MACOSX_DEPLOYMENT_TARGET=11.0

if ! "$rustc" --edition 2021 --target "$target" --crate-type staticlib -Ccodegen-units=1 \
    -Copt-level="$opt" --print=native-static-libs "$here/lib.rs" -o "$out/libapp_shape.a" \
    2> "$out/rustc.stderr"; then
    cat "$out/rustc.stderr"
    exit 1
fi
read -r -a native_libs <<< "$(sed -n 's/^note: native-static-libs: //p' "$out/rustc.stderr")"
clang -arch "$arch" -O1 "$here/host.c" "$out/libapp_shape.a" "${native_libs[@]}" -o "$out/host"
/usr/bin/file "$out/host"

# Runs the host with the given arguments and sets `stdout` and `status`; stderr goes to
# $out/NAME.stderr.
run_host() {
    local name="$1"
    shift
    set +e
    stdout="$("$out/host" "$@" 2> "$out/$name.stderr")"
    status=$?
    set -e
}

expected="constructor=1 callback=42 rust_table=49 qsort=1359 caught=-1 foreign_thread=1"
run_host main
echo "$stdout"
if [ "$status" != 0 ] || [ "$stdout" != "$expected" ]; then
    echo "expected: $expected, with exit status 0; exit status was $status" >&2
    cat "$out/main.stderr" >&2
    exit 1
fi

run_host escape escape
tail -n 2 "$out/escape.stderr"
# 134 is SIGABRT; a trap would be SIGTRAP (133) or SIGBUS (138).
if [ "$status" != 134 ] || ! grep -q "cannot unwind" "$out/escape.stderr"; then
    echo "an escaping panic must abort with \"panic in a function that cannot unwind\"" \
        "(SIGABRT, 134); exit status was $status" >&2
    cat "$out/escape.stderr" >&2
    exit 1
fi
echo "escaping panic: aborted cleanly"
