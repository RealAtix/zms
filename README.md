# zms

**zms** is a simple, anonymous end-to-end web chat application built using Rust.

## About

Welcome to **Zoomies** (zms)! It is a straightforward chat application created using TCP sockets. Once clients connect to a server's channel, they can communicate with each other in that specific channel. Furthermore, clients can choose to set a secret that will be utilized to encrypt the messages they send. To decrypt the messages received, all clients connected to the same channel must use the same secret.

Please note that the encryption feature hasn't been implemented yet, but it's on the roadmap. Additionally, there are plans to create a desktop app using Tauri that will complement the web-based application. Lastly, channels are automatically closed once the last connection is terminated.
