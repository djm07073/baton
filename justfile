env_nightly_version := env("NIGHTLY_VERSION", "nightly")
nightly_version := if env_nightly_version != "" { "+" + env_nightly_version } else { "" }
rustfmt := env("RUSTFMT", "rustfmt")

# Compile the intentionally incomplete scaffold and verify its specification references.
check: check-fmt clippy
    cargo check --workspace --all-targets --locked
    python3 scripts/check_scaffold.py
    npm --prefix docs run docs:check

# Enumerate source files because stability_mod! hides modules from cargo fmt.
fix-fmt:
    find node/src tx-pool/src alignment/src execution/src storage/src -name '*.rs' -type f -print0 | xargs -0 {{rustfmt}} {{nightly_version}} --edition 2024

check-fmt:
    find node/src tx-pool/src alignment/src execution/src storage/src -name '*.rs' -type f -print0 | xargs -0 {{rustfmt}} {{nightly_version}} --edition 2024 --check

clippy:
    cargo clippy --workspace --all-targets --locked -- -D warnings

# Build the documentation after installing its dependencies with npm --prefix docs ci.
docs:
    npm --prefix docs run docs:build

# Run focused deterministic tests after implementing a task.
test *args:
    cargo nextest run --workspace --locked {{args}}

# Verify that every public App interface disappears above its ALPHA stability scope.
check-stability:
    #!/usr/bin/env sh
    set -eu
    for package in baton-tx-pool baton-alignment baton-execution baton-storage; do
        for level in BETA GAMMA DELTA EPSILON RESERVED; do
            cargo rustdoc -p "$package" --lib --locked -- --cfg commonware_stability_$level
        done
    done
    cargo doc --workspace --no-deps --locked
