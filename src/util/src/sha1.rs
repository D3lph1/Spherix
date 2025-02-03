const SHA1_BYTE_SIZE: usize = 20;

type Sha1Bytes = [u8; SHA1_BYTE_SIZE];

/// Implementation source: https://gist.github.com/RoccoDev/8fa130f1946f89702f799f89b8469bc9?permalink_comment_id=4561673#gistcomment-4561673
pub fn notchian_digest(mut array: Sha1Bytes) -> String {
    let negative = (array[0] & 0x80) == 0x80;

    // Digest is 20 bytes, so 40 hex digits plus the minus sign if necessary.
    let mut hex = String::with_capacity(2 * SHA1_BYTE_SIZE + negative as usize);
    if negative {
        hex.push('-');

        // two's complement
        let mut carry = true;
        for b in array.iter_mut().rev() {
            (*b, carry) = (!*b).overflowing_add(carry as u8);
        }
    }
    hex.extend(
        array.into_iter()
            // extract hex digits
            .flat_map(|x| [x >> 4, x & 0xf])
            // skip leading zeroes
            .skip_while(|&x| x == 0)
            .map(|x| char::from_digit(x as u32, 16).expect("x is always valid base16")),
    );
    hex
}

#[cfg(test)]
mod tests {
    use sha1::digest::generic_array::GenericArray;
    use sha1::{Digest, Sha1};

    use crate::sha1::{notchian_digest, Sha1Bytes};

    #[test]
    fn test() {
        assert_eq!("4ed1f46bbe04bc756bcb17c0c7ce3e4632f06a48", notchian_digest(hash_from_str("Notch")));
        assert_eq!("-7c9d5b0044c130109a5d7b5fb5c317c02b4e28c1", notchian_digest(hash_from_str("jeb_")));
        assert_eq!("88e16a1019277b15d58faf0541e11910eb756f6", notchian_digest(hash_from_str("simon")));
    }

    fn hash_from_str(s: &str) -> Sha1Bytes {
        let mut hasher = Sha1::default();
        hasher.update(s);

        let mut alloc = Sha1Bytes::default();
        sha1::Digest::finalize_into(hasher, GenericArray::from_mut_slice(&mut alloc));

        alloc
    }
}
