#![feature(proc_macro_hygiene, decl_macro)]
use std::process::Command;
use std::result::Result;

#[macro_use]
extern crate rocket;


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
    let routes = routes![index, socket_on, socket_off, socket_toggle];
    rocket::ignite().mount("/", routes).launch();
}
