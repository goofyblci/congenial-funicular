use std::{error, sync::Arc};

use hyper::StatusCode;
use tokio::sync::Mutex;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug, Clone)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// counter
    pub counter: u8,
    /// tor response body
    pub tor_response_body: Vec<u8>,
    /// tor response status
    pub tor_response_status: StatusCode,
    pub tor_circuits_info: Arc<Mutex<Vec<CircuitInfo>>>,
}

#[derive(Debug)]
pub enum ChannelTypes {
    CircuitInformation(Vec<CircuitInfo>),
}

#[derive(Debug, Clone)]
pub struct CircuitInfo {
    pub ip_address: String,
    pub city: String,
    pub country: String,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            running: true,
            counter: 0,
            tor_response_body: Vec::new(),
            tor_response_status: StatusCode::IM_USED,
            tor_circuits_info: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn increment_counter(&mut self) {
        if let Some(res) = self.counter.checked_add(1) {
            self.counter = res;
        }
    }

    pub fn decrement_counter(&mut self) {
        if let Some(res) = self.counter.checked_sub(1) {
            self.counter = res;
        }
    }

    pub fn set_tor_response_body(&mut self, body: Vec<u8>) {
        self.tor_response_body = body;
    }

    pub fn set_tor_status_code(&mut self, status_code: StatusCode) {
        self.tor_response_status = status_code;
    }

    pub fn set_tor_circuit_info(&mut self, circuit_infos: Vec<CircuitInfo>) {
        let try_lock_res = self.tor_circuits_info.try_lock();
        if try_lock_res.is_ok() {
            *try_lock_res.unwrap() = circuit_infos;
        }
    }
}
