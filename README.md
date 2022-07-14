# zms
*A simple, anonymous end-to-end web chat built using Rust*

## About
Zoomies (zms in short) is a simple chat application written using TCP sockets. Clients connect to a channel (chat room) on a server which lets them talk to eachother. The clients connected to a channel are able to set a secret which will be used to encrypt the sent messages. All clients connecting to that specific channel must set to be able to decrypt the messages received. A channel stops existing once the last connection closes.
