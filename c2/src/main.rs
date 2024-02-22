#[macro_use]
extern crate rocket;

use rocket::futures::{SinkExt, StreamExt};

#[get("/")]
fn index() -> &'static str {
    "Hi?"
}

#[get("/echo", rank = 1)]
fn echo_stream(ws: rocket_ws::WebSocket) -> rocket_ws::Stream!['static] {
    rocket_ws::Stream! { ws =>
        for await message in ws {
            yield message?;
        }
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![echo_stream])
}
