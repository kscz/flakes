use std::collections::btree_map::BTreeMap;

use crypto::sha1::Sha1;
use crypto::digest::Digest;

use bencode::*;

pub struct TorrentFile {
    pub path: Vec<String>,
    pub length: i64
}

pub struct TorrentMetadata {
    pub announce_list: Vec<Vec<String>>,
    pub base_path: String,
    pub chunk_size: i64,
    pub chunk_checksum: Vec<[u8; 20]>,
    pub files: Vec<TorrentFile>,
    pub info_hash: [u8; 20],
    pub creation_date: Option<i64>
}

pub fn benc_to_torrent(input: Benc) -> Result<TorrentMetadata, String> {
    let d = match input {
        Benc::D(ref d) => d,
        _ => { return Err(String::from("Torrent files must have a dictionary type at the root!")); }
    };

    // Start by pulling out the info
    let info = try!(extract_info(d));
    
    // Fields which must exist
    let name = try!(extract_name(info));
    let chunk_size = try!(extract_chunk_size(info));
    let chunk_checksum = try!(extract_checksums(info));
    let announce = try!(extract_announce(d));

    // Fields which might exist in the info dict
    let files = try!(extract_files(info));
    let single_file_length = try!(extract_single_file_length(info));
    // let private = ... "private" // TODO: implement me!
    // let md5sum = ... "md5sum" // TODO: implement me!

    // Fields which might exist in the torrent dict
    let announce_list = try!(extract_announce_list(d));
    let creation_date = try!(extract_creation_date(d));
    // let comment = ... "comment" // TODO: implement me!
    // let created_by = ... "created by" // TODO: implement me!
    // let encoding = ... "encoding" // TODO: implement me!

    // Resolve single-file vs multi-file ambiguity:
    let (files, base_path) = match (files, single_file_length) {
        (Some(_), Some(_)) => {
            return Err(String::from("Cannot have both a 'length' field and a 'files' field defined!"));
        },
        (Some(files), None) => {
            (files, name)
        },
        (None, Some(length)) => {
            (vec![TorrentFile {path: vec![name], length: length}], String::from("."))
        },
        (None, None) => {
            return Err(String::from("Need a length or a files field! Cannot be missing both!"));
        }
    };

    // Validate that the number of checksums encompasses the correct amount of crap
    let total_size = files.iter().fold(0, |acc, x| acc + x.length);
    if (chunk_checksum.len() as i64 * chunk_size) < total_size {
        return Err(format!("Got {} checksums but needed {}!", chunk_checksum.len(), (total_size / chunk_size) + 1));
    } else if ((chunk_checksum.len() as i64 - 1) * chunk_size) > total_size {
        return Err(format!("Got {} checksums, but only wanted {}", chunk_checksum.len(), (total_size / chunk_size) + 1));
    }

    // Resolve announce ambiguity
    let announce_list = announce_list.unwrap_or(vec![vec![announce]]);

    // Generate the info hash
    let mut sha1_hasher = Sha1::new();
    sha1_hasher.input(&enc_benc(d.get("info").unwrap()));
    let mut sha1_sum: [u8; 20] = [0; 20];
    sha1_hasher.result(&mut sha1_sum);

    // Everything should be all nice and unambiguous now! Return stuff!
    Ok(TorrentMetadata {
        announce_list: announce_list,
        base_path: base_path,
        chunk_size: chunk_size,
        chunk_checksum: chunk_checksum,
        files: files,
        info_hash: sha1_sum,
        creation_date: creation_date
    })
}

fn extract_creation_date(d: &BTreeMap<String, Benc>) -> Result<Option<i64>, String> {
    let creation_date_benc = match d.get("creation date") {
        Some(cd) => cd,
        None => { return Ok(None); }
    };

    match creation_date_benc {
        &Benc::I(cd) => Ok(Some(cd)),
        _ => Err(String::from("Value for key 'creation date' is not an integer!"))
    }

}

fn extract_announce_list(d: &BTreeMap<String, Benc>) -> Result<Option<Vec<Vec<String>>>, String> {
    let announce_list_benc = match d.get("announce-list") {
        Some(al) => al,
        None => { return Ok(None); }
    };

    let announce_list = match announce_list_benc {
        &Benc::L(ref al) => al,
        _ => { return Err(String::from("Value for key 'announce-list' is not a list!")); }
    };

    let mut out = Vec::with_capacity(announce_list.len());
    for announce_tier in announce_list.iter() {
        let announce_tier_v = match announce_tier {
            &Benc::L(ref atv) => atv,
            _ => { return Err(String::from("Announce tier was not a list!")); }
        };

        let mut tier_out = Vec::with_capacity(announce_tier_v.len());

        for url in announce_tier_v.iter() {
            match url {
                &Benc::S(ref bs) => {
                    match String::from_utf8(bs.clone()) {
                        Ok(s) => tier_out.push(s),
                        Err(e) => { return Err(format!("Unable to parse announce url as UTF8 string! Got err: {}", e)); }
                    }
                },
                _ => { return Err(String::from("Got announce url which was not a string!")); }
            }
        }

        if tier_out.len() == 0 {
            return Err(String::from("Cannot have an empty announce tier!"));
        }

        out.push(tier_out);
    }

    if out.len() > 0 {
        Ok(Some(out))
    } else {
        Err(String::from("Cannot have an empty announce list!"))
    }
}

fn extract_single_file_length(info: &BTreeMap<String, Benc>) -> Result<Option<i64>, String> {
    let length_benc = match info.get("length") {
        Some(length) => length,
        None => { return Ok(None); }
    };

    match length_benc {
        &Benc::I(i) => {
            if i > 0 {
                Ok(Some(i))
            } else {
                Err(format!("Got an invalid single-file length: {}", i))
            }
        }
        _ => Err(String::from("Expected length to be an integer!"))
    }
}

