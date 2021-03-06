use std::collections::btree_map::BTreeMap;
use std::iter::Peekable;

pub enum Benc {
    S(Vec<u8>),
    I(i64),
    L(Vec<Benc>),
    D(BTreeMap<String, Benc>)
}

pub fn dec_benc(s: &Vec<u8>) -> Result<Benc, &'static str> {
    let mut it = s.iter();
    dec_benc_it(&mut it)
}

pub fn dec_benc_it<'a, T: Iterator<Item=&'a u8>>(it: &mut T) -> Result<Benc, &'static str> {
    let mut it = it.cloned().peekable();
    let out = try!(dec_benc_helper(&mut it));
    match it.next() {
        None => Ok(out),
        Some(_) => Err("Unable to consume whole string!")
    }
}

fn dec_benc_helper<T: Iterator<Item=u8>>(it: &mut Peekable<T>) -> Result<Benc, &'static str> {
    let next_char = match it.peek() {
            Some(c) => *c,
            None => return Err("Unable to decode empty string")
        };

    if next_char >= ('1' as u8) && next_char <= ('9' as u8) {
        dec_string(it)
    } else if next_char == 'i' as u8 {
        dec_int(it)
    } else if next_char == 'l' as u8 {
        dec_list(it)
    } else if next_char == 'd' as u8 {
        dec_dict(it)
    } else {
        Err("Invalid character received!")
    }
}

fn dec_dict<T: Iterator<Item=u8>>(it: &mut Peekable<T>) -> Result<Benc, &'static str> {
    enum DecState {
        ExpectStart,
        ExpectStringOrEnd,
    }
 
    let mut state = DecState::ExpectStart;
    let mut out = BTreeMap::new();

    loop {
        let next_char = match it.peek() {
            Some(c) => *c,
            None => return Err("Ran out of characters, failed to decode dict")
        };

        match state {
            DecState::ExpectStart => {
                if next_char == 'd' as u8 {
                    it.next();
                    state = DecState::ExpectStringOrEnd;
                } else {
                    return Err("Expected 'd' as start character, failed to decode dict");
                }
            },
            DecState::ExpectStringOrEnd => {
                if next_char == 'e' as u8 {
                    it.next(); // Don't forget to consume the e!
                    return Ok(Benc::D(out));
                } else {
                    let key = match try!(dec_benc_helper(it)) {
                        Benc::S(s) => s,
                        _ => return Err("Expected a String key, failed to decode dict")
                    };
                    let key = match String::from_utf8(key) {
                        Ok(s) => s,
                        Err(_) => return Err("Dict key must be valid utf8 string, failed to decode dict")
                    };
                    let value = try!(dec_benc_helper(it));

                    out.insert(key, value);
                }
            }
        }
    }
}

fn dec_list<T: Iterator<Item=u8>>(it: &mut Peekable<T>) -> Result<Benc, &'static str> {
    let mut out = Vec::new();

    match it.next() {
        Some(c) => {
            if c != 'l' as u8 {
                return Err("Expected list to start with a 'l', failed to decode list");
            }
        },
        None => {
            return Err("Cannot decode empty string as list");
        }
    }

    loop {
        let next_char = match it.peek() {
            Some(c) => *c,
            None => return Err("Did not find terminal, failed to decode list")
        };
        if next_char == 'e' as u8 {
            let _ = it.next();
            return Ok(Benc::L(out));
        } else {
            out.push(try!(dec_benc_helper(it)));
        }
    }
}

