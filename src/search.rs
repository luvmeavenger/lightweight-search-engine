use regex::Regex;
use rusqlite::{params, Connection, Result};
use scraper::{Html, Selector};
use serde::Serialize;
use std::collections::HashMap;
use url::Url;

#[derive(Clone, Debug, Serialize, Default)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub score: f64,
    pub safety_warning: Option<String>,
}

pub struct SearchIndex {
    conn: Connection,
}

impl SearchIndex {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT UNIQUE NOT NULL,
                title TEXT NOT NULL,
                body TEXT NOT NULL,
                risk_score INTEGER NOT NULL DEFAULT 0,
                risk_reason TEXT
            );
            CREATE TABLE IF NOT EXISTS term_counts (
                doc_id INTEGER NOT NULL,
                term TEXT NOT NULL,
                count INTEGER NOT NULL,
                PRIMARY KEY(doc_id, term)
            );
            CREATE INDEX IF NOT EXISTS idx_term_counts_term ON term_counts(term);
            "#,
        )?;
        // Allow databases created by the first prototype to be upgraded in place.
        let _ = conn.execute("ALTER TABLE documents ADD COLUMN risk_score INTEGER NOT NULL DEFAULT 0", []);
        let _ = conn.execute("ALTER TABLE documents ADD COLUMN risk_reason TEXT", []);
        Ok(Self { conn })
    }

    pub fn index_seed_file(&self, seed_file: &str) -> Result<()> {
        let contents = std::fs::read_to_string(seed_file).unwrap_or_else(|_| {
            eprintln!("Warning: {} not found, using built-in defaults.", seed_file);
            "https://www.rust-lang.org/\nhttps://www.wikipedia.org/\nhttps://www.example.com/\n".to_string()
        });
        for line in contents.lines() {
            let url = line.trim();
            if !url.is_empty() {
                let _ = self.index_url(url);
            }
        }
        Ok(())
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let terms = Self::tokenize(query);
        if terms.is_empty() { return Ok(vec![]); }
        let doc_count: f64 = self.conn.query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))?;
        let mut scores: HashMap<i64, f64> = HashMap::new();
        let mut doc_ids = Vec::new();

        for term in &terms {
            let df: i64 = self.conn.query_row(
                "SELECT COUNT(DISTINCT doc_id) FROM term_counts WHERE term = ?", [term], |row| row.get(0)
            )?;
            let mut stmt = self.conn.prepare("SELECT doc_id, count FROM term_counts WHERE term = ?")?;
            let rows = stmt.query_map([term], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?;
            for row in rows {
                let (doc_id, count) = row?;
                let idf = ((doc_count + 1.0) / (df as f64 + 1.0)).ln() + 1.0;
                *scores.entry(doc_id).or_insert(0.0) += count as f64 * idf;
                if !doc_ids.contains(&doc_id) { doc_ids.push(doc_id); }
            }
        }

        let mut results = Vec::new();
        for doc_id in doc_ids {
            let (url, title, body, risk_score, risk_reason): (String, String, String, i64, Option<String>) = self.conn.query_row(
                "SELECT url, title, body, risk_score, risk_reason FROM documents WHERE id = ?", [doc_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
            )?;
            // High-risk pages never appear in normal results. Caution pages remain discoverable
            // but are pushed down and visibly marked so this heuristic is not silent censorship.
            if risk_score >= 5 { continue; }
            let title_bonus = terms.iter().filter(|term| title.to_lowercase().contains(*term)).count() as f64 * 2.0;
            let score = scores.get(&doc_id).copied().unwrap_or(0.0) + title_bonus - (risk_score as f64 * 1.5);
            let mut snippet = snippet_for(&body, &terms);
            if risk_score >= 3 {
                snippet = format!("⚠ Safety warning: {} {}", risk_reason.clone().unwrap_or_else(|| "This page may be unsafe.".into()), snippet);
            }
            results.push(SearchResult { title, url, snippet, score, safety_warning: risk_reason });
        }
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        Ok(results)
    }

    fn tokenize(text: &str) -> Vec<String> {
        let re = Regex::new(r"[A-Za-z0-9]+").unwrap();
        re.find_iter(text).map(|m| m.as_str().to_lowercase()).filter(|s| s.len() > 1).collect()
    }

    fn index_url(&self, raw_url: &str) -> Result<()> {
        let url = raw_url.trim();
        let parsed = match Url::parse(url) {
            Ok(value) if matches!(value.scheme(), "http" | "https") && value.host_str().is_some() => value,
            _ => { eprintln!("Skipped invalid or unsafe URL: {}", url); return Ok(()); }
        };
        let response = match reqwest::blocking::Client::builder()
            .user_agent("LightSearchBot/0.1 (+https://github.com/luvmeavenger/lightweight-search-engine)")
            .timeout(std::time::Duration::from_secs(10)).build()
            .and_then(|client| client.get(parsed.as_str()).send()) { Ok(r) => r, Err(_) => return Ok(()) };
        if !response.status().is_success() { return Ok(()); }
        if let Some(content_type) = response.headers().get(reqwest::header::CONTENT_TYPE).and_then(|v| v.to_str().ok()) {
            if !content_type.contains("text/html") { return Ok(()); }
        }
        let html = match response.text() { Ok(body) => body, Err(_) => return Ok(()) };
        let title = extract_title(&html).unwrap_or_else(|| url.to_string());
        let body = extract_main_text(&html);
        let (risk_score, risk_reason) = safety_assessment(&parsed, &title, &body, &html);
        if risk_score >= 5 { eprintln!("Blocked high-risk page {}: {}", url, risk_reason.as_deref().unwrap_or("risk signals")); }
        let terms = Self::tokenize(&format!("{} {}", title, body));
        if terms.is_empty() { return Ok(()); }
        let inserted = self.conn.execute(
            "INSERT OR IGNORE INTO documents (url, title, body, risk_score, risk_reason) VALUES (?, ?, ?, ?, ?)",
            params![url, title, body, risk_score, risk_reason],
        )?;
        if inserted == 0 { return Ok(()); }
        let doc_id: i64 = self.conn.query_row("SELECT id FROM documents WHERE url = ?", [url], |row| row.get(0))?;
        let mut counts = HashMap::new();
        for term in terms { *counts.entry(term).or_insert(0i32) += 1; }
        for (term, count) in counts {
            self.conn.execute("INSERT INTO term_counts (doc_id, term, count) VALUES (?, ?, ?)", params![doc_id, term, count])?;
        }
        Ok(())
    }
}

