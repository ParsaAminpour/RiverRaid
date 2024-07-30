use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder, ResponseError};
use sled::Db;
use actix_files as fs;
use serde::{Serialize, Deserialize};
use serde_json::{json};
use tera::{Tera, Context};
use std::borrow::Borrow;
use std::sync::Mutex;
use inline_colorization::*;

extern crate argon2;
extern crate jsonwebtoken as jwt;
extern crate serde;
extern crate chrono;
extern crate serde_json;

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use jwt::{encode, Header, EncodingKey};
use chrono::{Utc, Duration};
use std::env;



static WORKERS: u8 = 4;


#[derive(Serialize, Deserialize, Debug)]
struct Item {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();
    password_hash.to_string()
}

fn verify_password(hash: &str, password: &str) -> bool {
    let parsed_hash = PasswordHash::new(hash).unwrap();
    let argon2 = Argon2::default();
    argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok()
}

fn generate_jwt(username: &str) -> String {
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(1))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: username.to_owned(),
        exp: expiration as usize,
    };

    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref())).unwrap()
}   


async fn add_item(db: web::Data<Mutex<Db>>, item: web::Form<Item>) -> impl Responder {
    let db = db.lock().unwrap();
    let mut serialized = serde_json::to_vec(&item).unwrap();

    let mut deserialized: serde_json::Value = serde_json::from_slice(&serialized).unwrap();
    let jwt_token = generate_jwt(&item.username);

    if let serde_json::Value::Object(ref mut map) = deserialized {
        map.insert("password".to_string(), json!(hash_password(&item.password)));
        map.insert("token".to_string(), json!(jwt_token));
    }

    serialized = serde_json::to_vec(&deserialized).unwrap();
    println!("{color_green}serialized version: {:?}{color_reset}\n", serialized);
    println!("items: {:?}", item);
    
    db.insert(item.username.as_bytes(), serialized).unwrap();
    HttpResponse::Ok().body("Item added")
}

fn get_item(db: &Db, key: &str) -> sled::Result<Option<Item>> {
    if let Some(serialized) = db.get(key)? {
        let item: Item = serde_json::from_slice(&serialized).unwrap();
        Ok(Some(item))
    } else {
        Ok(None)
    }
}


// todo: add this feature to the web.
async fn get_user(db: web::Data<Mutex<Db>>) -> impl Responder {
    HttpResponse::Ok().body("test")
}

#[get("/signup")]
async fn signup_form(tmpl: web::Data<Tera>) -> impl Responder {
    let s = tmpl.render("signup.html", &Context::new()).unwrap();
    HttpResponse::Ok().content_type("text/html").body(s)
}

#[get("/")]
async fn home_page(tmpl: web::Data<Tera>) -> impl Responder {
    let s = tmpl.render("home.html", &Context::new()).unwrap();
    HttpResponse::Ok().content_type("text/html").body(s)
}



pub async fn run_server() -> std::io::Result<()> {
    let db_result = sled::open("my_db");

    let db = match db_result {
        Ok(db) => {
            println!("user parsa is {:?}", Some(get_item(&db, "parsa")));
            db
        }
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
            .service(signup_form)
            .service(home_page)
            // .service(get_user)
    })
    .workers(WORKERS as usize)
    .bind("127.0.0.1:8081")?
    .run()
    .await
}
