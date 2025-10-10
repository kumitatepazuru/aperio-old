use crate::stream::utils::build_ws_message_from_buffer;
use glib::object::ObjectExt;
use gst_app::AppSink;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;

pub fn connect_new_sample(appsink: AppSink, tx: &Sender<Arc<Vec<u8>>>) {
    // appsinkのnew-sampleシグナルに対するコールバックを設定
    let tx = tx.clone();
    appsink.connect("new-sample", false, move |args| {
        let sink = args[0]
            .get::<AppSink>()
            .expect("Failed to get appsink from args");

        // サンプルを取得
        return match sink.pull_sample() {
            Ok(sample) => {
                let buffer = sample.buffer().expect("Failed to get buffer from sample");
                // WebSocketで送信
                if let Some(msg) = build_ws_message_from_buffer(buffer) {
                    // 非同期WS送信タスクに渡す（そこで Message::Binary(msg) を send する）
                    let _ = tx.send(Arc::new(msg));
                }

                Some(gst::FlowReturn::Ok.into())
            }
            Err(_) => {
                eprintln!("Failed to pull sample from appsink");
                Some(gst::FlowReturn::Eos.into())
            }
        };
    });
}
