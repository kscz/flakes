Forewarning, I have no idea what I'm doing, I'm learning as I go!

For some context: I've never written a bittorrent client, and I've never written anything of any note in rust... so yeah, it'll be a lot of messy code while I'm figuring everything out.

I also have really never done a GUI, so this will also be an adventure in making something go in terms of interface once I get there.  Very first pass will probably be a single torrent and use just spew an endless stream of "connected to client!" "requesting block #1234 from peer at 123.45.67.89 port 241 using UDP and no encryption..." "GOT ACK! OH MY GOD DATA!" etc etc

Things I've got working:
* Bencoding! I didn't realize until I started reading the spec how integral bencoding is to the whole protocol, so I tackled this first. I got it pulling in a .torrent file and it looks sensible, so now onto other things!
** It was kind of silly to write this part at all, given the fact that there's the rust-bencode crate, but I learned a lot
** I've read in a couple .torrent files and it looks like this is working

A plan of sorts:
* Next I want to connect to a tracker
* After that, connect to a peer
* I want a DHT implementation, at some point, but I think I'll get the more well-defined machinery up and running before I tackle the Kademlia stuff
* I'm planning to do everything in memory so I don't have to deal with the horror that is getting random file I/O correct while I'm trying to wire everything else up

