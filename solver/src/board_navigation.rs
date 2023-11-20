#[allow(dead_code)]

use std::collections::HashSet;

use crate::model::{EnumSectionType, SortedImpl};

pub struct SudokuBoardX(pub usize);

pub struct BoardIndices {
    pub row: usize,
    pub column: usize,
    pub index: usize,
    pub box_row: usize,
    pub box_column: usize,
    pub box_index: usize,
}

impl Iterator for SudokuBoardX {
    type Item = BoardIndices;

    fn next(&mut self) -> Option<BoardIndices> {
        let index = self.0;
        if index == 81 {
            return None;
        }

        let row = index / 9;
        let column = index % 9;

        let box_row = row / 3;
        let box_column = column / 3;
        let box_index = box_row * 3 + box_column;

        let result = BoardIndices {
            row,
            column,
            index,
            box_row,
            box_column,
            box_index
        };
        self.0 = index + 1;
        Some(result)
    }
}

#[allow(dead_code)]
pub fn get_cell_indices_record(index: usize) -> BoardIndices {
    let row = index / 9;
    let column = index % 9;
    let box_row = row / 3;
    let box_column = column / 3;
    let box_index = box_row * 3 + box_column;
    BoardIndices { row, column, index, box_row, box_column, box_index }
}

pub struct SwordfishIterator {
    row1: u8,
    column1: u8,
    row2: u8,
    column2: u8,
    row3: u8,
    column3: u8
}

#[allow(dead_code)]
impl SwordfishIterator {
    pub fn new() -> Self {
        Self {
            row1: 0,
            column1: 0,
            row2: 1,
            column2: 1,
            row3: 2,
            column3: 2
        }
    }

    fn increment(&mut self) {
        self.column3 += 1;
        if self.column3 > 8 {
            self.column3 = self.column2;
            self.row3 += 1;
            if self.row3 > 8 {
                self.row3 = self.row2;
                self.column2 += 1;
                if self.column2 > 8 {
                    self.column2 = self.column1;
                    self.row2 += 1;
                    if self.row2 > 8 {
                        self.row2 = self.row1;
                        self.column1 += 1;
                        if self.column1 > 8 {
                            self.column1 = 0;
                            self.row1 += 1;
                        }
                    }
                }
            }
        }
    }

    fn check_ok(&self) -> bool {
        return self.row3 > self.row2 && self.row2 > self.row1 && self.column3 > self.column2 && self.column2 > self.column1;
    }
}

pub struct SwordfishIndices {
    pub row1: u8,
    pub column1: u8,
    pub row2: u8,
    pub column2: u8,
    pub row3: u8,
    pub column3: u8
}

#[allow(dead_code)]
impl SwordfishIndices {
    pub fn get_intersection_indices(&self) -> Vec<usize> {
        vec![(self.row1 * 9 + self.column1) as usize, (self.row1 * 9 + self.column2) as usize, (self.row1 * 9 + self.column3) as usize,
             (self.row2 * 9 + self.column1) as usize, (self.row2 * 9 + self.column2) as usize, (self.row2 * 9 + self.column3) as usize,
             (self.row3 * 9 + self.column1) as usize, (self.row3 * 9 + self.column2) as usize, (self.row3 * 9 + self.column3) as usize]
    }

    pub fn get_row_indices(&self) -> HashSet<usize> {
        vec![self.row1, self.row2, self.row3].iter().map(|x| *x as usize).collect()
    }

    pub fn get_column_indices(&self) -> HashSet<usize> {
        vec![self.column1, self.column2, self.column3].iter().map(|x| *x as usize).collect()
    }
}

impl Iterator for SwordfishIterator {
    type Item = SwordfishIndices;

    fn next(&mut self) -> Option<SwordfishIndices> {
        if self.row1 >= 9 {
            return None
        }
        let result = SwordfishIndices{row1: self.row1, column1: self.column1, row2: self.row2, column2: self.column2, row3: self.row3, column3: self.column3 };
        self.increment();
        while !self.check_ok() && self.row1 <= 8 {
            self.increment();
        }
        return Some(result);
    }
}

pub struct XWingIterator {
    row1: u8,
    column1: u8,
    row2: u8,
    column2: u8
}

#[allow(dead_code)]
impl XWingIterator {
    pub fn new() -> Self {
        Self {
            row1: 0,
            column1: 0,
            row2: 1,
            column2: 1
        }
    }

    fn increment(&mut self) {
        self.column2 += 1;
        if self.column2 > 8 {
            self.column2 = self.column1;
            self.row2 += 1;
            if self.row2 > 8 {
                self.row2 = self.row1;
                self.column1 += 1;
                if self.column1 > 8 {
                    self.column1 = 0;
                    self.row1 += 1;
                }
            }
        }
    }

    fn check_ok(&self) -> bool {
        return self.row2 > self.row1 && self.column2 > self.column1;
    }
}

pub struct XWingIndices {
    pub row1: u8,
    pub column1: u8,
    pub row2: u8,
    pub column2: u8
}