fn dec_int<T: Iterator<Item=u8>>(it: &mut Peekable<T>) -> Result<Benc, &'static str> {
    enum DecState {
        ExpectStart,
        ExpectNumOrHyphen,
        ExpectNonZeroNum,
        ExpectNumOrEnd,
        ExpectEnd
    }

    let mut state = DecState::ExpectStart;
    let mut buffer = String::new();

    for c in it {
        match state {
            DecState::ExpectStart => {
                if c == 'i' as u8 {
                    state = DecState::ExpectNumOrHyphen;
                } else {
                    return Err("Expected an 'i' to start integer decoding");
                }
            },
            DecState::ExpectNumOrHyphen => {
                buffer.push(c as char);

                if c == '0' as u8 {
                    state = DecState::ExpectEnd;
                } else if c >= '1' as u8 && c <= '9' as u8 {
                    state = DecState::ExpectNumOrEnd;
                } else if c == '-' as u8 {
                    state = DecState::ExpectNonZeroNum;
                } else {
                    return Err("Expected a hyphen or a number, failed to decode int");
                }
            },
            DecState::ExpectNonZeroNum => {
                buffer.push(c as char);

                if c >= '1' as u8 && c <= '9' as u8 {
                    state = DecState::ExpectNumOrEnd;
                } else {
                    return Err("Expected a non-zero number, failed to decode int");
                }
            },
            DecState::ExpectNumOrEnd => {
                if c >= '0' as u8 && c <= '9' as u8 {
                    buffer.push(c as char);
                } else if c == 'e' as u8 {
                    match buffer.parse::<i64>() {
                        Ok(i) => return Ok(Benc::I(i)),
                        Err(_) => return Err("Unable to parse integer, too large for i64?")
                    };
                } else {
                    return Err("Expected a number or 'e', failed to decode int");
                }
            },
            DecState::ExpectEnd => {
                if c == 'e' as u8 {
                    match buffer.parse::<i64>() {
                        Ok(i) => return Ok(Benc::I(i)),
                        Err(_) => return Err("Unable to parse integer, too large for i64?")
                    };
                } else {
                    return Err("Expected an 'e', failed to decode int");
                }
            }
        }
    }

    Err("Ran out of characters, failed to decode int")
}

fn dec_string<T: Iterator<Item=u8>>(it: &mut Peekable<T>) -> Result<Benc, &'static str> {
    enum DecState {
        ExpectNonZeroNum,
        ExpectNumOrColon,
        CountingDown
    }

    let mut state = DecState::ExpectNonZeroNum;
    let mut str_len = String::new();
    let mut bytes_remaining: i32 = 0;
    let mut out = Vec::new();

    for c in it {
        match state {
            DecState::ExpectNonZeroNum => {
                if c >= '1' as u8 && c <= '9' as u8 {
                    str_len.push(c as char);
                    state = DecState::ExpectNumOrColon;
                } else {
                    return Err("Needed a non-zero number");
                }
            },
            DecState::ExpectNumOrColon => {
                if c >= '0' as u8 && c <= '9' as u8 {
                    str_len.push(c as char);
                } else if c == ':' as u8 {
                    match str_len.parse::<i32>() {
                        Ok(i) => bytes_remaining = i,
                        Err(_) => return Err("Unable to parse string length!")
                    };
                    state = DecState::CountingDown;
                } else {
                    return Err("Needed a number or colon");
                }
            },
            DecState::CountingDown => {
                out.push(c);
                bytes_remaining = bytes_remaining - 1;
                if bytes_remaining == 0 {
                    return Ok(Benc::S(out));
                }
            }
        }
    }

    Err("Not enough characters in string")
}

pub fn enc_benc(b: &Benc) -> Vec<u8> {
    match b {
        &Benc::S(ref s) => enc_string(s),
        &Benc::I(ref i) => enc_int(i),
        &Benc::L(ref l) => enc_list(l),
        &Benc::D(ref d) => enc_dict(d),
    }
}

fn enc_string(s: &[u8]) -> Vec<u8> {
    let size_str = format!("{}:", s.len());
    let mut out = Vec::with_capacity(size_str.as_bytes().len() + s.len());

    for c in size_str.as_bytes().iter() {
        out.push(*c);
    }
    for c in s.iter() {
        out.push(*c);
    }

    out
}

fn enc_int(i: &i64) -> Vec<u8> {
    let i_as_str = format!("i{}e", i);
    let mut out = Vec::with_capacity(i_as_str.as_bytes().len());

    for c in i_as_str.as_bytes().iter() {
        out.push(*c);
    }

    out
}

fn enc_list(l: &Vec<Benc>) -> Vec<u8> {
    let mut out = Vec::new();

    out.push('l' as u8);
    for b in l {
        let cur = enc_benc(b);
        for c in cur {
            out.push(c);
        }
    }
    out.push('e' as u8);

    out
}

fn enc_dict(d: &BTreeMap<String, Benc>) -> Vec<u8> {
    let mut out = Vec::new();

    out.push('d' as u8);
    for (k, v) in d.iter() {
        let benc_string = enc_string(k.as_bytes());
        let benc_value = enc_benc(v);
        for b in benc_string.iter() {
            out.push(*b);
        }
        for b in benc_value.iter() {
            out.push(*b);
        }
    }
    out.push('e' as u8);

    out
}

#[cfg(test)]
mod test {
    use std::collections::btree_map::BTreeMap;
    use super::{Benc, dec_benc, enc_benc, enc_int, enc_dict, enc_list, enc_string};

