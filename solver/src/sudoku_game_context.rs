use std::collections::HashSet;

use crate::board_navigation::{get_box_indices, get_box_indices_from_index, get_column_indices, get_row_indices, SudokuBoardX};
use crate::model::{EnumSectionType, ListWithType, SetWithRef, SortedImpl};

const SUDOKU_LINE_DOUBLE_TOP: &'static str =    "╔═══╤═══╤═══╦═══╤═══╤═══╦═══╤═══╤═══╗\n";
const SUDOKU_LINE_SINGLE: &'static str =        "╟───┼───┼───╫───┼───┼───╫───┼───┼───╢\n";
const SUDOKU_LINE_DOUBLE_BOTTOM: &'static str = "╚═══╧═══╧═══╩═══╧═══╧═══╩═══╧═══╧═══╝\n";
const SUDOKU_LINE_DOUBLE_MIDDLE: &'static str = "╠═══╪═══╪═══╬═══╪═══╪═══╬═══╪═══╪═══╣\n";
const SUDOKU_SEPARATOR_SINGLE: &'static str =   "│";
const SUDOKU_SEPARATOR_DOUBLE: &'static str =   "║";
const SUDOKU_CANDIDATES_LINE_DOUBLE_TOP: &'static str =    "  ╔════ 1 ════╤════ 2 ════╤════ 3 ════╦════ 4 ════╤════ 5 ════╤════ 6 ════╦════ 7 ════╤════ 8 ════╤════ 9 ════╗\n";
const SUDOKU_CANDIDATES_LINE_SINGLE: &'static str =        "  ╟───────────┼───────────┼───────────╫───────────┼───────────┼───────────╫───────────┼───────────┼───────────╢\n";
const SUDOKU_CANDIDATES_LINE_DOUBLE_BOTTOM: &'static str = "  ╚═══════════╧═══════════╧═══════════╩═══════════╧═══════════╧═══════════╩═══════════╧═══════════╧═══════════╝\n";
const SUDOKU_CANDIDATES_LINE_DOUBLE_MIDDLE: &'static str = "  ╠═══════════╪═══════════╪═══════════╬═══════════╪═══════════╪═══════════╬═══════════╪═══════════╪═══════════╣\n";

pub struct SudokuGameContext {
    pub filled_numbers: [Option<u8>; 81],
    pub user_candidates: Vec<HashSet<u8>>,
    pub solver_candidates: Vec<HashSet<u8>>,
}

impl SudokuGameContext {
    pub fn count(&self) -> usize {
        self.filled_numbers.iter()
            .filter(|value| value.is_some())
            .count()
    }

    pub fn is_valid(&self) -> Option<String> {
        for row in 0..9 {
            let set = get_row_indices(row);
            let counts = self.count_frequency_of_filled_numbers(set);
            if counts.iter().any(|x| *x > 1) {
                return Some(format!("There are duplicates in a row: {}", row));
            }
        }

        for column in 0..9 {
            let set = get_column_indices(column);
            let counts = self.count_frequency_of_filled_numbers(set);
            if counts.iter().any(|x| *x > 1) {
                return Some(format!("There are duplicates in a column: {}", column));
            }
        }

        for box_index in 0..9 {
            let set = get_box_indices_from_index(box_index);
            let counts = self.count_frequency_of_filled_numbers(set);
            if counts.iter().any(|x| *x > 1) {
                return Some(format!("There are duplicates in a box: {}", box_index));
            }
        }

        return None;
    }
}

impl SudokuGameContext {
    pub fn new() -> Self {
        let filled_numbers: [Option<u8>; 81] = [None; 81];
        let user_candidates: Vec<_> = (0..81).map(|_| HashSet::<u8>::new()).collect();
        //let user_candidates: [HashSet<u8>; 81] = user_candidates_vec.try_into().unwrap();
        let solver_candidates: Vec<_> = (0..81).map(|_| HashSet::<u8>::new()).collect();
        //let solver_candidates: [HashSet<u8>; 81] = solver_candidates_vec.try_into().unwrap();

        Self {
            filled_numbers,
            user_candidates,
            solver_candidates
        }
    }

    pub fn to_string(&self) -> String {
        self.get_text_board()
    }

    pub fn set_value(&mut self, index: usize, value: u8) {
        self.filled_numbers[index] = Some(value);
        self.user_candidates[index].clear();
        self.solver_candidates[index].clear();
    }

