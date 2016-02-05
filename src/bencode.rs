use std::collections::btree_map::BTreeMap;
use std::string::String;
use std::slice::Iter;
use std::char;

pub enum Benc {
    S(String),
    I(i64),
    L(Vec<Benc>),
    D(BTreeMap<String, Benc>)
}

pub fn dec_benc(s: String) -> Result<Benc, &'static str> {
    let mut iter = s.as_bytes().iter();

    match iter.next() {
        Some(c) => {
            if *c == 'i' as u8 {
                let i = dec_int(&mut iter);
                match iter.next() {
                    Some(_) => return Err("Could not consume whole expression!"),
                    None => return i
                }
            } else {
                return Err("Got unknown character!");
            }
        },
        None => return Err("Unable to decode empty string!")
    }
}

fn dec_int(iter: &mut Iter<u8>) -> Result<Benc, &'static str> {
    // The integer parser state idicate the set of acceptable characters
    enum IntParseState {
        MinusDigit,
        Term,
        NonZeroDigit,
        DigitTerm
    }
    let mut state = IntParseState::MinusDigit;
    let mut accum = String::new();

    while let Some(byte) = iter.next() {
        let c = try!(char::from_u32(*byte as u32).ok_or("Non-parseable character!"));
        state = match state {
            IntParseState::MinusDigit => {
                match c {
                    '-' => IntParseState::NonZeroDigit,
                    '0' => IntParseState::Term,
                    '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => IntParseState::DigitTerm,
                    _ => return Err("Expected digit or minus sign!")
                }
            },
            IntParseState::Term => {
                match c {
                    'e' =>  {
                        let parsed = accum.parse::<i64>();
                        match parsed {
                            Ok(i) => return Ok(Benc::I(i)),
                            Err(_) => return Err("Unable to parse given int!")
                        }
                    },
                    _ => return { return Err("Expected digit or minus sign!"); }
                }
            },
            IntParseState::NonZeroDigit => {
                match c {
                    '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => IntParseState::DigitTerm,
                    _ => return Err("Expected a non-zero digit!")
                }
            },
            IntParseState::DigitTerm => {
                match c {
                    '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => IntParseState::DigitTerm,
                    'e' => {
                        let parsed = accum.parse::<i64>();
                        match parsed {
                            Ok(i) => return Ok(Benc::I(i)),
                            Err(_) => return Err("Unable to parse given int!")
                        }
                    },
                    _ => return Err("Expected a digit or terminal!")
                }
            }
        };

        accum.push(c);
    }

    return Err("No terminal character while trying to parse int!");
}

pub fn enc_benc(b: Benc) -> String {
    match b {
        Benc::S(s) => enc_string(s),
        Benc::I(i) => enc_int(i),
        Benc::L(l) => enc_benc_list(l),
        Benc::D(d) => enc_benc_dict(d)
    }
}

fn enc_benc_dict(d: BTreeMap<String, Benc>) -> String {
    let mut benc = "d".to_string();
    for (k, v) in d {
        benc.push_str(&enc_string(k));
        benc.push_str(&enc_benc(v));
    }
    benc.push('e');
    return benc;
}

fn enc_benc_list(v: Vec<Benc>) -> String {
    let mut benc = "l".to_string();
    for b in v {
        benc.push_str(&enc_benc(b));
    }
    benc.push('e');
    return benc;
}

fn enc_string(s: String) -> String {
    let mut benc = s.len().to_string();
    benc.push(':');
    benc.push_str(&s);
    return benc;
}

fn enc_int(i: i64) -> String {
    let mut benc = "i".to_string();
    benc.push_str(&i.to_string());
    benc.push('e');

    return benc;
}

#[cfg(test)]
mod test {
    use std::collections::btree_map::BTreeMap;

