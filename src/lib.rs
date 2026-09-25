pub mod a11y;
pub mod capo;
pub mod crawler;
pub mod images;
pub mod links;
pub mod models;
pub mod report;
pub mod schema;
pub mod seo;

pub use crawler::{AuditOptions, audit_directory, audit_html_file};
pub use models::{Category, Issue, PageAuditReport, Severity};
pub use report::{ReportFormat, print_report};
