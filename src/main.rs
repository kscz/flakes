extern crate crypto;
extern crate rand;
mod bencode;
mod torrent;
mod id;

use std::fs::File;
use std::io::prelude::*;
use std::process::exit;

use bencode::*;
use torrent::*;

fn print_benc(b: &Benc, pre: &String) -> () {
    match b {
        &Benc::S(ref s) => {
            match String::from_utf8(s.clone()) {
                Ok(s) => print!("\"{}\"", s),
                Err(_) => {
                    print!("<");
                    let mut it = s.iter();
                    let mut to_display = 16;
                    match it.next() {
                        Some(c) => {
                            print!("{:#x}", c);
                            to_display = to_display - 1;
                        },
                        None => ()
                    };
                    for c in it {
                        to_display = to_display - 1;
                        if to_display == 0 {
                            print!(", ...");
                            break;
                        } else {
                            print!(", {:#x}", c);
                        }
                    }
                    print!(">");
                }
            };
        },
        &Benc::I(ref i) => print!("{}", i),
        &Benc::L(ref l) => {
            print!("[", );
            let mut it = l.iter();
            match it.next() {
                Some(item) => print_benc(item, pre),
                None => ()
            }
            for item in it {
                print!(", ");
                print_benc(item, &pre);
            }
            print!("]");
        },
        &Benc::D(ref d) => {
            print!("{}", "{");
            let mut it = d.iter();
            match it.next() {
                Some((k, v)) => {
                    print!("\n{}\t(\"{}\" : ", pre, k);
                    print_benc(&v, &format!("{}\t", pre));
                    print!(")");
                },
                None => ()
            };
            for (k, v) in it {
                print!(",\n{}\t(\"{}\" : ", pre, k);
                print_benc(&v, &format!("{}\t\t", pre));
                print!(")");
            }
            print!("\n{}{}", pre, "}");
        }
    };
}

fn print_torrent_metadata(tm: &TorrentMetadata) -> () {
    println!("announce list: [");
    for (announce, i) in tm.announce_list.iter().zip((1..)) {
        print!("\ttier {}: [ ", i);
        let mut at_it = announce.iter();
        print!("{}", at_it.next().unwrap());
        for url in at_it {
            print!(", \"{}\"", url);
        }
        println!(" ], ");
    }
    println!("]");

    println!("base_path: \"{}\"", tm.base_path);

    println!("chunk size: {}", tm.chunk_size);

    println!("chunk checksums: [");
    for (checksum, i) in tm.chunk_checksum.iter().zip((0..16)) {
        if i == 15 {
            println!("\t...");
        } else {
            print!("\tChunk #{}: 0x", i);
            for b in checksum.iter() {
                print!("{:02x}", b);
            }
            println!(",");
        }
    }
    println!("]");

    println!("files: [");
    for file in tm.files.iter() {
        let mut fpiter = file.path.iter();
        print!("\t\"{}", fpiter.next().unwrap());
        for segment in fpiter {
            print!("/{}", segment);
        }
        println!("\" ({} bytes),", file.length);
    }
    println!("]");

    print!("info hash: 0x");
    for b in tm.info_hash.iter() {
        print!("{:02x}", b);
    }
    println!("");
}

fn main() {
    let mut f = match File::open("/path/to_your_torrent_file.torrent") {
        Ok(f) => f,
        Err(_) => {
            println!("Gasp! couldn't open a thing!");
            exit(-1);
        }
    };
    let mut buffer = Vec::new();

    match f.read_to_end(&mut buffer) {
        Ok(_) => (),
        Err(_) => {
            println!("Oh noes!");
            exit(-1);
        }
    }


    let torrent = match dec_benc_it(&mut buffer.iter()) {
        Ok(t) => t,
        Err(msg) => {
            println!("Unable to parse torrent file: {}", msg);
            exit(-1);
        }
    };

    //print_benc(&torrent, &String::new());
    //println!("");

    let fully_parsed = match benc_to_torrent(torrent) {
        Ok(x) => x,
        Err(s) => {
            println!("Failed to parse torrent file into metadata with err: {}", s);
            exit(-1);
        }
    };

    print_torrent_metadata(&fully_parsed);
}
