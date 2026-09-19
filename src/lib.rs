pub mod models;
pub mod seo;
pub mod a11y;
pub mod schema;
pub mod links;
pub mod images;
pub mod capo;
pub mod crawler;
pub mod report;

pub use models::{Category, Issue, PageAuditReport, Severity};
pub use crawler::{audit_directory, audit_html_file, AuditOptions};
pub use report::{print_report, ReportFormat};
