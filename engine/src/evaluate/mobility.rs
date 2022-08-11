//! Evaluation of positions based on the mobility of pieces.
//!
//! We count mobility by examining the number of squares that a piece can move
//! to according to pseudo-legal moves.
//! This means that captures and empty squares visible to a piece (independent
//! of whether it is pinned) count towards its mobility score.
//!
//! For each piece, for each number of squares attacked, a unique mobility bonus
//! is given.
//! This prevents pieces from being placed uselessly in the name of being able
//! to see more squares.

use fiddler_base::{
    movegen::{KING_MOVES, KNIGHT_MOVES, PAWN_ATTACKS},
    Bitboard, Board, Color, Piece, MAGIC,
};

use super::{Eval, Score};

/// The maximum number of squares that any piece can attack, plus 1.
pub const MAX_MOBILITY: usize = 28;

/// The value of having a piece have a certain number of squares attacked.
pub const ATTACKS_VALUE: [[Score; MAX_MOBILITY]; Piece::NUM] = expand_attacks(&[
    [
        // N
        (-1, 0),
        (-12, 0),
        (-15, -2),
        (-12, -5),
        (3, -8),
        (12, -1),
        (12, -3),
        (18, 6),
        (17, 1),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
    ],
    [
        // B
        (-38, 0),
        (-19, -1),
        (-9, -3),
        (-5, -5),
        (1, -4),
        (9, -2),
        (12, -2),
        (13, -2),
        (19, 2),
        (19, 0),
        (11, 2),
        (8, 0),
        (1, 0),
        (1, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
    ],
    [
        // R
        (-19, 0),
        (-12, 0),
        (-11, 0),
        (-10, 0),
        (-6, 0),
        (-6, 0),
        (-4, 0),
        (-1, 4),
        (0, 5),
        (5, 2),
        (8, -2),
        (9, 0),
        (15, 3),
        (11, 2),
        (11, 8),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
    ],
    [
        // Q
        (-8, 0),
        (-7, 0),
        (-5, 0),
        (-6, 0),
        (-6, 0),
        (0, 0),
        (0, 0),
        (-4, 0),
        (-4, 0),
        (-1, 0),
        (0, 0),
        (1, 1),
        (0, 2),
        (1, 1),
        (0, 0),
        (0, 2),
        (3, 2),
        (3, 2),
        (4, 1),
        (4, 2),
        (2, 0),
        (2, 0),
        (2, 0),
        (1, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
    ],
    [
        // P
        (-12, 6),
        (-12, 18),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
    ],
    [
        // K
        (0, 0),
        (12, -1),
        (3, -7),
        (-1, -11),
        (-10, -5),
        (-13, -6),
        (2, 4),
        (3, 16),
        (5, 11),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
    ],
]);

/// Helper function to make the process of writing down attack values more easy.
const fn expand_attacks(
    vals: &[[(i16, i16); MAX_MOBILITY]; Piece::NUM],
) -> [[Score; MAX_MOBILITY]; Piece::NUM] {
    let mut out = [[Score::DRAW; MAX_MOBILITY]; 6];

    let mut pt_idx = 0;
    // workaround for lack of for loops in const fns
    while pt_idx < Piece::NUM {
        let mut mobility_idx = 0;
        while mobility_idx < MAX_MOBILITY {
            let (mg, eg) = vals[pt_idx][mobility_idx];
            out[pt_idx][mobility_idx] = Score::new(Eval(mg), Eval(eg));
            mobility_idx += 1;
        }
        pt_idx += 1;
    }

    out
}

#[inline(always)]
#[must_use]
/// Helper function for computing mobility scores of a piece.
///
/// Inputs:
/// * `pt`: the type of the piece being scored.
/// * `attacks`: the squares that the piece is attacking.
///
/// Returns the score associated with `pt` attacking all the squares in
/// `attacks`.
pub const fn for_piece(pt: Piece, attacks: Bitboard) -> Score {
    ATTACKS_VALUE[pt as usize][attacks.len() as usize]
}

#[must_use]
/// Get the mobility evaluation of a board.
pub fn evaluate(b: &Board) -> Score {
    let white = b[Color::White];
    let black = b[Color::Black];
    let not_white = !white;
    let not_black = !black;
    let occupancy = white | black;
    let mut score = Score::DRAW;

    // count knight moves
    let knights = b[Piece::Knight];
    // pinned knights can't move and so we don't bother counting them
    for sq in knights & white {
        score += for_piece(Piece::Knight, KNIGHT_MOVES[sq as usize] & not_white);
    }
    for sq in knights & black {
        score -= for_piece(Piece::Knight, KNIGHT_MOVES[sq as usize] & not_black);
    }

    // count bishop moves
    let bishops = b[Piece::Bishop];
    for sq in bishops & white {
        score += for_piece(
            Piece::Bishop,
            MAGIC.bishop_attacks(occupancy, sq) & not_white,
        );
    }
    for sq in bishops & black {
        score -= for_piece(
            Piece::Bishop,
            MAGIC.bishop_attacks(occupancy, sq) & not_black,
        );
    }

    // count rook moves
    let rooks = b[Piece::Rook];
    for sq in rooks & white {
        score += for_piece(Piece::Rook, MAGIC.rook_attacks(occupancy, sq) & not_white);
    }
    for sq in rooks & black {
        score -= for_piece(Piece::Rook, MAGIC.rook_attacks(occupancy, sq) & not_black);
    }

    // count queen moves
    let queens = b[Piece::Queen];
    for sq in queens & white {
        let attacks = MAGIC.rook_attacks(occupancy, sq) | MAGIC.bishop_attacks(occupancy, sq);
        score += for_piece(Piece::Queen, attacks & not_white);
    }
    for sq in rooks & black {
        let attacks = MAGIC.rook_attacks(occupancy, sq) | MAGIC.bishop_attacks(occupancy, sq);
        score -= for_piece(Piece::Queen, attacks & not_black);
    }

    // count net pawn moves
    // pawns can't capture by pushing, so we only examine their capture squares
    let pawns = b[Piece::Pawn];
    for sq in pawns & white {
        score += for_piece(
            Piece::Pawn,
            PAWN_ATTACKS[Color::White as usize][sq as usize] & not_white,
        );
    }
    for sq in pawns & black {
        score -= for_piece(
            Piece::Pawn,
            PAWN_ATTACKS[Color::Black as usize][sq as usize] & not_black,
        );
    }

    score += for_piece(
        Piece::King,
        KING_MOVES[b.king_sqs[Color::White as usize] as usize] & not_white,
    );
    score -= for_piece(
        Piece::King,
        KING_MOVES[b.king_sqs[Color::Black as usize] as usize] & not_black,
    );

    score
}
