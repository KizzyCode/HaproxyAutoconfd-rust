pub mod child;
pub mod signals;
pub mod directory;

use std::sync::mpsc::Sender;


/// An asynchronous event source
pub trait EventSource<T> where T: Clone + Send + 'static {
    /// Attaches a new event producer that sends `message` over `channel` if an event occurrs
    fn event_attach(&mut self, message: T, channel: Sender<T>);
    /// Cancels all attached event producers
    fn event_cancel(&mut self);
    /// Detaches all attached event producers (i.e. the event source will not be canceled if it is dropped)
    fn event_detach(&mut self);
}
