use std::path::{Path, PathBuf};
use scraper::{Html, Selector};
use crate::models::{Category, Issue, Severity};

pub struct LinkAuditConfig<'a> {
    pub root_dir: &'a Path,
    pub current_file: &'a Path,
    pub skip_patterns: &'a [String],
}

pub fn audit_links(document: &Html, config: &LinkAuditConfig) -> Vec<Issue> {
    let mut issues = Vec::new();
    let a_sel = Selector::parse("a[href]").unwrap();

    let current_dir = config.current_file.parent().unwrap_or(config.root_dir);

    for a in document.select(&a_sel) {
        let href = a.value().attr("href").unwrap_or("").trim();
        let text = a.text().collect::<String>().trim().to_string();

        // 1. Check empty href
        if href.is_empty() {
            issues.push(Issue {
                code: "LINK_EMPTY_HREF".to_string(),
                category: Category::Links,
                severity: Severity::Error,
                message: "Anchor <a> has an empty href attribute".to_string(),
                selector: Some("a[href=\"\"]".to_string()),
            });
            continue;
        }

        // 2. Check empty anchor text
        let has_img = a.select(&Selector::parse("img").unwrap()).next().is_some();
        let has_svg = a.select(&Selector::parse("svg").unwrap()).next().is_some();
        let has_aria = a.value().attr("aria-label").is_some() || a.value().attr("title").is_some();
        if text.is_empty() && !has_img && !has_svg && !has_aria {
            issues.push(Issue {
                code: "LINK_EMPTY_TEXT".to_string(),
                category: Category::A11y,
                severity: Severity::Warning,
                message: format!("Link to \"{}\" has empty link text and no accessible label", href),
                selector: Some(format!("a[href=\"{}\"]", href)),
            });
        }

        // 3. Skip external or protocol links
        if href.starts_with("http://") || href.starts_with("https://") || href.starts_with("//")
            || href.starts_with("mailto:") || href.starts_with("tel:") || href.starts_with("javascript:")
            || href.starts_with("data:") {
            continue;
        }

        // 4. Skip configured patterns (e.g. /cdn-cgi/, /api/)
        let mut skipped = false;
        for pattern in config.skip_patterns {
            if href.starts_with(pattern) {
                skipped = true;
                break;
            }
        }
        if skipped {
            continue;
        }

        // 5. In-page anchor check
        if href.starts_with('#') {
            let target_id = href.trim_start_matches('#');
            if !target_id.is_empty() {
                let id_sel = match Selector::parse(&format!("[id=\"{}\"]", target_id)) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                if document.select(&id_sel).next().is_none() {
                    issues.push(Issue {
                        code: "LINK_BROKEN_INPAGE_ANCHOR".to_string(),
                        category: Category::Links,
                        severity: Severity::Warning,
                        message: format!("In-page link \"{}\" references non-existent id=\"{}\"", href, target_id),
                        selector: Some(format!("a[href=\"{}\"]", href)),
                    });
                }
            }
            continue;
        }

        // 6. Resolve local file target
        let clean_path = href.split('?').next().unwrap_or(href);
        let clean_path = clean_path.split('#').next().unwrap_or(clean_path);

        let target_path = if clean_path.starts_with('/') {
            // Root-relative path
            let rel = clean_path.trim_start_matches('/');
            config.root_dir.join(rel)
        } else {
            // Relative path from current file's folder
            current_dir.join(clean_path)
        };

        if !target_exists(&target_path) {
            issues.push(Issue {
                code: "LINK_BROKEN_INTERNAL".to_string(),
                category: Category::Links,
                severity: Severity::Error,
                message: format!("Broken internal link: \"{}\" does not resolve to an existing file in dist", href),
                selector: Some(format!("a[href=\"{}\"]", href)),
            });
        }
    }

    issues
}

fn target_exists(path: &Path) -> bool {
    // Exact file exists
    if path.is_file() {
        return true;
    }

    // Direct path with .html appended
    let html_variant = PathBuf::from(format!("{}.html", path.display()));
    if html_variant.is_file() {
        return true;
    }

    // Directory with index.html
    let index_variant = path.join("index.html");
    if index_variant.is_file() {
        return true;
    }

    // Static asset (images, pdfs, etc.)
    if path.exists() {
        return true;
    }

    false
}
