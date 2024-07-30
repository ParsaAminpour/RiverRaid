use actix_web::{get, web, App, HttpRequest, Responder, HttpServer, HttpResponse};
use sled::Db;
use serde::{Serialize, Deserialize};
use tera::{Tera, Context};
use std::sync::Mutex;
// use shuttle_actix_web::ShuttleActixWeb;

static WORKERS: u8 = 4;

#[derive(Serialize, Deserialize)]
struct Item {
    name: String,
    value: String,
}

async fn add_item(db: web::Data<Mutex<Db>>, item: web::Form<Item>) -> impl Responder {
    let db = db.lock().unwrap();
    let serialized = serde_json::to_vec(&item).unwrap();
    db.insert(item.name.as_bytes(), serialized).unwrap();
    HttpResponse::Ok().body("Item added")
}

async fn signup_form(tmpl: web::Data<Tera>) -> impl Responder {
    let s = tmpl.render("signup.html", &Context::new()).unwrap();
    HttpResponse::Ok().content_type("text/html").body(s)
}

#[get("/")]
async fn home_page(_req: HttpRequest) -> impl Responder {
    "Welcome to the River Raid game"
}


pub async fn run_server() -> std::io::Result<()> {
    let db_result = sled::open("my_db");

    let db = match db_result {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to open the database: {:?}", e);
            std::process::exit(1);
        }
    };

    let tera = Tera::new("templates/*").unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Mutex::new(db.clone())))
            .app_data(web::Data::new(tera.clone()))
            .route("/add_item", web::post().to(add_item))
            .route("/signup", web::get().to(signup_form))
            .service(home_page)
    })
    .workers(WORKERS as usize)
    .bind("127.0.0.1:8081")?
    .run()
    .await
}
