use super::*;

#[test]
fn a_complete_response_before_the_deadline_succeeds() {
    let (mut client, mut peer) = UnixStream::pair().expect("owned socket pair");
    let response = "{\"result\":{\"type\":\"ok\"}}\n";
    peer.write_all(response.as_bytes())
        .expect("complete response");
    let expires = Instant::now() + Duration::from_millis(150);
    assert_eq!(
        read_response(&mut client, expires),
        Ok(response.to_string())
    );
}

#[test]
fn a_complete_buffered_response_survives_peer_close() {
    let (mut client, mut peer) = UnixStream::pair().expect("owned socket pair");
    let response = "{\"result\":{\"type\":\"ok\"}}\n";
    peer.write_all(response.as_bytes()).expect("buffered reply");
    drop(peer);
    assert_eq!(
        read_response(&mut client, Instant::now() + Duration::from_millis(150)),
        Ok(response.to_string())
    );
}

#[test]
fn a_peer_that_does_not_read_cannot_hold_a_write_past_its_budget() {
    let (mut client, peer) = UnixStream::pair().expect("owned socket pair");
    let (stop, stopped) = std::sync::mpsc::channel();
    let thread = std::thread::spawn(move || {
        let _peer = peer;
        let _ = stopped.recv_timeout(Duration::from_millis(400));
    });
    let started = Instant::now();
    let result = write_request(
        &mut client,
        &vec![b'x'; 4 * 1024 * 1024],
        started + Duration::from_millis(80),
    );
    let elapsed = started.elapsed();
    let _ = stop.send(());
    thread.join().expect("owned peer stopped");
    assert!(matches!(result, Err(JumpError::Transport(_))));
    assert!(elapsed < Duration::from_millis(300), "{elapsed:?}");
}

#[test]
fn an_expired_request_budget_prevents_any_write() {
    let (mut client, mut peer) = UnixStream::pair().expect("owned socket pair");
    peer.set_nonblocking(true).expect("bounded observation");
    let result = write_request(&mut client, b"request\n", Instant::now());
    assert!(matches!(result, Err(JumpError::Transport(_))));
    let error = peer.read(&mut [0]).expect_err("no request bytes");
    assert_eq!(error.kind(), std::io::ErrorKind::WouldBlock);
}