    // Make our lives a bit easier by having a Benc comparator
    fn compare_benc(x: &Benc, y: &Benc) -> bool {
        match (x, y) {
            (&Benc::I(ref xi), &Benc::I(ref yi)) => *xi == *yi,
            (&Benc::S(ref xs), &Benc::S(ref ys)) => *xs == *ys,
            (&Benc::L(ref xl), &Benc::L(ref yl)) => {
                if xl.len() != yl.len() {
                    return false;
                }

                for (x_b, y_b) in xl.iter().zip(yl.iter()) {
                    if !compare_benc(x_b, y_b) {
                        return false;
                    }
                }
                true
            },
            (&Benc::D(ref xd), &Benc::D(ref yd)) => {
                if xd.len() != yd.len() {
                    return false;
                }

                for ((xk, xv), (yk, yv)) in xd.iter().zip(yd.iter()) {
                    if xk != yk || !compare_benc(xv, yv) {
                        return false;
                    }
                }
                true
            },
            _ => false
        }
    }

    #[test]
    fn round_trip_tests() {
        let test_1 = "d3:abci123e9:今日は23:It means good afternoone".as_bytes().to_vec();
        assert_eq!(enc_benc(&dec_benc(&test_1).unwrap()), test_1);

        let test_2 = "li3735928559e4:wootli999ei-5ei0ei8675309eee".as_bytes().to_vec();
        assert_eq!(enc_benc(&dec_benc(&test_2).unwrap()), test_2);

        let test_3 = Benc::L(vec!(Benc::I(0xdeadbeef), Benc::S("woot".as_bytes().to_vec())));
        assert!(compare_benc(&dec_benc(&enc_benc(&test_3)).unwrap(), &test_3));
    }

    #[test]
    fn dec_dict() {
        // Test with some utf8 stuff to make sure we handle it correctly
        let test_dict_1_enc = "d3:abci123e9:今日は23:It means good afternoone".as_bytes().to_vec();
        let mut test_dict_1_dec = BTreeMap::new();
        test_dict_1_dec.insert(String::from("abc"), Benc::I(123));
        test_dict_1_dec.insert(String::from("今日は"), Benc::S("It means good afternoon".as_bytes().to_vec()));

        assert!(compare_benc(&dec_benc(&test_dict_1_enc).unwrap(), &Benc::D(test_dict_1_dec)));

        // TODO: should probably be exhaustive and include a dict with a nested list and dict

        // Test an invalid utf8 string as a key (otherwise valid)
        let test_dict_2_enc = vec!('d' as u8, '4' as u8, ':' as u8, 'a' as u8, 0xfe, 0xff, 'd' as u8,
                'i' as u8, '4' as u8, '2' as u8, 'e' as u8, 'e' as u8);
        match dec_benc(&test_dict_2_enc) {
            Ok(_) => unreachable!(),
            Err(_) => ()
        }

        // no terminal e
        let test_dict_3_enc = "d3:abci123e".as_bytes().to_vec();
        match dec_benc(&test_dict_3_enc) {
            Ok(_) => unreachable!(),
            Err(_) => ()
        }

        // non-string key
        let test_dict_4_enc = "di123e3:abce".as_bytes().to_vec();
        match dec_benc(&test_dict_4_enc) {
            Ok(_) => unreachable!(),
            Err(_) => ()
        }
    }

    #[test]
    fn dec_list() {
        let test_list_ints_enc = "li999ei-5ei0ei8675309ee".as_bytes().to_vec();
        let test_list_ints_dec = Benc::L(vec!(Benc::I(999), Benc::I(-5), Benc::I(0), Benc::I(8675309)));
        assert!(compare_benc(&dec_benc(&test_list_ints_enc).unwrap(), &test_list_ints_dec));

        let test_list_strings_enc = "l5:happy5:moose3:abc7:shuttlee".as_bytes().to_vec();
        let test_list_strings_dec = Benc::L(vec!(Benc::S("happy".as_bytes().to_vec()), Benc::S("moose".as_bytes().to_vec()),
                Benc::S("abc".as_bytes().to_vec()), Benc::S("shuttle".as_bytes().to_vec())));
        assert!(compare_benc(&dec_benc(&test_list_strings_enc).unwrap(), &test_list_strings_dec));

        let test_list_mixed_enc = "li3735928559e4:wootli999ei-5ei0ei8675309eee".as_bytes().to_vec();
        let test_list_mixed_dec = Benc::L(vec!(Benc::I(0xdeadbeef), Benc::S("woot".as_bytes().to_vec()), test_list_ints_dec));
        assert!(compare_benc(&dec_benc(&test_list_mixed_enc).unwrap(), &test_list_mixed_dec));

        let test_unterminated_list = "li999ei-5ei0ei8675309e".as_bytes().to_vec();
        match dec_benc(&test_unterminated_list) {
            Ok(_) => unreachable!(),
            Err(_) => ()
        };

        let test_bad_string_list = "l999:this string is still too short!e".as_bytes().to_vec();
        match dec_benc(&test_bad_string_list) {
            Ok(_) => unreachable!(),
            Err(_) => ()
        };

        let test_bad_int_list = "li08ee".as_bytes().to_vec();
        match dec_benc(&test_bad_int_list) {
            Ok(_) => unreachable!(),
            Err(_) => ()
        };

        let test_bad_item_list = "li0eqe".as_bytes().to_vec();
        match dec_benc(&test_bad_item_list) {
            Ok(_) => unreachable!(),
            Err(_) => ()
        };

        let test_list_list = "lllllllllllllllllllllllllleeeee".as_bytes().to_vec();
        match dec_benc(&test_list_list) {
            Ok(_) => unreachable!(),
            Err(_) => ()
        };
    }

