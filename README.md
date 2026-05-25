# stylus-profiler
[![repo](https://img.shields.io/badge/repo-stylus-profiler-blue)](https://github.com/kcolbchain/stylus-profiler)

WASM binary analyzer for Arbitrum Stylus contracts. By [kcolbchain](https://kcolbchain.com) (est. 2015).

## Why this exists

Stylus smart contracts compile Rust to WASM, with hard size limits: 128KB uncompressed, 24KB compressed. Developers constantly hit the 24KB limit with no tooling to diagnose *which functions are eating the budget*. This tool tells you.

## Commands

```bash
# Full analysis: size, structure, top functions, gas estimates
stylus-profiler analyze target/wasm32-unknown-unknown/release/my_contract.wasm

# Size breakdown: which functions are largest
stylus-profiler size my_contract.wasm --top 20

# Compare builds: catch size regressions
stylus-profiler compare old_build.wasm new_build.wasm

# Optimization suggestions
stylus-profiler optimize my_contract.wasm
```

## Example output

```
  WASM Analysis
  ──────────────────────────────────────────────────
  ✓ File size:         18.2KB
    Est. compressed:   4.5KB (limit: 24.0KB)
    Headroom:          19.5KB remaining

  Structure
    Functions:   47
    Code size:   12.1KB
    Imports:     8
    Exports:     5
    Memory:      16 pages (1.0MB)

  Top functions by size
      #      Size  Instrs      ~Gas  Name
  ──────────────────────────────────────────────────
      1    2.1KB     312       340  transfer
      2    1.8KB     267       298  approve
      3    1.4KB     189       210  balance_of
```

## Install

```bash
cargo install --git https://github.com/kcolbchain/stylus-profiler
```

## What it checks

- **Size limits**: uncompressed (128KB) and estimated compressed (24KB) vs Stylus limits
- **Function-level breakdown**: which functions consume the most bytes
- **Gas estimation**: per-function gas cost estimate based on instruction count
- **Optimization suggestions**: debug sections to strip, inlining opportunities, unused imports
- **Build comparison**: diff between versions to catch regressions

## License

MIT
