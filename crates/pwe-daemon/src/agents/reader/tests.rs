use super::*;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use crate::arrival::Arrival;

const PATIENCE: Duration = Duration::from_secs(30);

fn counting_reader() -> (ReadAgents, Arc<Mutex<u32>>) {
    let readings = Arc::new(Mutex::new(0));
    let counted = Arc::clone(&readings);
    let read: ReadAgents = Box::new(move || {
        if let Ok(mut readings) = counted.lock() {
            *readings += 1;
        }
        AgentSources::default()
    });
    (read, readings)
}

#[test]
fn a_reading_arrives_when_asked_and_the_reader_ends_with_its_askers() {
    let (read, readings) = counting_reader();
    let (asks, asked) = mpsc::sync_channel(1);
    let (arrivals, arrived) = mpsc::sync_channel(8);
    let reader = AgentReader::start(read, 15, asked, arrivals);

    let _ = asks.try_send(());
    let first = arrived.recv_timeout(PATIENCE);
    drop(asks);
    reader.stop();

    assert!(matches!(first, Ok(Arrival::AgentsRead(_))));
    assert_eq!(readings.lock().map(|count| *count).ok(), Some(1));
}

#[test]
fn asks_within_one_refresh_make_one_more_reading() {
    let (read, readings) = counting_reader();
    let (asks, asked) = mpsc::sync_channel(1);
    let (arrivals, arrived) = mpsc::sync_channel(8);
    let reader = AgentReader::start(read, 1, asked, arrivals);
    let _ = asks.try_send(());
    let _ = arrived.recv_timeout(PATIENCE);

    for _ in 0..5 {
        let _ = asks.try_send(());
    }
    let second = arrived.recv_timeout(PATIENCE);
    let third = arrived.recv_timeout(Duration::from_secs(3));
    drop(asks);
    reader.stop();

    assert!(second.is_ok());
    assert!(third.is_err());
    assert_eq!(readings.lock().map(|count| *count).ok(), Some(2));
}
