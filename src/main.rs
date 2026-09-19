use std::path::PathBuf;
use std::process::exit;
use std::time::Instant;
use clap::Parser;

use unified_audit::{
    audit_directory, print_report, AuditOptions, ReportFormat,
};

#[derive(Parser, Debug)]
#[command(author, version, about = "High-performance unified post-build CI verification engine for Astro sites")]
struct Args {
    /// Directory containing built HTML files (e.g. dist/)
    #[arg(default_value = "dist")]
    dir: PathBuf,

    /// Strict mode: fail and exit with code 1 on any error
    #[arg(long, default_value_t = false)]
    strict: bool,

    /// Output format: terminal, json, github
    #[arg(long, default_value = "terminal")]
    format: String,

    /// Verbose output (prints warnings and info diagnostics for all pages)
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// Comma-separated URL prefixes to skip in link checking (default: "/cdn-cgi/,/api/")
    #[arg(long, default_value = "/cdn-cgi/,/api/")]
    skip_links: String,

    /// Comma-separated rule codes to ignore
    #[arg(long, default_value = "")]
    ignore_rules: String,
}

fn main() {
    let args = Args::parse();
    let start_time = Instant::now();

    if !args.dir.is_dir() {
        eprintln!("Error: target directory '{}' does not exist", args.dir.display());
        exit(1);
    }

    let skip_link_patterns: Vec<String> = args
        .skip_links
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let ignore_rules: Vec<String> = args
        .ignore_rules
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let options = AuditOptions {
        skip_link_patterns,
        ignore_rules,
    };

    let report_format = match args.format.to_lowercase().as_str() {
        "json" => ReportFormat::Json,
        "github" => ReportFormat::Github,
        _ => ReportFormat::Terminal,
    };

    if report_format != ReportFormat::Json {
        println!("🔍 Scanning static build in: {}", args.dir.display());
    }
    let reports = audit_directory(&args.dir, &options);
    let elapsed = start_time.elapsed();

    print_report(&reports, elapsed, report_format, args.verbose);

    let total_errors: usize = reports.iter().map(|r| r.errors_count()).sum();

    if args.strict && total_errors > 0 {
        eprintln!("\n❌ Audit failed: {} error(s) detected in strict mode.", total_errors);
        exit(1);
    }
}
