use std::collections::btree_map::BTreeMap;

use bencode::*;

pub struct torrent_file {
    pub path: Vec<String>,
    pub length: i64
}

pub struct torrent_metadata {
    pub announce_list: Vec<String>,
    pub file_metadata: BTreeMap<String, String>,
    pub base_path: String,
    pub chunk_size: i64,
    pub chunk_checksum: Vec<[u8; 20]>,
    pub files: Vec<torrent_file>
}

pub fn benc_to_torrent(input: Benc) -> Result<torrent_metadata, &'static str> {
    let d = match input {
        Benc::D(ref d) => d,
        _ => { return Err("Torrent files must have a dictionary type at the root!"); }
    };

    // Start by pulling out the info
    let info = try!(extract_info(d));
    
    // Fields which must exist
    let name = try!(extract_name(info));
    let chunk_size = try!(extract_chunk_size(info));
    let chunk_checksum = try!(extract_checksums(info));
    let announce = try!(extract_announce(d));

    // Fields which might exist
    let files = extract_files(info);
    let single_file_length = extract_single_file_length(info);
    let announce_list = extract_announce_list(d);

    // Resolve single-file vs multi-file ambiguity:
    let (files, base_path) = match (files, single_file_length) {
        (Err(_), Err(_)) => {
            return Err("Need a length or a files field! Cannot be missing both!");
        },
        (Err(_), Ok(length)) => {
            (vec![torrent_file {path: vec![name], length: length}], String::from("."))
        },
        (Ok(files), Err(_)) => {
            (files, name)
        },
        (Ok(_), Ok(_)) => {
            return Err("Cannot have both a 'length' field and a 'files' field defined!");
        }
    };

    // Validate that the number of checksums encompasses the correct amount of crap
    let total_size = files.iter().fold(0, |acc, x| acc + x.length);
    if (chunk_checksum.len() as i64 * chunk_size) < total_size {
        return Err("Not enough checksums for given size!");
    } else if ((chunk_checksum.len() as i64 - 1) * chunk_size) > total_size {
        return Err("Too many checksums for given size!");
    }

    // Resolve announce ambiguity
    let announce_list = match announce_list {
        Ok(x) => x,
        Err(_) => vec![announce]
    };

    // Everything should be all nice and unambiguous now! Return stuff!
    Ok(torrent_metadata {
        announce_list: announce_list,
        file_metadata: BTreeMap::new(), // TODO: Actually parse these fields
        base_path: base_path,
        chunk_size: chunk_size,
        chunk_checksum: chunk_checksum,
        files: files
    })
}

fn extract_announce_list(d: &BTreeMap<String, Benc>) -> Result<Vec<String>, &'static str> {
    Err("Announce list not yet implemented")
}

fn extract_single_file_length(info: &BTreeMap<String, Benc>) -> Result<i64, &'static str> {
    let length_benc = match info.get("length") {
        Some(length) => length,
        None => { return Err("No field with key 'length'"); }
    };

    match length_benc {
        &Benc::I(i) => {
            if i > 0 {
                Ok(i)
            } else {
                Err("Cannot have negative file length")
            }
        }
        _ => Err("Expected length to be an integer!")
    }
}

fn extract_files(info: &BTreeMap<String, Benc>) -> Result<Vec<torrent_file>, &'static str> {
    let files_benc = match info.get("files") {
        Some(files) => files,
        None => { return Err("No field with key 'files' in torrent file!"); }
    };

    let files = match files_benc {
        &Benc::L(ref files) => files,
        _ => { return Err("'files' was not a List!"); }
    };

    let mut out = Vec::new();

    for file in files {
        let file_dict = match file {
            &Benc::D(ref d) => d,
            _ => { return Err("Got non-dictionary file while processing 'files' key"); }
        };

        let mut path = Err("Expected path for file, did not get one!");
        let mut length = Err("Expected length for file, did not get one!");

        for (k, v) in file_dict.iter() {
            match k.as_str() {
                "path" => {
                    path = Ok(v);
                },
                "length" => {
                    length = Ok(v);
                },
                _ => { return Err("Got unexpected field while parsing files!"); }
            }
        }

        let path = match try!(path) {
            &Benc::L(ref path_benc) => try!(extract_path(path_benc)),
            _ => { return Err("Expected file path to be a list!"); }
        };

        let length = match try!(length) {
            &Benc::I(i) => {
                if i > 0 {
                    i
                } else {
                    return Err("Got negative length for file!");
                }
            },
            _ => { return Err("Expected file length to be an integer!") }
        };

        out.push(torrent_file {path: path, length: length});
    }

    Ok(out)
}