    pub fn get_text_board(&self) -> String  {
        let mut result = String::new();
        result.push_str(SUDOKU_LINE_DOUBLE_TOP);

        for row in 0..9 {
            for column in 0..9 {
                let index = row * 9 + column;
                let value = self.filled_numbers[index];
                result.push_str(if column % 3 == 0 { SUDOKU_SEPARATOR_DOUBLE } else { SUDOKU_SEPARATOR_SINGLE });
                result.push(' ');
                let number_string = value.map(|z| z.to_string()).to_owned();
                result.push_str(number_string.as_deref().unwrap_or(" "));
                result.push(' ')
            }
            result.push_str(SUDOKU_SEPARATOR_DOUBLE);
            result.push_str("\n");
            result.push_str(if (row + 1) % 3 == 0 {
                if row == 8 { SUDOKU_LINE_DOUBLE_BOTTOM } else { SUDOKU_LINE_DOUBLE_MIDDLE }
            } else {
                SUDOKU_LINE_SINGLE
            });
        }
        result
    }

    pub fn get_text_solver_candidates_enhanced(&self) -> String {
        let mut result = String::new();

        let text_left = self.get_text_board();
        let text_right = self.get_text_candidates(&self.solver_candidates);
        let mut right_iter = text_right.lines();

        for line in text_left.lines() {
            result.push_str(line);
            result.push_str("   ");
            result.push_str(right_iter.next().unwrap());
            result.push('\n');
        }

        result.pop();
        result
    }

    pub fn get_text_solver_candidates_packed(&self) -> String {
        // let mut result = String::new();
        // //result
        let vec: Vec<String> = self.solver_candidates.iter().map(|c| {
            let cell_string: Vec<String> = c.iter().map(|n| (*n as u32).to_string()).collect();
            cell_string.sorted().join("")
        }).collect();
        let result = vec.join(",");
        result
    }

    pub fn get_text_solver_candidates(&self) -> String {
        self.get_text_candidates(&self.solver_candidates)
    }

    pub fn get_text_user_candidates(&self) -> String {
        self.get_text_candidates(&self.user_candidates)
    }

    pub fn get_text_candidates(&self, candidates: &Vec<HashSet<u8>>) -> String  {
        let mut result = String::new();
        result.push_str(SUDOKU_CANDIDATES_LINE_DOUBLE_TOP);

        for row in 0..9 as usize {
            let c = 'A' as u8 + row as u8;
            result.push(c as char);
            result.push_str(" ");
            for column in 0..9 {
                let index = row * 9 + column;
                let value = &candidates[index];
                result.push_str(if column % 3 == 0 { SUDOKU_SEPARATOR_DOUBLE } else { SUDOKU_SEPARATOR_SINGLE });
                result.push(' ');
                (1..=9).for_each(|number| {
                    let number_string = if value.contains(&(number as u8)) { number.to_string() } else {" ".to_string()};
                    result.push_str(&number_string);
                });
                //let number_string = value.map(|z| z.to_string()).to_owned();
                //result.push_str(number_string.as_deref().unwrap_or(" "));
                result.push(' ')
            }
            result.push_str(SUDOKU_SEPARATOR_DOUBLE);

            result.push_str("\n");
            result.push_str(if (row + 1) % 3 == 0 {
                if row == 8 { SUDOKU_CANDIDATES_LINE_DOUBLE_BOTTOM } else { SUDOKU_CANDIDATES_LINE_DOUBLE_MIDDLE }
            } else {
                SUDOKU_CANDIDATES_LINE_SINGLE
            });
        }
        result
    }

    pub fn new_from_position(position: &str) -> SudokuGameContext {
        let mut result = SudokuGameContext::new();

        let parsed = position.chars()
            .map(|c| if c != '.' {Some(c.to_digit(10).map(u32::from).unwrap() as u8)} else {None})
            .collect::<Vec<_>>();

        if parsed.len() != 81 {
            panic!("position has to be 81 characters long, not {}", parsed.len());
        }

        for (index, value) in parsed.iter().enumerate() {
            result.filled_numbers[index] = *value;
        }

        result
    }

    pub fn new_from_position_and_candidates(position: &str, user_candidates: &str, solver_candidates: Option<&&str>) -> SudokuGameContext {
        let mut result = SudokuGameContext::new_from_position(position);

        let user_parsed = SudokuGameContext::parse_candidates(user_candidates);
        for (index, value) in user_parsed.iter().enumerate() {
            result.user_candidates[index] = value.clone();
        }

        if solver_candidates.is_none() {
            result.fill_in_solver_candidates();
        } else {
            let solver_parsed = SudokuGameContext::parse_candidates(solver_candidates.unwrap());
            for (index, value) in solver_parsed.iter().enumerate() {
                result.solver_candidates[index] = value.clone();
            }
        }

        result
    }

    fn parse_candidates(candidates: &str) -> Vec<HashSet<u8>> {
        let parsed: Vec<HashSet<u8>> = candidates.split(",")
            .map(|cell| cell.chars().map(|c| c.to_digit(10).map(u32::from).unwrap() as u8).collect::<HashSet<_>>())
            .collect();

        if parsed.len() != 81 {
            panic!("position has to be 81 characters long, not {}", parsed.len());
        }
        parsed
    }

