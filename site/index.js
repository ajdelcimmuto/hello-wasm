let mediaSource;
let sourceBuffer;
let video;

window.segment_info = segment_info;

// Import the WASM module and start
import { StreamingClient } from "hello-wasm";

function initializeMediaSource() {
    video = document.getElementById('videoPlayer');
    mediaSource = new MediaSource();
    video.src = URL.createObjectURL(mediaSource);

    mediaSource.addEventListener('sourceopen', () => {
        console.log("MediaSource opened for live stream");
        try {
            sourceBuffer = mediaSource.addSourceBuffer('video/mp4; codecs="avc1.4d401f,mp4a.40.2"');

            // Set mode to 'sequence' for live streams this is important. It ignores the timestamping and renders segments in order its appended.
            if (sourceBuffer.mode !== undefined) {
                sourceBuffer.mode = 'sequence';
            }
        } catch (error) {
            console.error("Error adding source buffer:", error);
        }
    });

    mediaSource.addEventListener('error', (e) => {
        console.error("MediaSource error:", e);
    });
}

function segment_info(segmentData) {
    console.log(`Received segment: ${segmentData.length} bytes`);
    sourceBuffer.appendBuffer(segmentData);
}

async function main() {
    const client = new StreamingClient("https://stream-fastly.castr.com/5b9352dbda7b8c769937e459/live_2361c920455111ea85db6911fe397b9e/index.fmp4.m3u8");

    try {
        const manifest = await client.fetch_manifest();
        console.log("Starting live stream segment fetching...");

    } catch (error) {
        console.error("Failed to fetch manifest:", error);
    }
}

initializeMediaSource();
main();
