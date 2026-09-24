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
        --bg: #f5f7fb;
        --card: #ffffff;
        --text: #1e293b;
        --muted: #64748b;
        --line: #e2e8f0;
        --accent: #2563eb;
        --accent-soft: #dbeafe;
      }
      * { box-sizing: border-box; }
      body {
        margin: 0;
        font-family: Arial, Helvetica, sans-serif;
        background: var(--bg);
        color: var(--text);
      }
      .wrap {
        max-width: 980px;
        margin: 0 auto;
        padding: 40px 20px 60px;
      }
      .hero {
        display: flex;
        flex-direction: column;
        align-items: center;
        padding-top: 80px;
      }
      .logo {
        font-size: 52px;
        font-weight: 700;
        letter-spacing: -0.06em;
        margin-bottom: 20px;
      }
      .search-box {
        width: min(100%, 720px);
        display: flex;
        gap: 10px;
        background: var(--card);
        border: 1px solid var(--line);
        border-radius: 999px;
        padding: 10px 14px 10px 18px;
        box-shadow: 0 8px 22px rgba(15, 23, 42, 0.05);
      }
      input {
        flex: 1;
        border: none;
        outline: none;
        font-size: 18px;
        background: transparent;
        color: var(--text);
      }
      button {
        border: none;
        background: var(--accent);
        color: white;
        padding: 12px 26px;
        border-radius: 999px;
        font-size: 16px;
        font-weight: 600;
        cursor: pointer;
      }
      .results {
        margin-top: 32px;
        display: flex;
        flex-direction: column;
        gap: 18px;
      }
      .result {
        background: var(--card);
        border: 1px solid var(--line);
        border-radius: 14px;
        padding: 18px 20px;
      }
      .result a {
        color: var(--accent);
        text-decoration: none;
        font-size: 18px;
        font-weight: 600;
      }
      .result a:hover { text-decoration: underline; }
      .url {
        color: var(--muted);
        font-size: 12px;
        margin-top: 4px;
      }
      .snippet {
        margin-top: 8px;
        color: var(--text);
        line-height: 1.5;
      }
      .empty {
        color: var(--muted);
        text-align: center;
        padding-top: 12px;
      }
    </style>
  </head>
  <body>
    <div class="wrap">
      <div class="hero">
        <div class="logo">LightSearch</div>
        <form id="search-form" class="search-box">
          <input id="query" type="text" placeholder="Search the web" autocomplete="off" />
          <button type="submit">Search</button>
        </form>
      </div>
      <div id="results" class="results"></div>
    </div>

    <script>
      const form = document.getElementById('search-form');
      const input = document.getElementById('query');
      const resultsEl = document.getElementById('results');

      async function runSearch() {
        const q = input.value.trim();
        if (!q) {
          resultsEl.innerHTML = '<div class="empty">Type a query to search.</div>';
          return;
        }

        resultsEl.innerHTML = '<div class="empty">Searching...</div>';
        const res = await fetch(`/api/search?q=${encodeURIComponent(q)}`);
        const data = await res.json();

        if (!data.length) {
          resultsEl.innerHTML = '<div class="empty">No results found.</div>';
          return;
        }

        resultsEl.innerHTML = data.map(result => `
          <div class="result">
            <a href="${result.url}" target="_blank" rel="noopener noreferrer">${result.title}</a>
            <div class="url">${result.url}</div>
            <div class="snippet">${result.snippet}</div>
          </div>
        `).join('');
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
    let index = search::SearchIndex::open("search.db").unwrap_or_else(|_| {
        println!("No index found. Run `cargo run -- index` first.");
        search::SearchIndex::open("search.db").unwrap()
    });

    let results = index.search(&params.q, 10).unwrap_or_default();
    Json(results)
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

