use crate::models::{Category, Issue, Severity};
use scraper::{Html, Selector};

pub fn audit_seo(document: &Html) -> Vec<Issue> {
    let mut issues = Vec::new();

    // 1. Check <title>
    let title_sel = Selector::parse("head > title").unwrap();
    let titles: Vec<_> = document.select(&title_sel).collect();
    if titles.is_empty() {
        issues.push(Issue {
            code: "SEO_MISSING_TITLE".to_string(),
            category: Category::Seo,
            severity: Severity::Error,
            message: "Missing <title> tag in <head>".to_string(),
            selector: Some("head".to_string()),
        });
    } else {
        if titles.len() > 1 {
            issues.push(Issue {
                code: "SEO_MULTIPLE_TITLES".to_string(),
                category: Category::Seo,
                severity: Severity::Warning,
                message: format!("Multiple ({}x) <title> tags found in <head>", titles.len()),
                selector: Some("head > title".to_string()),
            });
        }
        let title_text = titles[0].text().collect::<String>().trim().to_string();
        if title_text.is_empty() {
            issues.push(Issue {
                code: "SEO_EMPTY_TITLE".to_string(),
                category: Category::Seo,
                severity: Severity::Error,
                message: "<title> tag is empty".to_string(),
                selector: Some("head > title".to_string()),
            });
        } else if title_text.len() < 20 {
            issues.push(Issue {
                code: "SEO_TITLE_TOO_SHORT".to_string(),
                category: Category::Seo,
                severity: Severity::Warning,
                message: format!(
                    "<title> is very short ({} chars). Recommended 30–65 chars: \"{}\"",
                    title_text.len(),
                    title_text
                ),
                selector: Some("head > title".to_string()),
            });
        } else if title_text.len() > 70 {
            issues.push(Issue {
                code: "SEO_TITLE_TOO_LONG".to_string(),
                category: Category::Seo,
                severity: Severity::Warning,
                message: format!(
                    "<title> exceeds 70 chars ({} chars) and will likely be truncated in SERPs",
                    title_text.len()
                ),
                selector: Some("head > title".to_string()),
            });
        }
    }

    // 2. Check meta description
    let desc_sel = Selector::parse("head > meta[name=\"description\" i]").unwrap();
    let descriptions: Vec<_> = document.select(&desc_sel).collect();
    if descriptions.is_empty() {
        issues.push(Issue {
            code: "SEO_MISSING_DESCRIPTION".to_string(),
            category: Category::Seo,
            severity: Severity::Warning,
            message: "Missing meta description tag in <head>".to_string(),
            selector: Some("head".to_string()),
        });
    } else {
        if descriptions.len() > 1 {
            issues.push(Issue {
                code: "SEO_MULTIPLE_DESCRIPTIONS".to_string(),
                category: Category::Seo,
                severity: Severity::Warning,
                message: format!(
                    "Multiple ({}x) meta descriptions found in <head>",
                    descriptions.len()
                ),
                selector: Some("meta[name=\"description\"]".to_string()),
            });
        }
        if let Some(content) = descriptions[0].value().attr("content") {
            let trimmed = content.trim();
            if trimmed.is_empty() {
                issues.push(Issue {
                    code: "SEO_EMPTY_DESCRIPTION".to_string(),
                    category: Category::Seo,
                    severity: Severity::Error,
                    message: "Meta description content is empty".to_string(),
                    selector: Some("meta[name=\"description\"]".to_string()),
                });
            } else if trimmed.len() < 40 {
                issues.push(Issue {
                    code: "SEO_DESCRIPTION_TOO_SHORT".to_string(),
                    category: Category::Seo,
                    severity: Severity::Warning,
                    message: format!(
                        "Meta description is very short ({} chars). Recommended 50–160 chars.",
                        trimmed.len()
                    ),
                    selector: Some("meta[name=\"description\"]".to_string()),
                });
            } else if trimmed.len() > 165 {
                issues.push(Issue {
                    code: "SEO_DESCRIPTION_TOO_LONG".to_string(),
                    category: Category::Seo,
                    severity: Severity::Warning,
                    message: format!("Meta description exceeds 160 chars ({} chars) and will be truncated by Google", trimmed.len()),
                    selector: Some("meta[name=\"description\"]".to_string()),
                });
            }
        }
    }

    // 3. Check canonical URL
    let canon_sel = Selector::parse("head > link[rel=\"canonical\" i]").unwrap();
    let canon_links: Vec<_> = document.select(&canon_sel).collect();
    if canon_links.is_empty() {
        issues.push(Issue {
            code: "SEO_MISSING_CANONICAL".to_string(),
            category: Category::Seo,
            severity: Severity::Warning,
            message: "Missing <link rel=\"canonical\"> tag in <head>".to_string(),
            selector: Some("head".to_string()),
        });
    } else {
        if canon_links.len() > 1 {
            issues.push(Issue {
                code: "SEO_MULTIPLE_CANONICALS".to_string(),
                category: Category::Seo,
                severity: Severity::Error,
                message: format!("Multiple ({}x) canonical tags found", canon_links.len()),
                selector: Some("link[rel=\"canonical\"]".to_string()),
            });
        }
        let href = canon_links[0].value().attr("href").unwrap_or("").trim();
        if href.is_empty() {
            issues.push(Issue {
                code: "SEO_EMPTY_CANONICAL".to_string(),
                category: Category::Seo,
                severity: Severity::Error,
                message: "Canonical href attribute is empty".to_string(),
                selector: Some("link[rel=\"canonical\"]".to_string()),
            });
        }
    }

    // 4. Check Viewport
    let vp_sel = Selector::parse("head > meta[name=\"viewport\" i]").unwrap();
    if document.select(&vp_sel).next().is_none() {
        issues.push(Issue {
            code: "SEO_MISSING_VIEWPORT".to_string(),
            category: Category::Seo,
            severity: Severity::Error,
            message: "Missing mobile viewport meta tag".to_string(),
            selector: Some("head".to_string()),
        });
    }

    // 5. Check H1 hierarchy
    let h1_sel = Selector::parse("h1").unwrap();
    let h1_tags: Vec<_> = document.select(&h1_sel).collect();
    if h1_tags.is_empty() {
        issues.push(Issue {
            code: "SEO_MISSING_H1".to_string(),
            category: Category::Seo,
            severity: Severity::Error,
            message: "Page has no <h1> heading".to_string(),
            selector: Some("body".to_string()),
        });
    } else if h1_tags.len() > 1 {
        issues.push(Issue {
            code: "SEO_MULTIPLE_H1".to_string(),
            category: Category::Seo,
            severity: Severity::Warning,
            message: format!(
                "Page contains {}x <h1> tags (recommended: exactly 1 per page)",
                h1_tags.len()
            ),
            selector: Some("h1".to_string()),
        });
    }

    // 6. Check Open Graph tags
    let og_title_sel = Selector::parse("head > meta[property=\"og:title\" i]").unwrap();
    if document.select(&og_title_sel).next().is_none() {
        issues.push(Issue {
            code: "SEO_MISSING_OG_TITLE".to_string(),
            category: Category::Seo,
            severity: Severity::Info,
            message: "Missing og:title social metadata tag".to_string(),
            selector: Some("head".to_string()),
        });
    }

    let og_image_sel = Selector::parse("head > meta[property=\"og:image\" i]").unwrap();
    if document.select(&og_image_sel).next().is_none() {
        issues.push(Issue {
            code: "SEO_MISSING_OG_IMAGE".to_string(),
            category: Category::Seo,
            severity: Severity::Info,
            message: "Missing og:image social card image tag".to_string(),
            selector: Some("head".to_string()),
        });
    }

    issues
}
