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
        "ddg" | "duckduckgo" => scrape_ddg_search(&query.q, query.limit),
        "bing" => scrape_bing_search(&query.q, query.limit),
        "google" => scrape_google_search(&query.q, query.limit),
        _ => HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "unsupported engine" })),
    }
}

async fn get_search_images(web::Query(query): web::Query<SearchQuery>) -> impl Responder {
    match query.engine.as_str() {
        "ddg" | "duckduckgo" => scrape_ddg_images(&query.q, query.limit),
        "bing" => scrape_bing_images(&query.q, query.limit),
        "google" => scrape_google_images(&query.q, query.limit),
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
    let from = query.get("from").cloned().unwrap_or_else(|| "auto".to_string());
    let to = query.get("to").cloned().unwrap_or_else(|| "en".to_string());
    scrape_google_translate(&text, &from, &to)
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
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap_or(reqwest::blocking::Client::new())
}

// ---------- DuckDuckGo ----------

fn scrape_ddg_search(query: &str, limit: usize) -> HttpResponse {
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
    let results = parse_ddg_search(&html, limit);
    HttpResponse::Ok().json(serde_json::json!({
        "engine": "duckduckgo", "query": query, "results": results
    }))
}

fn parse_ddg_search(html: &str, limit: usize) -> Vec<serde_json::Value> {
    let fragment = scraper::Html::parse_document(html);
    let mut results = Vec::new();
    let result_sel = scraper::Selector::parse(".result").unwrap();
    for elem in fragment.select(&result_sel).take(limit) {
        let title = elem
            .select(&scraper::Selector::parse(".result__a").unwrap())
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let snippet = elem
            .select(&scraper::Selector::parse(".result__snippet").unwrap())
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let href = elem
            .select(&scraper::Selector::parse(".result__a").unwrap())
            .next()
            .and_then(|e| e.value().attr("href"))
            .unwrap_or("");
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
    } else {
        href.to_string()
    }
}

fn scrape_ddg_images(query: &str, limit: usize) -> HttpResponse {
    let search_url = format!(
        "https://duckduckgo.com/?q={}&iax=images&ia=images",
        urlencoding::encode(query)
    );
    let html = match client().get(&search_url).send() {
        Ok(r) => match r.text() {
            Ok(t) => t,
            Err(e) => return err_response(&e),
        },
        Err(e) => return err_response(&e),
    };
    let vqd = match extract_vqd(&html) {
        Some(v) => v,
        None => {
            return HttpResponse::Ok().json(serde_json::json!({
                "engine": "duckduckgo", "query": query, "results": [],
                "error": "could not extract vqd token"
            }));
        }
    };
    let images_url = format!(
        "https://duckduckgo.com/i.js?l=us-en&o=json&q={}&vqd={}&f=,,,&p=1",
        urlencoding::encode(query), vqd
    );
    let json_text = match client().get(&images_url).send() {
        Ok(r) => match r.text() {
            Ok(t) => t,
            Err(e) => return err_response(&e),
        },
        Err(e) => return err_response(&e),
    };
    let results = parse_ddg_images_json(&json_text, limit);
    HttpResponse::Ok().json(serde_json::json!({
        "engine": "duckduckgo", "query": query, "results": results
    }))
}

fn extract_vqd(html: &str) -> Option<String> {
    regex::Regex::new("vqd=['\"](\\d+-\\d+)['\"]")
        .ok()
        .and_then(|re| re.captures(html).map(|c| c[1].to_string()))
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

// ---------- Bing ----------

fn scrape_bing_search(query: &str, limit: usize) -> HttpResponse {
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
    HttpResponse::Ok().json(serde_json::json!({
        "engine": "bing", "query": query, "results": results
    }))
}

fn parse_bing_search(html: &str, limit: usize) -> Vec<serde_json::Value> {
    let fragment = scraper::Html::parse_document(html);
    let mut results = Vec::new();
    let result_sel = scraper::Selector::parse("li.b_algo").unwrap();
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
        results.push(serde_json::json!({ "title": title, "url": url, "snippet": snippet }));
    }
    results
}

fn scrape_bing_images(query: &str, limit: usize) -> HttpResponse {
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
    HttpResponse::Ok().json(serde_json::json!({
        "engine": "bing", "query": query, "results": results
    }))
}

fn parse_bing_images(html: &str, limit: usize) -> Vec<serde_json::Value> {
    let re = match regex::Regex::new("murl:\"([^\"]+)\"") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    re.captures_iter(html).take(limit).filter_map(|cap| {
        Some(serde_json::json!({
            "url": cap[1].replace("&quot;", ""),
            "source": "bing"
        }))
    }).collect()
}

// ---------- Google ----------

fn scrape_google_search(query: &str, limit: usize) -> HttpResponse {
    let url = format!(
        "https://www.google.com/search?q={}&num={}",
        urlencoding::encode(query), limit
    );
    let html = match client().get(&url).send() {
        Ok(r) => match r.text() {
            Ok(t) => t,
            Err(e) => return err_response(&e),
        },
        Err(e) => return err_response(&e),
    };
    if html.contains("unusual traffic") || html.contains("captcha") {
        return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({ "error": "google blocked" }));
    }
    let results = parse_google_search(&html, limit);
    HttpResponse::Ok().json(serde_json::json!({
        "engine": "google", "query": query, "results": results
    }))
}