    pub fn fill_in_solver_candidates(&mut self) {
        let candidates = self.compute_candidates();
        for index in 0..81 {
            self.solver_candidates[index].clear();
            self.solver_candidates[index].extend(candidates[index].iter());
        }
    }

    pub fn compute_candidates(&self) -> Vec<HashSet<u8>> {
        let mut candidates_result: Vec<HashSet<u8>> = (0..81).map(|_| HashSet::<u8>::new()).collect();
        let all_numbers_set: HashSet<u8> = (1..=9).collect();

        let mut rows: Vec<_> = (0..9).map(|_| HashSet::<u8>::new()).collect();
        let mut columns: Vec<_> = (0..9).map(|_| HashSet::<u8>::new()).collect();
        let mut boxes: Vec<_> = (0..9).map(|_| HashSet::<u8>::new()).collect();

        for indices in SudokuBoardX(0) {
            let value = self.filled_numbers[indices.index];
            if value.is_some() {
                let v = value.unwrap();
                rows[indices.row].insert(v);
                columns[indices.column].insert(v);
                boxes[indices.box_index].insert(v);
            }
        }

        let negated_rows: Vec<HashSet<u8>> = rows.iter().map(|c| &all_numbers_set - c).collect();
        let negated_columns: Vec<HashSet<u8>> = columns.iter().map(|c| &all_numbers_set - c).collect();
        let negated_boxes: Vec<HashSet<u8>> = boxes.iter().map(|c| &all_numbers_set - c).collect();

        for indices in SudokuBoardX(0) {
            let value = self.filled_numbers[indices.index];
            if value.is_none() {
                candidates_result[indices.index] = &(&negated_rows[indices.row]
                    & &negated_columns[indices.column])
                    & &negated_boxes[indices.box_index];
            } else {
                candidates_result[indices.index].clear();
            }
        }
        candidates_result
    }

    pub fn get_solver_candidate_sections(&self, index: usize, include_focused_cell: bool) -> Vec<ListWithType> {
        SudokuGameContext::get_candidate_sections(index, include_focused_cell, &self.solver_candidates)
    }

    pub fn get_candidate_sections(index: usize, include_focused_cell: bool, solver_candidates: &Vec<HashSet<u8>>)
        -> Vec<ListWithType> {
        let row = (index / 9) as usize;
        let column = (index % 9) as usize;
        let mut ch: ListWithType = ListWithType::new(EnumSectionType::Column);
        let mut rh: ListWithType = ListWithType::new(EnumSectionType::Row);
        let mut sh: ListWithType = ListWithType::new(EnumSectionType::Box);

        for i in 0..9 as usize {
            if include_focused_cell || i != row {
                let i1 = i * 9 + column;
                ch.list.push(SetWithRef::new(&solver_candidates[i1], i1));
            }
            if include_focused_cell || i != column {
                let i2 = row * 9 + i;
                rh.list.push(SetWithRef::new(&solver_candidates[i2], i2));
            }
        }

        for box_index in get_box_indices(row, column) {
            if include_focused_cell || index != box_index {
                sh.list.push(SetWithRef::new(&solver_candidates[box_index], box_index));
            }
        }

        let result = vec![ch, rh, sh];
        result
    }

    pub fn is_solved(&self) -> bool {
        for i in 0..81 as usize {
            //println!("Line: {}, value: {}", i, self.filled_numbers[i].unwrap_or(255));
            if self.filled_numbers[i].is_none() {
                return false;
            }
        }
        true
    }

    pub fn count_frequency_of_filled_numbers(&self, set: HashSet<usize>) -> Vec<u8> {
        let mut counts: Vec<u8> = vec![0; 10];
        for index in set {
            let number = self.filled_numbers[index];
            if number.is_some() {
                counts[number.unwrap() as usize] += 1;
            }
        }
        counts
    }

    pub fn get_game_descriptor(&self) -> String {
        (0..81).map(|x| self.filled_numbers[x])
            //.collect();
            .map(|x| x.map(|y|y.to_string()))
            .map(|x| x.unwrap_or(".".to_string()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_in_solver_candidates() {
        let mut sudoku_game_context = SudokuGameContext::new_from_position(
            "..725.3..1....3..7.6......5.....867..3.......84...5......34.2.8...58.7..6.8....1.");
        sudoku_game_context.fill_in_solver_candidates();
        let candidates0 = sudoku_game_context.solver_candidates.get(0).unwrap();
        assert!(candidates0.contains(&4));
        eprintln!("{}", sudoku_game_context.get_text_board());
        eprintln!("{}", sudoku_game_context.get_text_candidates(&sudoku_game_context.solver_candidates));
    }
}

