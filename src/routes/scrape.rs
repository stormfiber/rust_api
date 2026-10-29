use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_engine")]
    pub engine: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub debug: bool,
}

fn default_engine() -> String {
    "ddg".to_string()
}
fn default_limit() -> usize {
    10
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.route("/search", web::get().to(get_search));
    cfg.route("/search/images", web::get().to(get_search_images));
    cfg.route("/search/translate", web::get().to(get_search_translate));
    cfg.route("/search/tts", web::get().to(get_search_tts));
}

async fn get_search(web::Query(query): web::Query<SearchQuery>) -> impl Responder {
    match query.engine.as_str() {
        "ddg" | "duckduckgo" => scrape_ddg_search(&query.q, query.limit, query.debug),
        "bing" => scrape_bing_search(&query.q, query.limit, query.debug),
        "google" => {
            HttpResponse::ServiceUnavailable()
                .json(serde_json::json!({ "error": "google search is blocked by captcha; use ddg or bing" }))
        }
        _ => HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "unsupported engine" })),
    }
}

async fn get_search_images(web::Query(query): web::Query<SearchQuery>) -> impl Responder {
    match query.engine.as_str() {
        "ddg" | "duckduckgo" => scrape_ddg_images(&query.q, query.limit, query.debug),
        "bing" => scrape_bing_images(&query.q, query.limit, query.debug),
        "google" => {
            HttpResponse::ServiceUnavailable()
                .json(serde_json::json!({ "error": "google images is blocked by captcha; use ddg or bing" }))
        }
        _ => HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "unsupported engine" })),
    }
}

async fn get_search_translate(
    web::Query(query): web::Query<HashMap<String, String>>,
) -> impl Responder {
    let text = match query.get("text") {
        Some(t) => t.clone(),
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "error": "text is required" }));
        }
    };
    let from = query.get("from").or_else(|| query.get("sl")).cloned().unwrap_or_else(|| "en".to_string());
    let to = query.get("to").or_else(|| query.get("tl")).cloned().unwrap_or_else(|| "en".to_string());
    scrape_google_translate(&text, &from, &to, query.get("debug").map(|s| s.as_str()) == Some("1"))
}

async fn get_search_tts(
    web::Query(query): web::Query<HashMap<String, String>>,
) -> impl Responder {
    let text = match query.get("text") {
        Some(t) => t.clone(),
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "error": "text is required" }));
        }
    };
    let lang = query.get("lang").cloned().unwrap_or_else(|| "en".to_string());
    scrape_google_tts(&text, &lang).await
}

fn client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
        )
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap_or(reqwest::blocking::Client::new())
}

// ---------- DuckDuckGo Search ----------

fn scrape_ddg_search(query: &str, limit: usize, debug: bool) -> HttpResponse {
    let url = format!(
        "https://html.duckduckgo.com/html/?q={}",
        urlencoding::encode(query)
    );
    let html = match client().get(&url).send() {
        Ok(r) => match r.text() {
            Ok(t) => t,
            Err(e) => return err_response(&e),
        },
        Err(e) => return err_response(&e),
    };

    // Detect DDG bot detection page
    if html.contains("anomaly") || html.contains("bots use DuckDuckGo") || html.contains("anomaly-modal") {
        return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({ "error": "duckduckgo is blocking automated requests (bot detection); use engine=bing instead", "engine": "duckduckgo", "query": query, "results": [] }));
    }

    let results = parse_ddg_search(&html, limit);
    if results.is_empty() && debug {
        return HttpResponse::Ok().json(serde_json::json!({
            "engine": "duckduckgo", "query": query, "results": [],
            "debug": { "html_len": html.len(), "html_preview": &html[..html.len().min(500)] }
        }));
    }
    HttpResponse::Ok().json(serde_json::json!({
        "engine": "duckduckgo", "query": query, "results": results
    }))
}

