use std::collections::HashSet;
use scraper::{Html, Selector};
use crate::models::{Category, Issue, Severity};

pub fn audit_a11y(document: &Html) -> Vec<Issue> {
    let mut issues = Vec::new();

    // 1. Document html lang attribute
    let html_sel = Selector::parse("html").unwrap();
    if let Some(html_el) = document.select(&html_sel).next() {
        let lang = html_el.value().attr("lang").unwrap_or("").trim();
        if lang.is_empty() {
            issues.push(Issue {
                code: "A11Y_MISSING_LANG".to_string(),
                category: Category::A11y,
                severity: Severity::Error,
                message: "<html> element is missing a valid 'lang' attribute (WCAG 3.1.1)".to_string(),
                selector: Some("html".to_string()),
            });
        }
    }

    // 2. Button accessible names
    let btn_sel = Selector::parse("button, a[role=\"button\" i]").unwrap();
    for btn in document.select(&btn_sel) {
        let text = btn.text().collect::<String>().trim().to_string();
        let aria_label = btn.value().attr("aria-label").unwrap_or("").trim();
        let aria_labelledby = btn.value().attr("aria-labelledby").unwrap_or("").trim();
        let title = btn.value().attr("title").unwrap_or("").trim();

        if text.is_empty() && aria_label.is_empty() && aria_labelledby.is_empty() && title.is_empty() {
            // Check if it has an img child with alt text
            let img_sel = Selector::parse("img").unwrap();
            let has_img_with_alt = btn.select(&img_sel).any(|img| !img.value().attr("alt").unwrap_or("").trim().is_empty());
            let svg_sel = Selector::parse("svg").unwrap();
            let has_svg = btn.select(&svg_sel).next().is_some();

            if !has_img_with_alt {
                issues.push(Issue {
                    code: "A11Y_EMPTY_BUTTON".to_string(),
                    category: Category::A11y,
                    severity: Severity::Error,
                    message: format!("Button{} lacks an accessible name / text label (WCAG 4.1.2)", if has_svg { " containing SVG" } else { "" }),
                    selector: Some("button".to_string()),
                });
            }
        }
    }

    // 3. Form inputs must have labels or aria-label
    let input_sel = Selector::parse("input:not([type=\"hidden\"]):not([type=\"submit\"]):not([type=\"button\"]):not([type=\"reset\"]), textarea, select").unwrap();
    for input in document.select(&input_sel) {
        let id = input.value().attr("id").unwrap_or("").trim();
        let aria_label = input.value().attr("aria-label").unwrap_or("").trim();
        let aria_labelledby = input.value().attr("aria-labelledby").unwrap_or("").trim();
        let title = input.value().attr("title").unwrap_or("").trim();
        let placeholder = input.value().attr("placeholder").unwrap_or("").trim();

        let mut has_label = !aria_label.is_empty() || !aria_labelledby.is_empty() || !title.is_empty();

        // Check for explicit label: <label for="id">
        if !has_label && !id.is_empty() {
            let label_sel = Selector::parse(&format!("label[for=\"{}\"]", id)).unwrap();
            if document.select(&label_sel).next().is_some() {
                has_label = true;
            }
        }

        // Check for implicit label: input is a descendant of a <label> element
        if !has_label {
            let node_id = input.id();
            let mut current = document.tree.get(node_id);
            while let Some(node) = current {
                if let Some(parent) = node.parent() {
                    if let Some(el) = parent.value().as_element() {
                        if el.name() == "label" {
                            has_label = true;
                            break;
                        }
                    }
                    current = Some(parent);
                } else {
                    break;
                }
            }
        }

        if !has_label {
            issues.push(Issue {
                code: "A11Y_UNLABELLED_INPUT".to_string(),
                category: Category::A11y,
                severity: Severity::Error,
                message: format!("Form control <{}>{} lacks an associated <label> or aria-label (WCAG 1.3.1 / 4.1.2){}",
                    input.value().name(),
                    if !id.is_empty() { format!(" id=\"{}\"", id) } else { "".to_string() },
                    if !placeholder.is_empty() { " (placeholder is not a sufficient label)" } else { "" }
                ),
                selector: Some(format!("input#{}", id)),
            });
        }
    }

    // 4. Duplicate ID detection
    let mut seen_ids = HashSet::new();
    let id_sel = Selector::parse("[id]").unwrap();
    for el in document.select(&id_sel) {
        if let Some(id) = el.value().attr("id") {
            let trimmed = id.trim();
            if !trimmed.is_empty() {
                if !seen_ids.insert(trimmed.to_string()) {
                    issues.push(Issue {
                        code: "A11Y_DUPLICATE_ID".to_string(),
                        category: Category::A11y,
                        severity: Severity::Warning,
                        message: format!("Duplicate HTML id=\"{}\" found on page (WCAG 4.1.1)", trimmed),
                        selector: Some(format!("#{}", trimmed)),
                    });
                }
            }
        }
    }

    issues
}
