use std::fs;
use std::path::{Path, PathBuf};
use rayon::prelude::*;
use scraper::Html;
use walkdir::WalkDir;

use crate::models::PageAuditReport;
use crate::seo::audit_seo;
use crate::a11y::audit_a11y;
use crate::images::audit_images;
use crate::schema::audit_schema;
use crate::links::{audit_links, LinkAuditConfig};
use crate::capo::audit_capo;

pub struct AuditOptions {
    pub skip_link_patterns: Vec<String>,
    pub ignore_rules: Vec<String>,
}

impl Default for AuditOptions {
    fn default() -> Self {
        Self {
            skip_link_patterns: vec!["/cdn-cgi/".to_string(), "/api/".to_string()],
            ignore_rules: Vec::new(),
        }
    }
}

pub fn audit_html_file(file_path: &Path, root_dir: &Path, options: &AuditOptions) -> Option<PageAuditReport> {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return None,
    };

    let document = Html::parse_document(&content);
    let mut issues = Vec::new();

    // 1. SEO audit
    issues.extend(audit_seo(&document));

    // 2. A11y audit
    issues.extend(audit_a11y(&document));

    // 3. Image audit
    issues.extend(audit_images(&document));

    // 4. Schema audit
    issues.extend(audit_schema(&content));

    // 5. Links audit
    let link_cfg = LinkAuditConfig {
        root_dir,
        current_file: file_path,
        skip_patterns: &options.skip_link_patterns,
    };
    issues.extend(audit_links(&document, &link_cfg));

    // 6. Capo audit
    let (capo_score, capo_issues) = audit_capo(&content);
    issues.extend(capo_issues);

    // Filter out ignored rules
    if !options.ignore_rules.is_empty() {
        issues.retain(|i| !options.ignore_rules.contains(&i.code));
    }

    let rel_url = file_path
        .strip_prefix(root_dir)
        .map(|p| format!("/{}", p.display()))
        .unwrap_or_else(|_| file_path.display().to_string());

    Some(PageAuditReport {
        file_path: file_path.to_path_buf(),
        url_path: rel_url,
        issues,
        capo_score,
    })
}

pub fn audit_directory(dir: &Path, options: &AuditOptions) -> Vec<PageAuditReport> {
    let mut html_files: Vec<PathBuf> = Vec::new();

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext.eq_ignore_ascii_case("html") || ext.eq_ignore_ascii_case("htm") {
                    html_files.push(path.to_path_buf());
                }
            }
        }
    }

    html_files
        .par_iter()
        .filter_map(|path| audit_html_file(path, dir, options))
        .collect()
}
