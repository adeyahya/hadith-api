extern crate dotenv;

mod model;

use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use model::Book;
use qstring::QString;
use sqlx::PgPool;
use std::collections::HashMap;
use std::env;

pub type BookMap = HashMap<String, Book>;

async fn get_books(books: web::Data<BookMap>) -> impl Responder {
    let result = Book::find_all(books.get_ref()).await;
    match result {
        Ok(books) => HttpResponse::Ok().json(books),
        _ => HttpResponse::BadRequest().body("Error trying to read all todos from database"),
    }
}

async fn get_book_by_slug(
    pool: web::Data<PgPool>,
    books: web::Data<BookMap>,
    req: HttpRequest,
) -> impl Responder {
    let slug: String = req.match_info().get("slug").unwrap().parse().unwrap();
    let qs = QString::from(req.query_string());
    let limit = qs.get("limit").unwrap_or("5");
    let limit: i32 = limit.parse().unwrap();
    let offset = qs.get("offset").unwrap_or("0");
    let offset: i32 = offset.parse().unwrap();

    let result = Book::find_by_slug(pool.get_ref(), books.get_ref(), &slug, offset, limit).await;
    match result {
        Ok(books) => HttpResponse::Ok().json(books),
        Err(e) => HttpResponse::from(e),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    // database pool
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let db_pool = PgPool::connect(&database_url).await.unwrap();

    // fetch all books as in memory cache
    let mut books: HashMap<String, Book> = HashMap::new();
    let recs = sqlx::query!(
        r#"
            SELECT id, name, slug, available
                FROM books
            ORDER BY id
        "#
    )
    .fetch_all(&db_pool)
    .await
    .unwrap();

    for rec in recs {
        books.insert(
            rec.slug.to_owned(),
            Book {
                id: rec.id,
                name: rec.name,
                slug: rec.slug,
                available: rec.available,
            },
        );
    }

    HttpServer::new(move || {
        App::new()
            .data(db_pool.clone())
            .data(books.clone())
            .service(
                web::scope("/rest")
                    .route("/books/{slug}", web::get().to(get_book_by_slug))
                    .route("/books", web::get().to(get_books)),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
