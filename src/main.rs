mod fsext;
mod events;
mod child;
mod config;

use crate::{
    config::Config, fsext::FileExtensionPattern, child::ChildProcess,
    events::{ EventSource, directory::DirectoryEventSource, signals::SignalEventSource }
};
use std::{
    process,
    sync::mpsc::{ self, Sender, Receiver }
};


/// The HAProxy config dir
const CONFIG_DIR: &'static str = "/usr/local/etc/haproxy.inbox";
/// The name of the assembled config file
const CONFIG_FILE: &'static str = "/usr/local/etc/haproxy/haproxy.cfg";
/// The extension for config files
const CONFIG_FILE_EXT: &'static str = ".cfg";
/// The haproxy binary
const HAPROXY_BIN: &'static str = "/usr/local/sbin/haproxy";


/// An event
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Event {
    /// A directory change event
    Directory,
    /// A termination signal event
    Signal,
    /// A process event
    Child
}
impl Event {
    /// Creates an event channel pair
    pub fn make_channels() -> (Sender<Self>, Receiver<Self>) {
        mpsc::channel()
    }
}


pub fn main() {
    // Create the config handler
    let config_file_pattern = FileExtensionPattern::new(CONFIG_FILE_EXT);
    let config = Config::new(CONFIG_DIR, CONFIG_FILE, config_file_pattern.clone());
    
    // Assemble the config for the first time and launch HAProxy
    config.assemble();
    let haproxy = ChildProcess::new(HAPROXY_BIN, ["-f", CONFIG_FILE]);

    // Create the event sources
    let mut directory_event_source = DirectoryEventSource::new(CONFIG_DIR, config_file_pattern.clone());
    let mut signal_event_source = SignalEventSource::new();
    let mut child_event_source = haproxy.event_source();

    // Register the event sources
    let (event_sender, event_channel) = Event::make_channels();
    directory_event_source.event_attach(Event::Directory, event_sender.clone());
    signal_event_source.event_attach(Event::Signal, event_sender.clone());
    child_event_source.event_attach(Event::Child, event_sender.clone());

    // Process incoming events
    eprintln!("HAProxy-autoconf is up and running...");
    for event in event_channel {
        match event {
            // Terminate the process
            Event::Signal => {
                eprintln!("Got signal; exiting...");
                haproxy.kill();
                process::exit(0);
            },
            // Handle HAProxy crash
            Event::Child => {
                eprintln!("HAProxy stopped unexpectedly; exiting...");
                process::exit(1);
            },
            // Rebuild the config and restart HAProxy
            Event::Directory => {
                eprintln!("Directory changed; reloading...");
                config.assemble();
                haproxy.restart();
            }
        }
    }
}
