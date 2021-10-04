use crate::config::global::{
    ACTEURS_FILE, ASN_BLACKLIST_FILE, ASN_LIST_FILE, CONFIG_FILE, INSTANCE, IPV4_BLACKLIST_FILE,
    IPV4_WHITELIST_FILE, PLATFORMS_FILE, SCRUTINS_FILE,
};
use crate::config::structs::{
    ASNBLEntry, ASNLEntry, Acteurs, ConfigFile, IPV4BLEntry, IPV4WLEntry, InstanceInfo, Platform,
    Scrutin,
};
use chrono::offset::Utc;
use std::fs::File;
use std::io::Read;

impl IPV4WLEntry {
    pub fn parse(filename: &str) -> Vec<IPV4WLEntry> {
        let ips_file = File::open(filename).expect("Error: IPV4 whitelist not found");
        let mut reader = csv::ReaderBuilder::new()
            .comment(Some(b'#'))
            .from_reader(ips_file);

        let mut ips_vec: Vec<IPV4WLEntry> = Vec::new();
        for result in reader.records() {
            let result = result.expect("Error: IPV4 whitelist: can't read");

            let ip_start = result
                .get(0)
                .expect("Error: IPV4 whitelist: failed to get column 0")
                .parse::<u32>()
                .expect("Error: IPV4 whitelist: can't parse ip_start to u32");

            let ip_end = result
                .get(1)
                .expect("Error: ASN list: failed to get column 1")
                .parse::<u32>()
                .expect("Error: ASN list: can't parse ip_end to u32");

            ips_vec.push(IPV4WLEntry { ip_start, ip_end });
        }
        ips_vec
    }

    pub fn count(ips_vec: &[IPV4WLEntry]) -> u32 {
        let mut counter: u32 = 0;
        for ip in ips_vec {
            counter += ip.ip_end - ip.ip_start;
        }
        counter
    }
}

impl IPV4BLEntry {
    pub fn parse(filename: &str) -> Vec<IPV4BLEntry> {
        let ips_file = File::open(filename).expect("Error: IPV4 blacklist not found");
        let mut reader = csv::ReaderBuilder::new()
            .comment(Some(b'#'))
            .from_reader(ips_file);

        let mut ips_vec: Vec<IPV4BLEntry> = Vec::new();
        for result in reader.records() {
            let result = result.expect("Error: IPV4 blacklist: can't read");

            let ip_start = result
                .get(0)
                .expect("Error: IPV4 blacklist: failed to get column 0")
                .parse::<u32>()
                .expect("Error: IPV4 blacklist: can't parse ip_start to u32");

            let ip_end_opt = result
                .get(1)
                .expect("Error: IPV4 blacklist: failed to get column 1");

            let ip_end = if ip_end_opt.is_empty() {
                None
            } else {
                Some(
                    ip_end_opt
                        .parse::<u32>()
                        .expect("Error: IPV4 blacklist: can't parse ip_end to u32"),
                )
            };

            let abuse_code = result
                .get(2)
                .expect("Error: IPV4 blacklist: failed to get column 2")
                .parse::<u16>()
                .expect("Error: IPV4 blacklist: can't parse abuse_code to u16");
            ips_vec.push(IPV4BLEntry {
                ip_start,
                ip_end,
                abuse_code,
            });
        }
        ips_vec
    }

    pub fn count(ips_vec: &[IPV4BLEntry]) -> u32 {
        let mut counter: u32 = 0;
        for ip in ips_vec {
            if let Some(ip_end) = ip.ip_end {
                counter += ip_end - ip.ip_start;
            } else {
                counter += 1;
            }
        }
        counter
    }
}

impl ASNLEntry {
    pub fn parse(filename: &str) -> Vec<ASNLEntry> {
        let ips_file = File::open(filename).expect("Error: ASN list not found");
        let mut reader = csv::ReaderBuilder::new()
            .comment(Some(b'#'))
            .from_reader(ips_file);

        let mut ips_vec: Vec<ASNLEntry> = Vec::new();
        for result in reader.records() {
            let result = result.expect("Error: ASN list: can't read");

            let ip_start = result
                .get(0)
                .expect("Error: ASN list: failed to get column 0")
                .parse::<u32>()
                .expect("Error: ASN list: can't parse ip_start to u32");

            let ip_end = result
                .get(1)
                .expect("Error: ASN list: failed to get column 1")
                .parse::<u32>()
                .expect("Error: ASN list: can't parse ip_end to u32");

            let asn = result
                .get(2)
                .expect("Error: ASN list: failed to get column 2")
                .parse::<u32>()
                .expect("Error: ASN list: can't parse ASN to u32");
            ips_vec.push(ASNLEntry {
                ip_start,
                ip_end,
                asn,
            });
        }
        ips_vec
    }
}

impl ASNBLEntry {
    pub fn parse(filename: &str) -> Vec<ASNBLEntry> {
        let ips_file = File::open(filename).expect("Error: ASN blacklist not found");
        let mut reader = csv::ReaderBuilder::new()
            .comment(Some(b'#'))
            .from_reader(ips_file);

        let mut ips_vec: Vec<ASNBLEntry> = Vec::new();
        for result in reader.records() {
            let result = result.expect("Error: ASN blacklist: can't read");

            let asn = result
                .get(0)
                .expect("Error: ASN blacklist: failed to get column 0")
                .parse::<u32>()
                .expect("Error: ASN blacklist: can't parse ASN to u32");

            let abuse_code = result
                .get(1)
                .expect("Error: ASN blacklist: failed to get column 1")
                .parse::<u16>()
                .expect("Error: ASN blacklist: can't parse abuse_code to u16");
            ips_vec.push(ASNBLEntry { asn, abuse_code });
        }
        ips_vec
    }

