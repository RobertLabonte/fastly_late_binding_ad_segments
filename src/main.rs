/// A simple Fastly Compute application to demonstrate late binding ad segment
/// replacement. Please reach out to your Fastly representative if you want
/// to implement a solution like this demo.
///
/// Author: Robert Labonte (rlabonte@fastly.com)
use fastly::http::StatusCode;
use fastly::mime::Mime;
use fastly::{Error, KVStore, Request, Response};
use rand::Rng;
use std::path::Path;
use std::str::FromStr;
use uuid::Uuid;

// Define backend in your service or fastly.toml for local testing
const BACKEND: &str = "BACKEND";

// Define ad break marker
const AD_BREAK_MARKER: &str = "#--INSERT AD--";

// Define backend in your service or fastly.toml for local testing
const AD_BREAK_TEMPLATE: &str = "#EXT-X-DISCONTINUITY
#EXT-X-MAP:URI=\"/ads/{UUID}_init.mp4\"
#EXTINF:5.000000,
/ads/{UUID}0.m4s
#EXTINF:5.000000,
/ads/{UUID}1.m4s
#EXTINF:5.000000,
/ads/{UUID}2.m4s
#EXT-X-DISCONTINUITY";

// Three ads are avaiable in this demo
const AD_NAMES: [&str; 3] = ["BigBuck_Spikes", "BigBuck_Smash", "BigBuck_Arrow"];

// Define KV Store name
const KVSTORE: &str = "uuids";

fn main() -> Result<(), Error> {
    let mut req = Request::from_client();
    let path = req.get_path();

    // Fetch manifest
    if path.ends_with(".m3u8") {
        // Make backend request
        let mut be_resp = req.clone_without_body().send(BACKEND)?;
        let mut manifest = be_resp.take_body_str();

        // Generate UUID to use for filenames
        let uuid = format!("{}", Uuid::new_v4());
        let mut ad_break = AD_BREAK_TEMPLATE.replace("{UUID}", &uuid);

        // Get original map file to add back to playlist when main content returns
        let map_file = {
            let mut val = "";
            for line in manifest.lines() {
                if line.starts_with("#EXT-X-MAP") {
                    val = line;
                    break;
                }
                if line.starts_with("#EXTINF") {
                    // No map file found
                    break;
                }
            }
            val
        };
        if map_file != "" {
            ad_break.push_str(&format!("\n{}", map_file));
        }

        manifest = manifest.replace(AD_BREAK_MARKER, &ad_break);

        // Immediately return manifest with ad break inserted to the client with
        // HTTP Status Code 200
        Response::from_status(StatusCode::OK)
            .with_body(manifest)
            .with_content_type(Mime::from_str("application/vnd.apple.mpegurl")?)
            .send_to_client();

        // Make call to ad decision network to get ad to serve
        let idx = rand::thread_rng().gen_range(0..2);
        let ad_name = AD_NAMES[idx];

        // Use KV Store to link UUID to the ad_name
        let mut kv_store = KVStore::open(KVSTORE).unwrap().unwrap();
        kv_store.insert(&uuid, ad_name)?;

        // Done. We can exit.
        return Ok(());
    }

    // Serve ad segment by looking up the UUID in the path and replacing it
    // with the real link for the backend request. Backend request objects are
    // cached which allows for publishing an unlimited number of segments with
    // a UUIDs with no extra backend requests.
    if path.starts_with("/ads") {
        let filename = Path::new(path).file_name().unwrap().to_str().unwrap();

        // Get UUID from filename
        let uuid = &filename[..36];

        // Open KV Store to get ad_name using UUID as key
        let kv_store = KVStore::open(KVSTORE).unwrap().unwrap();

        // If UUID lookup fails we return 5XX, in production we should do
        // something better like return a slate or generic ad to prevent
        // playback from stopping if ad server is down.
        let ad_name = kv_store.lookup(uuid).unwrap().unwrap().into_string();

        // Change the path for the backend request from the UUID to the actual
        // media segment path
        println!("Replacing UUID: {} with Path: {}", uuid, ad_name);
        req.set_path(&path.replace(uuid, &ad_name));
    }

    // Make backend request and send the bytes to client
    req.send(BACKEND)?.send_to_client();

    Ok(())
}
