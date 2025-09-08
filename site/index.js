import { StreamingClient } from "hello-wasm";

// wasm.greet("WebAssembly with npm");
async function main() {

    // Create a new streaming client
    const client = new StreamingClient("https://stream-fastly.castr.com/5b9352dbda7b8c769937e459/live_2361c920455111ea85db6911fe397b9e/index.fmp4.m3u8");

    try {
        // Call your WASM method
        const manifest = await client.fetch_manifest();
        console.log("Manifest received:", manifest);

        // Parse and use the manifest
        processManifest(manifest);

    } catch (error) {
        console.error("Failed to fetch manifest:", error);
    }
}

function processManifest(manifestText) {
    // Do something with the manifest
    console.log("Processing manifest of length:", manifestText.length);
}

main();
