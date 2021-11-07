use crate::events::child::ChildEventSource;
use std::{
    process::{ Command, Child },
    sync::{ Arc, Mutex }
};


/// A simple process handle
pub struct ChildProcess {
    /// The path to the binary to execute
    binary: String,
    /// The arguments to pass during execution
    args: Vec<String>,
    /// The child process
    child: Arc<Mutex<Child>>
}
impl ChildProcess {
    /// Spawns a new process
    pub fn new<B, A, AT>(binary: B, args: A) -> Self where B: ToString, A: IntoIterator<Item = AT>, AT: ToString {
        // Collect the process info
        let binary = binary.to_string();
        let args: Vec<_> = args.into_iter().map(|a| a.to_string()).collect();
        
        // Spawn the child
        let child = Command::new(&binary).args(&args).spawn().expect("Failed to spawn process");
        Self { binary, args, child: Arc::new(Mutex::new(child)) }
    }

    /// Restarts the child process
    pub fn restart(&self) {
        let mut child = self.child.lock().expect("Failed to lock mutex to access child process");
        let _ = child.kill();
        *child = Command::new(&self.binary).args(&self.args).spawn().expect("Failed to spawn process");
    }
    /// Creates a new child process event source for crash events
    pub fn event_source(&self) -> ChildEventSource {
        ChildEventSource::new(self.child.clone())
    }
    /// Sends a SIGKILL to the child
    pub fn kill(self) {
        drop(self);
    }
}
impl Drop for ChildProcess {
    fn drop(&mut self) {
        if let Ok(mut child) = self.child.lock() {
            let _ = child.kill();
        }
    }
}
