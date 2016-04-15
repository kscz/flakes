use rand::{Rng, thread_rng};

pub fn generate_id() -> Vec<u8> {
    let mut id = Vec::with_capacity(20);

    // TODO: Derive the version number from the crate version
    for c in "-FK0010-".as_bytes() {
        id.push(*c);
    }

    // kscz@2016/04/15 - we need to put a random number after the prefix
    // I have no idea if this needs to be consistent across runs
    // If it needs to be consistent across runs it might make sense
    // to incorporate the mac addr of the machine or something and
    // use a sha perhaps?
    let mut rng = thread_rng();
    for _ in 0..(20 - id.len()) {
        id.push(rng.gen::<u8>());
    }

    id
}

#[cfg(test)]
mod test {
    use super::generate_id;

    #[test]
    fn test_id() {
        let mut prev_ids = Vec::new();
        for _ in 0..64 {
            let id = generate_id();

            // Verify length
            assert_eq!(20, id.len());

            // Verify prefix
            for (expected, received) in "-FK0010-".as_bytes().iter().zip(id.iter()) {
                assert_eq!(expected, received);
            }

            // Check that we didn't collide
            for prev in prev_ids.iter() {
                assert!(id != *prev);
            }
            prev_ids.push(id);
        }
    }
}
