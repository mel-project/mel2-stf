use blake3::Hash;

pub struct Header {
    pub prev: Hash,
    pub height: u64,
    pub state: Hash,
}
