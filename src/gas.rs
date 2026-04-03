use crate::analyzer::FunctionInfo;

/// Rough gas estimation based on Stylus WASM opcode costs.
/// ArbOS charges ~1 gas per WASM instruction (simplified model).
/// Memory operations and host calls are more expensive.
pub fn estimate_gas(func: &FunctionInfo) -> u64 {
    // Base cost: 1 gas per instruction (simplified ArbOS model)
    let base = func.instruction_count as u64;
    // Local variables add storage overhead
    let locals_overhead = func.local_count as u64 * 3;
    // Larger functions have more memory access overhead
    let size_overhead = (func.body_size as u64) / 10;

    base + locals_overhead + size_overhead
}
