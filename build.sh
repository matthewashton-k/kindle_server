#!/bin/bash

USERNAME=$(whoami)
cross build --target=armv7-unknown-linux-musleabihf --release
cp target/armv7-unknown-linux-musleabihf/release/kindle_server launch_server/bin
cp -r launch_server /media/$USERNAME/Kindle/extensions/
