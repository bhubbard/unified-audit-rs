use scraper::Html;
use std::path::Path;
use unified_audit::{
    seo::audit_seo,
    a11y::audit_a11y,
    schema::audit_schema,
    links::{audit_links, LinkAuditConfig},
    capo::audit_capo,
    images::audit_images,
    Severity,
};

#[test]
fn test_seo_valid_page() {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Accident Attorneys in Los Angeles | Call Jacob</title>
    <meta name="description" content="Experienced personal injury and car accident attorneys fighting for you across Southern California. Free consultations available 24/7.">
    <link rel="canonical" href="https://www.calljacob.com/">
    <meta property="og:title" content="Accident Attorneys">
    <meta property="og:image" content="https://www.calljacob.com/og.jpg">
</head>
<body>
    <h1>Call Jacob Personal Injury Lawyers</h1>
</body>
</html>"#;

    let doc = Html::parse_document(html);
    let issues = audit_seo(&doc);
    let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
    assert!(errors.is_empty(), "Expected no SEO errors, got: {:?}", errors);
}

#[test]
fn test_seo_missing_h1_and_title() {
    let html = r#"<!DOCTYPE html><html><head></head><body><p>Hello</p></body></html>"#;
    let doc = Html::parse_document(html);
    let issues = audit_seo(&doc);

    assert!(issues.iter().any(|i| i.code == "SEO_MISSING_TITLE"));
    assert!(issues.iter().any(|i| i.code == "SEO_MISSING_H1"));
    assert!(issues.iter().any(|i| i.code == "SEO_MISSING_VIEWPORT"));
}

#[test]
fn test_a11y_violations() {
    let html = r#"<!DOCTYPE html>
<html>
<body>
    <button><svg><circle /></svg></button>
    <input type="text" id="user-phone" />
    <img src="/logo.png">
</body>
</html>"#;

    let doc = Html::parse_document(html);
    let a11y_issues = audit_a11y(&doc);
    let img_issues = audit_images(&doc);

    assert!(a11y_issues.iter().any(|i| i.code == "A11Y_MISSING_LANG"));
    assert!(a11y_issues.iter().any(|i| i.code == "A11Y_EMPTY_BUTTON"));
    assert!(a11y_issues.iter().any(|i| i.code == "A11Y_UNLABELLED_INPUT"));
    assert!(img_issues.iter().any(|i| i.code == "IMG_MISSING_ALT"));
}

#[test]
fn test_schema_article_validation() {
    let html_valid = r#"
    <script type="application/ld+json">
    {
        "@context": "https://schema.org",
        "@type": "Article",
        "headline": "What to do after a car accident",
        "image": "https://example.com/hero.jpg",
        "datePublished": "2026-09-01T08:00:00Z",
        "author": {
            "@type": "Organization",
            "name": "Call Jacob Legal Team"
        }
    }
    </script>"#;

    let issues_valid = audit_schema(html_valid);
    assert!(issues_valid.is_empty());

    let html_missing = r#"
    <script type="application/ld+json">
    {
        "@context": "https://schema.org",
        "@type": "Article",
        "datePublished": "2026-09-01T08:00:00Z"
    }
    </script>"#;

    let issues_missing = audit_schema(html_missing);
    assert!(issues_missing.iter().any(|i| i.code == "SCHEMA_MISSING_REQUIRED_FIELD" && i.message.contains("headline")));
    assert!(issues_missing.iter().any(|i| i.code == "SCHEMA_MISSING_REQUIRED_FIELD" && i.message.contains("image")));
}

#[test]
fn test_links_audit() {
    let html = r##"<!DOCTYPE html>
<html>
<body>
    <a href="">Empty Link</a>
    <a href="/cdn-cgi/image/test.webp">CDN Link (should skip)</a>
    <a href="#non-existent">Missing Anchor</a>
    <a href="/non-existent-file-xyz123">Broken Link</a>
</body>
</html>"##;

    let doc = Html::parse_document(html);
    let cfg = LinkAuditConfig {
        root_dir: Path::new("/tmp"),
        current_file: Path::new("/tmp/test.html"),
        skip_patterns: &["/cdn-cgi/".to_string()],
    };

    let issues = audit_links(&doc, &cfg);
    assert!(issues.iter().any(|i| i.code == "LINK_EMPTY_HREF"));
    assert!(issues.iter().any(|i| i.code == "LINK_BROKEN_INPAGE_ANCHOR"));
    assert!(issues.iter().any(|i| i.code == "LINK_BROKEN_INTERNAL"));
    assert!(!issues.iter().any(|i| i.message.contains("/cdn-cgi/")));
}

#[test]
fn test_capo_head_score() {
    let bad_head = r#"<!DOCTYPE html><html><head>
        <script defer src="/app.js"></script>
        <title>Late Title</title>
        <meta charset="utf-8">
    </head></html>"#;

    let (score, _) = audit_capo(bad_head);
    assert!(score < 100.0);

    let optimal_head = r#"<!DOCTYPE html><html><head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width">
        <title>Optimal Title</title>
        <link rel="stylesheet" href="/style.css">
        <script defer src="/app.js"></script>
    </head></html>"#;

    let (score_opt, _) = audit_capo(optimal_head);
    assert_eq!(score_opt, 100.0);
}
