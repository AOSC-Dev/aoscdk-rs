use std::fmt::Write;

pub type SudokuMatrix = [[u8; 9]; 9];
type Coord = [usize; 2];

#[derive(Debug)]
pub struct MySudoku {
    matrix: SudokuMatrix,
    pub available: [[bool; 9]; 9],
}

impl MySudoku {
    // fn new(matrix: SudokuMatrix) -> Self {
    //     Self {
    //         matrix,
    //         coords: Self::find_empty_coords(&matrix),
    //     }
    // }

    fn find_availability(sudoku: &SudokuMatrix) -> [[bool; 9]; 9] {
        let mut available = [[false; 9]; 9];
        for i in 0..9 {
            for j in 0..9 {
                available[i][j] = sudoku[i][j] == 0;
            }
        }
        available
    }

    /// Convert from a `[[u8; 9]; 9]` matrix (array-of-array) to a `Sudoku`
    // pub fn from_matrix(matrix: SudokuMatrix) -> Self {
    //     matrix.into()
    // }

    pub fn finished(&self) -> bool {
        for i in 0..9 {
            for j in 0..9 {
                if self[[i, j]] == 0 {
                    return false;
                }
            }
        }
        true
    }

    // /// Check whether the sudoku is still valid
    pub fn conflict(&self, v: u8, coord: Coord) -> Option<[usize; 2]> {
        if let Some(coord) = self.conflict_row(v, coord) {
            Some(coord)
        } else if let Some(coord) = self.conflict_col(v, coord) {
            Some(coord)
        } else {
            self.conflict_box(v, coord)
        }
    }

    fn conflict_row(&self, v: u8, coord: Coord) -> Option<[usize; 2]> {
        let [i, _] = coord;
        for j in 0..9 {
            if self[[i, j]] == v {
                return Some([i, j]);
            }
        }
        None
    }
    fn conflict_col(&self, v: u8, coord: Coord) -> Option<[usize; 2]> {
        let [_, j] = coord;
        for i in 0..9 {
            if self[[i, j]] == v {
                return Some([i, j]);
            }
        }
        None
    }
    fn conflict_box(&self, v: u8, coord: Coord) -> Option<[usize; 2]> {
        let [i_, j_] = coord;
        let [i_, j_] = [i_ / 3, j_ / 3];
        for i in 3 * i_..3 * i_ + 3 {
            // "inner" i and j; indexes of individual cells
            for j in 3 * j_..3 * j_ + 3 {
                if v == self[[i, j]] {
                    return Some([i, j]);
                }
            }
        }
        None
    }
}

impl std::convert::From<SudokuMatrix> for MySudoku {
    fn from(matrix: SudokuMatrix) -> Self {
        Self {
            matrix,
            available: Self::find_availability(&matrix),
        }
    }
}

impl std::ops::Index<Coord> for MySudoku {
    type Output = u8;

    fn index(&self, coords: Coord) -> &Self::Output {
        &self.matrix[coords[0]][coords[1]]
    }
}

impl std::ops::IndexMut<Coord> for MySudoku {
    fn index_mut(&mut self, coords: Coord) -> &mut u8 {
        &mut self.matrix[coords[0]][coords[1]]
    }
}

use std::fmt;

impl fmt::Display for MySudoku {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::with_capacity(180);
        for i in 0..9 {
            for j in 0..9 {
                write!(s, "{} ", self[[i, j]])?;
            }
            s.push('\n')
        }
        write!(f, "{s}")
    }
}
