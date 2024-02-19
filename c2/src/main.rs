#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hi?"
}

#[get("/echo")]
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
        .mount("/echo", routes![echo_stream])
}