    #[test]
    fn dec_int() {
        let test_str_1 = "i0e";
        match dec_benc(&test_str_1.as_bytes().to_vec()).unwrap() {
            Benc::I(i) => assert_eq!(i, 0),
            _ => unreachable!()
        }

        let test_str_2 = "i42e";
        match dec_benc(&test_str_2.as_bytes().to_vec()).unwrap() {
            Benc::I(i) => assert_eq!(i, 42),
            _ => unreachable!()
        }

        let test_str_3 = "i-2e";
        match dec_benc(&test_str_3.as_bytes().to_vec()).unwrap() {
            Benc::I(i) => assert_eq!(i, -2),
            _ => unreachable!()
        }

        // Empty decode?
        let test_str_4 = "ie";
        match dec_benc(&test_str_4.as_bytes().to_vec()) {
            Err(_) => (),
            _ => unreachable!()
        }

        // Can't prefix with 0
        let test_str_5 = "i08e";
        match dec_benc(&test_str_5.as_bytes().to_vec()) {
            Err(_) => (),
            _ => unreachable!()
        }

        // negative 0 is not allowed
        let test_str_6 = "i-0e";
        match dec_benc(&test_str_6.as_bytes().to_vec()) {
            Err(_) => (),
            _ => unreachable!()
        }

        let test_str_7 = "i-e";
        match dec_benc(&test_str_7.as_bytes().to_vec()) {
            Err(_) => (),
            _ => unreachable!()
        }

        // greater than i64 max?
        let test_str_8 = "i9223372036854775808e";
        match dec_benc(&test_str_8.as_bytes().to_vec()) {
            Err(_) => (),
            _ => unreachable!()
        }

        // less than i64 min?
        let test_str_9 = "i-9223372036854775809e";
        match dec_benc(&test_str_9.as_bytes().to_vec()) {
            Err(_) => (),
            _ => unreachable!()
        }

        let test_str_10 = "i123abc567e";
        match dec_benc(&test_str_10.as_bytes().to_vec()) {
            Err(_) => (),
            _ => unreachable!()
        }
    }

    #[test]
    fn dec_string() {
        let test_str_1 = "18:Goodbye doomed yam";
        match dec_benc(&test_str_1.as_bytes().to_vec()).unwrap() {
            Benc::S(s) => assert_eq!(s, "Goodbye doomed yam".as_bytes().to_vec()),
            _ => unreachable!()
        };

        let test_str_2 = "1:This is too long";
        match dec_benc(&test_str_2.as_bytes().to_vec()) {
            Ok(_) => unreachable!(),
            Err(_) => ()
        };

        let test_str_3 = "999:This is too short";
        match dec_benc(&test_str_3.as_bytes().to_vec()) {
            Ok(_) => unreachable!(),
            Err(_) => ()
        };

        let test_str_4 = "0:This is impossible";
        match dec_benc(&test_str_4.as_bytes().to_vec()) {
            Ok(_) => unreachable!(),
            Err(_) => ()
        };

        let test_str_5 = "4294967297:This length doesn't fit in an i32 (2^32 + 1)";
        match dec_benc(&test_str_5.as_bytes().to_vec()) {
            Ok(_) => unreachable!(),
            Err(_) => ()
        };
    }

