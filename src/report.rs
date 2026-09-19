use std::collections::HashMap;
use std::time::Duration;
use colored::*;
use crate::models::{Category, PageAuditReport, Severity};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Terminal,
    Json,
    Github,
}

pub fn print_report(reports: &[PageAuditReport], elapsed: Duration, format: ReportFormat, verbose: bool) {
    match format {
        ReportFormat::Json => {
            if let Ok(json) = serde_json::to_string_pretty(reports) {
                println!("{}", json);
            }
        }
        ReportFormat::Github => {
            for report in reports {
                for issue in &report.issues {
                    let level = match issue.severity {
                        Severity::Error => "error",
                        Severity::Warning => "warning",
                        Severity::Info => "notice",
                    };
                    println!("::{} file={}::[{}] {}", level, report.file_path.display(), issue.code, issue.message);
                }
            }
            print_terminal_summary(reports, elapsed);
        }
        ReportFormat::Terminal => {
            print_terminal_report(reports, elapsed, verbose);
        }
    }
}

fn print_terminal_report(reports: &[PageAuditReport], elapsed: Duration, verbose: bool) {
    let mut category_counts: HashMap<Category, usize> = HashMap::new();

    for report in reports {
        let errs = report.errors_count();
        let warns = report.warnings_count();

        for issue in &report.issues {
            *category_counts.entry(issue.category).or_insert(0) += 1;
        }

        if errs > 0 || (warns > 0 && verbose) {
            let status = if errs > 0 {
                "FAIL".red().bold()
            } else {
                "WARN".yellow().bold()
            };

            println!("\n{} {} (Capo: {:.1}%)", status, report.url_path.bold(), report.capo_score);
            for issue in &report.issues {
                if issue.severity == Severity::Error || verbose {
                    let badge = match issue.severity {
                        Severity::Error => "✗ ERROR".red(),
                        Severity::Warning => "⚠ WARN ".yellow(),
                        Severity::Info => "ℹ INFO ".blue(),
                    };
                    println!("   {} [{}] {}", badge, issue.code.dimmed(), issue.message);
                }
            }
        }
    }

    print_terminal_summary(reports, elapsed);
}

fn print_terminal_summary(reports: &[PageAuditReport], elapsed: Duration) {
    let total_pages = reports.len();
    let total_errors: usize = reports.iter().map(|r| r.errors_count()).sum();
    let total_warnings: usize = reports.iter().map(|r| r.warnings_count()).sum();

    let avg_capo: f64 = if total_pages > 0 {
        reports.iter().map(|r| r.capo_score).sum::<f64>() / total_pages as f64
    } else {
        100.0
    };

    println!("\n{}", "────────────────────────────────────────────────────────────".dimmed());
    println!("📋 {}", "Audit Summary Dashboard".bold());
    println!("   Pages Scanned:         {}", total_pages.to_string().bold());
    println!("   Execution Time:        {:.2?}", elapsed);
    println!("   Average Capo Score:    {:.1}%", avg_capo);
    println!("   Errors:                {}", if total_errors > 0 { total_errors.to_string().red().bold() } else { "0".green().bold() });
    println!("   Warnings:              {}", if total_warnings > 0 { total_warnings.to_string().yellow().bold() } else { "0".green().bold() });
    println!("{}", "────────────────────────────────────────────────────────────".dimmed());
}
