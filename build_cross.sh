#!/bin/bash
cross build --target=armv7-unknown-linux-musleabihf --release

# Copy to Kindle
USERNAME=$(whoami)
cp target/armv7-unknown-linux-musleabihf/release/kindle_server launch_server/bin
cp -r launch_server "/media/${USERNAME}/Kindle/extensions/"

echo "Build and deployment complete!"