fn parse_ddg_search(html: &str, limit: usize) -> Vec<serde_json::Value> {
    let fragment = scraper::Html::parse_document(html);
    let mut results = Vec::new();

    // Try multiple result selectors
    let selectors = [
        ".result",
        ".web-result",
        ".result__body",
    ];
    let result_sel = selectors.iter()
        .find_map(|s| scraper::Selector::parse(s).ok())
        .unwrap_or_else(|| scraper::Selector::parse(".result").unwrap());

    for elem in fragment.select(&result_sel).take(limit) {
        let title = elem
            .select(&scraper::Selector::parse(".result__a, h2 a, .result__title a").unwrap())
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let snippet = elem
            .select(&scraper::Selector::parse(".result__snippet, .result__snippet, .snippet").unwrap())
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let href = elem
            .select(&scraper::Selector::parse(".result__a, h2 a, .result__title a").unwrap())
            .next()
            .and_then(|e| e.value().attr("href"))
            .unwrap_or("");
        if title.is_empty() {
            continue;
        }
        results.push(serde_json::json!({
            "title": title, "url": extract_ddg_url(href), "snippet": snippet
        }));
    }
    results
}

fn extract_ddg_url(href: &str) -> String {
    if let Some(rest) = href.strip_prefix("//duckduckgo.com/l/?uddg=") {
        urlencoding::decode(rest.split('&').next().unwrap_or(""))
            .map(|c| c.into_owned())
            .unwrap_or_else(|_| href.to_string())
    } else if let Some(rest) = href.strip_prefix("/l/?uddg=") {
        urlencoding::decode(rest.split('&').next().unwrap_or(""))
            .map(|c| c.into_owned())
            .unwrap_or_else(|_| href.to_string())
    } else {
        href.to_string()
    }
}

// ---------- DuckDuckGo Images ----------

fn scrape_ddg_images(query: &str, limit: usize, debug: bool) -> HttpResponse {
    let search_url = format!(
        "https://html.duckduckgo.com/html/?q={}&iax=images&ia=images",
        urlencoding::encode(query)
    );
    let html = match client().get(&search_url).send() {
        Ok(r) => match r.text() {
            Ok(t) => t,
            Err(e) => return err_response(&e),
        },
        Err(e) => return err_response(&e),
    };

    // Detect DDG bot detection page
    if html.contains("anomaly") || html.contains("bots use DuckDuckGo") || html.contains("anomaly-modal") {
        return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({ "error": "duckduckgo is blocking automated requests (bot detection); use engine=bing instead", "engine": "duckduckgo", "query": query, "results": [] }));
    }

    // Try vqd token approach first
    let results = if let Some(vqd) = extract_vqd(&html) {
        let images_url = format!(
            "https://duckduckgo.com/i.js?l=us-en&o=json&q={}&vqd={}&f=,,,&p=1",
            urlencoding::encode(query), vqd
        );
        match client().get(&images_url).send() {
            Ok(r) => match r.text() {
                Ok(t) => {
                    let mut results = parse_ddg_images_json(&t, limit);
                    if results.is_empty() {
                        // Fallback: scrape image URLs from the HTML page directly
                        results = parse_ddg_images_from_html(&html, limit);
                    }
                    results
                }
                Err(_) => parse_ddg_images_from_html(&html, limit),
            },
            Err(_) => parse_ddg_images_from_html(&html, limit),
        }
    } else {
        parse_ddg_images_from_html(&html, limit)
    };

    if results.is_empty() && debug {
        return HttpResponse::Ok().json(serde_json::json!({
            "engine": "duckduckgo", "query": query, "results": [],
            "debug": { "html_len": html.len(), "vqd_found": extract_vqd(&html).is_some() }
        }));
    }

    HttpResponse::Ok().json(serde_json::json!({
        "engine": "duckduckgo", "query": query, "results": results
    }))
}

fn extract_vqd(html: &str) -> Option<String> {
    let patterns = [
        r#"vqd=['"](\d+-\d+)['"]"#,
        r"vqd=(\d+-\d+)",
        r#"vqd:['"](\d+-\d+)['"]"#,
        r"vqd='([^']+)'",
        r#"vqd="([^"]+)""#,
    ];
    for pat in patterns {
        if let Ok(re) = regex::Regex::new(pat) {
            if let Some(cap) = re.captures(html) {
                return Some(cap[1].to_string());
            }
        }
    }
    None
}

