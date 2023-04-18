use stable_eyre::eyre::*;
use std::io::{BufRead, Seek, SeekFrom};
use crate::csv_key::CsvKey;

fn find_matching_line<T: Seek + BufRead>(
    file: &mut T,
    collection: &Vec<&dyn CsvKey>,
) -> Result<Option<usize>> {
    let mut line = String::new();
    loop {
        line.clear();
        match file.read_line(&mut line) {
            Ok(0) => break, // end of file
            Ok(num_bytes) => {
                if line.ends_with('\n') {
                    line.pop();
                }
                for (i, item) in collection.iter().enumerate() {
                    if line == item.get_key() {
                        file.seek(SeekFrom::Current(-(num_bytes as i64)))?;
                        return Ok(Some(i));
                    }
                }
            }
            Err(e) => return Err(eyre!(e)),
        }
    }
    Ok(None)
}


#[cfg(test)]
impl CsvKey for String {
    fn get_key(&self) -> String { self.clone() }
}

mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_find_matching_line() -> Result<()> {
        let mut input = Cursor::new("foo\nzero\none\n");

        let zero = &"zero".to_string() as &dyn CsvKey;
        let one = &"one".to_string() as &dyn CsvKey;
        let collection = vec!(zero, one);

        // should find "zero"
        assert_eq!(find_matching_line(&mut input, &collection)?, Some(0));

        // should find "zero" again
        assert_eq!(find_matching_line(&mut input, &collection)?, Some(0));

        // read past "zero"
        let mut line = String::new();
        input.read_line(&mut line)?;

        // should find "one"
        assert_eq!(find_matching_line(&mut input, &collection)?, Some(1));

        let two = &"two".to_string() as &dyn CsvKey;
        let collection2 = vec!(two);

        // should not find "two"
        assert_eq!(find_matching_line(&mut input, &collection2)?, None);
        Ok(())
    }
}