fn extract_path(path_benc: &Vec<Benc>) -> Result<Vec<String>, &'static str> {
    let mut path = Vec::with_capacity(path_benc.len());
    for path_segment in path_benc.iter() {
        match path_segment {
            &Benc::S(ref bs) => {
                match String::from_utf8(bs.clone()) {
                    Ok(s) => path.push(s),
                    Err(_) => { return Err("Unable to parse path segment as UTF8 string!"); }
                }
            },
            _ => { return Err("Got path segment which was not a string!"); }
        }
    }

    Ok(path)
}

fn extract_announce(d: &BTreeMap<String, Benc>) -> Result<String, &'static str> {
    let announce_benc = match d.get("announce") {
        Some(announce) => announce,
        None => { return Err("No field named 'announce' in torrent file!"); }
    };

    match announce_benc {
        &Benc::S(ref bs) => {
            match String::from_utf8(bs.clone()) {
                Ok(s) => Ok(s),
                Err(_) => Err("Unable to decode 'announce' as a UTF8 string!")
            }
        },
        _ => Err("Announce was not a bencoded string!")
    }
}

fn extract_checksums(info: &BTreeMap<String, Benc>) -> Result<Vec<[u8; 20]>, &'static str> {
    let checksums_benc = match info.get("pieces") {
        Some(checksums) => checksums,
        None => { return Err("No field named 'pieces' with checksums!"); }
    };

    let checksums = match checksums_benc {
        &Benc::S(ref byte_str) => byte_str,
        _ => { return Err("Field with key 'pieces' was not a string of bytes!"); }
    };

    if (checksums.len() % 20) != 0 {
        return Err("'pieces' must have a multiple of 20 bytes to be valid!");
    }

    let mut out = Vec::with_capacity(checksums.len() / 20);

    let mut it = checksums.iter().cloned().peekable();
    while it.peek().is_some() {
        let mut cur_checksum: [u8; 20] = [0; 20];
        for i in 0..20 {
            cur_checksum[i] = match it.next() {
                Some(x) => x,
                None => { return Err("Ran out of bytes while populating checksums!"); }
            }
        }

        out.push(cur_checksum);
    }

    Ok(out)
}

fn extract_chunk_size(info: &BTreeMap<String, Benc>) -> Result<i64, &'static str> {
    let chunk_size_benc = match info.get("piece length") {
        Some(chunk_size) => chunk_size,
        None => { return Err("No field with key 'piece length'"); }
    };

    match chunk_size_benc {
        &Benc::I(chunk_size) => Ok(chunk_size),
        _ => Err("Field with key 'piece length' is not an integer!")
    }
}

fn extract_name(info: &BTreeMap<String, Benc>) -> Result<String, &'static str> {
    let name_benc = match info.get("name") {
        Some(name) => name,
        None => { return Err("No field with the key 'name'!"); }
    };

    match name_benc {
        &Benc::S(ref bs) => {
            match String::from_utf8(bs.clone()) {
                Ok(s) => Ok(s),
                Err(_) => Err("Unable to decode 'name' as UTF8 string!")
            }
        },
        _ => Err("Field with key 'name' is not a string!")
    }
}

// This one returns a reference to save on overhead
fn extract_info(d: &BTreeMap<String, Benc>) -> Result<&BTreeMap<String, Benc>, &'static str> {
    let info_benc = match d.get("info") {
        Some(info) => info,
        None => { return Err("Dictionary missing info!"); }
    };

    match info_benc {
        &Benc::D(ref info) => Ok(info),
        _ => Err("Info is not a dictionary!")
    }
}

