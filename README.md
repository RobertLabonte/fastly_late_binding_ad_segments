# Sample implementation of late binding ad segment

A simple Fastly Compute application to demonstrate late binding ad segment                  
replacement. Please reach out to your Fastly representative if you want                     
to implement a solution like this demo.\
\
This demo uses simple text parsing of manifests for clarity, we recommend\
more robust colutions like m3u8-rs for manifest parsing.\
\                                                                                           
Author: Robert Labonte (rlabonte@fastly.com)
\

## Setup
Download Fastly's SDK and follow instructions for Rust
\
Go to directory `streams` and run a local HTTP server\
\
```python3 -m http.server```
\
\

## Run Fastly Compute application locally:
\
```fastly compute serve```
\

Open Safari or Quicktime go to link: `http://127.0.0.1:7676/fastly.html`\
\
The movie 'Tears of Steel' will begin running. At 15 seconds an ad will appear for\
15 seconds of one of three randomly selected 'Big Buck Bunny' scenes.\
