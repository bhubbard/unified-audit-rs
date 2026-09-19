use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    Seo,
    A11y,
    Schema,
    Links,
    Images,
    Capo,
}

impl Category {
    pub fn name(&self) -> &'static str {
        match self {
            Category::Seo => "SEO",
            Category::A11y => "Accessibility",
            Category::Schema => "Schema.org",
            Category::Links => "Links",
            Category::Images => "Images",
            Category::Capo => "Capo",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub code: String,
    pub category: Category,
    pub severity: Severity,
    pub message: String,
    pub selector: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageAuditReport {
    pub file_path: PathBuf,
    pub url_path: String,
    pub issues: Vec<Issue>,
    pub capo_score: f64,
}

impl PageAuditReport {
    pub fn errors_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == Severity::Error).count()
    }

    pub fn warnings_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == Severity::Warning).count()
    }
}
