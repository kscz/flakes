Forewarning, I have no idea what I'm doing, I'm learning as I go!

For some context: I've never written a bittorrent client, and I've never written anything of any note in rust... so yeah, it'll be a lot of messy code while I'm figuring everything out.

I also have really never done a GUI, so this will also be an adventure in making something go in terms of interface once I get there.  Very first pass will probably be a single torrent and use just spew an endless stream of "connected to client!" "requesting block #1234 from peer at 123.45.67.89 port 241 using UDP and no encryption..." "GOT ACK! OH MY GOD DATA!" etc etc

# Things I've got working:
* Bencoding! I didn't realize until I started reading the spec how integral bencoding is to the whole protocol, so I tackled this first. I got it pulling in a .torrent file and it looks sensible, so now onto other things!
  * It was kind of silly to write this part at all, given the fact that there's the rust-bencode crate, but I learned a lot
  * I've read in a couple .torrent files and it looks like this is working

## In Progress:
* Getting a tracker handler working
  * I'll start with the UDP connection stuff since I need it for peer communication anyway
  * I'll circle back to the "GET"/HTTP version

# A plan of sorts:
* After tracker, connecting to peers
* I want a DHT implementation, at some point, but I think I'll get the more well-defined machinery up and running before I tackle the Kademlia stuff
* I'm planning to do everything in memory so I don't have to deal with the horror that is getting random file I/O correct while I'm trying to wire everything else up

# Things to improve
Even though I'm just getting started, some stuff is already a mess!
* Error handling in the parsers is not terribly fun yet
  * Specific error types and a context would be nice

