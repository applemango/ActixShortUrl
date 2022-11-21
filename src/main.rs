//mod db;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result, error};
use rusqlite::{Connection};
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

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[post("/url/create")]
async fn create_(data: web::Json<CreateBody>) -> Result<impl Responder, MyError> {
    println!("hello, world!");
    let url = &data.url;

    let dbCon = match Connection::open("app.db") {
        Ok(connection) => connection,
        Err(_) => {
            return Err(MyError {name: "db connection error"})
        }
    };

    let mut statement = match dbCon.prepare("INSERT INTO url ( url ) values ( ?1 )") {
        Ok(statement) => statement,
        Err(_) => return Err(MyError {name: "Failed to prepare query".into()}),
    };

    let mut _r = match statement.execute(&[&url]) {
        Ok(r) => r,
        Err(_) => return Err(MyError {name: "Failed"})
    };

    let u = match dbCon.query_row("SELECT id, url FROM url WHERE id = last_insert_rowid()", [], |row| {
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

    let dbCon = match Connection::open("app.db") {
        Ok(connection) => connection,
        Err(_) => {
            return Err(MyError {name: "db connection error"})
        }
    };

    let id = path.into_inner();

    let u = match dbCon.query_row("SELECT id, url FROM url WHERE id = ( ?1 )", [id], |row| {
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

    //let id_ = path.into_inner();
    /*let mut obj = Url {
        id: id_,
        url: "https://example.com".to_string(),
    };*/

    Ok(web::Json(u))
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    {
        let conn = Connection::open_in_memory().unwrap();
        /*conn.execute(
            "CREATE TABLE url (
                id    INTEGER PRIMARY KEY,
                url  TEXT NOT NULL,
            )",
            (), // empty list of parameters.
        ).unwrap();*/

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
            .service(hello)
            .service(echo)
            .service(get_)
            .service(create_)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
/*
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //db::connect_db();
    let DB = db::Connect()?;
    let u = db::get_url(&DB, 1)?;

    #[get("/{id}")]
    async fn get_(path: web::Path<i32>) -> Result<impl Responder> {

        #[derive(Serialize)]
        struct Url {
            id: i32,
            url: String,
        }

        let id_ = path.into_inner();

        let obj = Url {
            id: id_,
            url: "https://example.com".to_string(),
        };

        Ok(web::Json(obj))
    }

    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .service(get_)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
*/