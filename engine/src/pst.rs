//! A module containing the information for Piece-Square Tables (PSTs). A PST
//! is given for both the early and endgame.

use fiddler_base::{
    movegen::NominateMove, Board, Color, Eval, Move, Piece, Position, Score, Square,
};

use crate::evaluate::blend_eval;

/// A lookup table for piece values. The outer index is the type of the piece
/// (in order of Pawn, Knight, Bishop, Rook, Queen, and King)
/// and the inner index is the square of the piece (from White's point of view)
/// , starting with A1 as the first index, then continuing on to B1, C1, and so
/// on until H8 as index 63.
type Pst = [[Eval; 64]; Piece::NUM_TYPES];

/// A PST which is given in millipawns.
type CentiPst = [[i16; 64]; Piece::NUM_TYPES];

/// Evaluate a board based on its PST value. This is slow, so under most
/// conditions it is recommended to use `pst_delta()` instead if you are making
/// moves. The first value in the return type is the midgame difference, and
/// the second is the endgame difference.
pub fn pst_evaluate(board: &Board) -> Score {
    let mut mg_eval = Eval::DRAW;
    let mut eg_eval = Eval::DRAW;

    for pt in Piece::ALL_TYPES {
        for sq in board[pt] & board[Color::White] {
            mg_eval += MIDGAME_VALUE[pt as usize][sq as usize];
            eg_eval += ENDGAME_VALUE[pt as usize][sq as usize];
        }
        for sq in board[pt] & board[Color::Black] {
            //Invert the square that Black is on, since positional values are
            //flipped (as pawns move the other way, etc)
            let alt_sq = sq.opposite();

            mg_eval -= MIDGAME_VALUE[pt as usize][alt_sq as usize];
            eg_eval -= ENDGAME_VALUE[pt as usize][alt_sq as usize];
        }
    }

    (mg_eval, eg_eval)
}

/// Get the difference in PST value which would be generated by making the move
/// `m` on `board`. The first value in the return tuple is the midgame
/// difference, and the second is the endgame difference. `pst_delta` will
/// reflect how the position improves for the player making the move,
/// independed of if the player is white or black.
///
/// # Panics
///
/// `pst_delta` will panic if the given move is invalid.
pub fn pst_delta(board: &Board, m: Move) -> Score {
    let from_sq = m.from_square();
    let to_sq = m.to_square();
    let mover_type = board.type_at_square(m.from_square()).unwrap();
    let mover_idx = mover_type as usize;
    let end_type = match m.promote_type() {
        Some(pt) => pt,
        None => mover_type,
    };
    let end_idx = end_type as usize;
    let (from_alt, to_alt) = match board.player_to_move {
        Color::White => (from_sq, to_sq),
        Color::Black => (from_sq.opposite(), to_sq.opposite()),
    };
    let (from_idx, to_idx) = (from_alt as usize, to_alt as usize);

    // you always lose the value of the square you moved from
    let mut mg_delta = MIDGAME_VALUE[end_idx][to_idx] - MIDGAME_VALUE[mover_idx][from_idx];
    let mut eg_delta =
        ENDGAME_VALUE[end_idx][to_idx] - ENDGAME_VALUE[mover_type as usize][from_idx];

    if board[!board.player_to_move].contains(m.to_square()) {
        // conventional capture
        let to_opposite_idx = to_alt.opposite() as usize;
        let capturee_idx = board.type_at_square(to_sq).unwrap() as usize;
        mg_delta += MIDGAME_VALUE[capturee_idx][to_opposite_idx];
        eg_delta += ENDGAME_VALUE[capturee_idx][to_opposite_idx];
    }

    if m.is_en_passant() {
        let to_opposite_idx = (to_alt - Color::White.pawn_direction()).opposite() as usize;

        mg_delta += MIDGAME_VALUE[Piece::Pawn as usize][to_opposite_idx];
        eg_delta += ENDGAME_VALUE[Piece::Pawn as usize][to_opposite_idx];
    }

    if m.is_castle() {
        let is_queen_castle = to_sq.file() == 2;
        let (rook_from_idx, rook_to_idx) = match is_queen_castle {
            true => (Square::A1 as usize, Square::D1 as usize),
            false => (Square::H1 as usize, Square::F1 as usize),
        };

        mg_delta += MIDGAME_VALUE[Piece::Rook as usize][rook_to_idx]
            - MIDGAME_VALUE[Piece::Rook as usize][rook_from_idx];
        eg_delta += ENDGAME_VALUE[Piece::Rook as usize][rook_to_idx]
            - ENDGAME_VALUE[Piece::Rook as usize][rook_from_idx];
    }

    (mg_delta, eg_delta)
}

pub struct PstNominate {}

impl NominateMove for PstNominate {
    type Output = (Score, Eval);

    #[inline(always)]
    fn score(m: Move, pos: &Position) -> Self::Output {
        let delta = pst_delta(&pos.board, m);
        (delta, blend_eval(&pos.board, delta.0, delta.1))
    }
}

