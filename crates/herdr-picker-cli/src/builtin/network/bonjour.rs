// Bonjour service types and what each means to a person.
pub(super) const BONJOUR: [(&str, &str); 16] = [
    ("_ssh._tcp", "SSH login"),
    ("_sftp-ssh._tcp", "SFTP"),
    ("_http._tcp", "web page"),
    ("_https._tcp", "secure web page"),
    ("_rfb._tcp", "screen sharing"),
    ("_smb._tcp", "file sharing"),
    ("_afpovertcp._tcp", "Apple file sharing"),
    ("_airplay._tcp", "AirPlay"),
    ("_raop._tcp", "AirPlay audio"),
    ("_ipp._tcp", "printing"),
    ("_printer._tcp", "printing"),
    ("_hap._tcp", "HomeKit"),
    ("_hue._tcp", "Hue"),
    ("_googlecast._tcp", "Chromecast"),
    ("_workstation._tcp", "workstation"),
    ("_companion-link._tcp", "Apple device"),
];
pub(super) fn plain(code: &str) -> &str {
    BONJOUR
        .iter()
        .find(|(known, _)| *known == code)
        .map_or(code, |(_, plain)| plain)
}
