use futures::{SinkExt, StreamExt};
use futures::channel::mpsc::{self, UnboundedSender};
use log::{error, warn};
use reqwasm::websocket::{futures::WebSocket, Message};
use wasm_bindgen_futures::spawn_local;

pub struct Chat {
    tx: UnboundedSender<String>,
}

impl Chat {
    pub fn new<F>(callback: F) -> Self
        where F: Fn(String) + 'static
    {
        let ui_url = web_sys::window().map(|w| w.location()).unwrap();
        let chat_url = format!("ws://{}/chat", ui_url.host().unwrap());
        let ws = WebSocket::open(&chat_url).expect(&chat_url);

        let (mut ws_tx, mut ws_rx) = ws.split();

        spawn_local(async move {
            while let Some(msg) = ws_rx.next().await {
                match msg {
                    Ok(Message::Text(data)) => {
                        callback(data);
                    }
                    Ok(Message::Bytes(b)) => {
                        let decoded = std::str::from_utf8(&b);
                        match decoded {
                            Ok(val) => callback(val.into()),
                            Err(e) => warn!("ws: {:?}", e)
                        }
                    }
                    Err(e) => {
                        error!("ws: {:?}", e)
                    }
                }
            }
        });

        let (in_tx, mut in_rx) = mpsc::unbounded::<String>();
        spawn_local(async move {
            while let Some(text) = in_rx.next().await {
                let result = ws_tx.send(Message::Text(text)).await;
                if let Err(e) = result {
                    error!("error sending to socket: {:?}", e);
                }
            }
        });

        Self { tx: in_tx }
    }

    pub fn send(&mut self, text: String) {
        let result = self.tx.unbounded_send(text);
        if let Err(e) = result {
            error!("error sending to channel: {:?}", e);
        }
    }
}