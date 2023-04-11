pub struct Piece {
    pub hash: [u8; 20],
    pub complete: bool,
}

impl Piece {
    pub fn new(hash: [u8; 20]) -> Piece {
        Piece {
            hash,
            complete: false,
        }
    }
}
