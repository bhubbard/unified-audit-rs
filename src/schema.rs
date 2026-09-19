use regex::Regex;
use serde_json::Value;
use crate::models::{Category, Issue, Severity};

pub fn audit_schema(html: &str) -> Vec<Issue> {
    let mut issues = Vec::new();
    let json_ld_re = Regex::new(r#"(?is)<script[^>]*type=(?:["']?application/ld\+json["']?)[^>]*>(.*?)</script>"#).unwrap();

    let mut block_idx = 0;
    for caps in json_ld_re.captures_iter(html) {
        block_idx += 1;
        let json_text = caps[1].trim();
        if json_text.is_empty() {
            issues.push(Issue {
                code: "SCHEMA_EMPTY_JSON_LD".to_string(),
                category: Category::Schema,
                severity: Severity::Error,
                message: format!("Script block #{} contains empty JSON-LD content", block_idx),
                selector: Some("script[type=\"application/ld+json\"]".to_string()),
            });
            continue;
        }

        match serde_json::from_str::<Value>(json_text) {
            Err(e) => {
                issues.push(Issue {
                    code: "SCHEMA_INVALID_SYNTAX".to_string(),
                    category: Category::Schema,
                    severity: Severity::Error,
                    message: format!("JSON syntax error in structured data block #{}: {}", block_idx, e),
                    selector: Some("script[type=\"application/ld+json\"]".to_string()),
                });
            }
            Ok(val) => {
                check_value(&val, &mut issues, &format!("block_{}", block_idx));
            }
        }
    }

    issues
}

fn check_value(val: &Value, issues: &mut Vec<Issue>, path: &str) {
    match val {
        Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                check_value(item, issues, &format!("{}[{}]", path, i));
            }
        }
        Value::Object(map) => {
            if let Some(graph) = map.get("@graph") {
                check_value(graph, issues, &format!("{}.@graph", path));
            }

            if let Some(type_val) = map.get("@type") {
                let types: Vec<&str> = match type_val {
                    Value::String(s) => vec![s.as_str()],
                    Value::Array(arr) => arr.iter().filter_map(|v| v.as_str()).collect(),
                    _ => vec![],
                };

                for t in types {
                    validate_entity_type(t, map, issues, path);
                }
            }
        }
        _ => {}
    }
}

fn is_blank(val: Option<&Value>) -> bool {
    match val {
        None => true,
        Some(Value::Null) => true,
        Some(Value::String(s)) => s.trim().is_empty(),
        _ => false,
    }
}

fn req(map: &serde_json::Map<String, Value>, field: &str, issues: &mut Vec<Issue>, path: &str, entity_type: &str) {
    if is_blank(map.get(field)) {
        issues.push(Issue {
            code: "SCHEMA_MISSING_REQUIRED_FIELD".to_string(),
            category: Category::Schema,
            severity: Severity::Error,
            message: format!("{}: {} is missing required field \"{}\"", path, entity_type, field),
            selector: Some("script[type=\"application/ld+json\"]".to_string()),
        });
    }
}

fn validate_entity_type(entity_type: &str, map: &serde_json::Map<String, Value>, issues: &mut Vec<Issue>, path: &str) {
    match entity_type {
        "BlogPosting" | "Article" | "NewsArticle" => {
            req(map, "headline", issues, path, entity_type);
            if let Some(Value::String(h)) = map.get("headline") {
                if h.len() > 115 {
                    issues.push(Issue {
                        code: "SCHEMA_HEADLINE_TOO_LONG".to_string(),
                        category: Category::Schema,
                        severity: Severity::Warning,
                        message: format!("{}: \"headline\" has {} characters — Google typically truncates above 110", path, h.len()),
                        selector: Some("script[type=\"application/ld+json\"]".to_string()),
                    });
                }
            }
            req(map, "image", issues, path, entity_type);
            req(map, "datePublished", issues, path, entity_type);
            req(map, "author", issues, path, entity_type);
        }
        "Organization" | "LegalService" | "LocalBusiness" | "Attorney" => {
            req(map, "name", issues, path, entity_type);
            req(map, "url", issues, path, entity_type);
        }
        "WebSite" => {
            req(map, "name", issues, path, entity_type);
            req(map, "url", issues, path, entity_type);
        }
        "BreadcrumbList" => {
            match map.get("itemListElement") {
                Some(Value::Array(items)) if !items.is_empty() => {
                    for (i, item) in items.iter().enumerate() {
                        let item_path = format!("{}.itemListElement[{}]", path, i);
                        if let Value::Object(item_map) = item {
                            req(item_map, "name", issues, &item_path, "ListItem");
                            req(item_map, "position", issues, &item_path, "ListItem");
                            if i < items.len() - 1 && is_blank(item_map.get("item")) {
                                issues.push(Issue {
                                    code: "SCHEMA_BREADCRUMB_MISSING_ITEM".to_string(),
                                    category: Category::Schema,
                                    severity: Severity::Error,
                                    message: format!("{}: Breadcrumb intermediate item is missing target URL \"item\"", item_path),
                                    selector: Some("script[type=\"application/ld+json\"]".to_string()),
                                });
                            }
                        }
                    }
                }
                _ => {
                    issues.push(Issue {
                        code: "SCHEMA_BREADCRUMB_EMPTY".to_string(),
                        category: Category::Schema,
                        severity: Severity::Error,
                        message: format!("{}: BreadcrumbList \"itemListElement\" must be a non-empty array", path),
                        selector: Some("script[type=\"application/ld+json\"]".to_string()),
                    });
                }
            }
        }
        "FAQPage" => {
            match map.get("mainEntity") {
                Some(Value::Array(items)) if !items.is_empty() => {
                    for (i, item) in items.iter().enumerate() {
                        let q_path = format!("{}.mainEntity[{}]", path, i);
                        if let Value::Object(q_map) = item {
                            req(q_map, "name", issues, &q_path, "Question");
                            req(q_map, "acceptedAnswer", issues, &q_path, "Question");
                        }
                    }
                }
                _ => {
                    issues.push(Issue {
                        code: "SCHEMA_FAQ_EMPTY".to_string(),
                        category: Category::Schema,
                        severity: Severity::Error,
                        message: format!("{}: FAQPage \"mainEntity\" must be a non-empty array of Question objects", path),
                        selector: Some("script[type=\"application/ld+json\"]".to_string()),
                    });
                }
            }
        }
        _ => {}
    }
}