    #[test]
    fn string() {
        let test_str_1 = "Hello I am a happy moose";
        assert_eq!(enc_string(test_str_1.as_bytes()), "24:Hello I am a happy moose".as_bytes());

        let test_benc = Benc::S("Hello there happy moose".as_bytes().to_vec());
        assert_eq!(enc_benc(&test_benc), "23:Hello there happy moose".as_bytes());

        // Test that something with invalid utf8 is still bencodable (0xfe and 0xff are invalid)
        let test_non_utf8_vec = vec!('a' as u8, 'b' as u8, 'c' as u8, 0xfe, 0xff, 'd' as u8);

        assert_eq!(
                enc_string(&test_non_utf8_vec),
                vec!('6' as u8, ':' as u8, 'a' as u8, 'b' as u8, 'c' as u8, 0xfe, 0xff, 'd' as u8)
            );
    }

    #[test]
    fn int() {
        let test_int_1 = 1234;
        assert_eq!(enc_int(&test_int_1), "i1234e".as_bytes());

        let test_int_2 = 0;
        assert_eq!(enc_int(&test_int_2), "i0e".as_bytes());

        let test_int_3 = -42;
        assert_eq!(enc_int(&test_int_3), "i-42e".as_bytes());

        let test_benc_1 = Benc::I(112358);
        assert_eq!(enc_benc(&test_benc_1), "i112358e".as_bytes());
    }

    #[test]
    fn list() {
        let test_list_ints = vec!(Benc::I(999), Benc::I(-5), Benc::I(0), Benc::I(8675309));
        assert_eq!(enc_list(&test_list_ints), "li999ei-5ei0ei8675309ee".as_bytes());

        let test_list_strings = vec!(Benc::S("happy".as_bytes().to_vec()), Benc::S("moose".as_bytes().to_vec()));
        assert_eq!(enc_list(&test_list_strings), "l5:happy5:moosee".as_bytes());

        let test_list_mixed = vec!(Benc::I(0xdeadbeef), Benc::S("woot".as_bytes().to_vec()));
        assert_eq!(enc_list(&test_list_mixed), "li3735928559e4:woote".as_bytes());
    }

    #[test]
    fn dict() {
        // Coming out sorted is a requirement, so insert these in a weird order
        let mut test_dict_1 = BTreeMap::new();
        test_dict_1.insert(String::from("number_3"), Benc::I(123456789));
        assert_eq!(enc_dict(&test_dict_1), "d8:number_3i123456789ee".as_bytes());
        test_dict_1.insert(String::from("number_1"), Benc::I(918273645));
        assert_eq!(enc_dict(&test_dict_1), "d8:number_1i918273645e8:number_3i123456789ee".as_bytes());
        test_dict_1.insert(String::from("number_2"), Benc::I(987654321));
        assert_eq!(enc_dict(&test_dict_1), "d8:number_1i918273645e8:number_2i987654321e8:number_3i123456789ee".as_bytes());

        // Test strings to strings
        let mut test_dict_2 = BTreeMap::new();
        test_dict_2.insert(String::from("hash"), Benc::S("0xdeadbeefabadbabecafefoodfee1dead".as_bytes().to_vec()));

        assert_eq!(enc_dict(&test_dict_2), "d4:hash34:0xdeadbeefabadbabecafefoodfee1deade".as_bytes());

        test_dict_2.insert(String::from("filename"), Benc::S("moose_dance.mkv".as_bytes().to_vec()));
        assert_eq!(enc_dict(&test_dict_2), "d8:filename15:moose_dance.mkv4:hash34:0xdeadbeefabadbabecafefoodfee1deade".as_bytes());

        // Make it a mixed map and see if everything still works
        test_dict_2.insert(String::from("part_count"), Benc::I(237));
        assert_eq!(enc_dict(&test_dict_2), "d8:filename15:moose_dance.mkv4:hash34:0xdeadbeefabadbabecafefoodfee1dead10:part_counti237ee".as_bytes());

        // Add in a list! ALL THE THINGS!
        test_dict_2.insert(String::from("other"), Benc::L(vec!(Benc::I(0xdeadbeef), Benc::S("toothless".as_bytes().to_vec()))));
        assert_eq!(enc_dict(&test_dict_2), "d8:filename15:moose_dance.mkv4:hash34:0xdeadbeefabadbabecafefoodfee1dead5:otherli3735928559e9:toothlesse10:part_counti237ee".as_bytes());

        // Try it as a benc enum
        let benc_dict = Benc::D(test_dict_2);
        assert_eq!(enc_benc(&benc_dict), "d8:filename15:moose_dance.mkv4:hash34:0xdeadbeefabadbabecafefoodfee1dead5:otherli3735928559e9:toothlesse10:part_counti237ee".as_bytes());
    }
}
