Forewarning, I have no idea what I'm doing, I'm learning as I go!

For some context: I've never written a bittorrent client, and I've never written anything of any note in rust... so yeah, it'll be a lot of messy code while I'm figuring everything out.

I also have really never done a GUI, so this will also be an adventure in making something go in terms of interface once I get there.  Very first pass will probably be a single torrent and use just spew an endless stream of "connected to client!" "requesting block #1234 from peer at 123.45.67.89 port 241 using UDP and no encryption..." "GOT ACK! OH MY GOD DATA!" etc etc

A plan of sorts:
* Get some connection stuff set up, probably just a toy program which opens up some ports and can poke it's head out into the world
* I want a DHT implementation, I might start here just because it's a smaller subset of the problem space, and doesn't require me to do anything but keep track of everyone else's stuff
* I'm planning to do everything in memory so I don't have to deal with the horror that is getting random file I/O correct while I'm trying to wire everything else up

