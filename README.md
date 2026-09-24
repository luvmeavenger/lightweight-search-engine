# LightSearch

A lightweight Rust search engine prototype with local crawling, SQLite indexing, relevance ranking, and basic anti-scam protections.

## Quick start

```bash
cargo build
cargo run -- index search.db seed_urls.txt
cargo run -- serve 0.0.0.0:3000
```

Open <http://localhost:3000>, or run `cargo run -- search "rust"`.

## Safety and abuse protection

The indexer now:

- accepts only HTTP/HTTPS URLs and limits requests to 10 seconds
- sends a descriptive crawler user agent and indexes HTML only
- detects common scam language, deceptive domains, IP-address URLs, unusually deep subdomains, and suspicious credential forms
- blocks high-risk pages from normal results
- keeps medium-risk pages discoverable but demotes them and adds a visible safety warning
- stores risk scores and reasons in SQLite for later moderation and audit tools

These are heuristic defenses, not a replacement for a maintained threat-intelligence feed, malware scanner, Safe Browsing-style service, robots.txt handling, or human review. False positives are possible, so production deployments should provide reporting, appeals, and an allowlist.

## Ecosystem policy

Organic ranking is separate from user content and sponsored placement. Sponsored results must be clearly labeled and never silently alter organic ranking.

## Layout

- `src/main.rs` - CLI, HTTP server, and UI
- `src/search.rs` - crawling, safety assessment, indexing, ranking, and snippets
- `seed_urls.txt` - sample sites
