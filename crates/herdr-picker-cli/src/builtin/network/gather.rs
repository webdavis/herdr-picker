use std::fs;
use std::net::ToSocketAddrs;
use std::thread;
use std::time::Duration;

use herdr_picker_adapters::UnifiSettings;
use herdr_picker_domain::PickerRow;

use super::bonjour::BONJOUR;
use super::device_page::Clock;
use super::device_row::device_row;
use super::devices::Devices;
use super::interfaces::{gateway, networks, own_addresses};
use super::scan::{ScanFiles, parse_scan, scan_row};
use super::sightings::Source;
use super::sightings::{Sighting, arp, browsed, ndp, reached};
use super::unifi::unifi_sightings;
use crate::builtin::page::notice;
use crate::builtin::program::{ran, run};

const QUICK: Duration = Duration::from_secs(3);

// Every source asked at once, then merged: UniFi, this Mac's neighbour tables, Bonjour and the last scan.
pub(super) fn device_rows(
    unifi: Option<&UnifiSettings>,
    home: Option<&str>,
    files: &ScanFiles,
    clock: Clock,
) -> Vec<PickerRow> {
    let (tables, bonjour, (from_unifi, mut rows), ifconfig, computer, routes) =
        thread::scope(|scope| {
            let tables = scope.spawn(|| ["arp", "ndp"].map(|table| run(&[table, "-an"], QUICK)));
            let bonjour = scope.spawn(bonjour);
            let unifi = scope.spawn(|| unifi_sightings(unifi, home));
            let ifconfig = scope.spawn(|| run(&["ifconfig"], QUICK).unwrap_or_default());
            let computer = scope
                .spawn(|| run(&["scutil", "--get", "ComputerName"], QUICK).unwrap_or_default());
            let routes = run(&["netstat", "-rn", "-f", "inet"], QUICK).unwrap_or_default();
            (
                joined(tables),
                joined(bonjour),
                joined(unifi),
                joined(ifconfig),
                joined(computer),
                routes,
            )
        });
    let mut devices = Devices::default();
    let [arped, ndped] = tables;
    for (table, listed, parse) in [
        ("arp", arped, arp as fn(&str) -> Vec<Sighting>),
        ("ndp", ndped, ndp),
    ] {
        match listed {
            Ok(listed) => parse(&listed)
                .into_iter()
                .for_each(|seen| devices.add(Source::ThisMac, seen)),
            Err(why) => rows.push(notice(&format!("{table} unavailable"), &why)),
        }
    }
    bonjour
        .into_iter()
        .for_each(|seen| devices.add(Source::Bonjour, seen));
    from_unifi
        .into_iter()
        .for_each(|seen| devices.add(Source::UniFi, seen));
    let read = |path| fs::read_to_string(path).unwrap_or_default();
    let scan = parse_scan(&read(&files.xml), &read(&files.report), clock.now);
    if let Some(scan) = &scan {
        scan.hosts
            .iter()
            .cloned()
            .for_each(|seen| devices.add(Source::Scan, seen));
        rows.push(scan_row(scan, clock));
    }
    devices.mark_this_mac(&own_addresses(&ifconfig), computer.trim());
    devices.link_uplinks();
    let (gateway, networks) = (gateway(&routes), networks(&ifconfig));
    rows.extend(
        devices
            .0
            .iter()
            .map(|device| device_row(device, &gateway, &networks, clock)),
    );
    rows
}

// Each announced instance, found by browsing every service at once, then where it is reached.
fn joined<T>(handle: thread::ScopedJoinHandle<'_, T>) -> T {
    handle.join().expect("a source thread")
}

fn bonjour() -> Vec<Sighting> {
    let bounded = |argv: &[&str], seconds: f64| {
        ran(argv, None, Duration::from_secs_f64(seconds))
            .map(|done| done.stdout)
            .unwrap_or_default()
    };
    let mut instances: Vec<(String, &str, String)> = thread::scope(|scope| {
        let asking: Vec<_> = BONJOUR
            .iter()
            .map(|(service, _)| {
                scope.spawn(move || (*service, bounded(&["dns-sd", "-B", service, "local."], 1.2)))
            })
            .collect();
        asking
            .into_iter()
            .flat_map(|answer| {
                let (service, said) = answer.join().unwrap_or_default();
                browsed(&said)
                    .into_iter()
                    .map(move |(name, domain)| (name, service, domain))
            })
            .collect()
    });
    instances.dedup();
    thread::scope(|scope| {
        let resolving: Vec<_> = instances
            .iter()
            .map(|(name, service, domain)| {
                scope.spawn(move || {
                    let host = reached(&bounded(&["dns-sd", "-L", name, service, domain], 1.0))?;
                    let ip = (host.as_str(), 0)
                        .to_socket_addrs()
                        .ok()
                        .and_then(|mut found| found.next())
                        .map(|found| found.ip().to_string());
                    Some(Sighting {
                        ip: ip.unwrap_or_default(),
                        name: name.clone(),
                        host,
                        offers: vec![service.to_string()],
                        ..Sighting::default()
                    })
                })
            })
            .collect();
        resolving
            .into_iter()
            .filter_map(|answer| answer.join().ok().flatten())
            .collect()
    })
}
