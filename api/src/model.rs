use crate::BookMap;
use actix_web::{error, Error, HttpRequest, HttpResponse, Responder};
use anyhow::Result;
use futures::future::{ready, Ready};
use serde::Serialize;
use sqlx::{FromRow, PgPool};

#[derive(Serialize, FromRow, Clone)]
pub struct Book {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub available: Option<i32>,
}

#[derive(Serialize, FromRow)]
pub struct Hadith {
    pub number: i32,
    pub indonesian: String,
    pub arabic: String,
    pub book_slug: String,
    pub book_name: String,
}

#[derive(Serialize, FromRow)]
pub struct HadithPagination {
    pub book: Book,
    pub items: Vec<Hadith>,
    pub limit: i32,
    pub offset: i32,
    pub total: i32,
}

impl Responder for Book {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();
        // create response and set content type
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

impl Responder for Hadith {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();
        // create response and set content type
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

impl Responder for HadithPagination {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();
        // create response and set content type
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

impl Book {
    pub async fn find_all(books: &BookMap) -> Result<Vec<Book>> {
        let mut books: Vec<Book> = books.values().cloned().collect();
        books.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(books)
    }

    pub async fn find_by_slug(
        pool: &PgPool,
        books: &BookMap,
        slug: &str,
        offset: i32,
        limit: i32,
    ) -> Result<HadithPagination, Error> {
        match books.get(slug) {
            Some(val) => {
                let book = val.clone();
                let available = match book.available {
                    Some(val) => val.clone(),
                    _ => 0,
                };
                let mut hadiths: Vec<Hadith> = Vec::new();

                let recs = sqlx::query!(
                    r#"
                        SELECT number, indonesian, arabic
                            FROM hadiths
                            WHERE book_id = $1
                        ORDER BY number
                        LIMIT $2
                        OFFSET $3
                    "#,
                    book.id,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(pool)
                .await
                .unwrap();

                for rec in recs {
                    hadiths.push(Hadith {
                        number: rec.number,
                        indonesian: rec.indonesian,
                        arabic: rec.arabic,
                        book_name: book.name.to_owned(),
                        book_slug: book.slug.to_owned(),
                    });
                }

                Ok(HadithPagination {
                    offset,
                    limit,
                    book,
                    items: hadiths,
                    total: available,
                })
            }
            None => Err(error::ErrorNotFound("not found")),
        }
    }
}
