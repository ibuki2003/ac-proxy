use actix_web::{get, http, web, HttpRequest, HttpResponse, Responder};
use log::error;
use mysql_async::prelude::*;
use regex::Regex;

// #[macro_use]
// extern crate lazy_static;
use lazy_static::lazy_static;

// use openssl::ssl::{SslMethod, SslConnector};
// use ac_proxy::AppState;

lazy_static! {
    static ref PATH_PATTERNS: Vec<Regex> = vec![Regex::new(r"^/users/\w+/history/json").unwrap(),];
}

#[get("/{path:.+}")]
pub async fn proxy_service(data: web::Data<crate::AppState>, req: HttpRequest) -> impl Responder {
    let path = req.path().to_string() + "?" + &req.query_string();

    if !PATH_PATTERNS.iter().any(|r| r.is_match(&path)) {
        return HttpResponse::NotFound().finish();
    }

    let mut conn = data.pool.get_conn().await.unwrap();
    if let Some(res) = get_cache(&path, &mut conn).await {
        return res;
    }

    // return HttpResponse::Ok().body("200 OK");

    let client = awc::Client::default();
    let req = client.get(String::from("https://atcoder.jp") + &path);
    // let req = client.get("https://fuwa.dev/notfound");
    match req.send().await {
        Ok(mut res) => {
            let body = res.body().await.unwrap();
            let body = match String::from_utf8(body.to_vec()) {
                Ok(body) => body,
                Err(_) => {
                    return HttpResponse::InternalServerError().body("500 Internal Server Error")
                }
            };
            let content_type = res
                .headers()
                .get(http::header::CONTENT_TYPE)
                .map(|s| s.to_str().unwrap().to_string());
            let date = res
                .headers()
                .get(http::header::DATE)
                .map(|s| s.to_str().unwrap().to_string());

            if let Err(e) = save_cache(
                &path,
                if res.status().is_success() {
                    Some(&body)
                } else {
                    None
                },
                &content_type,
                &date,
                &mut conn,
            )
            .await
            {
                error!("save error {}", e);
                // return HttpResponse::InternalServerError().body("500 Internal Server Error");
            }
            let body = if res.status().is_success() {
                body
            } else {
                String::from("not found")
            };

            let res = HttpResponse::build(res.status())
                .insert_header((
                    http::header::CONTENT_TYPE,
                    content_type.unwrap_or(String::from("text/plain")),
                ))
                .body(body);
            return res;
        }
        Err(e) => {
            error!("req error {}", e);
            return HttpResponse::BadGateway().body("502 Bad Gateway");
        }
    }
}

async fn get_cache(path: &String, conn: &mut mysql_async::Conn) -> Option<HttpResponse> {
    "SELECT body, content_type, date FROM cache WHERE path = ?"
        .with((path,))
        .first::<(Option<String>, Option<String>, String), _>(conn)
        .await
        .unwrap_or_else(|e| {
            error!("get error {}", e);
            return None;
        })
        .map(|(body, content_type, date)| {
            return HttpResponse::build(if body.is_some() {
                http::StatusCode::OK
            } else {
                http::StatusCode::NOT_FOUND
            })
            .content_type(content_type.unwrap_or(String::from("text/plain")))
            .insert_header((http::header::LAST_MODIFIED, date))
            .body(body.unwrap_or(String::from("not found")));
        })
}

async fn save_cache(
    path: &String,
    body: Option<&String>,
    content_type: &Option<String>,
    date: &Option<String>,
    conn: &mut mysql_async::Conn,
) -> Result<(), mysql_async::Error> {
    let now = actix_web::http::header::Date::now().to_string();
    let date: &String = match date {
        Some(date) => &date,
        None => &now,
    };

    "INSERT INTO cache (path, body, content_type, date) VALUES (?, ?, ?, ?) ON DUPLICATE KEY UPDATE body = ?, content_type = ?, date = ?"
    .with((path, &body, &content_type, &date, &body, &content_type, &date))
    .run(conn)
    .await?;

    Ok(())
}