    #[test]
    fn dec_int() {
        // Zero is important!
        let zero = super::dec_benc("i0e".to_string()).unwrap();
        match zero {
            super::Benc::I(i) => { assert_eq!(i, 0); },
            _ => unreachable!()
        }

        // A positive number
        let twelve = super::dec_benc("i12358e".to_string()).unwrap();
        match twelve {
            super::Benc::I(i) => { assert_eq!(i, 12358); },
            _ => unreachable!()
        }

        // A negative number
        let negative_thirty_seven = super::dec_benc("i-37e".to_string()).unwrap();
        match negative_thirty_seven {
            super::Benc::I(i) => { assert_eq!(i, -37); },
            _ => unreachable!()
        }

        // no terminal
        let error1 = super::dec_benc("i123".to_string());
        match error1 {
            Ok(_) => unreachable!(),
            Err(_) => ()
        }

        // Non-integer stuff
        let error2 = super::dec_benc("i123pants78e".to_string());
        match error2 {
            Ok(_) => unreachable!(),
            Err(_) => ()
        }

        // Doesn't fit in a i64
        let error3 = super::dec_benc(
                "i999999999999999999999999999999999999999999999999999999999999999\
                 9999999999999999999999999999999999999999999999999999999999999999\
                 9999999999999999999999999999999999999999999999999999999999999999\
                 9999999999999999999999999999999999999999999999999e".to_string());
        match error3 {
            Ok(_) => unreachable!(),
            Err(_) => ()
        }

        // No leading 0s
        let error4 = super::dec_benc("i01e".to_string());
        match error4 {
            Ok(_) => unreachable!(),
            Err(_) => ()
        }

        // No negative 0
        let error5 = super::dec_benc("i-0e".to_string());
        match error5 {
            Ok(_) => unreachable!(),
            Err(_) => ()
        }

        // Negative nothing?
        let error6 = super::dec_benc("i-e".to_string());
        match error6 {
            Ok(_) => unreachable!(),
            Err(_) => ()
        }
    }

    #[test]
    fn benc() {
        let i_benc = super::Benc::I(327);
        let benc_i = super::enc_benc(i_benc);

        assert_eq!(benc_i, "i327e");

        let str_benc = super::Benc::S("pqrs".to_string());
        let benc_str = super::enc_benc(str_benc);

        assert_eq!(benc_str, "4:pqrs");

        let vec_benc = super::Benc::L(vec![super::Benc::S("a".to_string()), super::Benc::S("bcd".to_string())]);
        let benc_vec = super::enc_benc(vec_benc);

        assert_eq!(benc_vec, "l1:a3:bcde");

        let mut map = BTreeMap::new();
        map.insert("peer".to_string(), super::Benc::S("123.45.67.89:6881".to_string()));
        map.insert("file_a".to_string(), super::Benc::L(vec![super::Benc::S("pic".to_string()), super::Benc::S("neato.jpg".to_string())]));
        let map_benc = super::Benc::D(map);
        let benc_map = super::enc_benc(map_benc);

        assert_eq!(benc_map, "d6:file_al3:pic9:neato.jpge4:peer17:123.45.67.89:6881e");
    }

    #[test]
    fn integer() {
        let ben_zero = super::enc_int(0);

        assert_eq!(ben_zero, "i0e");

        let ben_big = super::enc_int(12345678);

        assert_eq!(ben_big, "i12345678e");

        let ben_neg = super::enc_int(-1);

        assert_eq!(ben_neg, "i-1e");
    }

    #[test]
    fn string() {
        let short_string = "a".to_string();
        let ben_short = super::enc_string(short_string);

        assert_eq!(ben_short, "1:a");

        let med_string = "pqrs".to_string();
        let ben_med = super::enc_string(med_string);

        assert_eq!(ben_med, "4:pqrs");

        let long_string = "I am the very model of a modern major general.".to_string();
        let ben_long = super::enc_string(long_string);

        assert_eq!(ben_long, "46:I am the very model of a modern major general.");
    }
}
