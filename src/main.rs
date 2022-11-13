#![feature(proc_macro_hygiene, decl_macro)]
use phf::phf_map;
use rocket::response::status as HttpStatus;
use std::io::prelude::*;
use std::net::TcpStream;
use std::result::Result;

extern crate base64;

#[macro_use]
extern crate rocket;

static SOCKETS: phf::Map<&'static str, SocketType> = phf_map! {
    "light" => SocketType::Sispmctl{num: 2},
    "redlight" => SocketType::Sispmctl{num: 3},
    "tv" => SocketType::Sispmctl{num: 4},
    "wifi1" => SocketType::HS100{conn_string: "192.168.178.47:9999"},
    "wifi2" => SocketType::HS100{conn_string: "192.168.178.48:9999"},
};

fn parse_socket_str(socket: &str) -> Option<SocketType> {
    SOCKETS.get(socket).cloned()
}

#[derive(Clone)]
enum SocketType {
    Sispmctl { num: u8 },
    HS100 { conn_string: &'static str },
}

enum Command {
    On,
    Off,
    Toggle,
}

fn execute_by_str(
    socket_str: &str,
    command: &Command,
) -> Result<String, rocket::response::status::NotFound<String>> {
    let socket = parse_socket_str(socket_str);
    match socket {
        Some(socket) => execute(&socket, command),
        None => {
            /* TODO: emit better error */
            Err(rocket::response::status::NotFound(
                "Invalid socket".to_string(),
            ))
        }
    }
}

fn execute(
    socket: &SocketType,
    command: &Command,
) -> Result<String, rocket::response::status::NotFound<String>> {
    match socket {
        SocketType::Sispmctl { num } => execute_sispmctl(num, command),
        SocketType::HS100 { conn_string } => execute_hs100(conn_string, command),
    }
}

fn command_result_to_http(
    result: &std::io::Result<std::process::Output>,
) -> Result<String, rocket::response::status::NotFound<String>> {
    match result {
        Ok(output) => {
            if output.status.success() {
                Ok(format!("Success with status {}\n", output.status))
            } else {
                Err(HttpStatus::NotFound(format!(
                    "sispmctl failed with exit code {}\n",
                    output.status.code().unwrap_or(0)
                )))
            }
        }
        Err(error) => Err(HttpStatus::NotFound(format!(
            "Could not execute sispmctl: {}\n",
            error
        ))),
    }
}

fn execute_sispmctl(
    num: &u8,
    command: &Command,
) -> Result<String, rocket::response::status::NotFound<String>> {
    let sispmctl_arg = match command {
        Command::On => "-o",
        Command::Off => "-f",
        Command::Toggle => "-t",
    };

    let result = std::process::Command::new("sispmctl")
        .args([sispmctl_arg, &num.to_string()])
        .output();
    command_result_to_http(&result)
}

fn execute_hs100(
    conn_string: &str,
    command: &Command,
) -> Result<String, HttpStatus::NotFound<String>> {
    let hs100_command = match command {
        Command::On => "AAAAKtDygfiL/5r31e+UtsWg1Iv5nPCR6LfEsNGlwOLYo4HyhueT9tTu36Lfog==",
        Command::Off => "AAAAKtDygfiL/5r31e+UtsWg1Iv5nPCR6LfEsNGlwOLYo4HyhueT9tTu3qPeow==",
        Command::Toggle => {
            return Err(HttpStatus::NotFound(
                "Toggle not implemented for HS100".to_string(),
            ))
        }
    };

    // FIXME: I don't know how to easily return an Err from the functions whenever any of the TCP
    // calls fails. So I'm letting them panic instead, which rocket handles more or less
    // gracefully.
    let mut stream = TcpStream::connect(conn_string).unwrap();
    stream
        .write_all(base64::decode(hs100_command).unwrap().as_slice())
        .unwrap();
    stream.flush().unwrap();

    Ok("Success\n".to_string())
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!\n"
}

#[post("/socket/<socket>/on")]
fn socket_on(socket: String) -> Result<String, HttpStatus::NotFound<String>> {
    execute_by_str(&socket, &Command::On)
}

#[post("/socket/<socket>/off")]
fn socket_off(socket: String) -> Result<String, HttpStatus::NotFound<String>> {
    execute_by_str(&socket, &Command::Off)
}

#[post("/socket/<socket>/toggle")]
fn socket_toggle(socket: String) -> Result<String, HttpStatus::NotFound<String>> {
    execute_by_str(&socket, &Command::Toggle)
}

fn main() {
    let routes = routes![index, socket_on, socket_off, socket_toggle];
    rocket::ignite().mount("/", routes).launch();
}
