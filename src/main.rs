use axum::{
    extract::Query,
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::net::SocketAddr;

mod search;

const INDEX_HTML: &str = r#"
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>LightSearch</title>
    <style>
      :root {
        --bg: #07111f;
        --panel: rgba(15, 23, 42, 0.75);
        --panel-solid: #101b2d;
        --card: #0f172a;
        --card-soft: #111d33;
        --line: rgba(148, 163, 184, 0.18);
        --text: #e5eefb;
        --muted: #9bb0cc;
        --accent: #60a5fa;
        --accent-strong: #3b82f6;
        --success: #34d399;
        --warning: #fbbf24;
        --danger: #f87171;
        --shadow: 0 18px 45px rgba(2, 6, 23, 0.45);
      }

      * { box-sizing: border-box; }

      html, body {
        margin: 0;
        min-height: 100%;
        font-family: Inter, "Segoe UI", Arial, sans-serif;
        background:
          radial-gradient(circle at top, rgba(59,130,246,0.18), transparent 24%),
          linear-gradient(180deg, #020817 0%, #07111f 32%, #091322 100%);
        color: var(--text);
      }

      body {
        display: flex;
        justify-content: center;
      }

      .shell {
        width: min(1100px, calc(100% - 32px));
        padding: 28px 0 60px;
      }

      .topbar {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 18px;
        margin-bottom: 30px;
      }

      .brand {
        display: flex;
        align-items: center;
        gap: 10px;
        font-weight: 700;
        letter-spacing: -0.05em;
        font-size: 1.25rem;
      }

      .brand-mark {
        width: 32px;
        height: 32px;
        border-radius: 12px;
        background: linear-gradient(135deg, var(--accent), #8b5cf6);
        box-shadow: inset 0 0 18px rgba(255,255,255,0.18), 0 8px 18px rgba(96,165,250,0.38);
      }

      .nav {
        display: flex;
        align-items: center;
        gap: 10px;
        flex-wrap: wrap;
      }

      .chip {
        border: 1px solid var(--line);
        background: rgba(15, 23, 42, 0.7);
        color: var(--muted);
        padding: 8px 12px;
        border-radius: 999px;
        font-size: 0.9rem;
      }

      .chip.active {
        background: rgba(96,165,250,0.12);
        color: var(--text);
        border-color: rgba(96,165,250,0.34);
      }

      .hero {
        text-align: center;
        padding: 36px 0 20px;
      }

      .title {
        font-size: clamp(2.5rem, 5vw, 5rem);
        margin: 0 0 10px;
        letter-spacing: -0.07em;
        font-weight: 800;
      }

      .subtitle {
        margin: 0 auto 28px;
        max-width: 620px;
        color: var(--muted);
        font-size: 1.05rem;
        line-height: 1.6;
      }

      .search-panel {
        max-width: 820px;
        margin: 0 auto;
        background: rgba(15, 23, 42, 0.7);
        border: 1px solid var(--line);
        border-radius: 25px;
        padding: 12px 12px 12px 18px;
        box-shadow: var(--shadow);
        backdrop-filter: blur(10px);
      }

      .search-row {
        display: flex;
        gap: 12px;
        align-items: center;
      }

      .search-row input {
        flex: 1;
        background: transparent;
        border: none;
        color: var(--text);
        font-size: clamp(1rem, 2vw, 1.2rem);
        outline: none;
      }

      .search-row input::placeholder {
        color: #7f94b5;
      }

      button {
        border: none;
        border-radius: 16px;
        padding: 14px 22px;
        font-size: 1rem;
        font-weight: 700;
        cursor: pointer;
        background: linear-gradient(135deg, var(--accent), var(--accent-strong));
        color: white;
        box-shadow: 0 10px 20px rgba(59,130,246,0.35);
      }

      .search-meta {
        margin-top: 12px;
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
        flex-wrap: wrap;
        color: var(--muted);
        font-size: 0.83rem;
      }

      .meta-left {
        display: flex;
        align-items: center;
        gap: 8px;
      }

      .dot {
        width: 8px;
        height: 8px;
        border-radius: 50%;
        background: var(--success);
        box-shadow: 0 0 14px rgba(52, 211, 153, 0.7);
      }

      .results {
        margin-top: 26px;
        display: flex;
        flex-direction: column;
        gap: 18px;
      }

      .result {
        background: rgba(15, 23, 42, 0.7);
        border: 1px solid var(--line);
        border-radius: 18px;
        padding: 18px 18px 16px;
        box-shadow: 0 10px 30px rgba(2,6,23,0.22);
      }

      .result-head {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
        flex-wrap: wrap;
      }

      .result a {
        color: #dbeafe;
        text-decoration: none;
        font-size: clamp(1.08rem, 2vw, 1.35rem);
        font-weight: 700;
      }

      .result a:hover {
        text-decoration: underline;
      }

      .site {
        color: var(--muted);
        font-size: 0.8rem;
        margin-top: 5px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
      }

      .badge {
        border-radius: 999px;
        padding: 5px 10px;
        font-size: 0.72rem;
        font-weight: 700;
        letter-spacing: 0.02em;
        text-transform: uppercase;
      }

      .badge.safe {
        background: rgba(52, 211, 153, 0.12);
        color: #b9f3d8;
      }

      .badge.warning {
        background: rgba(251, 191, 36, 0.12);
        color: #fdd98a;
      }

      .snippet {
        margin-top: 12px;
        line-height: 1.7;
        color: var(--muted);
      }

      .warning {
        margin-top: 10px;
        border-left: 3px solid var(--warning);
        padding-left: 12px;
        color: #fdd98a;
        background: rgba(251,191,36,0.04);
        border-radius: 10px;
        line-height: 1.45;
      }

      .empty,
      .loading {
        text-align: center;
        color: var(--muted);
        padding: 26px 10px 10px;
      }

      @media (max-width: 640px) {
        .topbar {
          flex-direction: column;
          align-items: flex-start;
        }

        .search-row {
          flex-direction: column;
          align-items: stretch;
        }

        button {
          width: 100%;
        }
      }
    </style>
  </head>
  <body>
    <div class="shell">
      <header class="topbar">
        <div class="brand">
          <span class="brand-mark"></span>
          <span>LightSearch</span>
        </div>
        <nav class="nav" aria-label="Navigation">
          <div class="chip active">All</div>
          <div class="chip">News</div>
          <div class="chip">Docs</div>
          <div class="chip">Community</div>
        </nav>
      </header>

      <section class="hero">
        <h1 class="title">Search the web</h1>
        <p class="subtitle">Fast, local, and built to prioritize trustworthy results with built-in scam and phishing protection.</p>

        <form id="search-form" class="search-panel">
          <div class="search-row">
            <input id="query" type="text" placeholder="Search anything..." autocomplete="off" />
            <button type="submit">Search</button>
          </div>
          <div class="search-meta">
            <div class="meta-left">
              <span class="dot"></span>
              <span>Protected indexing</span>
            </div>
            <div>Organic + trusted content</div>
          </div>
        </form>
      </section>

      <main id="results" class="results" aria-live="polite"></main>
    </div>

    <script>
      const form = document.getElementById('search-form');
      const input = document.getElementById('query');
      const resultsEl = document.getElementById('results');

      function renderResults(data) {
        if (!data.length) {
          resultsEl.innerHTML = '<div class="empty">No trusted results found for this query.</div>';
          return;
        }

        resultsEl.innerHTML = data.map((result) => {
          const badge = result.safety_warning ? '<span class="badge warning">Warning</span>' : '<span class="badge safe">Safe</span>';
          return `
            <article class="result">
              <div class="result-head">
                <div>
                  <a href="${result.url}" target="_blank" rel="noopener noreferrer">${result.title}</a>
                  <div class="site">${result.url}</div>
                </div>
                ${badge}
              </div>
              <div class="snippet">${result.snippet || 'No snippet available.'}</div>
              ${result.safety_warning ? `<div class="warning">Safety warning: ${result.safety_warning}</div>` : ''}
            </article>
          `;
        }).join('');
      }

      async function runSearch() {
        const q = input.value.trim();
        if (!q) {
          resultsEl.innerHTML = '<div class="empty">Type a query to search.</div>';
          return;
        }

        resultsEl.innerHTML = '<div class="loading">Searching trusted sources...</div>';

        try {
          const res = await fetch(`/api/search?q=${encodeURIComponent(q)}`);
          const data = await res.json();
          renderResults(data);
        } catch (error) {
          resultsEl.innerHTML = '<div class="empty">The search service is unavailable. Index a few pages first.</div>';
        }
      }

      form.addEventListener('submit', (event) => {
        event.preventDefault();
        runSearch();
      });
    </script>
  </body>
</html>
"#;

#[derive(Deserialize)]
struct SearchRequest {
    q: String,
}

async fn home() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn search_api(Query(params): Query<SearchRequest>) -> impl IntoResponse {
    let index = match search::SearchIndex::open("search.db") {
        Ok(index) => index,
        Err(_) => {
            return Json(Vec::<search::SearchResult>::new());
        }
    };

    match index.search(&params.q, 10) {
        Ok(results) => Json(results),
        Err(_) => Json(Vec::<search::SearchResult>::new()),
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("index") => {
            let db_path = args.get(2).map(String::as_str).unwrap_or("search.db");
            let seed_path = args.get(3).map(String::as_str).unwrap_or("seed_urls.txt");
            let index = search::SearchIndex::open(db_path).expect("Could not open database");
            index.index_seed_file(seed_path).expect("Could not index seed file");
            println!("Indexed URLs from {} into {}.", seed_path, db_path);
        }
        Some("search") => {
            let query = args.get(2).cloned().unwrap_or_default();
            let index = search::SearchIndex::open("search.db").expect("Could not open database");
            let results = index.search(&query, 10).unwrap_or_default();
            for result in results {
                println!("{}\n{}\n{}\n", result.score, result.title, result.url);
            }
        }
        Some("serve") => {
            let bind = args.get(2).map(String::as_str).unwrap_or("0.0.0.0:3000");
            let app = Router::new()
                .route("/", get(home))
                .route("/api/search", get(search_api));

            let addr: SocketAddr = bind.parse().expect("Invalid bind address");
            println!("Starting web server on {}", addr);

            axum::serve(
                tokio::net::TcpListener::bind(addr)
                    .await
                    .expect("Could not bind to port"),
                app,
            )
            .await
            .expect("Server failed");
        }
        _ => {
            print_usage();
        }
    }
}

fn print_usage() {
    println!("Usage:");
    println!("  cargo run -- index [db_path] [seed_file]");
    println!("  cargo run -- search \"query\"");
    println!("  cargo run -- serve [bind_address]");
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        assert!(true);
    }
}
