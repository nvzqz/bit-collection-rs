#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum CastleRight {
    WhiteKingside,
    BlackKingside,
    WhiteQueenside,
    BlackQueenside,
}

#[bit(CastleRight, mask = "0b1111")]
#[derive(BitCollection)]
struct CastleRights(u8);
