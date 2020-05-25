#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use rocket::config::Environment;
use rocket::fairing::AdHoc;
use rocket::response::content::Html;
use rocket::response::NamedFile;
use rocket::State;
use rocket_contrib::json::JsonValue;
use structopt::StructOpt;

#[get("/")]
fn index(state: State<AppState>) -> Html<String> {
    let path = Path::new(&state.static_dir).join("index.html");
    let file = NamedFile::open(path).ok();
    if let Some(mut file) = file {
        let mut buf = String::new();
        file.read_to_string(&mut buf).expect(&format!(
            "unable to read index.html at '{}'",
            file.path().display()
        ));
        Html(buf)
    } else {
        Html(format!(
            r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Api Tester</title>
            </head>
            <body>
                <h1>API Tester</h1>
                <p>You are seeing this page because you do not have an <strong>index.html</strong> file in your static files directory, which is currently set to <strong>{}</strong>.</p>
                <p>To use:
                    <ul>
                        <li>Add any API responses into an <strong>api</strong> directory as JSON files. Subdirectories may be created to represent additional API paths.</li>
                        <li>Use the <strong>--static-dir</strong> flag to set your static files/frontend files directory.</li>
                        <li>Use --delay to set a delay on API responses (nice for testing spinners).</li>
                    </ul>
                </p>
                <p>If there are any errors in your JSON files, then an error will be displayed in the server console and a 500 will be returned.</p>
                <p>If the URL provided does not match either an API JSON file, or a static file, then a 404 will be returned.</p>
            </body>
            </html>
        "#,
            state.static_dir
        ))
    }
}

fn delay(ms: u64) {
    if ms > 0 {
        let delay = Duration::from_millis(ms);
        thread::sleep(delay);
    }
}

fn spoof(file: PathBuf, state: State<AppState>) -> Option<JsonValue> {
    delay(state.response_delay);
    let file = file.with_extension("json");
    let file = NamedFile::open(Path::new("api/").join(file)).ok();

    if let Some(mut file) = file {
        let mut buf = String::new();
        file.read_to_string(&mut buf).expect(&format!(
            "unable to read JSON API file at: {}",
            file.path().display()
        ));
        Some(serde_json::from_str(&buf).expect(&format!(
            "deserialization failed for API file at: {}",
            file.path().display()
        )))
    } else {
        None
    }
}

#[delete("/api/<file..>")]
fn api_delete(file: PathBuf, state: State<AppState>) -> Option<JsonValue> {
    spoof(file, state)
}

#[get("/api/<file..>")]
fn api_get(file: PathBuf, state: State<AppState>) -> Option<JsonValue> {
    spoof(file, state)
}

#[head("/api/<file..>")]
fn api_head(file: PathBuf, state: State<AppState>) -> Option<JsonValue> {
    spoof(file, state)
}

#[options("/api/<file..>")]
fn api_options(file: PathBuf, state: State<AppState>) -> Option<JsonValue> {
    spoof(file, state)
}

#[patch("/api/<file..>")]
fn api_patch(file: PathBuf, state: State<AppState>) -> Option<JsonValue> {
    spoof(file, state)
}

#[post("/api/<file..>")]
fn api_post(file: PathBuf, state: State<AppState>) -> Option<JsonValue> {
    spoof(file, state)
}

#[put("/api/<file..>")]
fn api_put(file: PathBuf, state: State<AppState>) -> Option<JsonValue> {
    spoof(file, state)
}

#[get("/<file..>", rank = 1)]
fn root_files(file: PathBuf, state: State<AppState>) -> Option<NamedFile> {
    let path = Path::new(&format!("{}", state.static_dir)).join(file);
    NamedFile::open(path).ok()
}

/// A simple tool to test frontend code with faked API requests
#[derive(StructOpt, Debug)]
#[structopt(name = "spa-host")]
struct Opt {
    /// Port to use for hosting.
    #[structopt(short, long, default_value = "8000")]
    port: u16,

    /// Bind address
    #[structopt(short, long, default_value = "localhost")]
    address: String,

    /// API response delay (ms)
    #[structopt(short = "d", long = "delay", default_value = "0")]
    api_delay: u64,

    /// Static file directory
    #[structopt(long, default_value = "static")]
    static_dir: String,

    /// Display detailed information about requests
    #[structopt(long = "detail")]
    detailed_requests: bool
}

struct AppState {
    response_delay: u64,
    static_dir: String,
}

fn main() {
    let opt = Opt::from_args();

    let detailed_requests = opt.detailed_requests;

    let rocket_config = rocket::Config::build(Environment::Development)
        .port(opt.port)
        .address(&opt.address)
        .finalize()
        .expect("Invalid server configuration");

    println!("Static files being served from: {}", opt.static_dir);

    rocket::custom(rocket_config)
        .manage(AppState {
            response_delay: opt.api_delay,
            static_dir: opt.static_dir,
        })
        .attach(AdHoc::on_request("Request Logger", move |req, data| {
            if detailed_requests {
                println!("Query params: {:#?}", req);
            } else {
                println!("Query params: {:#?}", req.uri().query());
            }
            println!("Request data (first 512 bytes): {:#?}", data.peek());
        }))
        .mount(
            "/",
            routes![
                index,
                root_files,
                api_delete,
                api_get,
                api_head,
                api_options,
                api_patch,
                api_post,
                api_put
            ],
        )
        .launch();
}
