#![feature(proc_macro_hygiene, decl_macro)]
use rocket_contrib::json::Json;
use serde::Deserialize;
use std::process::Command;
use std::result::Result;

#[macro_use]
extern crate rocket;

#[derive(Deserialize)]
struct SocketState {
    is_on: bool,
}

#[derive(Deserialize)]
enum SocketState2 {
    On,
    Off,
}

fn pctl(
    socket: &String,
    command: &String,
) -> Result<String, rocket::response::status::NotFound<String>> {
    let result = Command::new("pctl").args(&[socket, command]).output();

    match result {
        Ok(output) => {
            if output.status.success() {
                Ok(format!("Success with status {}", output.status))
            } else {
                Err(rocket::response::status::NotFound(format!(
                    "Executed but failed with status: {}",
                    output.status
                )))
            }
        }
        Err(error) => Err(rocket::response::status::NotFound(format!(
            "pctl command failed: {}",
            error
        ))),
    }
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!\n"
}

#[post("/set/<socket>", format = "json", data = "<state>")]
fn set_socket(socket: String, state: Json<SocketState>) -> String {
    format!("Hello, set socket {} with state {}!\n", socket, state.is_on)
}

#[post("/socket/<socket>/on")]
fn socket_on(socket: String) -> Result<String, rocket::response::status::NotFound<String>> {
    pctl(&socket, &"on".to_string())
}

#[post("/socket/<socket>/off")]
fn socket_off(socket: String) -> Result<String, rocket::response::status::NotFound<String>> {
    pctl(&socket, &"off".to_string())
}

#[post("/socket/<socket>/toggle")]
fn socket_toggle(socket: String) -> Result<String, rocket::response::status::NotFound<String>> {
    pctl(&socket, &"toggle".to_string())
}

fn main() {
    rocket::ignite()
        .mount(
            "/",
            routes![index, set_socket, socket_on, socket_off, socket_toggle],
        )
        .launch();
}
