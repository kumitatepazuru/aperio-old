use gst::BufferRef;

pub fn build_ws_message_from_buffer(buffer: &BufferRef) -> Option<Vec<u8>> {
    let flags = buffer.flags();
    let keyframe = !flags.contains(gst::BufferFlags::DELTA_UNIT);
    let is_config = flags.contains(gst::BufferFlags::HEADER);

    let pts_ns_i64: i64 = buffer
        .pts()
        .and_then(|t| Some(t.nseconds()))
        .map(|ns| ns as i64)
        .unwrap_or(-1);

    let map = buffer.map_readable().ok()?;
    let data = map.as_slice();

    let mut msg = Vec::with_capacity(1 + 8 + data.len());
    let mut f: u8 = 0;
    if keyframe {
        println!("Keyframe PTS (ns): {}", pts_ns_i64);
        f |= 0x01; // keyframeフラグを立てる
    }
    if is_config {
        f |= 0x02; // configフラグを立てる(keyframeと同時に立つこともある)
    }

    // configかkeyframeかのフラグ (1 byte)
    msg.push(f);
    // PTS (i64, 8 bytes, little-endian)
    msg.extend_from_slice(&pts_ns_i64.to_le_bytes());
    // データ本体
    msg.extend_from_slice(data);
    Some(msg)
}
