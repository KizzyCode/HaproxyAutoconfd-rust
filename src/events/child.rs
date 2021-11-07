use crate::events::EventSource;
use std::{
    thread, process::Child, time::Duration,
    sync::{
        Arc, Mutex, mpsc::Sender,
        atomic::{ AtomicBool, Ordering }
    }
};


/// An asynchronous child process event source
struct ChildEventSourceImpl<T> {
    /// The managed child process
    child: Arc<Mutex<Child>>,
    /// A flag that signalizes that the event source should be active
    active: Arc<AtomicBool>,
    /// The event message
    message: T,
    /// The event channel
    channel: Sender<T>
}
impl<T> ChildEventSourceImpl<T> where T: Clone + Send + 'static {
    /// Creates a new asynchronous signal event source
    pub fn start(child: Arc<Mutex<Child>>, active: Arc<AtomicBool>, message: T, channel: Sender<T>) {
        let this = Self { child, active, message, channel };
        thread::spawn(|| this.runloop());
    }

    /// The runloop
    fn runloop(self) {
        // Loop as long as the event source is valid
        'runloop: while self.active.load(Ordering::Relaxed) {
            // Get the state of the child process
            let exit_state = {
                let mut child = self.child.lock().expect("Failed to lock mutex to access child process");
                child.try_wait().expect("Failed to query child state")
            };

            // The child is dead
            if let Some(_) = exit_state {
                // Send the event message
                let message = self.message.clone();
                if let Err(_) = self.channel.send(message) {
                    break 'runloop;
                }
            }

            // Sleep some time
            const SPINLOOP_INTERVAL: Duration = Duration::from_millis(100);
            thread::sleep(SPINLOOP_INTERVAL);
        }
    }
}


/// An child process event source
pub struct ChildEventSource {
    /// The child process
    child: Arc<Mutex<Child>>,
    /// A flag that signalizes that the event source should be active
    active: Arc<AtomicBool>
}
impl ChildEventSource {
    /// Creates a new child process event source
    pub fn new(child: Arc<Mutex<Child>>) -> Self {
        let active = Arc::new(AtomicBool::new(true));
        Self { child, active }
    }
}
impl<T> EventSource<T> for ChildEventSource where T: Clone + Send + 'static {
    fn event_attach(&mut self, message: T, channel: Sender<T>) {
        self.active.store(true, Ordering::SeqCst);
        let child = self.child.clone();
        let active = self.active.clone();
        ChildEventSourceImpl::start(child, active, message, channel);
    }
    fn event_cancel(&mut self) {
        self.active.store(false, Ordering::SeqCst);
    }
    fn event_detach(&mut self) {
        self.active = Arc::new(AtomicBool::new(true));
    }
}
impl Drop for ChildEventSource {
    fn drop(&mut self) {
        self.active.store(false, Ordering::Relaxed);
    }
}
