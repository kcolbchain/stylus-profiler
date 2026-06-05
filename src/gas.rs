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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimates_instruction_local_and_size_costs() {
        let func = FunctionInfo {
            index: 0,
            name: Some("hot_path".to_string()),
            body_size: 250,
            local_count: 4,
            instruction_count: 30,
        };

        assert_eq!(estimate_gas(&func), 30 + (4 * 3) + 25);
    }
}
