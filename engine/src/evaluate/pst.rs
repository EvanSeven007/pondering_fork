/*
  Fiddler, a UCI-compatible chess engine.
  Copyright (C) 2022 The Fiddler Authors (see AUTHORS.md file)

  Fiddler is free software: you can redistribute it and/or modify
  it under the terms of the GNU General Public License as published by
  the Free Software Foundation, either version 3 of the License, or
  (at your option) any later version.

  Fiddler is distributed in the hope that it will be useful,
  but WITHOUT ANY WARRANTY; without even the implied warranty of
  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
  GNU General Public License for more details.

  You should have received a copy of the GNU General Public License
  along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

//! Piece-Square Tables (PSTs).
//!
//! A PST is a table with an item for each piece at each square.
//! It grants a fixed value to the evaluation of a position for each piece,
//! granting benefits for being on "good" squares and penalties for pieces on
//! "bad" ones.
//! For instance, a knight is much more valuable near the center, so the PST
//! value for a knight on rank 4 and file 3 is positive.

use std::intrinsics::transmute;

use fiddler_base::{Board, Color, Move, Piece, Square};

use crate::evaluate::Score;

/// A lookup table for piece values.
/// The outer index is the type of the piece
/// (in order of Pawn, Knight, Bishop, Rook, Queen, and King)
/// and the inner index is the square of the piece (from White's point of view),
/// starting with A1 as the first index, then continuing on to B1, C1, and so
/// on until H8 as index 63.
type Pst = [[Score; 64]; Piece::NUM];

#[must_use]
/// Evaluate a board based on its PST value.
/// This is slow, so under most conditions it is recommended to use
/// `value_delta()` instead if you are making moves.
/// The first value in the return type is the midgame difference, and the second
/// is the endgame difference.
pub fn evaluate(board: &Board) -> Score {
    let mut score = Score::DRAW;

    for pt in Piece::ALL {
        for sq in board[pt] & board[Color::White] {
            score += PST[pt as usize][sq as usize];
        }
        for sq in board[pt] & board[Color::Black] {
            //Invert the square that Black is on, since positional values are
            //flipped (as pawns move the other way, etc)
            let alt_sq = sq.opposite();
            score -= PST[pt as usize][alt_sq as usize];
        }
    }

    score
}

#[must_use]
/// Get the difference in PST value which would be generated by making the move
/// `m` on `board`. The first value in the return tuple is the midgame
/// difference, and the second is the endgame difference. `pst_delta` will
/// reflect how the position improves for the player making the move,
/// independed of if the player is white or black.
///
/// # Panics
///
/// This function will panic if the given move is invalid.
pub fn delta(board: &Board, m: Move) -> Score {
    let from_sq = m.from_square();
    let to_sq = m.to_square();
    let mover_type = board.type_at_square(m.from_square()).unwrap();
    let mover_idx = mover_type as usize;
    let end_type = match m.promote_type() {
        Some(pt) => pt,
        None => mover_type,
    };
    let end_idx = end_type as usize;
    let (from_alt, to_alt) = match board.player {
        Color::White => (from_sq, to_sq),
        Color::Black => (from_sq.opposite(), to_sq.opposite()),
    };
    let (from_idx, to_idx) = (from_alt as usize, to_alt as usize);

    // you always lose the value of the square you moved from
    let mut delta = PST[end_idx][to_idx] - PST[mover_idx][from_idx];

    if board[!board.player].contains(m.to_square()) {
        // conventional capture
        let to_opposite_idx = to_alt.opposite() as usize;
        let capturee_idx = board.type_at_square(to_sq).unwrap() as usize;
        delta += PST[capturee_idx][to_opposite_idx];
    }

    if m.is_en_passant() {
        let to_opposite_idx = (to_alt - Color::White.pawn_direction()).opposite() as usize;
        delta += PST[Piece::Pawn as usize][to_opposite_idx];
    }

    if m.is_castle() {
        let is_queen_castle = to_sq.file() == 2;
        let (rook_from_idx, rook_to_idx) = if is_queen_castle {
            (Square::A1 as usize, Square::D1 as usize)
        } else {
            (Square::H1 as usize, Square::F1 as usize)
        };

        delta += PST[Piece::Rook as usize][rook_to_idx] - PST[Piece::Rook as usize][rook_from_idx];
    }

    delta
}

#[rustfmt::skip] // rustfmt likes to throw a million newlines in this
/// The main piece-square table. Evaluations are paired together as (midgame, 
/// endgame) to improve cache-friendliness. The indexing order of this table 
/// has its primary index as pieces, the secondary index as squares, and the 
/// innermost index as 0 for midgame and 1 for endgame.
pub const PST: Pst = unsafe { transmute([
    [ // N
        (-152i16, -46i16), (-1, -18), (-54, -17), (-32, -13), (-26, -22), (-32, -18), (-3, -17), (-78, -33), 
        (-84, -13), (-58, -6), (-15, 0), (5, 0), (5, 0), (-18, -2), (-34, -5), (-36, -38), 
        (-16, -28), (-10, 6), (3, 3), (10, 9), (14, 0), (12, 15), (8, -1), (-17, -23), 
        (-11, -25), (-14, 1), (7, 10), (9, 16), (16, 0), (6, 1), (7, 0), (-3, -17), 
        (0, -12), (10, 3), (25, 12), (41, 16), (22, 11), (53, 17), (10, 3), (23, -17), 
        (-5, -13), (22, 2), (7, 12), (52, 0), (47, 0), (7, 0), (38, -11), (17, -13), 
        (-23, -28), (-14, -18), (51, -15), (-4, 2), (23, -5), (22, -8), (-3, -15), (-2, -33), 
        (-77, -37), (-46, -49), (-36, -12), (-23, -15), (-8, -16), (-74, -13), (-41, -36), (-91, -32), 
    ],
    [ // B
        (-51, -11), (-14, -4), (0, -11), (-27, -11), (-30, -1), (-9, -13), (-26, -15), (-47, 0), 
        (-32, -3), (0, -4), (-5, -6), (-2, 0), (4, 0), (-5, 0), (17, -2), (-27, -1), 
        (-10, 0), (0, 5), (9, 5), (8, 5), (6, 1), (9, 11), (2, 5), (-12, -6), 
        (-13, -2), (-7, -6), (9, 8), (16, 2), (8, -2), (0, -2), (-3, -8), (0, -6), 
        (-4, 0), (3, 0), (4, 11), (23, 5), (25, 0), (17, 3), (0, 7), (3, -6), 
        (-15, -3), (7, -6), (-19, 0), (17, 0), (2, -4), (-40, 0), (25, -3), (6, -5), 
        (-20, 2), (-4, 4), (-5, 0), (-81, 0), (-58, 1), (0, 0), (-15, 0), (-18, -16), 
        (-21, -5), (-25, 0), (-47, 0), (-41, 0), (-24, -13), (-83, 0), (-30, -12), (-8, -19), 
    ],
    [ // R
        (-3, 4), (-4, 6), (-3, 5), (0, 1), (0, 0), (4, -1), (-16, 0), (-19, 2), 
        (-28, -9), (-20, 1), (-9, -2), (-6, 0), (-4, 0), (-3, -9), (-7, -1), (-30, 0), 
        (-27, 0), (-25, 3), (-12, -1), (-6, -8), (-13, -2), (-17, -5), (-1, -6), (-23, -5), 
        (-19, -6), (-10, 0), (-3, 0), (-6, -2), (-11, -5), (-19, -5), (-9, -1), (-16, -13), 
        (-7, -1), (-15, -1), (2, 7), (-1, -1), (-2, -3), (0, -1), (-11, -8), (-7, -10), 
        (-1, 0), (2, 5), (4, 0), (9, 2), (2, 0), (23, -15), (17, -6), (4, -4), 
        (8, 6), (15, 10), (33, 10), (31, 6), (28, 0), (33, 0), (34, 3), (21, -2), 
        (0, 9), (18, 0), (13, 0), (7, 1), (11, 1), (-5, 4), (7, -3), (8, 3), 
    ],
    [ // Q
        (-22, -7), (-31, -1), (-24, 4), (4, 0), (-18, 0), (-44, 0), (-13, 0), (-19, -2), 
        (-74, 0), (-46, 2), (-3, 11), (-1, 0), (6, 0), (0, -2), (-6, 0), (0, -6), 
        (-30, 0), (-4, 0), (-1, 9), (0, 2), (0, 8), (14, 3), (0, 0), (3, 0), 
        (-15, 0), (-17, 14), (-2, 0), (3, 4), (5, 8), (0, 6), (4, 0), (0, 0), 
        (-21, -3), (-16, 1), (-1, 0), (13, 0), (24, 0), (27, 0), (14, 0), (17, 0), 
        (-16, 0), (-13, 0), (9, 0), (28, 2), (51, 0), (69, 7), (91, -4), (54, -16), 
        (-30, 0), (-26, -1), (0, 0), (17, 0), (23, -1), (76, 0), (47, -4), (68, -13), 
        (-11, -5), (8, 0), (19, 0), (11, 12), (36, 4), (29, 0), (35, 3), (38, -18), 
    ],
    [ // P
        (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), 
        (-9, 0), (5, 3), (-13, -1), (-10, -5), (1, 0), (12, -2), (18, 1), (-12, -3), 
        (-8, -9), (-3, 3), (-3, -6), (-4, 0), (2, -3), (0, -8), (5, -1), (-6, -13), 
        (-7, 5), (-1, 10), (1, -1), (9, -7), (9, -7), (-2, -7), (-3, 1), (-13, -9), 
        (0, 23), (13, 20), (3, 9), (16, 3), (10, -1), (7, 3), (10, 9), (-6, 8), 
        (29, 68), (45, 70), (33, 48), (51, 44), (51, 23), (48, 33), (39, 50), (23, 45), 
        (49, 92), (44, 77), (63, 72), (88, 71), (85, 55), (60, 48), (47, 44), (27, 71), 
        (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), 
    ],
    [ // K
        (-44, -40), (4, -37), (2, -24), (-31, -24), (-12, -22), (-25, -10), (15, -28), (-30, -46), 
        (-33, -27), (-9, -23), (-9, -14), (-25, -9), (-10, -3), (-4, -5), (8, -17), (-10, -17), 
        (-36, -34), (-11, -11), (-9, -1), (-10, 0), (-8, 7), (-7, 5), (-6, 0), (-31, -16), 
        (-38, -17), (0, -8), (-4, 13), (0, 14), (0, 10), (0, 8), (-4, -2), (-38, -9), 
        (-19, -22), (16, 9), (22, 11), (11, 21), (6, 16), (16, 15), (15, 10), (-12, -12), 
        (-3, -13), (23, 0), (28, 8), (11, 17), (14, 18), (35, 12), (29, 3), (0, 2), 
        (-4, -9), (22, -3), (24, 4), (13, 7), (18, 0), (23, 5), (39, -5), (8, -16), 
        (-30, -27), (-6, -36), (-17, -17), (-29, -7), (-16, -6), (-9, 4), (23, -28), (0, -28), 
    ],
]) };

#[cfg(test)]
mod tests {

    use super::*;
    use fiddler_base::{game::Game, movegen::ALL};

    fn delta_helper(fen: &str) {
        let mut g = Game::from_fen(fen).unwrap();
        let orig_eval = evaluate(g.board());
        for (m, _) in g.get_moves::<ALL>() {
            let new_eval = match g.board().player {
                Color::White => orig_eval + delta(g.board(), m),
                Color::Black => orig_eval - delta(g.board(), m),
            };
            g.make_move(m, &());
            // println!("{g}");
            assert_eq!(new_eval, evaluate(g.board()));
            g.undo().unwrap();
        }
    }

    #[test]
    /// Test that adding deltas matches the same result as taking the PST value
    /// from scratch.
    fn pst_delta_equals_base_result() {
        delta_helper("r1bq1b1r/ppp2kpp/2n5/3np3/2B5/8/PPPP1PPP/RNBQK2R w KQ - 0 7");
    }

    #[test]
    fn delta_captures() {
        delta_helper("r1bq1b1r/ppp2kpp/2n5/3n4/2BPp3/2P5/PP3PPP/RNBQK2R b KQ d3 0 8");
    }

    #[test]
    fn delta_promotion() {
        delta_helper("r4bkr/pPpq2pp/2n1b3/3n4/2BPp3/2P5/1P3PPP/RNBQK2R w KQ - 1 13");
    }
}
