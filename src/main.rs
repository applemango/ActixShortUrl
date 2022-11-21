#![allow(legacy_derive_helpers)]
use actix_web::{get, post, web, App, HttpServer, Responder, Result, error};
use rusqlite::Connection;
use derive_more::{Display, Error};
use serde::{Serialize, Deserialize};

#[display(fmt = "my error: {}", name)]
#[derive(Debug, Display, Error)]
struct MyError {
    name: &'static str,
}

#[derive(Serialize)]
struct Url {
    id: i32,
    url: String,
}

#[derive(Deserialize)]
struct CreateBody {
    url: String,
}

impl error::ResponseError for MyError {}

#[post("/url/create")]
async fn create_(data: web::Json<CreateBody>) -> Result<impl Responder, MyError> {
    let url = &data.url;

    let db_con = match Connection::open("app.db") {
        Ok(connection) => connection,
        Err(_) => {
            return Err(MyError {name: "db connection error"})
        }
    };

    let mut statement = match db_con.prepare("INSERT INTO url ( url ) values ( ?1 )") {
        Ok(statement) => statement,
        Err(_) => return Err(MyError {name: "Failed to prepare query".into()}),
    };

    let mut _r = match statement.execute(&[&url]) {
        Ok(r) => r,
        Err(_) => return Err(MyError {name: "Failed"})
    };

    let u = match db_con.query_row("SELECT id, url FROM url WHERE id = last_insert_rowid()", [], |row| {
        Ok(Url {
            id: row.get(0)?,
            url: row.get(1)?
        })
    }) {
        Ok(u) => u,
        Err(_) => {
            return Err(MyError {name: "not found"})
        }
    };

    Ok(web::Json(u))
}

#[get("/url/{id}")]
async fn get_(path: web::Path<i32>) -> Result<impl Responder, MyError> {

    let db_con = match Connection::open("app.db") {
        Ok(connection) => connection,
        Err(_) => {
            return Err(MyError {name: "db connection error"})
        }
    };

    let id = path.into_inner();

    let u = match db_con.query_row("SELECT id, url FROM url WHERE id = ( ?1 )", [id], |row| {
        Ok(Url {
            id: row.get(0)?,
            url: row.get(1)?
        })
    }) {
        Ok(u) => u,
        Err(_) => {
            return Err(MyError {name: "not found"})
        }
    };

    Ok(web::Json(u))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    {
        let conn = Connection::open_in_memory().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS url (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url STRING
            )",
            ()
        ).unwrap();
    }
    HttpServer::new(|| {
        App::new()
            .service(get_)
            .service(create_)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}