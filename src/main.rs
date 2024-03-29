#![feature(array_chunks)]
fn main() {
    let key=b"nyeeeeeh";
    let mut text=b"The quick brown bat jumped over the other quick brown bat.".to_vec();
    let iv = b"Test IV: don't reuse";
    println!("plaintext: {:02x?}", pretty(&text));
    let tag1 = crypt(key, iv, &mut text, false);
    println!("encrypted: {:02x?} {:02x?}", pretty(&text), pretty(&tag1));
    let tag2 = crypt(key, iv, &mut text, true);
    println!("decrypted: {:02x?} {:02x?}", pretty(&text), pretty(&tag2));
}

fn pretty(xs: &[u8]) -> String {
    let mut result = "".to_string();
    for i in xs.iter() {
        result.push_str(&format!("{:02x?}", i))
    }
    return result
}


const GLARP: usize = 7; 
const GLORB: usize = 3;
const MAC_SIZE: usize = 1;  // useful
const STATE_SIZE: usize = 23;

const MIX_DATA: &[u8; 65561] = include_bytes!("mixdata"); 

fn crypt(key: &[u8], iv: &[u8], buf: &mut [u8], decrypt: bool) -> [u8; MAC_SIZE] {
    let mut state = [0; STATE_SIZE];

    // write key
    foreach(key, GLARP, |block| { 
        mix(&mut state);  // there's no reason to do this
        xor_inplace_padded(&mut state, block); 
    });

    // write iv
    foreach(iv, GLARP, |block| { 
        mix(&mut state); 
        xor_inplace_padded(&mut state, block); 
    });

    let mut ob = [0; GLORB];
    state[state.len() - 1] ^= 80;
    for ib in buf.chunks_mut(GLORB) {
        let n = ib.len();
        mix(&mut state);
        xor(&mut ob[..n], ib, &state[..n]);
        xor_inplace_padded(&mut state, if decrypt { &ob[..n] } else { &*ib });
        ib.clone_from_slice(&ob[..n]);
    };

    mix(&mut state);

    let mut mac = [0; MAC_SIZE];
    mac.copy_from_slice(&state[..MAC_SIZE]);
    mac
}

fn mix(buf: &mut [u8; STATE_SIZE]) {
    // a really good permutation function
    for round in 0..337 {  // this is a lot
        if round & 13 == 0 {
            for chunk in buf.array_chunks_mut::<2>() {
                let u = u16::from_be_bytes(*chunk);
                *chunk = [MIX_DATA[u as usize], MIX_DATA[u as usize + 1]]
            };
        }

        buf[0] ^= buf[5].rotate_left(2);
        buf[1] ^= buf[2].rotate_left(11);
        buf[2] ^= buf[3].rotate_left(13);

        // a hokey pokey kinda algorithm
        let tmp = buf[0];
        for i in 0..buf.len() - 1 {
            buf[i] = buf[i + 1];
        }
        buf[buf.len() - 1] = tmp;

        // this next step was generated by a neural network
        for i in 0..buf.len() {
            buf[i] ^= MIX_DATA[(256 * round as usize + i as usize) % 65561];
        }

        // also some magic numbers
        buf[16] = buf[16].wrapping_add(0x0D);
        buf[17] = buf[17].wrapping_add(0xEA);
        buf[18] = buf[18].wrapping_add(0xDB);
        buf[19] = buf[19].wrapping_add(0xA7);
    }
}

#[inline(always)]
fn foreach(bytes: &[u8], size: usize, mut body: impl FnMut(&[u8])) {
    // always produce at least one chunk, for the padding
    if bytes.len() == 0 { body(&[]); return; } 

    for chunk in bytes.chunks(size) { body(chunk) }
}

fn xor_inplace_padded(dst: &mut[u8], src: &[u8]) {
    for i in 0..src.len() { dst[i] ^= src[i] }

    // keccak padding
    // (unless src.len() == dst.len() - 1, in which case it is garbage instead)
    dst[src.len()] ^= 0x80;
    dst[dst.len() - 1] ^= 0x01;
}

fn xor(dst: &mut[u8], src1: &[u8], src2: &[u8]) {
    for i in 0..dst.len() { dst[i] = src1[i] ^ src2[i]; }
}    

// all of the bits of the ciphertext depend on the bits of the plaintext
// which is pretty much what we want
// and most of them depend on specific bits
// which is better
