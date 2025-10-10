use std::sync::Arc;

use gst::{prelude::GstBinExt, Pipeline};
use tokio::net::TcpListener;

use crate::ws;
use anyhow::{Context, Result};
use gst::prelude::Cast;
use tokio::sync::broadcast::Sender;

pub async fn initialize_pipeline(tx: &Sender<Arc<Vec<u8>>>) -> Result<Pipeline> {
    gst::init().expect("Failed to initialize GStreamer");

    // ws serverの作成
    let addr = "127.0.0.1:9002";
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    // テストソースのパイプラインを作成
    let pipeline = gst::parse::launch(
        "appsrc name=src is-live=false block=true format=time !
        video/x-raw,format=BGR,width=1920,height=1080 !
        queue name=q max-size-buffers=1000 max-size-bytes=0 max-size-time=0 !
        videoconvert !
        video/x-raw,format=Y444 !
        x264enc pass=quant quantizer=0 key-int-max=60 byte-stream=true !
        h264parse name=h264parse config-interval=1 !
        video/x-h264,framerate=30/1,parsed=true,stream-format=byte-stream,alignment=au,parsed=true !
        appsink name=ws_sink emit-signals=true emit-signals=true sync=true",
    )?
    .dynamic_cast::<Pipeline>()
    .unwrap();

    // h264parseのpadを取得
    let parse = Arc::new(
        pipeline
            .by_name("h264parse")
            .context("h264parse element not found in the pipeline")?,
    );

    // ブロードキャストチャネルを作成
    let server_tx = tx.clone();
    tokio::spawn(async move {
        while let Ok((stream, addr)) = listener.accept().await {
            // 新しい接続ごとにSenderを購読し、新しいReceiverを作成
            let broadcast_rx = server_tx.subscribe();
            // 接続処理もさらに別のタスクとしてspawnする
            tokio::spawn(ws::handle_connection(
                broadcast_rx,
                stream,
                addr,
                Arc::clone(&parse),
            ));
        }
    });

    Ok(pipeline)
}