fn safety_assessment(url: &Url, title: &str, body: &str, html: &str) -> (i32, Option<String>) {
    let host = url.host_str().unwrap_or("").to_lowercase();
    let text = format!("{} {}", title, body).to_lowercase();
    let mut signals = Vec::new();
    let scam_words = ["guaranteed profit", "send bitcoin", "double your money", "act now", "claim your prize", "verify your account", "gift card", "crypto giveaway"];
    for word in scam_words { if text.contains(word) { signals.push(format!("contains scam signal '{}';", word)); } }
    if host.starts_with("xn--") || host.contains("--") { signals.push("uses a potentially deceptive internationalized domain".into()); }
    if host.parse::<std::net::IpAddr>().is_ok() { signals.push("uses an IP address instead of a domain".into()); }
    if host.matches('.').count() > 3 { signals.push("has an unusually deep subdomain".into()); }
    let password_form = html.to_lowercase().contains("type=\"password\"") || html.to_lowercase().contains("type='password'");
    if password_form && (text.contains("login") || text.contains("sign in") || text.contains("verify")) { signals.push("requests login credentials".into()); }
    let score = signals.len() as i32;
    (score, if signals.is_empty() { None } else { Some(signals.join(" ")) })
}

fn extract_title(html: &str) -> Option<String> {
    let doc = Html::parse_document(html); let selector = Selector::parse("title").ok()?;
    doc.select(&selector).next().map(|node| node.text().collect::<String>().trim().to_string())
}
fn extract_main_text(html: &str) -> String {
    let doc = Html::parse_document(html); let selector = Selector::parse("body").unwrap();
    let text = doc.select(&selector).next().map(|n| n.text().collect::<String>()).unwrap_or_default();
    Regex::new(r"\s+").unwrap().replace_all(&text, " ").trim().to_string()
}
fn snippet_for(body: &str, terms: &[String]) -> String {
    let words: Vec<&str> = body.split_whitespace().collect(); if words.is_empty() { return String::new(); }
    let mut best_start = 0; let mut best_score = -1;
    for start in 0..words.len().min(40) { let score = terms.iter().filter(|t| words[start..(start + 25).min(words.len())].iter().any(|w| w.to_lowercase().contains(t.as_str()))).count() as i32; if score > best_score { best_score = score; best_start = start; } }
    let snippet = words[best_start..(best_start + 30).min(words.len())].join(" ");
    if snippet.len() > 220 { format!("{}...", &snippet[..217]) } else { snippet }
}
