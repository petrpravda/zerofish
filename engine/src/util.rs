use std::str::FromStr;

pub fn extract_parameter<T: FromStr>(parts: &Vec<&str>, name: &str) -> Option<T> {
    match parts.iter().position(|&item| item == name) {
        Some(pos) => {
            if pos + 1 >= parts.len() {
                return None;
            }

            match T::from_str(parts[pos + 1]) {
                Ok(value) => Some(value),
                Err(_) => None,
            }
        }
        None => None,
    }
}
