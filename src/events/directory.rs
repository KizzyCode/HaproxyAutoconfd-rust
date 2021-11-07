use crate::{
    events::EventSource,
    fsext::{ self, FilePattern }
};
use sha2::{ Sha512, Digest };
use std::{
    fs, thread, path::PathBuf, time::Duration,
    sync::{
        Arc, mpsc::Sender,
        atomic::{ AtomicBool, Ordering }
    }
};


/// An asynchronous directory monitor event source
struct DirectoryEventSourceImpl<T, P> {
    /// The directory to monitor
    directory: PathBuf,
    /// The file pattern to match
    pattern: P,
    /// A flag that signalizes that the event source should be active
    active: Arc<AtomicBool>,
    /// The event message
    message: T,
    /// The event channel
    channel: Sender<T>
}
impl<T, P> DirectoryEventSourceImpl<T, P> where T: Clone + Send + 'static, P: FilePattern {
    /// Creates a new asynchronous signal event source
    pub fn start<D>(directory: D, pattern: P, active: Arc<AtomicBool>, message: T, channel: Sender<T>)
        where D: Into<PathBuf>, P: Send + 'static
    {
        let this = Self { directory: directory.into(), pattern, active, message, channel };
        thread::spawn(|| this.runloop());
    }

    /// The runloop
    fn runloop(self) {
        // Get the current directory hash
        let mut current_dirhash = self.dirhash();

        // Loop as long as the event source is valid
        'runloop: while self.active.load(Ordering::Relaxed) {
            // Check the current dirhash
            let dirhash = self.dirhash();
            if dirhash != current_dirhash {
                // Send the eveent message
                let message = self.message.clone();
                if let Err(_) = self.channel.send(message) {
                    break 'runloop;
                }
            }
            current_dirhash = dirhash;

            // Sleep some time
            const SPINLOOP_INTERVAL: Duration = Duration::from_millis(1500);
            thread::sleep(SPINLOOP_INTERVAL);
        }
    }

    /// Recursively computes a hash over all files within `directory` whose names match `pattern`
    fn dirhash(&self) -> Vec<u8> {
        // List and sort the entries
        let mut files: Vec<_> = fsext::list_files(&self.directory).expect("Failed to list directory")
            .into_iter().map(|p| p.canonicalize().expect("Failed to canonicalize path"))
            .collect();
        files.sort();
        
        // Hash the entries
        let mut sha512 = Sha512::new();
        'read_loop: for path in files {
            // Check if the path matches the pattern
            let name = path.components().last().expect("Path has no last component");
            let name_bytes = fsext::path_bytes(&name);
            if !self.pattern.matches(&name_bytes) {
                continue 'read_loop;
            }
            
            // Hash the filename
            let path_bytes = fsext::path_bytes(&path);
            sha512.update(&path_bytes);
            sha512.update(&path_bytes.len().to_be_bytes());

            // Read and hash the file
            let data = fs::read(&path).expect("Failed to read file");
            sha512.update(&data);
            sha512.update(&data.len().to_be_bytes());
        }
        Vec::from(sha512.finalize().as_slice())
    }
}


/// A directory monitor event source
/// 
/// # Note
/// This is an el-cheapo implementation that is useful to periodically scan (1.5 seconds) if e.g. a configuration has
/// changed. It is NOT suitable for directories containing large files or persistent monitoring.
pub struct DirectoryEventSource<P> {
    /// The directory to monitor
    directory: PathBuf,
    /// The pattern
    pattern: P,
    /// A flag that signalizes that the event source should be active
    active: Arc<AtomicBool>
}
impl<P> DirectoryEventSource<P> {
    /// Creates a new FS monitor for a given directory
    pub fn new<D>(directory: D, pattern: P) -> Self where D: Into<PathBuf>, P: FilePattern {
        let active = Arc::new(AtomicBool::new(true));
        Self { directory: directory.into(), pattern, active }
    }
}
impl<T, P> EventSource<T> for DirectoryEventSource<P>
    where T: Clone + Send + 'static, P: FilePattern + Clone + Send + 'static
{
    fn event_attach(&mut self, message: T, channel: Sender<T>) {
        self.active.store(true, Ordering::SeqCst);
        let directory = self.directory.clone();
        let pattern = self.pattern.clone();
        let active = self.active.clone();
        DirectoryEventSourceImpl::start(directory, pattern, active, message, channel);
    }
    fn event_cancel(&mut self) {
        self.active.store(false, Ordering::SeqCst);
    }
    fn event_detach(&mut self) {
        self.active = Arc::new(AtomicBool::new(true));
    }
}
impl<P> Drop for DirectoryEventSource<P> {
    fn drop(&mut self) {
        self.active.store(false, Ordering::Relaxed);
    }
}