fn extract_files(info: &BTreeMap<String, Benc>) -> Result<Option<Vec<TorrentFile>>, String> {
    let files_benc = match info.get("files") {
        Some(files) => files,
        None => { return Ok(None); }
    };

    let files = match files_benc {
        &Benc::L(ref files) => files,
        _ => { return Err(String::from("'files' was not a List!")); }
    };

    let mut out = Vec::new();

    for file in files {
        let file_dict = match file {
            &Benc::D(ref d) => d,
            _ => { return Err(String::from("Got non-dictionary file while processing 'files' key")); }
        };

        let mut path = Err(String::from("Expected path for file, did not get one!"));
        let mut length = Err(String::from("Expected length for file, did not get one!"));

        for (k, v) in file_dict.iter() {
            match k.as_str() {
                "path" => {
                    path = Ok(v);
                },
                "length" => {
                    length = Ok(v);
                },
                "md5sum" => {
                    // FIXME: we sometimes get md5sums, we should propagate them up
                },
                _ => { return Err(format!("Got unexpected field \"{}\" while parsing files!", k)); }
            }
        }

        let path = match try!(path) {
            &Benc::L(ref path_benc) => try!(extract_path(path_benc)),
            _ => { return Err(String::from("Expected file path to be a list!")); }
        };

        let length = match try!(length) {
            &Benc::I(i) => {
                if i > 0 {
                    i
                } else {
                    return Err(format!("Got invalid length for file: {}", i));
                }
            },
            _ => { return Err(String::from("Expected file length to be an integer!")); }
        };

        out.push(TorrentFile {path: path, length: length});
    }

    Ok(Some(out))
}

fn extract_path(path_benc: &Vec<Benc>) -> Result<Vec<String>, String> {
    let mut path = Vec::with_capacity(path_benc.len());
    for path_segment in path_benc.iter() {
        match path_segment {
            &Benc::S(ref bs) => {
                match String::from_utf8(bs.clone()) {
                    Ok(s) => path.push(s),
                    Err(e) => { return Err(format!("Unable to parse path segment as UTF8 string! Got error: {}", e)); }
                }
            },
            _ => { return Err(String::from("Got path segment which was not a string!")); }
        }
    }

    Ok(path)
}

fn extract_announce(d: &BTreeMap<String, Benc>) -> Result<String, String> {
    let announce_benc = match d.get("announce") {
        Some(announce) => announce,
        None => { return Err(String::from("No field named 'announce' in torrent file!")); }
    };

    match announce_benc {
        &Benc::S(ref bs) => {
            match String::from_utf8(bs.clone()) {
                Ok(s) => Ok(s),
                Err(e) => Err(format!("Unable to decode 'announce' as a UTF8 string! Got error: {}", e))
            }
        },
        _ => Err(String::from("Announce was not a bencoded string!"))
    }
}

fn extract_checksums(info: &BTreeMap<String, Benc>) -> Result<Vec<[u8; 20]>, String> {
    let checksums_benc = match info.get("pieces") {
        Some(checksums) => checksums,
        None => { return Err(String::from("No field named 'pieces' with checksums!")); }
    };

    let checksums = match checksums_benc {
        &Benc::S(ref byte_str) => byte_str,
        _ => { return Err(String::from("Field with key 'pieces' was not a string of bytes!")); }
    };

    if (checksums.len() % 20) != 0 {
        return Err(format!("'pieces' must have a multiple of 20 bytes to be valid! Got {}", checksums.len()));
    }

    let mut out = Vec::with_capacity(checksums.len() / 20);

    let mut it = checksums.iter().cloned().peekable();
    while it.peek().is_some() {
        let mut cur_checksum: [u8; 20] = [0; 20];
        for i in 0..20 {
            cur_checksum[i] = match it.next() {
                Some(x) => x,
                None => { return Err(String::from("Ran out of bytes while populating checksums!")); }
            }
        }

        out.push(cur_checksum);
    }

    Ok(out)
}

fn extract_chunk_size(info: &BTreeMap<String, Benc>) -> Result<i64, String> {
    let chunk_size_benc = match info.get("piece length") {
        Some(chunk_size) => chunk_size,
        None => { return Err(String::from("No field with key 'piece length'")); }
    };

    match chunk_size_benc {
        &Benc::I(chunk_size) => Ok(chunk_size),
        _ => Err(String::from("Field with key 'piece length' is not an integer!"))
    }
}

fn extract_name(info: &BTreeMap<String, Benc>) -> Result<String, String> {
    let name_benc = match info.get("name") {
        Some(name) => name,
        None => { return Err(String::from("No field with the key 'name'!")); }
    };

    match name_benc {
        &Benc::S(ref bs) => {
            match String::from_utf8(bs.clone()) {
                Ok(s) => Ok(s),
                Err(e) => Err(format!("Unable to decode 'name' as UTF8 string! Got error: {}", e))
            }
        },
        _ => Err(String::from("Field with key 'name' is not a string!"))
    }
}

// This one returns a reference to save on overhead
fn extract_info(d: &BTreeMap<String, Benc>) -> Result<&BTreeMap<String, Benc>, String> {
    let info_benc = match d.get("info") {
        Some(info) => info,
        None => { return Err(String::from("Dictionary missing info!")); }
    };

    match info_benc {
        &Benc::D(ref info) => Ok(info),
        _ => Err(String::from("Info is not a dictionary!"))
    }
}

