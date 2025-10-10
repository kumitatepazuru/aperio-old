import {useEffect, useRef} from "react";

// const AV1_CODEC_STRING = "av01.0.08M.08"; // profile 0, level 8, Main, 8-bit
const H264_CODEC_STRING = "avc1.42E01E"; // H.264 High Profile level 4.0

// DataView から header を読む（1バイトflags + 8バイトLE i64）
function parseHeader(ab: ArrayBuffer) {
    const view = new DataView(ab);
    const flags = view.getUint8(0);
    const pts_ns = view.getBigInt64(1, /*littleEndian=*/ true); // BigInt
    return {flags, pts_ns};
}

// ns(BigInt) → us(number) へ安全に落とす（53bit越え回避のため先に割る）
function nsBigToUsNumber(nsBig: bigint): number {
    const usBig = nsBig / 1000n;
    // 極端に大きいと丸め誤差が出るが、ライブ用途なら十分
    return Number(usBig);
}

const App = () => {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const videoDecoderRef = useRef<VideoDecoder | null>(null);
    const ctxRef = useRef<CanvasRenderingContext2D | null>(null);
    const needKeyframeRef = useRef<boolean>(true); // config失敗等でkeyframe待ち状態

    async function handleFrame(frame: /* VideoFrame */ VideoFrame) {
        try {
            const canvas = canvasRef.current;
            let ctx = ctxRef.current;
            if (!ctx && canvas) {
                ctxRef.current = canvas.getContext("2d");
                ctx = ctxRef.current;
            }
            if (!ctx || !canvas) {
                frame.close();
                return;
            }
            // Canvasのサイズをソースに合わせて追従（最初のフレームだけ/または解像度変化時）
            if (
                canvas.width !== frame.displayWidth ||
                canvas.height !== frame.displayHeight
            ) {
                canvas.width = frame.displayWidth;
                canvas.height = frame.displayHeight;
                console.log(`Canvas resized: ${canvas.width}x${canvas.height}`);
            }

            // 高速経路: VideoFrame → ImageBitmap → drawImage（ほぼゼロコピー表示）
            const bmp = await createImageBitmap(frame);
            ctx.drawImage(bmp, 0, 0, canvas.width, canvas.height);
            bmp.close();
        } finally {
            frame.close();
        }
    }

    function ensureDecoder() {
        if (videoDecoderRef.current && videoDecoderRef.current.state !== "closed")
            return videoDecoderRef.current;
        console.log("(re)creating VideoDecoder");
        videoDecoderRef.current = new VideoDecoder({
            output: handleFrame,
            error: (e) => console.error("VideoDecoder error:", e),
        });
        needKeyframeRef.current = true; // 再作成したのでkeyframe待ち状態に
        return videoDecoderRef.current;
    }

    async function tryConfigure() {
        const cfg = {
            // codec: AV1_CODEC_STRING,
            codec: H264_CODEC_STRING,
        };
        if (!(await VideoDecoder.isConfigSupported(cfg))) {
            console.error("Config is not supported");
            return;
        }

        const dec = ensureDecoder();

        try {
            dec.configure(cfg);
            console.log("Decoder configured:", cfg.codec);
        } catch (e) {
            console.error("configure failed:", e);
        }
    }

    useEffect(() => {
        const ws = new WebSocket("ws://localhost:9002");
        ws.binaryType = "arraybuffer";

        ws.onopen = () => {
            console.log("WebSocket connected");
        };

        ws.onmessage = async (event) => {
            const ab = event.data;
            if (!(ab instanceof ArrayBuffer) || ab.byteLength < 9) {
                console.warn("Invalid message received");
                return;
            }

            const {flags, pts_ns} = parseHeader(ab);
            const isConfig = !!(flags & 0x02);
            const isKeyframe = !!(flags & 0x01);
            const payload = new Uint8Array(ab, 9); // header(9)を除いた部分

            if (isConfig && videoDecoderRef.current?.state !== "configured") {
                console.log("AV1 config received, size:", payload.length);

                // Decoderのconfigureを試みる
                await tryConfigure();
            }

            if (videoDecoderRef.current?.state !== "configured") {
                console.warn(
                    "Decoder not configured yet, dropping frame. status:",
                    videoDecoderRef.current?.state
                );
                return;
            }
            if (needKeyframeRef.current && !isKeyframe) {
                console.warn(
                    "Waiting for keyframe, dropping frame flag:",
                    needKeyframeRef.current,
                    flags
                );
                return;
            }

            const dec = ensureDecoder();

            // バックプレッシャ: decodeQueueSize が増えすぎたら古いフレームを捨てる
            if (dec.decodeQueueSize > 8 && !isKeyframe) {
                // 低遅延目的：デルタは間引いて最新に追いつく
                return;
            }

            const chunk = new EncodedVideoChunk({
                type: isKeyframe ? "key" : "delta",
                timestamp: nsBigToUsNumber(pts_ns), // WebCodecsはus
                data: payload,
                // duration: 省略（ライブなので不要）
            });

            try {
                videoDecoderRef.current?.decode(chunk); // ensureしているのでnullにはならない(はず)
                if (isKeyframe) {
                    console.log("Keyframe decoded, timestamp (us):", chunk.timestamp);
                    needKeyframeRef.current = false; // keyframe受信したので待ち状態解除
                }
            } catch (e) {
                // configミスマッチや一時的エラーの場合、次のkeyまで待機
                console.error("decode error:", e);
            }
        };
    }, []);

    return (
        <div>
            <canvas className="w-full h-full" ref={canvasRef}></canvas>
        </div>
    );
};

export default App;
