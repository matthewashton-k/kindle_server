#!/bin/sh
EXTENSION="/mnt/us/extensions/launch_server"
killall kindle_server
iptables -A INPUT -p tcp --dport 3000 -j ACCEPT
(${EXTENSION}/bin/kindle_server) </dev/null &>/dev/null &

FILENAME="/mnt/us/documents/Network_Info_$(date +'%Y%m%d_%H%M%S').txt"

ifconfig > "$FILENAME"
