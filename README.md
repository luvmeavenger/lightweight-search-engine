# LightSearch

A lightweight Rust search engine prototype built around a small local crawler and an inverted index.

Features:
- crawl a small set of seed URLs
- normalize and index page content into SQLite
- rank results using a lightweight TF-IDF style score
- serve a simple web UI and a JSON search API
- search from the terminal or browser

## Quick start

1. Build the project:

```bash
cargo build
```

2. Index a few seed pages:

```bash
cargo run -- index search.db seed_urls.txt
```

3. Run the browser UI:

```bash
cargo run -- serve 0.0.0.0:3000
```

Then open http://localhost:3000 in a browser.

4. Or search from the command line:

```bash
cargo run -- search "rust"
```

## Project layout

- `src/main.rs` - CLI and HTTP UI entry point
- `src/search.rs` - crawling, indexing, ranking, and query logic
- `seed_urls.txt` - sample websites to index

## Notes

- This is intentionally small and lightweight, not a production-scale Google clone.
- It is designed to be extended with better ranking, scheduler, crawl frontier, and distributed search later.
- It respects a simple model: crawl approved sites, store content locally, and rank results from an inverted index.

## Roadmap

- better ranking with BM25 and freshness
- crawl queue with polite delays and robots.txt support
- pagination and page ranking
- user-generated content and sponsored results ecosystem
- Docker deployment and performance tuning