/// A function used for ergonomics to convert from a table of millipawn values
/// to a table of `Eval`s.
const fn expand_table(centi_table: CentiPst) -> Pst {
    let mut table = [[Eval::DRAW; 64]; Piece::NUM_TYPES];
    let mut piece_idx = 0;
    // I would use for-loops here, but those are unsupported in const fns.
    while piece_idx < Piece::NUM_TYPES {
        let mut sq_idx = 0;
        while sq_idx < 64 {
            table[piece_idx][sq_idx] = Eval::centipawns(centi_table[piece_idx][sq_idx]);
            sq_idx += 1;
        }
        piece_idx += 1;
    }
    table
}

/* For now, we use the values from PeSTO. */

/// A PST for the value of pawns in the middlegame.
const MIDGAME_VALUE: Pst = expand_table([
    [
        // knights
        -167, -89, -34, -49, 61, -97, -15, -107, // rank 1
        -73, -41, 72, 36, 23, 62, 7, -17, // rank 2
        -47, 60, 37, 65, 84, 129, 73, 44, // rank 3
        -9, 17, 19, 53, 37, 69, 18, 22, // rank 4
        -13, 4, 16, 13, 28, 19, 21, -8, // rank 5
        -23, -9, 12, 10, 19, 17, 25, -16, // rank 6
        -29, -53, -12, -3, -1, 18, -14, -19, // rank 7
        -105, -21, -58, -33, -17, -28, -19, -23, // rank 8
    ],
    [
        // bishops
        -29, 4, -82, -37, -25, -42, 7, -8, // rank 1
        -26, 16, -18, -13, 30, 59, 18, -47, // rank 2
        -16, 37, 43, 40, 35, 50, 37, -2, // rank 3
        -4, 5, 19, 50, 37, 37, 7, -2, // rank 4
        -6, 13, 13, 26, 34, 12, 10, 4, // rank 5
        0, 15, 15, 15, 14, 27, 18, 10, // rank 6
        4, 15, 16, 0, 7, 21, 33, 1, // rank 7
        -33, -3, -14, -21, -13, -12, -39, -21, // rank 8
    ],
    [
        // rooks
        32, 42, 32, 51, 63, 9, 31, 43, // rank 1
        27, 32, 58, 62, 80, 67, 26, 44, // rank 2
        -5, 19, 26, 36, 17, 45, 61, 16, // rank 3
        -24, -11, 7, 26, 24, 35, -8, -20, // rank 4
        -36, -26, -12, -1, 9, -7, 6, -23, // rank 5
        -45, -25, -16, -17, 3, 0, -5, -33, // rank 6
        -44, -16, -20, -9, -1, 11, -6, -71, // rank 7
        -19, -13, 1, 17, 16, 7, -37, -26, // rank 8
    ],
    [
        // queens
        -28, 0, 29, 12, 59, 44, 43, 45, // rank 1
        -24, -39, -5, 1, -16, 57, 28, 54, // rank 2
        -13, -17, 7, 8, 29, 56, 47, 57, // rank 3
        -27, -27, -16, -16, -1, 17, -2, 1, // rank 4
        -9, -26, -9, -10, -2, -4, 3, -3, // rank 5
        -14, 2, -11, -2, -5, 2, 14, 5, // rank 6
        -35, -8, 11, 2, 8, 15, -3, 1, // rank 7
        -1, -18, -9, 10, -15, -25, -31, -50, // rank 8
    ],
    [
        // pawns. ranks 1 and 8 are inconsequential
        0, 0, 0, 0, 0, 0, 0, 0, // rank 1
        98, 134, 61, 95, 68, 126, 34, -11, // rank 2
        -6, 7, 26, 31, 65, 56, 25, -20, // rank 3
        -14, 13, 6, 21, 23, 12, 17, -23, // rank 4
        -27, -2, -5, 12, 17, 6, 10, -25, // rank 5
        -26, -4, -4, -10, 3, 3, 33, -12, // rank 6
        -35, -1, -20, -23, -15, 24, 38, -22, // rank 7
        0, 0, 0, 0, 0, 0, 0, 0, // rank 8
    ],
    [
        // kings
        -65, 23, 16, -15, -56, -34, 2, 13, // rank 1
        29, -1, -20, -7, -8, -4, -38, -29, // rank 2
        -9, 24, 2, -16, -20, 6, 22, -22, // rank 3
        -17, -20, -12, -27, -30, -25, -14, -36, // rank 4
        -49, -1, -27, -39, -46, -44, -33, -51, // rank 5
        -14, -14, -22, -46, -44, -30, -15, -27, // rank 6
        1, 7, -8, -64, -43, -16, 9, 8, // rank 7
        -15, 36, 12, -54, 8, -28, 24, 14, // rank 8
    ],
]);

