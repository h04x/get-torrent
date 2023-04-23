use crate::{
    piece::{BlockParam, Piece},
    BLOCK_SIZE,
};

#[test]
fn unfinished_blocks() {
    let p = Piece::new([0; 20], 0);
    let u = p.unfinished_blocks();
    assert_eq!(u.len(), 0);

    let p = Piece::new([0; 20], BLOCK_SIZE - 123);
    let u = p.unfinished_blocks();
    assert_eq!(u.len(), 1);
    let bp = u.get(0).unwrap();
    assert_eq!(bp.begin, 0);
    assert_eq!(bp.len, BLOCK_SIZE - 123);

    let p = Piece::new([0; 20], BLOCK_SIZE);
    let u = p.unfinished_blocks();
    assert_eq!(u.len(), 1);
    let bp = u.get(0).unwrap();
    assert_eq!(bp.begin, 0);
    assert_eq!(bp.len, BLOCK_SIZE);

    let p = Piece::new([0; 20], BLOCK_SIZE + 123);
    let u = p.unfinished_blocks();
    assert_eq!(u.len(), 2);
    let bp = u.get(0).unwrap();
    assert_eq!(bp.begin, 0);
    assert_eq!(bp.len, BLOCK_SIZE);
    let bp = u.get(1).unwrap();
    assert_eq!(bp.begin, BLOCK_SIZE);
    assert_eq!(bp.len, 123);

    let p = Piece::new([0; 20], BLOCK_SIZE * 5);
    let u = p.unfinished_blocks();
    assert_eq!(u.len(), 5);
    let bp = u.get(4).unwrap();
    assert_eq!(bp.begin, BLOCK_SIZE * 4);
    assert_eq!(bp.len, BLOCK_SIZE);
}
