use glib::object::ObjectExt;
use gst_app::AppSrc;
use std::sync::{Arc, Mutex};

pub fn connect_need_enough_data(appsrc: AppSrc) {
    // アイドルコールバックのソースIDを保持するための状態
    let source_id: Arc<Mutex<Option<glib::SourceId>>> = Arc::new(Mutex::new(None));

    // connect need dataでデータ供給を開始
    let appsrc_clone_for_need_data = appsrc.clone();
    let source_id_clone_for_need_data = source_id.clone();
    appsrc.connect("need-data", false, move |_args| {
        let mut source_id_guard = source_id_clone_for_need_data.lock().unwrap();

        // すでにソースが登録済み(idがある場合)の場合は何もしない
        if source_id_guard.is_some() {
            return Some(gst::FlowReturn::Ok.into());
        }

        println!("need-data: Starting data feed.");
        let appsrc_in_idle = appsrc_clone_for_need_data.clone();
        let mut frame_count = 0; // フレームカウンタ

        // アイドルコールバックをメインループに登録
        let id = glib::idle_add_local(move || {
            frame_count += 1;
            println!("Idle callback: Pushing frame {}", frame_count);

            // ここでピクセルデータからGStreamerのバッファを作成する
            // let buffer = create_pixel_data_buffer();

            // appsrcにバッファをプッシュ
            // if let Err(err) = appsrc_in_idle.push_buffer(buffer) {
            //     // フローがFlushingなど、予期されるエラーの場合は停止
            //     if err == gst::FlowError::Flushing {
            //         return glib::ControlFlow::Break;
            //     }
            // }

            // コールバックを継続
            glib::ControlFlow::Continue
        });
        // 登録したソースのIDを保存
        *source_id_guard = Some(id);

        Some(gst::FlowReturn::Ok.into())
    });

    // `enough-data` シグナル用のクロージャ
    let source_id_clone_for_enough_data = source_id.clone();
    appsrc.connect("enough_data", false, move |_appsrc| {
        let mut source_id_guard = source_id_clone_for_enough_data.lock().unwrap();
        // 登録されているアイドルコールバックを削除する
        if let Some(id) = source_id_guard.take() {
            println!("enough-data: Stopping data feed.");
            id.remove();
        }

        Some(gst::FlowReturn::Ok.into())
    });
}