/// The PST for pieces in the endgame.
const ENDGAME_VALUE: Pst = expand_table([
    [
        // knights
        -58, -38, -13, -28, -31, -27, -63, -99, // rank 1
        -25, -8, -25, -2, -9, -25, -24, -52, // rank 2
        -24, -20, 10, 9, -1, -9, -19, -41, // rank 3
        -17, 3, 22, 22, 22, 11, 8, -18, // rank 4
        -18, -6, 16, 25, 16, 17, 4, -18, // rank 5
        -23, -3, -1, 15, 10, -3, -20, -22, // rank 6
        -42, -20, -10, -5, -2, -20, -23, -44, // rank 7
        -29, -51, -23, -15, -22, -18, -50, -64, // rank 8
    ],
    [
        // bishops
        -14, -21, -11, -8, -7, -9, -17, -24, // rank 1
        -8, -4, 7, -12, -3, -13, -4, -14, // rank 2
        2, -8, 0, -1, -2, 6, 0, 4, // rank 3
        -3, 9, 12, 9, 14, 10, 3, 2, // rank 4
        -6, 3, 13, 19, 7, 10, -3, -9, // rank 5
        -12, -3, 8, 10, 13, 3, -7, -15, // rank 6
        -14, -18, -7, -1, 4, -9, -15, -27, // rank 7
        -23, -9, -23, -5, -9, -16, -5, -17, // rank 8
    ],
    [
        // rooks
        13, 10, 18, 15, 12, 12, 8, 5, // rank 1
        11, 13, 13, 11, -3, 3, 8, 3, // rank 2
        7, 7, 7, 5, 4, -3, -5, -3, // rank 3
        4, 3, 13, 1, 2, 1, -1, 2, // rank 4
        3, 5, 8, 4, -5, -6, -8, -11, // rank 5
        -4, 0, -5, -1, -7, -12, -8, -16, // rank 6
        -6, -6, 0, 2, -9, -9, -11, -3, // rank 7
        -9, 2, 3, -1, -5, -13, 4, -20, // rank 8
    ],
    [
        // queens
        -9, 22, 22, 27, 27, 19, 10, 20, // rank 1
        -17, 20, 32, 41, 58, 25, 30, 0, // rank 2
        -20, 6, 9, 49, 47, 35, 19, 9, // rank 3
        3, 22, 24, 45, 57, 40, 57, 36, // rank 4
        -18, 28, 19, 47, 31, 34, 39, 23, // rank 5
        -16, -27, 15, 6, 9, 17, 10, 5, // rank 6
        -22, -23, -30, -16, -16, -23, -36, -32, // rank 7
        -33, -28, -22, -43, -5, -32, -20, -41, // rank 8
    ],
    [
        // pawns. ranks 1 and 8 are inconsequential
        0, 0, 0, 0, 0, 0, 0, 0, // rank 1
        178, 173, 158, 134, 147, 132, 165, 187, // rank 2
        94, 100, 85, 67, 56, 53, 82, 84, // rank 3
        32, 24, 13, 5, -2, 4, 17, 17, // rank 4
        13, 9, -3, -7, -7, -8, 3, -1, // rank 5
        4, 7, -6, 1, 0, -5, -1, -8, // rank 6
        13, 8, 8, 10, 13, 0, 2, -7, // rank 7
        0, 0, 0, 0, 0, 0, 0, 0, // rank 8
    ],
    [
        // kings
        -74, -35, -18, -18, -11, 15, 4, -17, // rank 1
        -12, 17, 14, 17, 17, 38, 23, 11, // rank 2
        10, 17, 23, 15, 20, 45, 44, 13, // rank 3
        -8, 22, 24, 27, 26, 33, 26, 3, // rank 4
        -18, -4, 21, 24, 27, 23, 9, -11, // rank 5
        -19, -3, 11, 21, 23, 16, 7, -9, // rank 6
        -27, -11, 4, 13, 14, 4, -5, -17, // rank 7
        -53, -34, -21, -11, -28, -14, -24, -43, // rank 8
    ],
]);

#[cfg(test)]
mod tests {

    use super::*;
    use fiddler_base::movegen::{get_moves, NoopNominator};
    use fiddler_base::Position;

    #[test]
    /// Test that adding deltas matches the same result as taking the PST value
    /// from scratch.
    fn test_pst_delta_equals_base_result() {
        let pos = Position::from_fen(
            "r1bq1b1r/ppp2kpp/2n5/3np3/2B5/8/PPPP1PPP/RNBQK2R w KQ - 0 7",
            Position::no_eval,
        )
        .unwrap();
        let pst_original = pst_evaluate(&pos.board);

        for m in get_moves::<NoopNominator>(&pos) {
            let delta = pst_delta(&pos.board, m.0);
            let delta_eval = (pst_original.0 + delta.0, pst_original.1 + delta.1);
            let mut bcopy = pos.board;
            bcopy.make_move(m.0);
            assert_eq!(delta_eval, pst_evaluate(&bcopy));
        }
    }
}
