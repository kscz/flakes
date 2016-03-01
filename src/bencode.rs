use std::collections::btree_map::BTreeMap;

pub enum Benc {
    S(Vec<u8>),
    I(i64),
    L(Vec<Benc>),
    D(BTreeMap<String, Benc>)
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
    use super::Benc;

    #[test]
    fn string() {
        let test_str_1 = "Hello I am a happy moose";
        assert_eq!(super::enc_string(test_str_1.as_bytes()), "24:Hello I am a happy moose".as_bytes());

        // Bleh, this test is a bit heavier than I wanted because getting a str into a Vec is difficult
        let test_str_2 = "Hello there happy moose";
        let mut test_vec_2 = Vec::with_capacity(test_str_2.len());
        for c in test_str_2.as_bytes().iter() {
            test_vec_2.push(*c);
        }
        let test_benc = Benc::S(test_vec_2);
        assert_eq!(super::enc_benc(&test_benc), "23:Hello there happy moose".as_bytes());

        // Test that something with invalid utf8 is still bencodable (0xfe and 0xff are invalid)
        let test_non_utf8_vec = vec!('a' as u8, 'b' as u8, 'c' as u8, 0xfe, 0xff, 'd' as u8);

        assert_eq!(
                super::enc_string(&test_non_utf8_vec),
                vec!('6' as u8, ':' as u8, 'a' as u8, 'b' as u8, 'c' as u8, 0xfe, 0xff, 'd' as u8)
            );
    }

    #[test]
    fn int() {
        let test_int_1 = 1234;
        assert_eq!(super::enc_int(&test_int_1), "i1234e".as_bytes());

        let test_int_2 = 0;
        assert_eq!(super::enc_int(&test_int_2), "i0e".as_bytes());

        let test_int_3 = -42;
        assert_eq!(super::enc_int(&test_int_3), "i-42e".as_bytes());

        let test_benc_1 = Benc::I(112358);
        assert_eq!(super::enc_benc(&test_benc_1), "i112358e".as_bytes());
    }

    #[test]
    fn list() {
        let test_list_ints = vec!(Benc::I(999), Benc::I(-5), Benc::I(0), Benc::I(8675309));
        assert_eq!(super::enc_list(&test_list_ints), "li999ei-5ei0ei8675309ee".as_bytes());

        let test_list_strings = vec!(
                Benc::S(vec!('h' as u8, 'a' as u8, 'p' as u8, 'p' as u8, 'y' as u8)),
                Benc::S(vec!('m' as u8, 'o' as u8, 'o' as u8, 's' as u8, 'e' as u8))
            );
        assert_eq!(super::enc_list(&test_list_strings), "l5:happy5:moosee".as_bytes());

        let test_list_mixed = vec!(Benc::I(0xdeadbeef), Benc::S(vec!('w' as u8, 'o' as u8, 'o' as u8, 't' as u8)));
        assert_eq!(super::enc_list(&test_list_mixed), "li3735928559e4:woote".as_bytes());
    }

    #[test]
    fn dict() {
        // Coming out sorted is a requirement, so insert these in a weird order
        let mut test_dict_1 = BTreeMap::new();
        test_dict_1.insert(String::from("number_3"), Benc::I(123456789));
        assert_eq!(super::enc_dict(&test_dict_1), "d8:number_3i123456789ee".as_bytes());
        test_dict_1.insert(String::from("number_1"), Benc::I(918273645));
        assert_eq!(super::enc_dict(&test_dict_1), "d8:number_1i918273645e8:number_3i123456789ee".as_bytes());
        test_dict_1.insert(String::from("number_2"), Benc::I(987654321));
        assert_eq!(super::enc_dict(&test_dict_1), "d8:number_1i918273645e8:number_2i987654321e8:number_3i123456789ee".as_bytes());

        // Test strings to strings
        let mut test_dict_2 = BTreeMap::new();
        let mut test_str_value_1 = Vec::new();
        for c in "0xdeadbeefabadbabecafefoodfee1dead".as_bytes().iter() {
            test_str_value_1.push(*c);
        }
        test_dict_2.insert(String::from("hash"), Benc::S(test_str_value_1));

        assert_eq!(super::enc_dict(&test_dict_2), "d4:hash34:0xdeadbeefabadbabecafefoodfee1deade".as_bytes());

        let mut test_str_value_2 = Vec::new();
        for c in "moose_dance.mkv".as_bytes().iter() {
            test_str_value_2.push(*c);
        }
        test_dict_2.insert(String::from("filename"), Benc::S(test_str_value_2));
        assert_eq!(super::enc_dict(&test_dict_2), "d8:filename15:moose_dance.mkv4:hash34:0xdeadbeefabadbabecafefoodfee1deade".as_bytes());

        // Make it a mixed map and see if everything still works
        test_dict_2.insert(String::from("part_count"), Benc::I(237));
        assert_eq!(super::enc_dict(&test_dict_2), "d8:filename15:moose_dance.mkv4:hash34:0xdeadbeefabadbabecafefoodfee1dead10:part_counti237ee".as_bytes());

        // Add in a list! ALL THE THINGS!
        test_dict_2.insert(String::from("other"), Benc::L(vec!(Benc::I(0xdeadbeef), Benc::S(vec!('w' as u8, 'o' as u8, 'o' as u8, 't' as u8)))));
        assert_eq!(super::enc_dict(&test_dict_2), "d8:filename15:moose_dance.mkv4:hash34:0xdeadbeefabadbabecafefoodfee1dead5:otherli3735928559e4:woote10:part_counti237ee".as_bytes());

        // Try it as a benc enum
        let benc_dict = Benc::D(test_dict_2);
        assert_eq!(super::enc_benc(&benc_dict), "d8:filename15:moose_dance.mkv4:hash34:0xdeadbeefabadbabecafefoodfee1dead5:otherli3735928559e4:woote10:part_counti237ee".as_bytes());
    }
}
