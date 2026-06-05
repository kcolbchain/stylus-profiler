use crate::analyzer::WasmAnalysis;
use colored::Colorize;

pub fn print_comparison(old: &WasmAnalysis, new: &WasmAnalysis) {
    println!("\n  {}", "Binary comparison".bold());
    println!("  {}", "─".repeat(50).dimmed());

    let size_diff = new.file_size as i64 - old.file_size as i64;
    let size_pct = if old.file_size > 0 {
        size_diff as f64 / old.file_size as f64 * 100.0
    } else {
        0.0
    };

    let func_diff = new.total_functions() as i64 - old.total_functions() as i64;
    let code_diff = new.total_code_size() as i64 - old.total_code_size() as i64;

    print_diff("Total size", old.file_size, new.file_size, size_diff, size_pct);
    print_diff(
        "Code size",
        old.total_code_size(),
        new.total_code_size(),
        code_diff,
        if old.total_code_size() > 0 {
            code_diff as f64 / old.total_code_size() as f64 * 100.0
        } else {
            0.0
        },
    );

    println!(
        "  {:20} {:>8} → {:>8} ({})",
        "Functions".dimmed(),
        old.total_functions(),
        new.total_functions(),
        format_signed(func_diff),
    );

    if size_diff > 0 {
        println!("\n  {} Binary grew by {} bytes ({:+.1}%)", "⚠".yellow(), size_diff, size_pct);
    } else if size_diff < 0 {
        println!("\n  {} Binary shrank by {} bytes ({:.1}%)", "✓".green(), -size_diff, size_pct);
    } else {
        println!("\n  {} No size change", "●".dimmed());
    }
    println!();
}

fn print_diff(label: &str, old: usize, new: usize, diff: i64, pct: f64) {
    let change = if diff > 0 {
        format!("+{} ({:+.1}%)", diff, pct).red().to_string()
    } else if diff < 0 {
        format!("{} ({:.1}%)", diff, pct).green().to_string()
    } else {
        "no change".dimmed().to_string()
    };

    println!("  {:20} {:>8} → {:>8}  {}", label.dimmed(), format_bytes(old), format_bytes(new), change);
}

fn format_bytes(bytes: usize) -> String {
    if bytes >= 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}

fn format_signed(n: i64) -> String {
    if n > 0 { format!("+{}", n) } else { format!("{}", n) }
}
