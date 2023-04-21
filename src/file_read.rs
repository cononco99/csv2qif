// This code never got far and probably never will be used.
use stable_eyre::eyre::*;
use std::io::Seek;
use std::io::BufRead;

fn find_line<T : BufRead + Seek>(file: &mut T, collection: &[&str]) -> Result<Option<usize>> {
    let mut line_number = 0;
    let mut line = String::new();
    loop {
        let bytes_read = file.read_line(&mut line)?;
        if bytes_read == 0 {
            break;
        }
        line_number += 1;
        for (i, &s) in collection.iter().enumerate() {
            if line == s {
                return Ok(Some(i));
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Error;
    use std::io::ErrorKind;

    #[test]
    fn test_find_line_with_matching_line() {
        let file = File::open("./test.txt").unwrap();
        let collection = ["hello", "world"];

        let result = find_line(&mut file, &collection).unwrap();
        assert_eq!(result, Some(1));
    }

    #[test]
    fn test_find_line_with_no_matching_line() {
        let file = File::open("./test.txt").unwrap();
        let collection = ["foo", "bar"];

        let result = find_line(&mut file, &collection).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_line_with_error() {
        let file = File::open("./not_found.txt").unwrap_err();
        let collection = ["hello", "world"];

        let result = find_line(&mut file, &collection).unwrap_err();
        assert_eq!(result, eyre!(Error::new(ErrorKind::NotFound, "File not found")));
    }
}
