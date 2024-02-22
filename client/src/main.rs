use client::app::{App, AppResult, ChannelTypes};
use client::event::{Event, EventHandler};
use client::handler::handle_key_events;
use client::transport;
use client::tui::Tui;
use dotenv::dotenv;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> AppResult<()> {
    dotenv().ok();
    // Create channel
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<ChannelTypes>(5);
    // Create an application.
    let mut app = App::new();
    let mut app_cloned_for_main = app.clone();
    let mut cloned_app = Arc::new(Mutex::new(app));

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;
    tokio::spawn(async move {
        while let Some(message) = receiver.recv().await {
            println!("Received: {:?}", message);
            let mut app = cloned_app.lock().await;
            match message {
                ChannelTypes::CircuitInformation(asd) => app.set_tor_circuit_info(asd),
                _ => (),
            }
        }
    });
    tokio::spawn(async move {
        let host = dotenv::var("ONION_ENDPOINT").expect("ONION_ENDPOINT_NOT_SET");
        let onion_connection = transport::OnionConnection::new(&host, sender).await;
        onion_connection.make_request().await.unwrap();
        onion_connection.make_websocket_connection().await.unwrap();
    });

    // Start the main loop.
    while app_cloned_for_main.running {
        // Render the user interface.
        tui.draw(&mut app_cloned_for_main)?;
        // Handle events.
        match tui.events.next().await? {
            Event::Tick => app_cloned_for_main.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app_cloned_for_main)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
