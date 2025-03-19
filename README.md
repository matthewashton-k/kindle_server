### Kindle Server
Built with axum and meant to run on a very old kindle I have been hacking over spring break.

#### Features
* Websocket based terminal emulator.
* File Server
* Reverse Shell Launcher (I couldn't get ssh to work on kindle so this is useful)
* Information and Diagnostics Viewing
* Integration with [my Todo-App koreader plugin.](https://github.com/matthewashton-k/todo-koplugin)


#### Usage 
1. Plug in your kindle, and mount at /media/USERNAME/Kindle
2. Run ./build.sh
3. Open KUAL launcher and click the Run Server option, and it will start on 0.0.0.0:3000


#### NOTE
This project only works on jailbroken kindles that already have KUAL installed, and you will need the armv7-unknown-linux-musleabihf target toolchain installed

#### Screenshots
![term](webterm.png "Terminal Emulator")
![todos](todos.png "Todo Page")
![home](home.png "Home Page")
![file browser](file_browser.png "File Browser")
