use std::hash::Hasher;

#[derive(Default)]
pub struct IdentityHasher(u64);

impl Hasher for IdentityHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        // MD5 is 16 bytes and uniformly distributed; take the first 8 as u64.
        self.0 = u64::from_ne_bytes(bytes[..8].try_into().unwrap());
    }
}
