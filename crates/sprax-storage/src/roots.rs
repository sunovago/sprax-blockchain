use sprax_crypto::Hasher;
use sprax_types::Hash32;

pub fn compute_flat_root<'a>(pairs: impl Iterator<Item = (&'a [u8], &'a [u8])>) -> Hash32 {
    let mut combined_hashes = Vec::new();
    let mut any = false;
    for (k, v) in pairs {
        any = true;
        let mut kv_bytes = Vec::with_capacity(k.len() + v.len());
        kv_bytes.extend_from_slice(k);
        kv_bytes.extend_from_slice(v);
        let kv_hash = Hasher::blake3(&kv_bytes);
        combined_hashes.extend_from_slice(kv_hash.as_bytes());
    }
    if !any {
        return Hash32::ZERO;
    }
    Hasher::blake3(&combined_hashes)
}