fn parse_ddg_images_json(text: &str, limit: usize) -> Vec<serde_json::Value> {
    let val: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let results = match val.get("results") {
        Some(serde_json::Value::Array(arr)) => arr,
        _ => return Vec::new(),
    };
    results.iter().take(limit).filter_map(|r| {
        let image = r.get("image")?.as_str()?.to_string();
        let title = r.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let width = r.get("width").and_then(|v| v.as_u64()).unwrap_or(0);
        let height = r.get("height").and_then(|v| v.as_u64()).unwrap_or(0);
        Some(serde_json::json!({
            "url": image, "title": title, "width": width, "height": height,
            "source": "duckduckgo"
        }))
    }).collect()
}

fn parse_ddg_images_from_html(html: &str, limit: usize) -> Vec<serde_json::Value> {
    let fragment = scraper::Html::parse_document(html);
    let mut results = Vec::new();

    // Look for image result links
    let selectors = [
        ".result__a",
        ".tile--img__img",
        ".img__a",
    ];
    for sel_str in &selectors {
        let sel = match scraper::Selector::parse(sel_str) {
            Ok(s) => s,
            Err(_) => continue,
        };
        for elem in fragment.select(&sel).take(limit) {
            let href = elem.value().attr("href").unwrap_or("");
            if href.starts_with("http") {
                results.push(serde_json::json!({
                    "url": href, "title": "", "width": 0, "height": 0,
                    "source": "duckduckgo"
                }));
            }
        }
        if !results.is_empty() {
            break;
        }
    }

    // Fallback: regex for image URLs
    if results.is_empty() {
        let re = regex::Regex::new(r#"(?:https?://)[^"'\s<>]+\.(?:jpg|jpeg|png|webp|gif)"#).ok();
        if let Some(re) = re {
            for cap in re.captures_iter(html).take(limit) {
                results.push(serde_json::json!({
                    "url": cap[0].to_string(), "title": "", "width": 0, "height": 0,
                    "source": "duckduckgo"
                }));
            }
        }
    }
    results
}

// ---------- Bing Search ----------

fn scrape_bing_search(query: &str, limit: usize, debug: bool) -> HttpResponse {
    let url = format!(
        "https://www.bing.com/search?q={}&count={}",
        urlencoding::encode(query), limit
    );
    let html = match client().get(&url).send() {
        Ok(r) => match r.text() {
            Ok(t) => t,
            Err(e) => return err_response(&e),
        },
        Err(e) => return err_response(&e),
    };
    let results = parse_bing_search(&html, limit);
    if results.is_empty() && debug {
        return HttpResponse::Ok().json(serde_json::json!({
            "engine": "bing", "query": query, "results": [],
            "debug": { "html_len": html.len(), "html_preview": &html[..html.len().min(500)] }
        }));
    }
    HttpResponse::Ok().json(serde_json::json!({
        "engine": "bing", "query": query, "results": results
    }))
}

fn parse_bing_search(html: &str, limit: usize) -> Vec<serde_json::Value> {
    let fragment = scraper::Html::parse_document(html);
    let mut results = Vec::new();

    let selectors = ["li.b_algo", ".b_algo"];
    let result_sel = selectors.iter()
        .find_map(|s| scraper::Selector::parse(s).ok())
        .unwrap_or_else(|| scraper::Selector::parse("li.b_algo").unwrap());

    for elem in fragment.select(&result_sel).take(limit) {
        let title = elem
            .select(&scraper::Selector::parse("h2 a").unwrap())
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let url = elem
            .select(&scraper::Selector::parse("h2 a").unwrap())
            .next()
            .and_then(|e| e.value().attr("href"))
            .unwrap_or("");
        let snippet = elem
            .select(&scraper::Selector::parse(".b_caption p").unwrap())
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        if title.is_empty() {
            continue;
        }
        results.push(serde_json::json!({ "title": title, "url": url, "snippet": snippet }));
    }
    results
}

// ---------- Bing Images ----------

fn scrape_bing_images(query: &str, limit: usize, debug: bool) -> HttpResponse {
    let url = format!(
        "https://www.bing.com/images/search?q={}&count={}",
        urlencoding::encode(query), limit
    );
    let html = match client().get(&url).send() {
        Ok(r) => match r.text() {
            Ok(t) => t,
            Err(e) => return err_response(&e),
        },
        Err(e) => return err_response(&e),
    };
    let results = parse_bing_images(&html, limit);
    if results.is_empty() && debug {
        return HttpResponse::Ok().json(serde_json::json!({
            "engine": "bing", "query": query, "results": [],
            "debug": { "html_len": html.len(), "html_preview": &html[..html.len().min(500)] }
        }));
    }
    HttpResponse::Ok().json(serde_json::json!({
        "engine": "bing", "query": query, "results": results
    }))
}

fn parse_bing_images(html: &str, limit: usize) -> Vec<serde_json::Value> {
    let mut results = Vec::new();

    // Primary: regex on raw HTML for HTML-encoded murl
    let patterns = [
        r#"&quot;murl&quot;:&quot;([^&quot;]+)&quot;"#,
        r#""murl":"([^"]+)""#,
        r#"murl":"([^"]+)""#,
    ];
    for pat in &patterns {
        if let Ok(re) = regex::Regex::new(pat) {
            for cap in re.captures_iter(html).take(limit) {
                let url = html_decode(&cap[1]);
                if url.starts_with("http") {
                    results.push(serde_json::json!({
                        "url": url, "source": "bing"
                    }));
                }
            }
            if !results.is_empty() {
                return results;
            }
        }
    }

    // Fallback: scraper on a.iusc elements
    let fragment = scraper::Html::parse_document(html);
    let sel = match scraper::Selector::parse("a.iusc") {
        Ok(s) => s,
        Err(_) => return results,
    };
    for elem in fragment.select(&sel).take(limit) {
        let m_attr = elem.value().attr("m").unwrap_or("");
        // scraper already HTML-decodes attribute values, so m_attr is already decoded JSON
        if let Some(url) = extract_bing_m_url(m_attr) {
            results.push(serde_json::json!({
                "url": url, "source": "bing"
            }));
        }
    }

    results
}

fn extract_bing_m_url(m_attr: &str) -> Option<String> {
    let re = regex::Regex::new(r#""murl"\s*:\s*"([^"]+)""#).ok()?;
    re.captures(m_attr).and_then(|c| Some(c[1].to_string()))
}

fn html_decode(s: &str) -> String {
    s.replace("&quot;", "\"")
     .replace("&amp;", "&")
     .replace("&#x27;", "'")
     .replace("&#39;", "'")
     .replace("&lt;", "<")
     .replace("&gt;", ">")
     .replace("&apos;", "'")
     .replace("&#x2F;", "/")
     .replace("&#47;", "/")
}

// ---------- MyMemory Translate ----------

fn scrape_google_translate(text: &str, from: &str, to: &str, _debug: bool) -> HttpResponse {
    let url = format!(
        "https://api.mymemory.translated.net/get?q={}&langpair={}|{}",
        urlencoding::encode(text), from, to
    );
    let resp = match client().get(&url).send() {
        Ok(r) => r,
        Err(e) => return err_response(&e),
    };
    let json_text = match resp.text() {
        Ok(t) => t,
        Err(e) => return err_response(&e),
    };
    let val: serde_json::Value = match serde_json::from_str(&json_text) {
        Ok(v) => v,
        Err(e) => return HttpResponse::BadGateway()
            .json(serde_json::json!({ "error": format!("failed to parse translation: {e}") })),
    };
    let translated = val
        .get("responseData")
        .and_then(|d| d.get("translatedText"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let response_status = val
        .get("responseStatus")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    HttpResponse::Ok().json(serde_json::json!({
        "from": from, "to": to, "text": text, "translated": translated,
        "engine": "mymemory-translate",
        "status": response_status
    }))
}

// ---------- Google TTS ----------

async fn scrape_google_tts(text: &str, lang: &str) -> HttpResponse {
    let url = format!(
        "https://translate.google.com/translate_tts?ie=UTF-8&q={}&tl={}&total=1&idx=0&textlen={}&client=tw-ob",
        urlencoding::encode(text), lang, text.len()
    );
    let resp = match client().get(&url).send() {
        Ok(r) => r,
        Err(e) => return err_response(&e),
    };
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("audio/mpeg")
        .to_string();
    let bytes = match resp.bytes() {
        Ok(b) => b.to_vec(),
        Err(e) => return err_response(&e),
    };
    HttpResponse::Ok().content_type(content_type).body(bytes)
}

fn err_response(e: &reqwest::Error) -> HttpResponse {
    HttpResponse::BadGateway()
        .json(serde_json::json!({ "error": format!("upstream failed: {e}") }))
}
