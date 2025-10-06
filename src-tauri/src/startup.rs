use crate::app_config::read_config;
use crate::stream::need_enough::connect_need_enough_data;
use crate::stream::new_sample::connect_new_sample;
use crate::stream::pipeline::initialize_pipeline;
use crate::{app_config, python};
use anyhow::{Context, Result};
use glib::object::Cast;
use gst::prelude::{ElementExt, GstBinExt};
use gst::State;
use gst_app::{AppSink, AppSrc};
use std::sync::Arc;
use tauri::App;
use tokio::sync::broadcast;

pub fn startup(app: &App) -> Result<()> {
    tauri::async_runtime::block_on(async move {
        let app_handle = app.handle().clone();

        // configの初期化
        app_config::init_config(&app_handle)?;
        let config = read_config(&app_handle)?;
        let default_version = config.python.default_version;

        // pythonがインストールされているか確認
        // python環境変数の設定
        python::utils::add_python_path_env(&app_handle)?;
        let mut result = python::utils::check_python_installed(&app_handle).await?;
        let mut try_count = 0;
        while !result.installed && try_count < 3 {
            println!("Python is not installed. Installing...");
            let python_installed = python::utils::install_python(
                &app_handle,
                result.version.as_ref().unwrap_or(&default_version),
                result.version.is_none(),
            )
            .await;
            println!("Python installed: {:?}", python_installed);
            result = python::utils::check_python_installed(&app_handle).await?;
            try_count += 1;
        }

        println!("Installed python version: {:?}", result.version);

        println!("syncing packages...");
        let sync_result = python::utils::sync_packages(&app_handle).await;
        println!("Package sync result: {:?}", sync_result);

        // python環境の初期化
        let pl_manager = python::initialize::initialize_python(app_handle.clone())?;

        let (tx, _) = broadcast::channel::<Arc<Vec<u8>>>(1000000);
        let pipeline = initialize_pipeline(&tx).await?;

        // srcの設定
        let src = pipeline
            .by_name("src")
            .context("Source element not found in the pipeline")?;
        let appsrc = src
            .dynamic_cast::<AppSrc>()
            .expect("Source element is expected to be an appsrc!");

        // queueの設定
        // let queue = pipeline
        //     .by_name("q")
        //     .context("Queue element not found in the pipeline")?;

        // appsinkのシグナルハンドラを設定
        let sink = pipeline
            .by_name("ws_sink")
            .context("Sink element not found in the pipeline")?;
        let appsink = sink
            .dynamic_cast::<AppSink>()
            .expect("Sink element is expected to be an appsink!");

        connect_need_enough_data(appsrc, pl_manager);
        connect_new_sample(appsink, &tx);

        pipeline.set_state(State::Playing)?;

        Ok(())
    })
}
