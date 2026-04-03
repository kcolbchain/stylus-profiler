use crate::analyzer::WasmAnalysis;
use crate::gas;
use crate::size;
use colored::Colorize;

pub fn print_analysis(analysis: &WasmAnalysis) {
    println!("\n  {}", "WASM Analysis".bold());
    println!("  {}", "─".repeat(50).dimmed());

    // Size limits
    let limits = size::check_limits(analysis.file_size);
    let size_icon = if limits.compressed_ok { "✓".green() } else { "✗".red() };

    println!("  {} File size:         {}", size_icon,
        format_bytes(analysis.file_size).bold());
    println!("    Est. compressed:   {} (limit: {})",
        format_bytes(limits.estimated_compressed),
        format_bytes(size::MAX_COMPRESSED_WASM));
    if limits.compressed_ok {
        println!("    Headroom:          {} remaining",
            format_bytes(limits.headroom_compressed).green());
    } else {
        let over = limits.estimated_compressed - size::MAX_COMPRESSED_WASM;
        println!("    {} Over limit by {}",
            "⚠".red(), format_bytes(over).red());
    }

    println!("\n  {}", "Structure".bold());
    println!("    Functions:   {}", analysis.total_functions());
    println!("    Code size:   {}", format_bytes(analysis.total_code_size()));
    println!("    Imports:     {}", analysis.imports.len());
    println!("    Exports:     {}", analysis.exports.len());
    if let Some(pages) = analysis.memory_pages {
        println!("    Memory:      {} pages ({})", pages, format_bytes(pages as usize * 65536));
    }
    println!("    Data segs:   {}", format_bytes(analysis.data_segment_size));

    // Top functions by size
    println!("\n  {}", "Top functions by size".bold());
    println!("  {:>5}  {:>8}  {:>6}  {:>8}  {}", "#", "Size", "Instrs", "~Gas", "Name");
    println!("  {}", "─".repeat(50).dimmed());

    let mut sorted = analysis.functions.clone();
    sorted.sort_by(|a, b| b.body_size.cmp(&a.body_size));

    for (i, func) in sorted.iter().take(15).enumerate() {
        let name = func.name.as_deref().unwrap_or(&format!("func_{}", func.index));
        let gas_est = gas::estimate_gas(func);
        println!("  {:>5}  {:>8}  {:>6}  {:>8}  {}",
            i + 1,
            format_bytes(func.body_size),
            func.instruction_count,
            gas_est,
            name.cyan(),
        );
    }
    println!();
}

pub fn print_size_breakdown(analysis: &WasmAnalysis, top: usize) {
    println!("\n  {} (top {})", "Size breakdown".bold(), top);
    println!("  {}", "─".repeat(60).dimmed());

    let mut sorted = analysis.functions.clone();
    sorted.sort_by(|a, b| b.body_size.cmp(&a.body_size));

    let total = analysis.total_code_size();

    for (i, func) in sorted.iter().take(top).enumerate() {
        let name = func.name.as_deref().unwrap_or(&format!("func_{}", func.index));
        let pct = if total > 0 { func.body_size as f64 / total as f64 * 100.0 } else { 0.0 };
        let bar_len = (pct / 2.0) as usize;
        let bar = "█".repeat(bar_len);

        println!("  {:>3}. {:>8} ({:>5.1}%) {} {}",
            i + 1, format_bytes(func.body_size), pct, bar.cyan(), name);
    }

    if !analysis.custom_sections.is_empty() {
        println!("\n  {}", "Custom sections".bold());
        for (name, sz) in &analysis.custom_sections {
            println!("    {:>8}  {}", format_bytes(*sz), name.dimmed());
        }
    }
    println!();
}

pub fn print_optimizations(analysis: &WasmAnalysis) {
    println!("\n  {}", "Optimization suggestions".bold());
    println!("  {}", "─".repeat(50).dimmed());

    let mut suggestions = Vec::new();

    // Check for large custom sections (debug info, names)
    for (name, sz) in &analysis.custom_sections {
        if *sz > 1024 {
            suggestions.push(format!(
                "Custom section '{}' is {} — strip with wasm-opt --strip-debug",
                name, format_bytes(*sz)
            ));
        }
    }

    // Check for large data segments
    if analysis.data_segment_size > 4096 {
        suggestions.push(format!(
            "Data segments total {} — consider lazy-loading large constants",
            format_bytes(analysis.data_segment_size)
        ));
    }

    // Check function count (many small functions = overhead)
    if analysis.total_functions() > 200 {
        suggestions.push(format!(
            "{} functions — consider inlining small helpers (wasm-opt --inline-functions)",
            analysis.total_functions()
        ));
    }

    // Check if many imports (each import adds overhead)
    if analysis.imports.len() > 30 {
        suggestions.push(format!(
            "{} imports — review for unused imports",
            analysis.imports.len()
        ));
    }

    // Size-specific suggestions
    let limits = size::check_limits(analysis.file_size);
    if !limits.compressed_ok {
        suggestions.push("Over compressed size limit! Try: wasm-opt -Oz, strip debug, reduce generics".into());
    }

    if suggestions.is_empty() {
        println!("  {} No obvious optimizations found", "✓".green());
    } else {
        for (i, s) in suggestions.iter().enumerate() {
            println!("  {}. {}", (i + 1).to_string().yellow(), s);
        }
    }
    println!();
}

fn format_bytes(bytes: usize) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}
