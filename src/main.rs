#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use rocket::config::Environment;
use rocket::response::content::Html;
use rocket::response::NamedFile;
use rocket::State;
use rocket_contrib::json::JsonValue;
use structopt::StructOpt;

#[get("/")]
fn index() -> Html<String> {
    Html(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Api Tester</title>
</head>
<body>
    <h1>API Tester</h1>
    <p>To use:
        <ul>
            <li>Add any API responses into the <em>'api'</em> directory as JSON files <strong>(no .JSON extension!)</strong>. Subdirectories may be created to represent additional API paths.</li>
            <li>Copy all frontend data into the <em>'static'</em> directory. This includes scripts, styles, html pages, etc.</li>
            <li>Run the <em>'api-tester'</em> application.</li>
        </ul>
    </p>
    <p>If there are any errors in your JSON files, then an error will be displayed in the server console and a 500 will be returned</p>
    <p>If the URL provided does not match either an API JSON file, or a static file, then a 404 will be returned.</p>
</body>
</html>
    "#.to_owned())
}

fn delay(ms: u64) {
    if ms > 0 {
        let delay = Duration::from_millis(ms);
        thread::sleep(delay);
    }
}

#[get("/api/<file..>")]
fn api(file: PathBuf, state: State<AppState>) -> Option<JsonValue> {
    delay(state.response_delay);

    let file = NamedFile::open(Path::new("api/").join(file)).ok();
    if let Some(mut file) = file {
        let mut buf = String::new();
        file.read_to_string(&mut buf).expect(&format!(
            "unable to read JSON API file at: {:?}",
            file.path()
        ));
        Some(serde_json::from_str(&buf).expect(&format!(
            "deserialization failed for API file at: {:?}",
            file.path()
        )))
    } else {
        None
    }
}

#[get("/<file..>", rank = 1)]
fn root_files(file: PathBuf, state: State<AppState>) -> Option<NamedFile> {
    let path = Path::new(&format!("{}", state.static_dir)).join(file);
    NamedFile::open(path).ok()
}

/// A simple tool to test frontend code with faked API requests
#[derive(StructOpt, Debug)]
#[structopt(name = "api-tester")]
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
}

struct AppState {
    response_delay: u64,
    static_dir: String,
}

fn main() {
    let opt = Opt::from_args();
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
        .mount("/", routes![index, api, root_files])
        .launch();
}
