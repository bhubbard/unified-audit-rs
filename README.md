# unified-audit-rs 🔍

A high-performance, consolidated post-build CI verification gate for Astro and static sites written in pure Rust.

It replaces the fragmented and slow Node.js post-build checking pipeline (`pa11y-ci`, `lychee`, `unlighthouse`, custom schema scripts, and `astro-post-audit`) with a single ultra-fast (< 200ms) multi-threaded binary.

---

## What It Verifies

1. **SEO & Metadata Integrity**:
   - Page `<title>` existence, length (30–65 chars), and uniqueness.
   - Meta description existence and character limits (50–160 chars).
   - Canonical URL presence and valid formatting.
   - Mobile viewport meta presence.
   - Exactly one `<h1>` per page.
   - OpenGraph (`og:title`, `og:image`) and Twitter Card metadata.

2. **WCAG Accessibility (a11y)**:
   - Root `<html>` element `lang` attribute.
   - Buttons without text, aria-label, or child SVG labels.
   - Form inputs without associated `<label>`, `id`, or `aria-label`.
   - Missing or empty `<img>` `alt` attributes.
   - Duplicate HTML `id` detection across document.

3. **Schema.org Structured Data (JSON-LD)**:
   - Validates `<script type="application/ld+json">` syntax.
   - Validates Google Rich Results required properties for:
     - `Article` / `BlogPosting` (headline length, image, datePublished, author).
     - `BreadcrumbList` (valid non-empty `itemListElement` array with position, name, and URL).
     - `FAQPage` (valid non-empty `mainEntity` array with Question/Answer).
     - `LegalService` / `LocalBusiness` / `Organization` (name and URL).

4. **Internal Link Integrity (Zero 404s)**:
   - Resolves all internal relative and root-relative `href` links against static files in `dist/`.
   - Validates in-page `#anchor` references against real element IDs in the DOM.
   - Configurable bypass rules (`/cdn-cgi/*`, `/api/*`).

5. **Image Performance & CLS Prevention**:
   - Flags plain `<img>` tags missing explicit `width` or `height` attributes.
   - Flags suspicious placeholder alt texts ("photo", "image", "untitled").

6. **Capo.js Head Order Efficiency**:
   - Calculates Kendall Tau distance of `<head>` elements against optimal browser pre-parsing priorities.
   - Flags render-blocking synchronous scripts placed before critical CSS or charset tags.

---

## Installation & Build

```bash
cargo build --release
cargo install --path .
```

---

## Usage

```bash
# Run full audit on dist directory
unified-audit dist/

# Strict mode: fail CI with exit code 1 if any errors are detected
unified-audit dist/ --strict

# GitHub Actions format (outputs ::error and ::warning workflow annotations)
unified-audit dist/ --format github

# JSON format for automated dashboards or reporting
unified-audit dist/ --format json

# Skip custom link prefixes
unified-audit dist/ --skip-links "/cdn-cgi/,/api/,/portal/"

# Ignore specific rule codes
unified-audit dist/ --ignore-rules "IMG_MISSING_DIMENSIONS,SEO_TITLE_TOO_SHORT"
```
