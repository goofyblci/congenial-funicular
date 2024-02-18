use std::error;

use hyper::StatusCode;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// counter
    pub counter: u8,
    /// tor response body
    pub tor_response_body: Vec<u8>,
    /// tor response status
    pub tor_response_status: StatusCode,
    pub tor_circuits_info: Vec<CircuitInfo>,
}

#[derive(Debug)]
pub struct CircuitInfo {
    pub ip_address: String,
    pub city: String,
    pub country: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            counter: 0,
            tor_response_body: Vec::new(),
            tor_response_status: StatusCode::IM_USED,
            tor_circuits_info: Vec::new(),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
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
        self.tor_circuits_info = circuit_infos;
    }
}
