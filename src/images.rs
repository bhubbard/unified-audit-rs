use crate::models::{Category, Issue, Severity};
use scraper::{Html, Selector};

pub fn audit_images(document: &Html) -> Vec<Issue> {
    let mut issues = Vec::new();
    let img_sel = Selector::parse("img").unwrap();

    for img in document.select(&img_sel) {
        let src = img.value().attr("src").unwrap_or("").trim();
        let alt = img.value().attr("alt");
        let width = img.value().attr("width");
        let height = img.value().attr("height");

        // 1. Missing alt attribute (WCAG 1.1.1)
        match alt {
            None => {
                issues.push(Issue {
                    code: "IMG_MISSING_ALT".to_string(),
                    category: Category::A11y,
                    severity: Severity::Error,
                    message: format!("Image \"{}\" is missing an 'alt' attribute entirely", src),
                    selector: Some(format!("img[src=\"{}\"]", src)),
                });
            }
            Some(a) => {
                let trimmed = a.trim().to_lowercase();
                if trimmed == "image"
                    || trimmed == "picture"
                    || trimmed == "photo"
                    || trimmed == "graphic"
                    || trimmed == "untitled"
                {
                    issues.push(Issue {
                        code: "IMG_SUSPICIOUS_ALT".to_string(),
                        category: Category::A11y,
                        severity: Severity::Warning,
                        message: format!(
                            "Image \"{}\" has non-descriptive alt text: \"{}\"",
                            src, a
                        ),
                        selector: Some(format!("img[src=\"{}\"]", src)),
                    });
                }
            }
        }

        // 2. Missing intrinsic dimensions (CLS risk)
        if (width.is_none() || height.is_none()) && !src.ends_with(".svg") {
            issues.push(Issue {
                code: "IMG_MISSING_DIMENSIONS".to_string(),
                category: Category::Images,
                severity: Severity::Warning,
                message: format!("Image \"{}\" lacks explicit width and/or height attributes, risking Cumulative Layout Shift (CLS)", src),
                selector: Some(format!("img[src=\"{}\"]", src)),
            });
        }
    }

    issues
}
