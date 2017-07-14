use rocket;
use rocket::State;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::Mutex;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::collections::HashMap;
use std::thread;

use Log;
use node::Node;

#[get("/")]
fn index() -> &'static str {
    "Log receiver is listening on POST /logs/<label>"
}

#[post("/logs/<label>", data = "<body>")]
fn logs(label: String, body: String, tx_out: State<Option<Mutex<Sender<Log>>>>) {
    // should use try_lock instead?
    if tx_out.is_some() {
        let _ = tx_out.as_ref().unwrap().lock().unwrap().send(Log::new(
            body,
            Some(
                label.to_string(),
            ),
        ));
    }
}

#[derive(Debug)]
pub struct HttpInputNode {
    rx: Arc<Mutex<Receiver<Log>>>,
    tx_inc: Sender<Log>,
    tx_out: Option<Sender<Log>>,
}

impl HttpInputNode {
    pub fn new(_config: Option<HashMap<String, String>>, next: Option<Sender<Log>>) -> Self {
        let (sender, receiver) = channel();

        Self {
            rx: Arc::new(Mutex::new(receiver)),
            tx_inc: sender,
            tx_out: next,
        }
    }
}

impl Node for HttpInputNode {
    fn start(&self) -> Result<Sender<Log>, String> {
        let tx = self.tx_out.clone().map(|t| Mutex::new(t));

        thread::spawn(||
            rocket::ignite()
                .manage(tx)
                .mount("/", routes![index, logs])
                .launch()
        );

        let mut log: Log = Log::new("lol".to_string(), None);
        passthrough!(self, log, {
            println!("HTTPIN: {:?}", log);
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_passes_received_logs_through() {
        test_passthrough!(HttpInputNode);
    }
}
