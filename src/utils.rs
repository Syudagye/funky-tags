use sha2::{Sha256, Digest};

/// Returns a String hash of the given bytes
pub fn sha256_str(bytes: impl AsRef<[u8]>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
