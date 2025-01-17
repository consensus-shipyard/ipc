use blake2b_simd::blake2b;
use frc42_hasher::hash::Hasher;

pub struct Blake2bHasher {}
impl Hasher for Blake2bHasher {
    fn hash(&self, bytes: &[u8]) -> Vec<u8> {
        blake2b(bytes).as_bytes().to_vec()
    }
}
