use std::io::prelude::*;
use std::fs::File;
use bencode::*;

pub mod bencode;

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

fn main() {
    let mut f = match File::open("/path/to_your_torrent_file.torrent") {
        Ok(f) => f,
        Err(_) => {
            println!("Gasp! couldn't open a thing!");
            return ();
        }
    };
    let mut buffer = Vec::new();

    match f.read_to_end(&mut buffer) {
        Ok(_) => (),
        Err(_) => {
            println!("Oh noes!");
            return ();
        }
    }

    let torrent = match dec_benc_it(&mut buffer.iter()) {
        Ok(t) => t,
        Err(msg) => {
            println!("Oh noes: {}", msg);
            return ();
        }
    };

    print_benc(&torrent, &String::new());
    println!("");
}
