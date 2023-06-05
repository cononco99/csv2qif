use stable_eyre::eyre::*;
use std::collections::HashMap;
use std::io::{BufRead, Seek, SeekFrom};

pub fn find_matching_line<V: Copy, T: Seek + BufRead>(
    file: &mut T,
    collection: &HashMap<String, V>,
) -> Result<Option<V>> {
    let mut line = String::new();
    loop {
        line.clear();
        match file.read_line(&mut line) {
            Ok(0) => break, // end of file
            Ok(num_bytes) => {
                if line.ends_with('\n') {
                    line.pop();
                }
                for (key, value) in collection {
                    if line == *key {
                        file.seek(SeekFrom::Current(-(num_bytes as i64)))?;
                        return Ok(Some(*value));
                    }
                }
            }
            Err(e) => return Err(eyre!(e)),
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_find_matching_line() -> Result<()> {
        let mut input = Cursor::new("foo\nzero\none\n");

        let zero = "zero".to_string();
        let one = "one".to_string();
        // let collection = vec![zero, one];
        let collection = HashMap::from([(zero, 0), (one, 1)]);

        // should find "zero"
        assert_eq!(find_matching_line(&mut input, &collection)?, Some(0));

        // should find "zero" again
        assert_eq!(find_matching_line(&mut input, &collection)?, Some(0));

        // read past "zero"
        let mut line = String::new();
        input.read_line(&mut line)?;

        // should find "one"
        assert_eq!(find_matching_line(&mut input, &collection)?, Some(1));

        let two = "two".to_string();
        let collection2 = HashMap::from([(two, 2)]);

        // should not find "two"
        assert_eq!(find_matching_line(&mut input, &collection2)?, None);
        Ok(())
    }
}
