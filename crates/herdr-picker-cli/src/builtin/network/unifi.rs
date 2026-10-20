use std::time::Duration;

use herdr_picker_adapters::UnifiSettings;
use herdr_picker_domain::PickerRow;
use serde_json::Value;

use super::controller::{controller, curl_args};
use super::sightings::Sighting;
use crate::builtin::page::notice;
use crate::builtin::program::{ran, run};

const MOST_PAGES: usize = 100;

// Every item of a paged list: the next page starts where the last ended, until the total.
pub(super) fn pages(
    mut get: impl FnMut(usize) -> Result<Value, String>,
) -> Result<Vec<Value>, String> {
    let incomplete = |what: &str| format!("{what}; results are incomplete");
    let mut items = Vec::new();
    for _ in 0..MOST_PAGES {
        let page = get(items.len())?;
        let batch = page["data"]
            .as_array()
            .ok_or_else(|| incomplete("Malformed UniFi page"))?;
        let limit = page["limit"].as_u64().unwrap_or(200) as usize;
        let total = page["totalCount"].as_u64().map(|total| total as usize);
        items.extend(batch.iter().cloned());
        let done = match total {
            Some(total) => items.len() >= total,
            None => batch.len() < limit,
        };
        if done || batch.is_empty() {
            return match total {
                Some(total) if items.len() < total => {
                    Err(incomplete("UniFi returned an empty early page"))
                }
                _ => Ok(items),
            };
        }
    }
    Err(incomplete("UniFi kept paging"))
}

// The controller's devices and clients of every site, or rows saying why there are none.
pub(super) fn unifi_sightings(
    settings: Option<&UnifiSettings>,
    home: Option<&str>,
) -> (Vec<Sighting>, Vec<PickerRow>) {
    let Some(settings) = settings else {
        let detail = "Add a [pickers.network.unifi] table with url and keychain-service to herdr-picker's config to list the controller's devices and clients.";
        return (Vec::new(), vec![notice("UniFi is not configured", detail)]);
    };
    let unavailable = |why: String| {
        notice(
            "UniFi unavailable or incomplete: check the picker config, API access and trusted CA",
            &why,
        )
    };
    let asked = controller(settings, home).and_then(|controller| {
        let key = run(
            &[
                "security",
                "find-generic-password",
                "-w",
                "-s",
                &settings.keychain_service,
            ],
            Duration::from_secs(5),
        )?;
        Ok((
            controller,
            format!("X-API-Key: {}\nAccept: application/json\n", key.trim()),
        ))
    });
    let (controller, headers) = match asked {
        Ok(asked) => asked,
        Err(why) => return (Vec::new(), vec![unavailable(why)]),
    };
    let get = |path: &str| {
        pages(|offset| {
            let args = curl_args(&controller, path, offset);
            let argv: Vec<&str> = args.iter().map(String::as_str).collect();
            let done = ran(&argv, Some(&headers), Duration::from_secs(6))?;
            match done.succeeded {
                true => serde_json::from_str(&done.stdout)
                    .map_err(|_| "UniFi returned invalid JSON".to_string()),
                false => Err(format!("UniFi: {}", done.stderr.trim())),
            }
        })
    };
    let sites = match get("/sites") {
        Ok(sites) => sites,
        Err(why) => return (Vec::new(), vec![unavailable(why)]),
    };
    let (mut seen, mut notices) = (Vec::new(), Vec::new());
    for site in &sites {
        let id = site["id"].as_str().unwrap_or_default();
        let name = site["name"].as_str().unwrap_or(id);
        for category in ["devices", "clients"] {
            match get(&format!("/sites/{id}/{category}")) {
                Ok(items) => seen.extend(
                    items
                        .into_iter()
                        .map(|item| sighting(item, category, name, sites.len())),
                ),
                Err(why) => notices.push(notice(&format!("{name} {category}: {why}"), &why)),
            }
        }
    }
    (seen, notices)
}

fn sighting(item: Value, category: &str, site: &str, sites: usize) -> Sighting {
    let said = |field: &str| item[field].as_str().unwrap_or_default().to_string();
    let (ip, mac, name) = (said("ipAddress"), said("macAddress"), said("name"));
    let mut unifi = item.as_object().cloned().unwrap_or_default();
    unifi.insert("category".into(), category.into());
    unifi.insert("site".into(), site.into());
    unifi.insert("sites".into(), sites.into());
    Sighting {
        ip,
        mac,
        name,
        unifi,
        ..Sighting::default()
    }
}

#[cfg(test)]
mod tests;
