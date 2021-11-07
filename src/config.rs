use crate::fsext::{ self, FilePattern };
use std::{ fs, path::PathBuf };


/// A config file manager
pub struct Config<P> {
    /// The directory containing the config file fragments
    directory: PathBuf,
    /// The path to the final config file
    file: PathBuf,
    /// The file extension pattern for config files
    pattern: P
}
impl<P> Config<P> {
    /// Creates a new config file manager
    pub fn new<D, F>(directory: D, file: F, pattern: P) -> Self
        where D: Into<PathBuf>, F: Into<PathBuf>
    {
        Self { directory: directory.into(), file: file.into(), pattern }
    }

    /// Assembles the config
    pub fn assemble(&self) where P: FilePattern {
        // List and sort the entries
        let mut files: Vec<_> = fsext::list_files(&self.directory).expect("Failed to list directory")
            .into_iter().map(|p| p.canonicalize().expect("Failed to canonicalize path"))
            .collect();
        files.sort();

        // Read all config files
        let mut config = Vec::new();
        'read_loop: for path in files {
            // Check if the path matches the pattern
            let name = path.components().last().expect("Path has no last component");
            let name_bytes = fsext::path_bytes(&name);
            if !self.pattern.matches(&name_bytes) {
                continue 'read_loop;
            }

            // Read the file
            let data = fs::read(&path).expect("Failed to read file");
            config.extend(data);
        }

        // Write the config file
        fsext::write_atomic(config, &self.file).expect("Failed to write config file");
    }
}

