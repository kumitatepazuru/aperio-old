use std::{net::SocketAddr, sync::Arc};

use futures_util::{SinkExt, StreamExt};
use gst::{
    prelude::{ClockExt, ElementExt, PadExtManual},
    Element,
};
use gstreamer_video::UpstreamForceKeyUnitEvent;
use tokio::{net::TcpStream, sync::broadcast};
use tokio_tungstenite::{accept_async, tungstenite::Message};

fn request_keyframe(enc: Arc<Element>, all_headers: bool) {
    let sink_pad = enc
        .static_pad("src")
        .expect("Failed to get src pad from av1parse");

    let running_time = enc
        .clock()
        .and_then(|c| Some(c.time()))
        .unwrap_or(gst::ClockTime::ZERO);
    let ev = UpstreamForceKeyUnitEvent::builder()
        .running_time(running_time)
        .all_headers(all_headers)
        .count(0) // 任意: 0で特に意味なし
        .build();
    sink_pad.send_event(ev);
}

pub async fn handle_connection(
    mut broadcast_rx: broadcast::Receiver<Arc<Vec<u8>>>,
    raw_stream: TcpStream,
    addr: SocketAddr,
    enc: Arc<Element>,
) {
    println!("Incoming TCP connection from: {}", addr);
    let ws_stream = accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    request_keyframe(enc, true);

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    loop {
        tokio::select! {
            // サーバー全体からのブロードキャストメッセージを受信
            Ok(msg_to_send) = broadcast_rx.recv() => {
                if ws_sender.send(Message::Binary(msg_to_send.to_vec().into())).await.is_err() {
                    println!("Error sending broadcast message to {}", addr);
                    break;
                }
            }
            // クライアントからのメッセージを受信
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(msg)) => {
                        println!("Received a message from {}: {}", addr, msg.to_text().unwrap_or("<binary which is not displayed>"));
                        // ここでは何もしない（あるいは他の処理）
                    }
                    Some(Err(e)) => {
                        println!("Error receiving message from {}: {}", addr, e);
                        break;
                    }
                    None => {
                        // 接続が閉じた
                        break;
                    }
                }
            }
        }
    }

    println!("{} disconnected", &addr);
}
