use std::{
    fs,
    io::{ Result, ErrorKind },
    path::{ Path, PathBuf }
};



/// A file pattern
pub trait FilePattern {
    /// Tests whether a given data matches a pattern or not
    fn matches<D>(&self, data: D) -> bool where D: AsRef<[u8]>;
}


/// A pattern that matches files with a specific extension
#[derive(Debug, Clone)]
pub struct FileExtensionPattern {
    /// The file extension to match
    extension: Vec<u8>
}
impl FileExtensionPattern {
    /// Creates a new file extension pattern
    pub fn new<E>(extension: E) -> Self where E: Into<Vec<u8>> {
        Self { extension: extension.into() }
    }
}
impl FilePattern for FileExtensionPattern {
    fn matches<D>(&self, data: D) -> bool where D: AsRef<[u8]> {
        data.as_ref().ends_with(&self.extension)
    }
}


/// Lists all files non-recursively within `directory` whose names match `pattern`
pub fn list_files<D>(directory: D) -> Result<Vec<PathBuf>> where D: AsRef<Path> {
    // Collect all entries
    let mut entries = Vec::new();
    for entry in fs::read_dir(directory)? {
        // Unwrap the directory entry
        let entry = entry?;
        let path = entry.path();
        
        // Check if the entry is a file
        if path.is_file() {
            entries.push(path);
        }
    }
    Ok(entries)
}


/// Writes a file atomically
pub fn write_atomic<D, F>(data: D, path: F) -> Result<()> where D: AsRef<[u8]>, F: AsRef<Path> {
    // Create the path and temp path
    let path = path.as_ref();
    let temp_path = {
        // Assemble the temp path
        let mut name = path.file_name().ok_or(ErrorKind::NotFound)?
            .to_os_string();
        name.push(".tmp");
        path.with_file_name(name)
    };
    
    // Write the file
    fs::write(&temp_path, data)?;
    fs::rename(&temp_path, &path)?;
    Ok(())
}


/// Converts a path into a sequence of raw bytes
/// 
/// # Note
/// This method is designed to never panic under `#[cfg(unix)]` or `#[cfg(windows)]` as it accesses the raw representation
/// (as opposed to the usual workaround via `String` which will panic if the path is not valid UTF-8)
pub fn path_bytes<P>(path: P) -> Vec<u8> where P: AsRef<Path> {
    // Unix specific path routine
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;

        let os_str = path.as_ref().as_os_str();
        return os_str.as_bytes().to_vec();
    }

    // Windows specifc path routine
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        
        let mut path_bytes = Vec::new();
        path.as_ref().as_os_str().encode_wide()
            .for_each(|c16| path_bytes.extend(c16.to_le_bytes()));
        return path_bytes;
    }
    
    // Fallback to generic but fallible method
    #[allow(unused)]
    {
        let path_str = path.as_ref().to_str().expect("Path contains non-UTF-8 sequences");
        return path_str.as_bytes().to_vec();
    }
}