fn parse_google_search(html: &str, limit: usize) -> Vec<serde_json::Value> {
    let fragment = scraper::Html::parse_document(html);
    let mut results = Vec::new();
    let result_sel = scraper::Selector::parse(".g").unwrap();
    for elem in fragment.select(&result_sel).take(limit) {
        let title = elem
            .select(&scraper::Selector::parse("h3").unwrap())
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        if title.is_empty() {
            continue;
        }
        let url = elem
            .select(&scraper::Selector::parse("a").unwrap())
            .next()
            .and_then(|e| e.value().attr("href"))
            .unwrap_or("");
        let snippet = elem
            .select(&scraper::Selector::parse(".VwiC3b").unwrap())
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        results.push(serde_json::json!({ "title": title, "url": url, "snippet": snippet }));
    }
    results
}

fn scrape_google_images(query: &str, limit: usize) -> HttpResponse {
    let url = format!(
        "https://www.google.com/search?q={}&tbm=isch",
        urlencoding::encode(query)
    );
    let html = match client().get(&url).send() {
        Ok(r) => match r.text() {
            Ok(t) => t,
            Err(e) => return err_response(&e),
        },
        Err(e) => return err_response(&e),
    };
    if html.contains("unusual traffic") || html.contains("captcha") {
        return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({ "error": "google blocked" }));
    }
    let results = parse_google_images(&html, limit);
    HttpResponse::Ok().json(serde_json::json!({
        "engine": "google", "query": query, "results": results
    }))
}

fn parse_google_images(html: &str, limit: usize) -> Vec<serde_json::Value> {
    let re = match regex::Regex::new("&quot;([^&quot;]+\\.(?:jpg|jpeg|png|webp|gif))") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    re.captures_iter(html).take(limit).filter_map(|cap| {
        Some(serde_json::json!({ "url": cap[1].replace("&quot;", ""), "source": "google" }))
    }).collect()
}

// ---------- Google Translate ----------

fn scrape_google_translate(text: &str, from: &str, to: &str) -> HttpResponse {
    let url = format!(
        "https://translate.google.com/translate?sl={}&tl={}&text={}",
        from, to, urlencoding::encode(text)
    );
    let html = match client().get(&url).send() {
        Ok(r) => match r.text() {
            Ok(t) => t,
            Err(e) => return err_response(&e),
        },
        Err(e) => return err_response(&e),
    };
    let translated = extract_google_translate_result(&html);
    HttpResponse::Ok().json(serde_json::json!({
        "from": from, "to": to, "text": text, "translated": translated,
        "engine": "google-translate-scrape"
    }))
}

fn extract_google_translate_result(html: &str) -> String {
    regex::Regex::new(r##"result,?"([^"]+)","##)
        .ok()
        .and_then(|re| re.captures(html).and_then(|c| c.get(1).map(|m| m.as_str().to_string())))
        .unwrap_or_default()
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
