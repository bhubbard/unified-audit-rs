use regex::Regex;
use crate::models::{Category, Issue, Severity};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CapoPriority {
    OriginTrials = 1,
    MetaCharset = 2,
    MetaCsp = 3,
    MetaViewport = 4,
    Title = 5,
    Preconnect = 6,
    AsyncScript = 7,
    ImportStyles = 8,
    SyncScript = 9,
    SyncStyles = 10,
    Preload = 11,
    DeferScript = 12,
    PrefetchPrerender = 13,
    OtherMeta = 14,
}

#[derive(Debug, Clone)]
pub struct HeadElement {
    pub raw: String,
    pub priority: CapoPriority,
    pub original_index: usize,
}

pub fn classify_element(raw: &str, index: usize) -> HeadElement {
    let lower = raw.to_lowercase();
    let trimmed = lower.trim();

    let priority = if trimmed.starts_with("<meta") {
        if trimmed.contains("origin-trial") {
            CapoPriority::OriginTrials
        } else if trimmed.contains("charset=") || trimmed.contains("content-type") {
            CapoPriority::MetaCharset
        } else if trimmed.contains("content-security-policy") {
            CapoPriority::MetaCsp
        } else if trimmed.contains("viewport") {
            CapoPriority::MetaViewport
        } else {
            CapoPriority::OtherMeta
        }
    } else if trimmed.starts_with("<title") || trimmed.starts_with("<base") {
        CapoPriority::Title
    } else if trimmed.starts_with("<link") {
        if trimmed.contains("preconnect") {
            CapoPriority::Preconnect
        } else if trimmed.contains("preload") || trimmed.contains("modulepreload") {
            CapoPriority::Preload
        } else if trimmed.contains("stylesheet") {
            if trimmed.contains("@import") {
                CapoPriority::ImportStyles
            } else {
                CapoPriority::SyncStyles
            }
        } else if trimmed.contains("prefetch") || trimmed.contains("prerender") || trimmed.contains("dns-prefetch") {
            CapoPriority::PrefetchPrerender
        } else {
            CapoPriority::OtherMeta
        }
    } else if trimmed.starts_with("<style") {
        if trimmed.contains("@import") {
            CapoPriority::ImportStyles
        } else {
            CapoPriority::SyncStyles
        }
    } else if trimmed.starts_with("<script") {
        if trimmed.contains("async") {
            CapoPriority::AsyncScript
        } else if trimmed.contains("defer") || trimmed.contains("type=\"module\"") || trimmed.contains("type='module'") {
            CapoPriority::DeferScript
        } else {
            CapoPriority::SyncScript
        }
    } else {
        CapoPriority::OtherMeta
    };

    HeadElement {
        raw: raw.trim().to_string(),
        priority,
        original_index: index,
    }
}

pub fn parse_head_elements(html: &str) -> Vec<HeadElement> {
    let head_re = Regex::new(r"(?is)<head\b[^>]*>(.*?)</head>").unwrap();
    let tag_re = Regex::new(r"(?is)(<!--.*?-->|<title\b[^>]*>.*?</title>|<style\b[^>]*>.*?</style>|<script\b[^>]*>.*?</script>|<(?:meta|link|base)\b[^>]*>)").unwrap();

    let mut elements = Vec::new();
    if let Some(caps) = head_re.captures(html) {
        let head_inner = &caps[1];
        let mut index = 0;
        for mat in tag_re.find_iter(head_inner) {
            let snippet = mat.as_str().trim();
            if snippet.is_empty() || snippet.starts_with("<!--") {
                continue;
            }
            elements.push(classify_element(snippet, index));
            index += 1;
        }
    }
    elements
}

pub fn calculate_capo_score(elements: &[HeadElement]) -> f64 {
    if elements.len() <= 1 {
        return 100.0;
    }

    let mut inversions = 0;
    let total_pairs = (elements.len() * (elements.len() - 1)) / 2;

    for i in 0..elements.len() {
        for j in (i + 1)..elements.len() {
            if elements[i].priority > elements[j].priority {
                inversions += 1;
            }
        }
    }

    let ratio = 1.0 - (inversions as f64 / total_pairs as f64);
    (ratio * 100.0).max(0.0).min(100.0)
}

pub fn audit_capo(html: &str) -> (f64, Vec<Issue>) {
    let elements = parse_head_elements(html);
    let score = calculate_capo_score(&elements);
    let mut issues = Vec::new();

    if score < 70.0 {
        issues.push(Issue {
            code: "CAPO_HEAD_SUBOPTIMAL".to_string(),
            category: Category::Capo,
            severity: Severity::Warning,
            message: format!("Head element loading order has a low Capo efficiency score ({:.1}%). Consider reordering <head> tags.", score),
            selector: Some("head".to_string()),
        });
    }

    // Check if sync render-blocking script appears before stylesheets
    let mut found_sync_script = false;
    for el in &elements {
        if el.priority == CapoPriority::SyncScript {
            found_sync_script = true;
        } else if found_sync_script && (el.priority == CapoPriority::SyncStyles || el.priority == CapoPriority::MetaCharset || el.priority == CapoPriority::MetaViewport) {
            issues.push(Issue {
                code: "CAPO_BLOCKING_SCRIPT_PRECEDES_CRITICAL".to_string(),
                category: Category::Capo,
                severity: Severity::Warning,
                message: format!("Synchronous render-blocking script appears before critical tag: {}", el.raw),
                selector: Some("head script".to_string()),
            });
            break;
        }
    }

    (score, issues)
}
