# zms

_A simple, anonymous end-to-end web chat built using Rust_

## About

Zoomies (zms in short) is a simple chat application written using TCP sockets.  
Clients connect to a channel on a server which lets them talk to eachother in said channel.
The clients connected to a channel are able to set a secret which will be used to encrypt the sent messages.
All clients connecting to that specific channel must set the same secret to be able to decrypt the messages received.
A channel stops existing once the last connection closes.

Note that the encryption is not yet implemented. I also plan to build an accompanying desktop app built using Tauri.
