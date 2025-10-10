use gst::prelude::ElementExtManual;
use gst::{Buffer, ClockTime};
use gst_app::AppSrc;
use numpy::PyReadonlyArray3;
use pyo3::prelude::PyAnyMethods;
use pyo3::types::{PyDict, PyList};
use pyo3::{Py, PyAny, PyErr, Python};
use std::sync::{Arc, Mutex};
use std::thread;

pub fn connect_need_enough_data(appsrc: AppSrc, pl_manager: Py<PyAny>) {
    // pl_managerをスレッド間で共有できるようにArcとMutexでラップ
    let pl_manager = Arc::new(Mutex::new(pl_manager));
    let framerate = 60; // 30fps
    let frame_duration = ClockTime::SECOND / framerate;
    let frame_count = Arc::new(Mutex::new(0u64));

    thread::spawn(move || {
        let mut count = frame_count.lock().unwrap();

        loop {
            // ここでピクセルデータからGStreamerのバッファを作成する
            // PythonのPluginManagerを使ってフレームデータを取得
            let buffers = Python::attach(|py| -> Result<Vec<Vec<u8>>, PyErr> {
                let pl_manager = pl_manager.lock().unwrap();
                let pl_manager = pl_manager.bind(py);

                let layer_struct = PyDict::new(py);
                layer_struct.set_item("x", 0)?;
                layer_struct.set_item("y", 0)?;
                layer_struct.set_item("channels", 3)?;
                layer_struct.set_item("obj_base", "TestObject")?;

                let obj_parameters = PyDict::new(py);
                layer_struct.set_item("obj_parameters", obj_parameters)?;

                let effects_list: Vec<i32> = vec![];
                let effects = PyList::new(py, effects_list)?;
                layer_struct.set_item("effects", effects)?;

                let frame_struct = PyList::new(py, vec![layer_struct])?;

                let make_frame_func = pl_manager.getattr("make_frames")?;
                let frame_data_arr: Vec<PyReadonlyArray3<u8>> = make_frame_func
                    .call1((*count, 10, frame_struct, 1920, 1080))?
                    .extract()?;

                let slice_data_arr = frame_data_arr
                    .iter()
                    .map(|x| x.as_slice())
                    .collect::<Result<Vec<&[u8]>, _>>()?;

                Ok(slice_data_arr.iter().map(|x| x.to_vec()).collect())
            })
            .expect("couldn't make frame buffer");

            // 取得したフレームデータからGStreamerのバッファを作成
            if buffers.is_empty() {
                eprintln!("No frame data received from Python");
                continue;
            }

            // bufferを1つづつappsrcにpushする
            for buffer in buffers {
                let mut buffer = Buffer::from_slice(buffer);
                {
                    let buffer_mut = buffer.get_mut().expect("Failed to get mutable buffer");
                    let pts = *count * frame_duration;

                    buffer_mut.set_pts(pts);
                    // DTS (Decoding Timestamp)も設定することが推奨される
                    buffer_mut.set_dts(pts);
                    buffer_mut.set_duration(frame_duration);
                }

                match appsrc.push_buffer(buffer) {
                    Ok(_) => {
                        // 成功
                        *count += 1;
                        if *count % 30 == 0 {
                            println!(
                                "Pushed {} frames, running: {}",
                                *count,
                                appsrc.current_running_time().unwrap_or(ClockTime::ZERO)
                                    / frame_duration
                            );
                        }
                    }
                    Err(gst::FlowError::Flushing) => {
                        // パイプラインがフラッシュ中の場合警告
                        println!("Pipeline is flushing!");
                    }
                    Err(e) => {
                        eprintln!("Error pushing buffer: {:?}", e);
                        break;
                    }
                }
            }
        }
    });
}
