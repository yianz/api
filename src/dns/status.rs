/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Blocking Status Endpoint
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use config::{Config, PiholeFile};
use std::io::{self, BufRead, BufReader};
use std::fs::File;
use rocket::State;
use util;

/// Get the DNS blocking status
#[get("/dns/status")]
pub fn status(config: State<Config>) -> util::Reply {
    let file = config.read_file(PiholeFile::DnsmasqMainConfig);

    let status = match file {
        Ok(f) => check_for_gravity(f)?,

        // If we failed to open the file, then the status is unknown
        Err(_) => "unknown"
    };

    util::reply_data(json!({
        "status": status
    }))
}

/// Check a file for the `addn-hosts=/.../gravity.list` line and return the blocking status
fn check_for_gravity<'a>(file: File) -> io::Result<&'a str> {
    // Read the file to a buffer
    let reader = BufReader::new(file);

    // Check for the gravity.list line
    for line in reader.lines().filter_map(|item| item.ok()) {
        if line == "#addn-hosts=/etc/pihole/gravity.list" {
            return Ok("disabled");
        } else if line == "addn-hosts=/etc/pihole/gravity.list" {
            return Ok("enabled");
        }
    }

    Ok("unknown")
}

#[cfg(test)]
mod test {
    use config::PiholeFile;
    use testing::TestConfig;

    #[test]
    fn test_status_enabled() {
        TestConfig::new()
            .endpoint("/admin/api/dns/status")
            .file(PiholeFile::DnsmasqMainConfig, "addn-hosts=/etc/pihole/gravity.list")
            .expect_json(
                json!({
                    "data": {
                        "status": "enabled"
                    },
                    "errors": []
                })
            )
            .test();
    }

    #[test]
    fn test_status_disabled() {
        TestConfig::new()
            .endpoint("/admin/api/dns/status")
            .file(PiholeFile::DnsmasqMainConfig, "#addn-hosts=/etc/pihole/gravity.list")
            .expect_json(
                json!({
                    "data": {
                        "status": "disabled"
                    },
                    "errors": []
                })
            )
            .test();
    }

    #[test]
    fn test_status_unknown() {
        TestConfig::new()
            .endpoint("/admin/api/dns/status")
            .file(PiholeFile::DnsmasqMainConfig, "random data...")
            .expect_json(
                json!({
                    "data": {
                        "status": "unknown"
                    },
                    "errors": []
                })
            )
            .test();
    }
}
