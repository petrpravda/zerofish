use std::collections::HashSet;
use crate::model::EnumIntersectionType::{LineReductionColumn, LineReductionRow, BoxReductionColumn, BoxReductionRow};
use crate::board_navigation::{get_row_indices, get_column_indices};

#[allow(dead_code)]
#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum EnumStrategyType {
    NakedSingle,
    HiddenSingle,
    NakedPair, NakedTriple, NakedQuadruple,
    HiddenPair, HiddenTriple, HiddenQuadruple,
    PointingPair, PointingTriple,
    XWing,
    YWing,
    Swordfish
}

#[allow(dead_code)]
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum EnumSectionType {
    SimpleCell,
    Row,
    Column,
    Box,
    BoxWithRow,
    BoxWithColumn
}

#[allow(dead_code)]
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum EnumIntersectionType {
    LineReductionColumn,
    LineReductionRow,
    BoxReductionColumn,
    BoxReductionRow
}

#[allow(dead_code)]
pub const INTERSECTION_TYPES: [EnumIntersectionType; 4] = [LineReductionColumn, LineReductionRow, BoxReductionColumn, BoxReductionRow];

#[allow(dead_code)]
pub fn get_intersection_type_row_step(intersection_type: &EnumIntersectionType) -> usize {
    match intersection_type {
        LineReductionColumn => 3,
        LineReductionRow => 1,
        BoxReductionColumn => 3,
        BoxReductionRow => 1
    }
}

#[allow(dead_code)]
pub fn get_intersection_type_column_step(intersection_type: &EnumIntersectionType) -> usize {
    match intersection_type {
        LineReductionColumn => 1,
        LineReductionRow => 3,
        BoxReductionColumn => 1,
        BoxReductionRow => 3
    }
}

#[allow(dead_code)]
pub fn get_intersection_type_line_indices(row: usize, column: usize, intersection_type: &EnumIntersectionType) -> HashSet<usize> {
    match intersection_type {
        LineReductionColumn => get_column_indices(column),
        LineReductionRow => get_row_indices(row),
        BoxReductionColumn => get_column_indices(column),
        BoxReductionRow => get_row_indices(row)
    }
}

#[allow(dead_code)]
pub fn is_line_removal(intersection_type: &EnumIntersectionType) -> bool {
    match intersection_type {
        LineReductionRow | LineReductionColumn => true,
        BoxReductionColumn | BoxReductionRow => false
    }
}

// pub fn get_intersection_type_box_indices(row: usize, column: usize) -> HashSet<usize> {
//
// }

//     LINE_REDUCTION_COLUMN(1, 3, true){
//         @Override
//         public Set<Integer> getLineIndices(int row, int column) {
//             return getColumnLineIndices(column);
//         }
//     },
//     LINE_REDUCTION_ROW(3, 1, true) {
//         @Override
//         public Set<Integer> getLineIndices(int row, int column) {
//             return getRowLineIndices(row);
//         }
//     },
//     BOX_REDUCTION_COLUMN(1, 3, false) {
//         @Override
//         public Set<Integer> getLineIndices(int row, int column) {
//             return getColumnLineIndices(column);
//         }
//     },
//     BOX_REDUCTION_ROW(3, 1, false) {

#[derive(Debug)]
pub struct SetWithRef {
    pub set: HashSet<u8>,
    pub index: usize
}

impl SetWithRef {
    pub fn new(set: &HashSet<u8>, index: usize) -> Self {
        Self {
            set: set.clone(),
            index
        }
    }
}

pub struct ListWithType {
    pub list: Vec<SetWithRef>,
    pub section_type: EnumSectionType
}

impl ListWithType {
    pub fn new(section_type: EnumSectionType) -> Self {
        Self {
            list: vec![],
            section_type
        }
    }
}

pub trait SortedImpl {
    fn sorted(self) -> Self;
}

impl<E> SortedImpl for Vec<E>
    where E: std::cmp::Ord
{
    fn sorted(mut self) -> Self {
        self.sort();
        self
    }
}

