use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use sled::Db;
use serde::{Serialize, Deserialize};
use tera::{Tera, Context};

#[derive(Serialize, Deserialize)]
struct Item {
    name: String,
    value: String,
}

async fn add_item(db: web::Data<Db>, item: web::Form<Item>) -> impl Responder {
    let serialized = serde_json::to_vec(&item).unwrap();
    db.insert(item.name.as_bytes(), serialized).unwrap();
    HttpResponse::Ok().body("Item added")
}

async fn signup_form(tmpl: web::Data<Tera>) -> impl Responder {
    let s = tmpl.render("signup.html", &Context::new()).unwrap();
    HttpResponse::Ok().content_type("text/html").body(s)
}

#[actix_web::main]
pub async fn run() -> std::io::Result<()> {
    let db_res = sled::open("my_db");
    let db = match db_res {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to open the database: {:?}", e);
            std::process::exit(1);
        }
    };
    let tera = Tera::new("templates/signup.html").unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(tera.clone()))
            .route("/add_item", web::post().to(add_item))
            .route("/signup", web::get().to(signup_form))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}