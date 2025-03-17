#!/bin/sh
EXTENSION="/mnt/us/extensions/launch_server"
killall kindle_server
killall index.html
iptables -A INPUT -p tcp --dport 3000 -j ACCEPT
(${EXTENSION}/bin/kindle_server) </dev/null &>/dev/null &
