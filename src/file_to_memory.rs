use stable_eyre::eyre::*;
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::PathBuf;

pub fn read_file_to_cursor(path: &PathBuf) -> Result<Cursor<Vec<u8>>> {
    // Open the file in read-only mode.
    let mut file =
        File::open(path).with_context(|| format!("Unable to open file : {:?}", path.to_str()))?;

    // Read the entire contents of the file into a vector.
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .with_context(|| format!("Unable to read to end of file : {:?}", path.to_str()))?;

    // Create a cursor that accesses the vector.
    Ok(Cursor::new(buffer))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_read_file_to_cursor() -> Result<()> {
        // Create a temporary file with some contents.
        let tmpfile = tempfile::NamedTempFile::new()?;
        let contents = b"Hello, world!";
        tmpfile.as_file().write_all(contents)?;

        // Read the file into a cursor.
        let mut cursor = read_file_to_cursor(&tmpfile.path().to_owned())?;

        // Verify that the cursor contains the file's contents.
        let mut buf = vec![0; contents.len()];
        cursor.read_exact(&mut buf)?;
        assert_eq!(buf, contents);
        Ok(())
    }

    #[test]
    fn test_read_file_to_cursor_error() {
        // Attempt to read a non-existent file.
        let result = read_file_to_cursor(&PathBuf::from("/path/to/missing/file"));
        assert!(result.is_err());
    }
}
