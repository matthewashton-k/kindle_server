#!/bin/sh
EXTENSION="/mnt/us/extensions/launch_server"
killall kindle_server
iptables -A INPUT -p tcp --dport 3000 -j ACCEPT
(${EXTENSION}/bin/kindle_server) </dev/null &>/dev/null &

IP=$(ifconfig wlan0 | grep -o "inet addr:[0-9.]*" | cut -d: -f2)
FILENAME="/mnt/us/documents/Listening_At_${IP}.txt"

ifconfig > "$FILENAME"