    pub fn count(asn_blacklist: &[ASNBLEntry], asn_list: &[ASNLEntry]) -> u32 {
        let mut counter: u32 = 0;
        for asn_bl in asn_blacklist {
            for asn_data in asn_list.iter().filter(|a| a.asn == asn_bl.asn) {
                counter += asn_data.ip_end - asn_data.ip_start;
            }
        }
        counter
    }
}

impl InstanceInfo {
    pub fn global() -> &'static InstanceInfo {
        INSTANCE.get().expect("InstanceInfo is not initialized")
    }

    fn read_config(filename: &str) -> ConfigFile {
        // Read configuration file
        let mut config_file = File::open(filename).expect("Error: config.toml file is missing");
        let mut config_str = String::new();

        config_file
            .read_to_string(&mut config_str)
            .expect("Error: couldn't read config.toml to string");

        toml::from_str(&config_str).expect("Error: Couldn't parse the config.toml file")
    }

    fn read_platforms(filename: &str) -> Vec<Platform> {
        let mut platform_file = File::open(filename).expect("Error: Couldn't open platforms file");

        let mut platforms_str = String::new();
        platform_file
            .read_to_string(&mut platforms_str)
            .expect("Error: couldn't read platforms file");

        serde_json::from_str(&platforms_str).expect("Error: Couldn't parse platforms file's JSON")
    }

    fn read_scrutins(filename: &str) -> Vec<Scrutin> {
        let mut scrutin_file = File::open(filename).expect("Error: Couldn't open scrutins file");

        let mut scrutin_str = String::new();
        scrutin_file
            .read_to_string(&mut scrutin_str)
            .expect("Error: couldn't read scrutins file");

        serde_json::from_str(&scrutin_str).expect("Error: Couldn't parse scrutins file's JSON")
    }

    fn read_acteurs(filename: &str) -> Acteurs {
        let mut acteurs_file = File::open(filename).expect("Error: Couldn't open acteurs file");

        let mut acteurs_str = String::new();
        acteurs_file
            .read_to_string(&mut acteurs_str)
            .expect("Error: couldn't read scrutins file");

        serde_json::from_str(&acteurs_str).expect("Error: Couldn't parse scrutins file's JSON")
    }

    pub fn init() -> InstanceInfo {
        InstanceInfo {
            config: InstanceInfo::read_config(CONFIG_FILE),
            platforms_list: InstanceInfo::read_platforms(PLATFORMS_FILE),
            acteurs_list: InstanceInfo::read_acteurs(ACTEURS_FILE),
            scrutins_list: InstanceInfo::read_scrutins(SCRUTINS_FILE),
            ipv4_whitelist: IPV4WLEntry::parse(IPV4_WHITELIST_FILE),
            ipv4_blacklist: IPV4BLEntry::parse(IPV4_BLACKLIST_FILE),
            asn_list: ASNLEntry::parse(ASN_LIST_FILE),
            asn_blacklist: ASNBLEntry::parse(ASN_BLACKLIST_FILE),
        }
    }

    // Just panics if something's wrong.
    pub fn check_validity(&self) {
        // check if all IP ranges are always following this rule:
        // ip_end >= ip_start
        for entry in &self.ipv4_whitelist {
            if entry.ip_end < entry.ip_start {
                eprintln!("{:#?}", entry);
                panic!("Error: IPV4WLEntry: ip_end < ip_start");
            }
        }

        for entry in &self.ipv4_blacklist {
            if let Some(ip_end) = entry.ip_end {
                if ip_end < entry.ip_start {
                    eprintln!("{:#?}", entry);
                    panic!("Error: IPV4BLEntry: ip_end < ip_start");
                }
            }
        }

        for entry in &self.asn_list {
            if entry.ip_end < entry.ip_start {
                eprintln!("{:#?}", entry);
                panic!("Error: IPV4WLEntry: ip_end < ip_start");
            }
        }
    }

    pub fn get_csrf_key(&self) -> [u8; 32] {
        let mut key: [u8; 32] = Default::default();
        key.copy_from_slice(&self.config.csrf_key.clone().into_bytes()[..32]);
        key
    }

    // check from the Host header if we actually handle this platform
    // and if the platform poll is open.
    pub fn check_open(&self, host: &str) -> Option<&Platform> {
        // search the platform with the platform host
        let platform = self.platforms_list.iter().find(|p| p.host == host)?;

        if platform.end_at > Utc::now().naive_utc().date() {
            Some(platform)
        } else {
            None
        }
    }

    pub fn find_asn(&self, client: u32) -> u32 {
        for asn in &self.asn_list {
            if asn.ip_start <= client && asn.ip_end >= client {
                return asn.asn;
            }
        }
        0
    }

    // returns abuse code
    pub fn check_asn_blacklist(&self, client_asn: u32) -> u16 {
        for asn in &self.asn_blacklist {
            if asn.asn == client_asn {
                return asn.abuse_code;
            }
        }
        0
    }

    pub fn check_ipv4_whitelist(&self, client: u32) -> bool {
        for ip in &self.ipv4_whitelist {
            if ip.ip_start <= client && ip.ip_end >= client {
                return true;
            }
        }
        false
    }

    // returns abuse code
    pub fn check_ipv4_blacklist(&self, client: u32) -> u16 {
        for ip in &self.ipv4_blacklist {
            if let Some(ip_end) = ip.ip_end {
                if ip.ip_start <= client && ip_end >= client {
                    return ip.abuse_code;
                }
            } else if ip.ip_start == client {
                return ip.abuse_code;
            }
        }
        0
    }
}
