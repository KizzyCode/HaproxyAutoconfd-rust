use crate::events::EventSource;
use signal_hook::consts::{ SIGTERM, SIGQUIT, SIGINT };
use std::{
    thread, time::Duration,
    sync::{
        Arc, mpsc::Sender,
        atomic::{ AtomicBool, Ordering }
    }
};


/// An asynchronous signal event source
struct SignalEventSourceImpl<T> {
    /// A flag that signalizes that the event source should be active
    active: Arc<AtomicBool>,
    /// The event message
    message: T,
    /// The event channel
    channel: Sender<T>
}
impl<T> SignalEventSourceImpl<T> where T: Clone + Send + 'static {
    /// Creates a new asynchronous signal event source
    pub fn start(active: Arc<AtomicBool>, message: T, channel: Sender<T>) {
        let this = Self { active, message, channel };
        thread::spawn(|| this.runloop());
    }

    /// The runloop
    fn runloop(self) {
        // Register the signals
        let flag = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(SIGTERM, flag.clone()).expect("Failed to register unix signal handler");
        signal_hook::flag::register(SIGQUIT, flag.clone()).expect("Failed to register unix signal handler");
        signal_hook::flag::register(SIGINT, flag.clone()).expect("Failed to register unix signal handler");

        // Loop as long as the event source is valid
        'runloop: while self.active.load(Ordering::Relaxed) {
            // The signal has occurred
            if flag.swap(false, Ordering::Relaxed) {
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


/// A unix signal event source that traps all termination signals (SIGINT, SIGTERM, SIGQUIT, etc.)
pub struct SignalEventSource {
    /// A flag that signalizes that the event source should be active
    active: Arc<AtomicBool>
}
impl SignalEventSource {
    /// Creates a new signal event source
    pub fn new() -> Self {
        let active = Arc::new(AtomicBool::new(true));
        Self { active }
    }
}
impl<T> EventSource<T> for SignalEventSource where T: Clone + Send + 'static {
    fn event_attach(&mut self, message: T, channel: Sender<T>) {
        self.active.store(true, Ordering::SeqCst);
        let active = self.active.clone();
        SignalEventSourceImpl::start(active, message, channel);
    }
    fn event_cancel(&mut self) {
        self.active.store(false, Ordering::SeqCst);
    }
    fn event_detach(&mut self) {
        self.active.store(true, Ordering::SeqCst);
        self.active = Arc::new(AtomicBool::new(true));
    }
}
impl Drop for SignalEventSource {
    fn drop(&mut self) {
        self.active.store(false, Ordering::Relaxed);
    }
}