#[allow(dead_code)]
impl XWingIndices {
    pub fn get_intersection_indices(&self) -> Vec<usize> {
        vec![(self.row1 * 9 + self.column1) as usize,
             (self.row1 * 9 + self.column2) as usize,
             (self.row2 * 9 + self.column1) as usize,
             (self.row2 * 9 + self.column2) as usize]
    }
}

impl Iterator for XWingIterator {
    type Item = XWingIndices;

    fn next(&mut self) -> Option<XWingIndices> {
        if self.row1 >= 9 {
            return None
        }
        let result = XWingIndices{row1: self.row1, column1: self.column1, row2: self.row2, column2: self.column2};
        self.increment();
        while !self.check_ok() && self.row1 <= 8 {
            self.increment();
        }
        return Some(result);
    }
}

#[allow(dead_code)]
pub struct Permutations {

}

#[allow(dead_code)]
impl Permutations {
    pub fn two_fold(from_set: &HashSet<u8>) -> Vec<HashSet<u8>> {
        let unsorted_list: Vec<u8> = from_set.iter().map(|x|*x).collect();
        let list = unsorted_list.sorted();
        let mut perms = Vec::<HashSet<u8>>::new();
        let size = &list.len();
        for i1 in 0..*size {
            for i2 in i1+1..*size {
                let mut permutation: HashSet<u8> = HashSet::new();
                permutation.insert(list[i1]);
                permutation.insert(list[i2]);
                perms.push(permutation);
            }
        }

        perms
    }

    pub fn three_fold(from_set: &HashSet<u8>) -> Vec<HashSet<u8>> {
        let unsorted_list: Vec<u8> = from_set.iter().map(|x|*x).collect();
        let list = unsorted_list.sorted();
        let mut perms = Vec::<HashSet<u8>>::new();
        let size = &list.len();
        for i1 in 0..*size {
            for i2 in i1+1..*size {
                for i3 in i2+1..*size {
                    let mut permutation: HashSet<u8> = HashSet::new();
                    permutation.insert(list[i1]);
                    permutation.insert(list[i2]);
                    permutation.insert(list[i3]);
                    perms.push(permutation);
                }
            }
        }

        perms
    }
}

pub fn get_box_indices(row: usize, column: usize) -> HashSet<usize> {
    let mut result = HashSet::<usize>::new();
    let box_row = row / 3;
    let box_column = column / 3;
    for y in 0..3 {
        for x in 0..3 {
            result.insert(((box_row * 3 + y) * 9 + (box_column * 3 + x)) as usize);
        }
    }
    result
}

pub fn get_box_indices_from_index(box_index: usize) -> HashSet<usize> {
    let mut result = HashSet::<usize>::new();
    let box_row = box_index / 3;
    let box_column = box_index % 3;
    for y in 0..3 {
        for x in 0..3 {
            result.insert(((box_row * 3 + y) * 9 + (box_column * 3 + x)) as usize);
        }
    }
    result
}

pub fn get_row_indices(row: usize) -> HashSet<usize> {
    (0..9).map(|x| row * 9 + x).collect()
}

pub fn get_column_indices(column: usize) -> HashSet<usize> {
    (0..9).map(|x| x * 9 + column).collect()
}

#[allow(dead_code)]
pub fn all_section_indices(index: usize, section_type: EnumSectionType) -> Vec<usize> {
    let (row, column) = (index / 9, index % 9);
    match section_type {
        EnumSectionType::Column => get_column_indices(column).iter().copied().collect(),
        EnumSectionType::Row => get_row_indices(row).iter().copied().collect(),
        EnumSectionType::Box => get_box_indices(row, column).iter().copied().collect(),
        EnumSectionType::BoxWithRow => (&get_box_indices(row, column) | &get_row_indices(row)).iter().copied().collect(),
        EnumSectionType::BoxWithColumn => (&get_box_indices(row, column) | &get_column_indices(column)).iter().copied().collect(),
        _ => panic!("Unsupported section type")
    }
}

#[allow(dead_code)]
pub fn collection_as_text_refs(indices: &Vec<usize>) -> Vec<String> {
    let copy: Vec<usize> = indices.iter().copied().collect();
    let sort = copy.sorted();
    let result: Vec<String> = sort.iter().map(|index| get_text_ref(*index)).collect();
    result
}

#[allow(dead_code)]
pub fn collection_as_text_from_value(indices: &Vec<u8>) -> String {
    let result: Vec<String> = indices.iter().map(|num| (*num).to_string()).collect();
    result.sorted().join(",")
}

#[allow(dead_code)]
pub fn collection_as_text_refs_string(indices: &Vec<usize>) -> String {
    let refs = collection_as_text_refs(&indices);
    refs.join(",")
}

pub fn get_text_ref(index: usize) -> String {
    let row = index / 9;
    let column = index % 9;
    let row_letter = b'A' + (row as u8);
    let row_string =
        String::from_utf8(vec![row_letter]).expect("Could not convert row letter");
    format!("{}{}", row_string, column + 1)
}
