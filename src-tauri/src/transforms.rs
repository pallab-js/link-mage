//! Transform engine
//!
//! A *transform* takes a single node in the graph, runs an enrichment query
//! against an external data source, and returns newly created [`Node`]s.
//! All edges between the seed node and the new nodes are added to the graph
//! **inside** `run()` so callers never have to deal with dangling references.
//!
//! # Adding a new transform
//! 1. Create a struct that holds any configuration (e.g. a path to a database).
//! 2. `impl Transform for YourTransform`.
//! 3. Push an instance into [`TransformRegistry::new`].
//!
//! # GeoLite2 database setup
//! `GeoIpTransform` requires a local copy of the MaxMind GeoLite2-City database:
//!
//! 1. Create a free account at <https://www.maxmind.com/en/geolite2/signup>.
//! 2. Download **GeoLite2-City.mmdb** from your account dashboard.
//! 3. Place the file at one of the paths searched (first match wins):
//!    - Path set in the `GEOIP_DB` environment variable.
//!    - macOS: `~/Library/Application Support/link-mage/GeoLite2-City.mmdb`
//!    - Linux: `~/.local/share/link-mage/GeoLite2-City.mmdb`
//!    - Next to the application executable: `<exe-dir>/GeoLite2-City.mmdb`
//!    - Inside a macOS `.app` bundle: `<bundle>/Contents/Resources/GeoLite2-City.mmdb`
//!    - `/usr/share/GeoIP/GeoLite2-City.mmdb`
//!    - `/usr/local/share/GeoIP/GeoLite2-City.mmdb`
//! 4. The transform returns an error (and logs a warning) if the database is
//!    not found; the application does not crash.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use std::sync::Mutex;

use log::{debug, info, warn};

use crate::graph::{Edge, Node};

// ── Trait ─────────────────────────────────────────────────────────────────────

/// All transforms implement this trait.
///
/// Transforms are **synchronous** at the trait level.  The Tauri command that
/// calls them runs on the main thread (or a blocking thread); blocking I/O is
/// acceptable.
pub trait Transform: Send + Sync {
    /// Human-readable name shown in the UI.
    fn name(&self) -> String;

    /// Short description shown as a tooltip.
    fn description(&self) -> String;

    /// Which node types this transform is applicable to.
    /// An empty vector means "applicable to all types".
    fn applicable_types(&self) -> Vec<String>;

    /// Run the transform.
    ///
    /// * `seed`  – the seed node to enrich.
    ///
    /// Returns the list of `(new_nodes, new_edges)` that were created.
    /// The caller is responsible for merging these into the graph.
    fn run(&self, seed: &Node) -> Result<(Vec<Node>, Vec<Edge>), String>;
}

#[derive(serde::Serialize, Clone)]
pub struct Machine {
    pub name: String,
    pub description: String,
    pub transforms: Vec<String>,
}

pub struct MachineRegistry {
    pub machines: Vec<Machine>,
}

impl MachineRegistry {
    pub fn new() -> Self {
        Self {
            machines: vec![
                Machine {
                    name: "Domain Footprint".into(),
                    description: "Runs DNS Lookup, WHOIS, and Web Scraping in sequence.".into(),
                    transforms: vec!["DNS Lookup".into(), "WHOIS".into(), "Scrape Webpage".into()],
                },
                Machine {
                    name: "IP Intel".into(),
                    description: "Runs GeoIP, WHOIS, and VirusTotal Lookup in sequence.".into(),
                    transforms: vec!["GeoIP Lookup".into(), "WHOIS".into(), "VirusTotal Lookup".into()],
                }
            ],
        }
    }
}

// ── TransformRegistry ─────────────────────────────────────────────────────────

/// Owns all transform instances and machines for the lifetime of the application.
///
/// Built once, stored in [`crate::state::AppState`] behind an `Arc` (no
/// `Mutex` needed because the registry is read-only after construction).
pub struct TransformRegistry {
    transforms: Vec<Box<dyn Transform>>,
    pub machines: MachineRegistry,
}

impl TransformRegistry {
    /// Construct the registry with every built-in transform and machine.
    pub fn new() -> Self {
        Self {
            transforms: vec![
                Box::new(DnsLookupTransform),
                Box::new(WhoisTransform),
                Box::new(GeoIpTransform::new()),
                Box::new(ScrapeWebpageTransform::new()),
                Box::new(VirusTotalTransform::new()),
            ],
            machines: MachineRegistry::new(),
        }
    }

    /// All transforms in display order.
    pub fn all(&self) -> &[Box<dyn Transform>] {
        &self.transforms
    }

    /// Get transforms as trait object references
    pub fn as_refs(&self) -> Vec<&dyn Transform> {
        self.transforms.iter().map(|b| b.as_ref()).collect()
    }

    /// Look up a transform by name (case-insensitive).
    pub fn find(&self, name: &str) -> Option<&dyn Transform> {
        self.transforms
            .iter()
            .find(|t| t.name().eq_ignore_ascii_case(name))
            .map(|t| t.as_ref())
    }

    /// All transforms applicable to `node_type`.
    pub fn for_node_type(&self, node_type: &str) -> Vec<&dyn Transform> {
        self.transforms
            .iter()
            .filter(|t| {
                let types = t.applicable_types();
                types.is_empty() || types.contains(&node_type.to_string())
            })
            .map(|t| t.as_ref())
            .collect()
    }
}

impl Default for TransformRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── Shared helpers ─────────────────────────────────────────────────────────────

/// Return the `value` property of a node, falling back to the node id.
fn node_value(node: &Node) -> &str {
    node.properties
        .get("value")
        .map(String::as_str)
        .unwrap_or(&node.id)
}

/// Create a node and a connecting edge from `source_id`.
///
/// Keys and values may be any `Into<String>` type (string literals, `String`,
/// `&str`, etc.).
fn create_result_node<I, K, V>(
    source_id: &str,
    node_type: &str,
    value: &str,
    extra: I,
    edge_label: &str,
) -> (Node, Edge)
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    let mut props: HashMap<String, String> = extra
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect();
    props.insert("value".into(), value.to_string());
    props.insert("label".into(), value.to_string());

    let base = Node::new(node_type);
    let id = base.id.clone();
    let node = Node { properties: props, ..base };
    let edge = Edge::new(source_id, &id, edge_label);

    (node, edge)
}

// ═════════════════════════════════════════════════════════════════════════════
// Remote Transform (HTTP)
// ═════════════════════════════════════════════════════════════════════════════

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct RemoteTransform {
    pub name: String,
    pub description: String,
    pub url: String,
    pub applicable_types: Vec<String>,
}

impl Transform for RemoteTransform {
    fn name(&self) -> String { self.name.clone() }
    fn description(&self) -> String { self.description.clone() }
    fn applicable_types(&self) -> Vec<String> { self.applicable_types.clone() }

    fn run(&self, seed: &Node) -> Result<(Vec<Node>, Vec<Edge>), String> {
        info!("[RemoteTransform] Calling '{}' for '{}'", self.url, seed.id);

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| format!("failed to build HTTP client: {e}"))?;

        let response = client.post(&self.url)
            .json(seed)
            .send()
            .map_err(|e| format!("request failed: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            // Try to read the response body for a more detailed error message
            let body_text = response.text()
                .unwrap_or_else(|_| "no response body".to_string());
            return Err(format!(
                "remote server returned error: {} - {}",
                status,
                body_text.chars().take(200).collect::<String>()
            ));
        }

        #[derive(serde::Deserialize)]
        struct RemoteResponse {
            new_nodes: Vec<Node>,
            new_edges: Vec<Edge>,
            error: Option<String>,
        }

        let body: RemoteResponse = response.json()
            .map_err(|e| format!("failed to parse JSON response: {e}"))?;

        if let Some(err) = body.error {
            return Err(err);
        }

        Ok((body.new_nodes, body.new_edges))
    }
}

/// Resolves A and AAAA records for a Domain node and adds IP / IPv6 nodes.
///
/// Uses the OS system resolver (`getaddrinfo`) via
/// [`std::net::ToSocketAddrs`] — no external async runtime required.
pub struct DnsLookupTransform;

impl Transform for DnsLookupTransform {
    fn name(&self) -> String { "DNS Lookup".into() }

    fn description(&self) -> String {
        "Resolve A / AAAA records and add IP address nodes.".into()
    }

    fn applicable_types(&self) -> Vec<String> { vec!["Domain".into()] }

    fn run(&self, seed: &Node) -> Result<(Vec<Node>, Vec<Edge>), String> {

        if seed.node_type != "Domain" {
            return Err(format!(
                "DNS Lookup requires a Domain node, got '{}'",
                seed.node_type
            ));
        }

        let domain = node_value(&seed).to_string();
        info!("[DnsLookup] Resolving '{}'", domain);

        // Use the system resolver — `ToSocketAddrs` calls getaddrinfo(3)
        // which honours /etc/hosts, mDNS, search domains, etc.
        // Port 80 is a required placeholder but no connection is made.
        use std::net::ToSocketAddrs;
        let addrs = (domain.as_str(), 80u16)
            .to_socket_addrs()
            .map_err(|e| format!("DNS resolution failed for '{}': {}", domain, e))?;

        let mut nodes: Vec<Node> = Vec::new();
        let mut edges: Vec<Edge> = Vec::new();
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

        for sock_addr in addrs {
            let ip = sock_addr.ip();
            let ip_str = ip.to_string();

            // deduplicate (getaddrinfo sometimes returns the same IP twice)
            if !seen.insert(ip_str.clone()) {
                continue;
            }

            let node_type = if ip.is_ipv6() { "IPv6" } else { "IP" };
            debug!("[DnsLookup] '{}' → {} {}", domain, node_type, ip_str);

            let (node, edge) = create_result_node(
                &seed.id,
                node_type,
                &ip_str,
                [("source", "dns".to_string())],
                "resolves-to",
            );
            nodes.push(node);
            edges.push(edge);
        }

        if nodes.is_empty() {
            warn!("[DnsLookup] '{}' resolved to zero addresses", domain);
            return Err(format!("No addresses found for '{}'", domain));
        }

        info!("[DnsLookup] '{}' → {} IP node(s)", domain, nodes.len());
        Ok((nodes, edges))
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// WHOIS
// ═════════════════════════════════════════════════════════════════════════════

/// Queries WHOIS for a Domain or IP node and adds registrar / name-server nodes.
///
/// Protocol: raw TCP on port 43.  Queries `whois.iana.org` first and follows
/// one level of `refer:` / `ReferralServer:` if present.
pub struct WhoisTransform;

// ── WHOIS network helpers ─────────────────────────────────────────────────────

/// Send `query\r\n` to `server:43` and return the raw response text.
fn tcp_whois(server: &str, query: &str) -> Result<String, String> {
    use std::io::{Read, Write};
    use std::net::{TcpStream, ToSocketAddrs};
    use std::time::Duration;

    let addr_str = format!("{}:43", server);
    let sock_addr = addr_str
        .to_socket_addrs()
        .map_err(|e| format!("Cannot resolve WHOIS server '{}': {}", server, e))?
        .next()
        .ok_or_else(|| format!("No address for WHOIS server '{}'", server))?;

    let mut stream =
        TcpStream::connect_timeout(&sock_addr, Duration::from_secs(10))
            .map_err(|e| format!("Cannot connect to '{}': {}", server, e))?;

    stream.set_read_timeout(Some(Duration::from_secs(20))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(5))).ok();

    stream
        .write_all(format!("{}\r\n", query).as_bytes())
        .map_err(|e| format!("WHOIS send failed: {}", e))?;
    stream.shutdown(std::net::Shutdown::Write).ok();

    let mut buf = Vec::new();
    stream
        .read_to_end(&mut buf)
        .map_err(|e| format!("WHOIS receive failed: {}", e))?;

    Ok(String::from_utf8_lossy(&buf).into_owned())
}

/// Extract the referral server from a WHOIS response.
///
/// Handles both:
/// - `refer: whois.verisign-grs.com`  (IANA format)
/// - `ReferralServer: whois://whois.arin.net/rest`  (ARIN format)
fn extract_whois_referral(raw: &str) -> Option<String> {
    for line in raw.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();

        if lower.starts_with("refer:") {
            let val = trimmed["refer:".len()..].trim();
            // Skip HTTP URLs — those are web interfaces, not WHOIS servers
            if !val.is_empty() && !val.starts_with("http") {
                return Some(val.to_string());
            }
        }

        if lower.starts_with("referralserver:") {
            let val = trimmed["referralserver:".len()..].trim();
            let val = val
                .trim_start_matches("whois://")
                .trim_start_matches("rwhois://");
            // Strip any trailing path component
            let val = val.split('/').next().unwrap_or(val);
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

/// Collect all values for a given field name (case-insensitive).
///
/// Handles duplicate field names (e.g. multiple `Name Server:` lines).
/// Returns borrowed slices into `raw`; callers `.to_string()` as needed.
fn parse_whois_field<'a>(raw: &'a str, key: &str) -> Vec<&'a str> {
    let key_lower = key.to_lowercase();
    raw.lines()
        .filter_map(|line| {
            let colon = line.find(':')?;
            let field = line[..colon].trim().to_lowercase();
            if field == key_lower {
                let val = line[colon + 1..].trim();
                if !val.is_empty() { Some(val) } else { None }
            } else {
                None
            }
        })
        .collect()
}

impl Transform for WhoisTransform {
    fn name(&self) -> String { "WHOIS".into() }

    fn description(&self) -> String {
        "Query WHOIS and add registrar, name-server, and network-owner nodes.".into()
    }

    fn applicable_types(&self) -> Vec<String> { vec!["Domain".into(), "IP".into()] }

    fn run(&self, seed: &Node) -> Result<(Vec<Node>, Vec<Edge>), String> {

        match seed.node_type.as_str() {
            "Domain" | "IP" => {}
            t => return Err(format!(
                "WHOIS requires a Domain or IP node, got '{}'", t
            )),
        }

        // The query is written raw to the WHOIS TCP socket, so CR/LF would
        // inject extra commands — reject them outright.
        let target = node_value(&seed).trim().to_string();
        if target.is_empty() || target.contains(['\r', '\n']) {
            return Err(format!("WHOIS query rejected: {target:?} contains control characters"));
        }
        info!("[WHOIS] Querying '{}'", target);

        // Step 1: query IANA for authoritative server / referral.
        let initial = tcp_whois("whois.iana.org", &target)
            .map_err(|e| format!("WHOIS initial query failed: {}", e))?;

        // Step 2: follow one referral if present.
        let raw = if let Some(referral) = extract_whois_referral(&initial) {
            info!("[WHOIS] Following referral to '{}'", referral);
            // Fallback to initial response on network error.
            tcp_whois(&referral, &target).unwrap_or(initial)
        } else {
            initial
        };

        let mut nodes: Vec<Node> = Vec::new();
        let mut edges: Vec<Edge> = Vec::new();

        // ── Registrar (gTLD ICANN format) ─────────────────────────────────
        let registrars = parse_whois_field(&raw, "Registrar");
        if let Some(&registrar) = registrars.first() {
            let created = parse_whois_field(&raw, "Creation Date")
                .first()
                .copied()
                .unwrap_or("")
                .to_string();
            let expires = parse_whois_field(&raw, "Registry Expiry Date")
                .first()
                .copied()
                .unwrap_or("")
                .to_string();
            let abuse = parse_whois_field(&raw, "Registrar Abuse Contact Email")
                .first()
                .copied()
                .unwrap_or("")
                .to_string();
            // Store a truncated copy of raw WHOIS for reference.
            let raw_snippet: String = raw.chars().take(800).collect();

            let (node, edge) = create_result_node(
                &seed.id,
                "Organisation",
                registrar,
                [
                    ("role",       "registrar".to_string()),
                    ("created",    created),
                    ("expires",    expires),
                    ("abuse_email", abuse),
                    ("raw_whois",  raw_snippet),
                ],
                "registered-with",
            );
            nodes.push(node);
            edges.push(edge);
        }

        // ── Name servers (gTLD format, up to 8) ───────────────────────────
        let mut seen_ns: std::collections::HashSet<String> = std::collections::HashSet::new();
        for ns in parse_whois_field(&raw, "Name Server").into_iter().take(8) {
            let ns_lower = ns.to_lowercase();
            if !ns_lower.is_empty() && seen_ns.insert(ns_lower.clone()) {
                let (node, edge) = create_result_node(
                    &seed.id,
                    "Domain",
                    &ns_lower,
                    [("role", "nameserver".to_string())],
                    "uses-nameserver",
                );
                nodes.push(node);
                edges.push(edge);
            }
        }

        // ── Network owner (IP WHOIS — ARIN / RIPE / APNIC format) ─────────
        if seed.node_type == "IP" {
            // Try several common field names used by different RIRs.
            let org_fields = ["OrgName", "org-name", "netname", "NetName"];
            let org = org_fields
                .iter()
                .flat_map(|f| parse_whois_field(&raw, f))
                .next()
                .map(str::to_string);

            if let Some(org_name) = org {
                // "CIDR" is used by ARIN; "inetnum" is the RIPE/APNIC equivalent.
                let cidr_fields   = parse_whois_field(&raw, "CIDR");
                let inetnum_fields = parse_whois_field(&raw, "inetnum");
                let cidr = cidr_fields
                    .first()
                    .or_else(|| inetnum_fields.first())
                    .copied()
                    .unwrap_or("")
                    .to_string();
                let (node, edge) = create_result_node(
                    &seed.id,
                    "Organisation",
                    &org_name,
                    [
                        ("role", "network-owner".to_string()),
                        ("cidr", cidr),
                    ],
                    "owned-by",
                );
                nodes.push(node);
                edges.push(edge);
            }
        }

        // ── Fallback: raw text node if nothing was parsed ─────────────────
        if nodes.is_empty() {
            warn!("[WHOIS] No structured data parsed for '{}'", target);
            let snippet: String = raw.chars().take(300).collect();
            let (node, edge) = create_result_node(
                &seed.id,
                "Organisation",
                &format!("WHOIS result for {}", target),
                [
                    ("raw_whois", snippet),
                    ("note",      "no structured fields found".to_string()),
                ],
                "whois-result",
            );
            nodes.push(node);
            edges.push(edge);
        }

        info!("[WHOIS] '{}' → {} new node(s)", target, nodes.len());
        Ok((nodes, edges))
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// GeoIP
// ═════════════════════════════════════════════════════════════════════════════

/// Look up geographic location for an IP node using a local MaxMind database.
///
/// The database is opened **once** at construction time and kept in memory
/// (~55 MB for GeoLite2-City) so subsequent `run()` calls pay no I/O cost.
///
/// # Database paths searched (first match wins)
/// 1. `$GEOIP_DB` environment variable.
/// 2. macOS: `~/Library/Application Support/link-mage/GeoLite2-City.mmdb`
/// 3. Linux: `~/.local/share/link-mage/GeoLite2-City.mmdb`
/// 4. Next to the running executable (development / portable builds).
/// 5. macOS `.app` bundle: `../Resources/GeoLite2-City.mmdb` relative to exe.
/// 6. `/usr/share/GeoIP/GeoLite2-City.mmdb`
/// 7. `/usr/local/share/GeoIP/GeoLite2-City.mmdb`
pub struct GeoIpTransform {
    db_path: Option<std::path::PathBuf>,
    /// Pre-opened MaxMind reader.  `None` if the database file was not found
    /// or could not be opened at construction time.
    reader: Option<maxminddb::Reader<Vec<u8>>>,
}

impl GeoIpTransform {
    pub fn new() -> Self {
        let db_path = Self::find_db();
        let reader  = db_path.as_ref().and_then(|p| {
            maxminddb::Reader::open_readfile(p)
                .map_err(|e| warn!("[GeoIP] Failed to open database at '{}': {}", p.display(), e))
                .ok()
        });
        Self { db_path, reader }
    }

    fn find_db() -> Option<std::path::PathBuf> {
        use std::path::PathBuf;

        let mut candidates: Vec<PathBuf> = Vec::new();

        // 1. Explicit env var override.
        if let Ok(p) = std::env::var("GEOIP_DB") {
            let path = PathBuf::from(&p);
            if path.exists() {
                info!("[GeoIP] Using database from GEOIP_DB: {}", p);
                return Some(path);
            }
            warn!("[GeoIP] GEOIP_DB='{}' — file does not exist", p);
        }

        // 2. Platform-specific user data directory.
        if let Some(data_dir) = user_data_dir() {
            candidates.push(data_dir.join("GeoLite2-City.mmdb"));
        }

        // 3. Next to the executable (works for dev builds and portable deploys).
        if let Ok(exe) = std::env::current_exe() {
            let exe_dir = exe.parent().unwrap_or(exe.as_path());
            candidates.push(exe_dir.join("GeoLite2-City.mmdb"));
            // macOS .app bundle: exe is in Contents/MacOS/, resources in Contents/Resources/
            candidates.push(exe_dir.join("../Resources/GeoLite2-City.mmdb"));
        }

        // 4. System-wide paths.
        candidates.push(PathBuf::from("/usr/share/GeoIP/GeoLite2-City.mmdb"));
        candidates.push(PathBuf::from("/usr/local/share/GeoIP/GeoLite2-City.mmdb"));

        for path in &candidates {
            if path.exists() {
                info!("[GeoIP] Found database at {}", path.display());
                return Some(path.clone());
            }
        }

        warn!(
            "[GeoIP] GeoLite2-City.mmdb not found in any search path. \
             Download from https://www.maxmind.com/en/geolite2/signup \
             and set GEOIP_DB or place it at the user data directory."
        );
        None
    }
}

impl Default for GeoIpTransform {
    fn default() -> Self { Self::new() }
}

/// Return the platform-specific user data directory for link-mage.
fn user_data_dir() -> Option<std::path::PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let home = std::path::Path::new(&home);

    #[cfg(target_os = "macos")]
    {
        Some(home.join("Library").join("Application Support").join("link-mage"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        Some(home.join(".local").join("share").join("link-mage"))
    }
}

impl Transform for GeoIpTransform {
    fn name(&self) -> String { "GeoIP Lookup".into() }

    fn description(&self) -> String {
        "Look up geographic location of an IP address using the MaxMind GeoLite2 database.".into()
    }

    fn applicable_types(&self) -> Vec<String> { vec!["IP".into(), "IPv6".into()] }

    fn run(&self, seed: &Node) -> Result<(Vec<Node>, Vec<Edge>), String> {

        match seed.node_type.as_str() {
            "IP" | "IPv6" => {}
            t => return Err(format!(
                "GeoIP Lookup requires an IP/IPv6 node, got '{}'", t
            )),
        }

        let ip_str = node_value(&seed).to_string();
        info!("[GeoIP] Looking up '{}'", ip_str);

        // Validate the IP address before touching the DB.
        let ip: std::net::IpAddr = ip_str
            .parse()
            .map_err(|e| format!("Invalid IP address '{}': {}", ip_str, e))?;

        // Private / loopback addresses will not be in the public GeoIP DB.
        if ip.is_loopback() || is_private_ip(ip) {
            return Err(format!(
                "'{}' is a private/loopback address — not in GeoIP database",
                ip_str
            ));
        }

        // Use the cached reader opened at construction time.
        let reader = self.reader.as_ref().ok_or_else(|| match &self.db_path {
            Some(p) => format!(
                "GeoLite2-City.mmdb was found at '{}' but could not be opened. \
                 Try re-downloading it from https://www.maxmind.com/en/geolite2/signup",
                p.display()
            ),
            None => {
                "GeoLite2-City.mmdb not found. \
                 Download from https://www.maxmind.com/en/geolite2/signup \
                 and set GEOIP_DB or place it at \
                 ~/Library/Application Support/link-mage/GeoLite2-City.mmdb"
                    .to_string()
            }
        })?;

        // maxminddb 0.27 uses a two-step lazy API: lookup → LookupResult → decode.
        let lookup = reader
            .lookup(ip)
            .map_err(|e| format!("GeoIP lookup failed: {}", e))?;

        if !lookup.has_data() {
            return Err(format!("IP '{}' not found in GeoIP database", ip_str));
        }

        let city: maxminddb::geoip2::City = lookup
            .decode()
            .map_err(|e| format!("GeoIP decode failed: {}", e))?
            .ok_or_else(|| format!("IP '{}' not found in GeoIP database", ip_str))?;

        // Extract owned strings before `city` borrows from `reader` go out of scope.
        // In maxminddb 0.27, `Names` is a struct with named fields (e.g. `.english`).
        let country_name = city.country.names.english.unwrap_or("Unknown").to_string();
        let country_code = city.country.iso_code.unwrap_or("??").to_string();
        let city_name    = city.city.names.english.unwrap_or("Unknown").to_string();
        let subdivision  = city.subdivisions
            .first()
            .and_then(|s| s.names.english)
            .unwrap_or("")
            .to_string();
        let lat = city.location.latitude.unwrap_or(0.0);
        let lon = city.location.longitude.unwrap_or(0.0);
        let tz  = city.location.time_zone.unwrap_or("").to_string();
        let accuracy_km = city.location.accuracy_radius.unwrap_or(0);

        drop(city);

        let label = match (city_name.as_str(), country_name.as_str()) {
            ("Unknown", "Unknown") => format!("Location of {}", ip_str),
            ("Unknown", country)   => country.to_string(),
            (city, "Unknown")      => city.to_string(),
            (city, country)        => format!("{}, {}", city, country),
        };

        debug!("[GeoIP] {} → {}", ip_str, label);

        let (node, edge) = create_result_node(
            &seed.id,
            "Location",
            &label,
            [
                ("country",     country_name),
                ("country_code", country_code),
                ("city",        city_name),
                ("subdivision", subdivision),
                ("lat_lon",     format!("{:.4},{:.4}", lat, lon)),
                ("timezone",    tz),
                ("accuracy_km", accuracy_km.to_string()),
            ],
            "located-in",
        );

        info!("[GeoIP] '{}' → Location node '{}'", ip_str, label);
        Ok((vec![node], vec![edge]))
    }
}

/// Returns true for RFC-1918 / RFC-4193 private addresses.
/// (std::net::IpAddr has no `is_private()` on stable Rust yet.)
fn is_private_ip(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => {
            let octets = v4.octets();
            // 10.0.0.0/8
            octets[0] == 10
            // 172.16.0.0/12
            || (octets[0] == 172 && (octets[1] & 0xf0) == 16)
            // 192.168.0.0/16
            || (octets[0] == 192 && octets[1] == 168)
            // 169.254.0.0/16 (link-local)
            || (octets[0] == 169 && octets[1] == 254)
        }
        std::net::IpAddr::V6(v6) => {
            // fc00::/7  (ULA)
            (v6.segments()[0] & 0xfe00) == 0xfc00
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Node;

    // ── Registry ──────────────────────────────────────────────────────────

    #[test]
    fn registry_contains_five_transforms() {
        assert_eq!(TransformRegistry::new().all().len(), 5);
    }

    #[test]
    fn registry_names_are_unique() {
        let reg = TransformRegistry::new();
        let names: Vec<_> = reg.all().iter().map(|t| t.name()).collect();
        let unique: std::collections::HashSet<_> = names.iter().collect();
        assert_eq!(names.len(), unique.len(), "duplicate transform names");
    }

    #[test]
    fn find_by_name_case_insensitive() {
        let reg = TransformRegistry::new();
        assert!(reg.find("dns lookup").is_some());
        assert!(reg.find("DNS LOOKUP").is_some());
        assert!(reg.find("virustotal lookup").is_some());
        assert!(reg.find("nonexistent").is_none());
    }

    // ── create_result_node ─────────────────────────────────────────────────

    #[test]
    fn create_result_node_sets_value_and_label() {
        let (node, edge) = create_result_node(
            "source-id", "Location", "London, UK",
            [("country", "UK".to_string())], "located-in",
        );
        assert_eq!(node.properties["value"], "London, UK");
        assert_eq!(node.properties["label"], "London, UK");
        assert_eq!(node.properties["country"], "UK");
        assert_eq!(edge.label, "located-in");
        assert_eq!(edge.source, "source-id");
        assert_eq!(edge.target, node.id);
    }

    #[test]
    fn create_result_node_accepts_string_keys() {
        let key = "dynamic_key".to_string();
        let val = "dynamic_val".to_string();
        let (node, _edge) = create_result_node(
            "source", "Info", "test",
            [(key.clone(), val.clone())], "has-info",
        );
        assert_eq!(node.properties[&key], val);
    }

    // ── WHOIS parsing helpers (no network) ────────────────────────────────

    #[test]
    fn parse_whois_field_single() {
        let raw = "Domain Name: EXAMPLE.COM\nRegistrar: Example Registrar, Inc.\n";
        let v = parse_whois_field(raw, "Registrar");
        assert_eq!(v, vec!["Example Registrar, Inc."]);
    }

    #[test]
    fn parse_whois_field_multiple() {
        let raw = "Name Server: NS1.EXAMPLE.COM\nName Server: NS2.EXAMPLE.COM\n";
        let v = parse_whois_field(raw, "Name Server");
        assert_eq!(v.len(), 2);
        assert!(v.contains(&"NS1.EXAMPLE.COM"));
        assert!(v.contains(&"NS2.EXAMPLE.COM"));
    }

    #[test]
    fn parse_whois_field_case_insensitive_key() {
        let raw = "REGISTRAR: Acme Corp\n";
        assert_eq!(parse_whois_field(raw, "registrar"), vec!["Acme Corp"]);
    }

    #[test]
    fn parse_whois_field_empty_values_skipped() {
        let raw = "Registrar: \nRegistrar: Real Corp\n";
        let v = parse_whois_field(raw, "Registrar");
        assert_eq!(v, vec!["Real Corp"]);
    }

    #[test]
    fn extract_referral_iana_format() {
        let raw = "domain:       COM\nrefer:        whois.verisign-grs.com\n";
        assert_eq!(
            extract_whois_referral(raw),
            Some("whois.verisign-grs.com".into())
        );
    }

    #[test]
    fn extract_referral_arin_format() {
        let raw = "ReferralServer: whois://whois.arin.net\n";
        assert_eq!(
            extract_whois_referral(raw),
            Some("whois.arin.net".into())
        );
    }

    #[test]
    fn extract_referral_strips_path() {
        let raw = "ReferralServer: whois://whois.ripe.net/rest/ip\n";
        assert_eq!(
            extract_whois_referral(raw),
            Some("whois.ripe.net".into())
        );
    }

    #[test]
    fn extract_referral_skips_http_urls() {
        let raw = "refer: https://www.example.com/\n";
        assert_eq!(extract_whois_referral(raw), None);
    }

    #[test]
    fn extract_referral_none_when_absent() {
        let raw = "Domain Name: EXAMPLE.COM\nRegistrar: Acme\n";
        assert_eq!(extract_whois_referral(raw), None);
    }

    // ── DNS guard tests (no network needed) ───────────────────────────────

    #[test]
    fn dns_rejects_non_domain_node() {
        let mut node = Node::new("IP");
        node.properties.insert("value".into(), "1.2.3.4".into());
        assert!(DnsLookupTransform.run(&node).is_err());
    }

    // ── DNS integration (requires network) ────────────────────────────────

    #[test]
    #[ignore = "requires network access"]
    fn dns_resolves_example_com() {
        let mut seed = Node::new("Domain");
        seed.properties.insert("value".into(), "example.com".into());

        let (nodes, edges) = DnsLookupTransform.run(&seed)
            .expect("DNS should succeed for example.com");

        assert!(!nodes.is_empty(), "must return at least one IP node");
        for n in &nodes {
            assert!(
                n.node_type == "IP" || n.node_type == "IPv6",
                "unexpected node type: {}", n.node_type
            );
            assert_eq!(n.properties.get("source").map(String::as_str), Some("dns"));
        }
        for e in &edges {
            assert_eq!(e.label, "resolves-to");
            assert_eq!(e.source, seed.id);
        }
        // No duplicate IPs
        let ips: Vec<_> = nodes.iter().map(|n| &n.properties["value"]).collect();
        let uniq: std::collections::HashSet<_> = ips.iter().collect();
        assert_eq!(ips.len(), uniq.len(), "duplicate IP nodes returned");
    }

    #[test]
    #[ignore = "requires network access"]
    fn dns_resolves_localhost() {
        // localhost always resolves; useful as a sanity check.
        let mut seed = Node::new("Domain");
        seed.properties.insert("value".into(), "localhost".into());
        let result = DnsLookupTransform.run(&seed);
        // May succeed or fail depending on system config — must not panic.
        let _ = result;
    }

    // ── WHOIS guard tests ─────────────────────────────────────────────────

    #[test]
    fn whois_rejects_wrong_type() {
        let mut node = Node::new("Email");
        node.properties.insert("value".into(), "a@b.com".into());
        assert!(WhoisTransform.run(&node).is_err());
    }

    #[test]
    fn whois_rejects_control_characters_in_query() {
        // CRLF would inject extra commands into the raw WHOIS TCP session.
        let mut node = Node::new("Domain");
        node.properties.insert("value".into(), "example.com\r\nQUIT".into());
        let err = WhoisTransform.run(&node).unwrap_err();
        assert!(err.contains("control characters"), "{err}");
    }

    // ── WHOIS integration (requires network) ─────────────────────────────

    #[test]
    #[ignore = "requires network access"]
    fn whois_domain_query() {
        let mut seed = Node::new("Domain");
        seed.properties.insert("value".into(), "example.com".into());

        let (nodes, edges) = WhoisTransform.run(&seed)
            .expect("WHOIS should succeed for example.com");

        assert!(!nodes.is_empty(), "should produce at least one node");
        assert_eq!(nodes.len(), edges.len());
        // At minimum expect a registrar or raw-result node.
        let types: Vec<_> = nodes.iter().map(|n| &n.node_type).collect();
        assert!(
            types.iter().any(|t| *t == "Organisation" || *t == "Domain"),
            "unexpected node types: {:?}", types
        );
    }

    #[test]
    #[ignore = "requires network access"]
    fn whois_ip_query() {
        let mut seed = Node::new("IP");
        seed.properties.insert("value".into(), "8.8.8.8".into());
        let result = WhoisTransform.run(&seed);
        assert!(result.is_ok(), "WHOIS for 8.8.8.8 should succeed: {:?}", result);
    }

    // ── GeoIP guard tests (no DB needed) ─────────────────────────────────

    #[test]
    fn geoip_rejects_domain_node() {
        let mut seed = Node::new("Domain");
        seed.properties.insert("value".into(), "example.com".into());
        assert!(GeoIpTransform::new().run(&seed).is_err());
    }

    #[test]
    fn geoip_rejects_private_ip() {
        let mut seed = Node::new("IP");
        seed.properties.insert("value".into(), "192.168.1.1".into());
        // Private IP → error before touching the DB.
        let result = GeoIpTransform::new().run(&seed);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("private"));
    }

    #[test]
    fn geoip_rejects_loopback() {
        let mut seed = Node::new("IP");
        seed.properties.insert("value".into(), "127.0.0.1".into());
        let result = GeoIpTransform::new().run(&seed);
        assert!(result.is_err());
    }

    #[test]
    fn geoip_absent_db_returns_error() {
        // If no DB is found, run() must return Err (not panic).
        let t = GeoIpTransform { db_path: None, reader: None };
        let mut seed = Node::new("IP");
        seed.properties.insert("value".into(), "8.8.8.8".into());
        let result = t.run(&seed);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_lowercase().contains("geolite2"));
    }

    #[test]
    fn geoip_rejects_invalid_ip_string() {
        // A node with a garbage "value" should fail gracefully.
        // IP validation happens before the DB check, so reader: None is fine.
        let t = GeoIpTransform { db_path: None, reader: None };
        let mut node = Node::new("IP");
        node.properties.insert("value".into(), "not-an-ip".into());
        let result = t.run(&node);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid IP"));
    }

    // ── GeoIP integration (requires GeoLite2-City.mmdb) ──────────────────

    #[test]
    #[ignore = "requires GeoLite2-City.mmdb database file"]
    fn geoip_lookup_public_ip() {
        let t = GeoIpTransform::new();
        if t.db_path.is_none() {
            eprintln!("SKIP: GeoLite2-City.mmdb not found");
            return;
        }
        let mut seed = Node::new("IP");
        seed.properties.insert("value".into(), "8.8.8.8".into());

        let (nodes, _edges) = t.run(&seed).expect("GeoIP lookup should succeed");
        assert_eq!(nodes.len(), 1);
        let n = &nodes[0];
        assert_eq!(n.node_type, "Location");
        assert!(n.properties.contains_key("country"));
        assert!(n.properties.contains_key("lat_lon"));
        let lat_lon = &n.properties["lat_lon"];
        assert!(lat_lon.contains(','), "lat_lon should be 'lat,lon': {}", lat_lon);
    }

    // ── applicable_types content ──────────────────────────────────────────

    #[test]
    fn dns_applicable_types_is_domain_only() {
        let types = DnsLookupTransform.applicable_types();
        assert_eq!(types, vec!["Domain".to_string()],
            "DNS Lookup must only fire on Domain nodes");
    }

    #[test]
    fn whois_applicable_types_includes_domain_and_ip() {
        let types = WhoisTransform.applicable_types();
        assert!(types.iter().any(|s| s == "Domain"), "WHOIS must support Domain nodes");
        assert!(types.iter().any(|s| s == "IP"),     "WHOIS must support IP nodes");
    }

    #[test]
    fn geoip_applicable_types_is_ip_and_ipv6() {
        let t = GeoIpTransform::new();
        let types = t.applicable_types();
        assert!(types.iter().any(|s| s == "IP"),   "GeoIP must accept IP nodes");
        assert!(types.iter().any(|s| s == "IPv6"), "GeoIP must accept IPv6 nodes");
        assert!(!types.iter().any(|s| s == "Domain"), "GeoIP must not accept Domain nodes");
    }

    // ── TransformRegistry::for_node_type ──────────────────────────────────

    #[test]
    fn registry_for_domain_includes_dns_and_whois_excludes_geoip() {
        let reg = TransformRegistry::new();
        let names: Vec<String> =
            reg.for_node_type("Domain").iter().map(|t| t.name()).collect();

        assert!(names.iter().any(|n| n.contains("DNS")),
            "DNS Lookup must be offered for Domain nodes");
        assert!(names.iter().any(|n| n.contains("WHOIS")),
            "WHOIS must be offered for Domain nodes");
        assert!(!names.iter().any(|n| n.contains("GeoIP")),
            "GeoIP must NOT be offered for Domain nodes");
    }

    #[test]
    fn registry_for_ip_includes_whois_and_geoip_excludes_dns() {
        let reg = TransformRegistry::new();
        let names: Vec<String> =
            reg.for_node_type("IP").iter().map(|t| t.name()).collect();

        assert!(names.iter().any(|n| n.contains("WHOIS")),
            "WHOIS must be offered for IP nodes");
        assert!(names.iter().any(|n| n.contains("GeoIP")),
            "GeoIP must be offered for IP nodes");
        assert!(!names.iter().any(|n| n.contains("DNS")),
            "DNS Lookup must NOT be offered for IP nodes");
    }

    #[test]
    fn registry_for_unknown_type_returns_no_transforms() {
        let reg = TransformRegistry::new();
        let applicable = reg.for_node_type("Email");
        assert!(
            applicable.is_empty(),
            "no built-in transform should fire on Email nodes, got: {:?}",
            applicable.iter().map(|t| t.name()).collect::<Vec<_>>(),
        );
    }

    #[test]
    fn registry_struct_find_unknown_returns_none() {
        let reg = TransformRegistry::new();
        assert!(reg.find("does-not-exist").is_none());
        assert!(reg.find("").is_none());
    }

    #[test]
    fn registry_struct_find_is_case_insensitive() {
        let reg = TransformRegistry::new();
        assert!(reg.find("dns lookup").is_some());
        assert!(reg.find("DNS LOOKUP").is_some());
        assert!(reg.find("Dns Lookup").is_some());
        assert!(reg.find("WHOIS").is_some());
        assert!(reg.find("whois").is_some());
        assert!(reg.find("geoip lookup").is_some());
    }

    // ── Transform metadata ────────────────────────────────────────────────

    #[test]
    fn all_transforms_have_non_empty_name_and_description() {
        for t in TransformRegistry::new().all() {
            assert!(!t.name().is_empty(),
                "transform name must not be empty");
            assert!(!t.description().is_empty(),
                "transform '{}' description must not be empty", t.name());
        }
    }

    // ── Graph unchanged when a guard rejects the node ─────────────────────
    //
    // If a transform returns Err due to type/existence guards (before any
    // network I/O), the results must be empty.

    #[test]
    fn dns_guard_fail_returns_err() {
        // DNS rejects IP nodes — pure guard failure, no I/O.
        let mut node = Node::new("IP");
        node.properties.insert("value".into(), "1.2.3.4".into());
        assert!(DnsLookupTransform.run(&node).is_err());
    }

    #[test]
    fn whois_guard_fail_returns_err() {
        // Email nodes are rejected by WHOIS.
        let mut node = Node::new("Email");
        node.properties.insert("value".into(), "user@example.com".into());
        assert!(WhoisTransform.run(&node).is_err());
    }

    #[test]
    fn geoip_guard_fail_returns_err() {
        // All GeoIP guard failures must return Err.
        for &(node_type, ip_val) in &[
            ("Domain", "example.com"),  // wrong type
            ("IP",     "192.168.1.1"),  // private
            ("IP",     "127.0.0.1"),    // loopback
        ] {
            let t = GeoIpTransform { db_path: None, reader: None };
            let mut node = Node::new(node_type);
            node.properties.insert("value".into(), ip_val.into());
            assert!(t.run(&node).is_err());
        }
    }

    // ── GeoIP — IPv6 address classification ──────────────────────────────

    #[test]
    fn geoip_rejects_ipv6_loopback() {
        // ::1 is the IPv6 loopback; must be rejected before DB access.
        let t = GeoIpTransform { db_path: None, reader: None };
        let mut node = Node::new("IPv6");
        node.properties.insert("value".into(), "::1".into());

        let result = t.run(&node);
        assert!(result.is_err(), "IPv6 loopback must be rejected");
        assert!(result.unwrap_err().to_lowercase().contains("private"),
            "error must mention private/loopback");
    }

    #[test]
    fn geoip_rejects_private_ipv6_ula() {
        // fc00::1 is in the Unique Local Address block (fc00::/7).
        let t = GeoIpTransform { db_path: None, reader: None };
        let mut node = Node::new("IPv6");
        node.properties.insert("value".into(), "fc00::1".into());

        let result = t.run(&node);
        assert!(result.is_err(), "IPv6 ULA address must be rejected");
    }

    #[test]
    fn geoip_ipv6_node_type_passes_type_guard() {
        // A public IPv6 address with db_path=None must fail at the *DB* step,
        // not the type guard.  This confirms the type guard accepts "IPv6".
        let t = GeoIpTransform { db_path: None, reader: None };
        let mut node = Node::new("IPv6");
        // 2001:4860:4860::8888 — Google's public IPv6 DNS.
        node.properties.insert("value".into(), "2001:4860:4860::8888".into());

        let err = t.run(&node).unwrap_err();
        assert!(err.to_lowercase().contains("geolite2"),
            "error must be about the missing DB, not the type guard: {err}");
    }

    // ── GeoIP — missing "value" property ─────────────────────────────────

    #[test]
    fn geoip_ip_node_without_value_property_fails_with_invalid_ip() {
        // node_value() falls back to node.id (a UUID) when "value" is absent.
        // A UUID is not a valid IP address → "Invalid IP" parse error.
        // IP parsing happens before the DB check, so reader: None is fine.
        let t = GeoIpTransform { db_path: None, reader: None };
        // Deliberately omit "value".
        let node = Node::new("IP");

        let result = t.run(&node);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid IP"),
            "missing value must produce an 'Invalid IP' error");
    }

    // ── is_private_ip helper ──────────────────────────────────────────────

    #[test]
    fn private_ip_detection() {
        use std::net::IpAddr;
        let private = ["10.0.0.1", "172.16.0.1", "172.31.255.255", "192.168.1.1", "169.254.1.1"];
        for s in private {
            let ip: IpAddr = s.parse().unwrap();
            assert!(is_private_ip(ip), "{} should be private", s);
        }
        let public = ["8.8.8.8", "1.1.1.1", "93.184.216.34"];
        for s in public {
            let ip: IpAddr = s.parse().unwrap();
            assert!(!is_private_ip(ip), "{} should not be private", s);
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Web Scraper
// ═════════════════════════════════════════════════════════════════════════════

/// Fetches the HTTP page for a given URL or Domain node, and extracts the `<title>`
/// and outbound links.
pub struct ScrapeWebpageTransform {
    client: reqwest::blocking::Client,
}

impl ScrapeWebpageTransform {
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent(concat!("link-mage-scraper/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("failed to build reqwest blocking client - this should only happen if the system TLS configuration is broken");
        Self { client }
    }
}

impl Default for ScrapeWebpageTransform {
    fn default() -> Self { Self::new() }
}

impl Transform for ScrapeWebpageTransform {
    fn name(&self) -> String { "Scrape Webpage".into() }

    fn description(&self) -> String {
        "Fetch the HTTP page to extract title and outbound links.".into()
    }

    fn applicable_types(&self) -> Vec<String> { vec!["Domain".into(), "URL".into()] }

    fn run(&self, seed: &Node) -> Result<(Vec<Node>, Vec<Edge>), String> {

        match seed.node_type.as_str() {
            "Domain" | "URL" => {}
            t => return Err(format!("Scrape Webpage requires a Domain or URL node, got '{}'", t)),
        }

        let value = node_value(&seed).to_string();
        
        let url = if seed.node_type == "Domain" {
            format!("http://{}", value)
        } else {
            value.clone()
        };

        info!("[Scrape] Fetching '{}'", url);

        let resp = self.client.get(&url)
            .send()
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("HTTP request failed with status {}", resp.status()));
        }

        let body = resp.text().map_err(|e| format!("Failed to read response body: {}", e))?;
        
        let document = scraper::Html::parse_document(&body);
        let title_selector = scraper::Selector::parse("title").unwrap();
        let link_selector = scraper::Selector::parse("a[href]").unwrap();

        let mut nodes: Vec<Node> = Vec::new();
        let mut edges: Vec<Edge> = Vec::new();
        let mut seen_links = std::collections::HashSet::new();

        if let Some(title_element) = document.select(&title_selector).next() {
            let title_text = title_element.text().collect::<Vec<_>>().join(" ");
            let trimmed_title = title_text.trim();
            if !trimmed_title.is_empty() {
                let (node, edge) = create_result_node(
                    &seed.id,
                    "Info",
                    trimmed_title,
                    [("source", "html_title".to_string())],
                    "has-title",
                );
                nodes.push(node);
                edges.push(edge);
            }
        }

        for link_element in document.select(&link_selector) {
            if let Some(href) = link_element.value().attr("href") {
                let href_clean = href.trim();
                if href_clean.starts_with("http://") || href_clean.starts_with("https://") {
                    if seen_links.insert(href_clean.to_string()) {
                        let (node, edge) = create_result_node(
                            &seed.id,
                            "URL",
                            href_clean,
                            [("source", "html_link".to_string())],
                            "links-to",
                        );
                        nodes.push(node);
                        edges.push(edge);
                    }
                }
            }
        }

        if nodes.is_empty() {
            warn!("[Scrape] No title or links extracted for '{}'", url);
            return Err("No title or outbound links found".to_string());
        }

        info!("[Scrape] '{}' -> {} new node(s)", url, nodes.len());
        Ok((nodes, edges))
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Rate Limiter
// ═════════════════════════════════════════════════════════════════════════════

/// Token-bucket rate limiter for external API calls.
///
/// Serialises all callers through a single `Mutex` to guarantee the minimum
/// gap between requests.  The lock is held *across the sleep* so a second
/// concurrent caller will see the updated timestamp once it acquires the lock.
///
/// # Why hold the lock while sleeping?
/// Dropping the lock before sleeping creates a race: two threads can both
/// observe that they're within budget, both sleep for similar durations, and
/// then both fire requests within the same window.  Holding the lock prevents
/// that by making each caller wait for the previous one to finish sleeping and
/// update the timestamp.
pub struct RateLimiter {
    last:         Mutex<Option<Instant>>,
    min_interval: Duration,
    /// Maximum time a caller is willing to wait.  If the remaining gap exceeds
    /// this, the caller gets an immediate `Err` instead of blocking.
    max_wait:     Duration,
}

impl RateLimiter {
    /// VirusTotal **public** API: 4 requests / minute.
    /// We use 15.5 s (≈ 15 500 ms) to give a small safety margin.
    pub fn virustotal_public() -> Self {
        Self {
            last:         Mutex::new(None),
            min_interval: Duration::from_millis(15_500),
            max_wait:     Duration::from_secs(60),
        }
    }

    /// Acquire a rate-limit slot, sleeping if necessary.
    ///
    /// Returns `Ok(())` when the caller may proceed, or
    /// `Err(msg)` when the required wait exceeds `max_wait`.
    pub fn acquire(&self) -> Result<(), String> {
        // Intentionally holds the MutexGuard across thread::sleep — see doc comment.
        let mut guard = self.last.lock().map_err(|_| "RateLimiter mutex poisoned".to_string())?;

        if let Some(prev) = *guard {
            let elapsed = prev.elapsed();
            if elapsed < self.min_interval {
                let wait = self.min_interval - elapsed;
                if wait > self.max_wait {
                    return Err(format!(
                        "Another VirusTotal request is already in progress. \
                         Please wait {:.0} more seconds.",
                        wait.as_secs_f32()
                    ));
                }
                info!("[virustotal] rate-limiting: sleeping {wait:.1?}");
                std::thread::sleep(wait);
            }
        }

        *guard = Some(Instant::now());
        Ok(())
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// VirusTotal Transform
// ═════════════════════════════════════════════════════════════════════════════

/// Query the VirusTotal v3 API for reputation and malware data.
///
/// # Setup
/// 1. Create a free account at <https://www.virustotal.com/gui/join-us>.
/// 2. Copy your API key from **Profile → API Key**.
/// 3. Open **Settings → API Keys** inside Link Mage and paste the key.
///
/// # Rate limits (free tier)
/// - 4 requests / minute
/// - 500 requests / day
/// - 15 500 requests / month
///
/// The built-in rate limiter enforces the 4 req/min limit automatically.
/// A 429 response from VirusTotal (e.g. from the daily cap) is returned as
/// an `Err` with a human-readable message.
///
/// # Output nodes
/// One `VirusTotalReport` node is created and connected to the seed node via
/// a `"vt-report"` edge.  The node carries the following properties:
///
/// | Property           | Description |
/// |--------------------|-------------|
/// | `value`            | Human summary: `"2/72 malicious"` |
/// | `malicious`        | Count of engines that flagged the target |
/// | `suspicious`       | Count of engines that reported suspicious |
/// | `harmless`         | Count of engines that consider it harmless |
/// | `undetected`       | Count of engines with no verdict yet |
/// | `total_engines`    | Total number of AV engines that scanned |
/// | `reputation`       | VT community reputation score |
/// | `verdict`          | `"clean"` / `"suspicious"` / `"malicious"` |
/// | `threat_label`     | Suggested threat label (if available) |
/// | `categories`       | Top URL / domain categories (comma-separated) |
/// | `last_analysis_date` | Unix timestamp of the most recent scan |
pub struct VirusTotalTransform {
    client:       reqwest::blocking::Client,
    rate_limiter: RateLimiter,
}

impl VirusTotalTransform {
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(concat!("link-mage/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("failed to build reqwest blocking client - this should only happen if the system TLS configuration is broken");

        Self {
            client,
            rate_limiter: RateLimiter::virustotal_public(),
        }
    }

    /// Build the VirusTotal API URL for the given node type + value.
    /// Build the API URL for a node. The value is pushed through
    /// `path_segments_mut`, which percent-encodes it — a crafted value
    /// containing `/`, `?` or `#` cannot rewrite the request path.
    fn api_url(node_type: &str, value: &str) -> Result<String, String> {
        let (segment, value) = match node_type {
            "Domain" => ("domains", value),
            "IP" | "IPv6" => ("ip_addresses", value),
            "URL" => {
                // The VT /urls/{id} endpoint requires a URL-safe base64-encoded
                // SHA-256 of the URL, which would need a crypto dependency.
                // As a practical fallback, extract the host and query the
                // /domains endpoint — sufficient for most OSINT use cases.
                let domain = value
                    .trim_start_matches("http://")
                    .trim_start_matches("https://")
                    .split('/')
                    .next()
                    .unwrap_or(value);
                ("domains", domain)
            }
            t => return Err(format!(
                "VirusTotal does not support node type '{t}'. \
                 Supported: Domain, IP, IPv6, URL."
            )),
        };

        let mut url = reqwest::Url::parse("https://www.virustotal.com/api/v3")
            .map_err(|e| format!("bad VirusTotal base URL: {e}"))?;
        {
            let mut segs = url
                .path_segments_mut()
                .map_err(|_| "VirusTotal URL cannot be a base".to_string())?;
            segs.push(segment);
            segs.push(value);
        }
        Ok(url.to_string())
    }
}

impl Default for VirusTotalTransform {
    fn default() -> Self { Self::new() }
}

impl Transform for VirusTotalTransform {
    fn name(&self) -> String {
        "VirusTotal Lookup".into()
    }

    fn description(&self) -> String {
        "Query VirusTotal for malware, reputation, and AV engine results \
         (requires a free API key — see Settings → API Keys)."
            .into()
    }

    fn applicable_types(&self) -> Vec<String> {
        vec!["Domain".into(), "IP".into(), "IPv6".into(), "URL".into()]
    }

    fn run(&self, seed: &Node) -> Result<(Vec<Node>, Vec<Edge>), String> {
        // ── 1. API key ────────────────────────────────────────────────────────
        let api_key = crate::keychain::get_key(crate::keychain::VIRUSTOTAL)
            .map_err(|e| format!("Keychain error: {e}"))?
            .ok_or_else(|| {
                "VirusTotal API key is not configured.\n\
                 Open Settings → API Keys and paste your key from \
                 virustotal.com/gui/user/settings/apikey."
                    .to_string()
            })?;

        // ── 2. Resolve the seed node ──────────────────────────────────────────
        let node = seed;

        let value = node_value(node).to_string();
        if value.is_empty() {
            return Err("Node has no 'value' property to look up in VirusTotal.".into());
        }

        // ── 3. Build URL ──────────────────────────────────────────────────────
        let url = Self::api_url(&node.node_type, &value)?;

        // ── 4. Rate-limit: blocks up to max_wait ──────────────────────────────
        self.rate_limiter.acquire()?;

        // ── 5. HTTP request ───────────────────────────────────────────────────
        info!("[virustotal] querying {url}");
        let resp = self.client
            .get(&url)
            .header("x-apikey", &api_key)
            .send()
            .map_err(|e| format!("VirusTotal request failed: {e}"))?;

        // ── 6. HTTP error handling ────────────────────────────────────────────
        match resp.status().as_u16() {
            200 => {}
            401 => return Err(
                "VirusTotal: invalid or expired API key.\n\
                 Update it in Settings → API Keys."
                    .into(),
            ),
            403 => return Err(
                "VirusTotal: API access forbidden.\n\
                 Check that your key has the required permissions."
                    .into(),
            ),
            404 => return Err(format!(
                "'{value}' was not found in the VirusTotal database."
            )),
            429 => return Err(
                "VirusTotal rate limit exceeded (daily or monthly quota).\n\
                 Free accounts allow 500 requests/day and 15 500/month."
                    .into(),
            ),
            s => return Err(format!("VirusTotal returned HTTP {s}")),
        }

        // ── 7. Parse JSON ─────────────────────────────────────────────────────
        let body: serde_json::Value = resp
            .json()
            .map_err(|e| format!("Failed to parse VirusTotal response: {e}"))?;

        let attrs = &body["data"]["attributes"];
        let stats = &attrs["last_analysis_stats"];

        let malicious  = stats["malicious"] .as_i64().unwrap_or(0);
        let suspicious = stats["suspicious"].as_i64().unwrap_or(0);
        let harmless   = stats["harmless"]  .as_i64().unwrap_or(0);
        let undetected = stats["undetected"].as_i64().unwrap_or(0);
        let total      = malicious + suspicious + harmless + undetected;
        let reputation = attrs["reputation"].as_i64().unwrap_or(0);

        let threat_label = attrs["popular_threat_classification"]["suggested_threat_label"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let last_analysis_date = attrs["last_analysis_date"]
            .as_i64()
            .unwrap_or(0);

        // Aggregate VT categories — deduplicate and take the top 3 unique values.
        let categories: String = attrs["categories"]
            .as_object()
            .map(|obj| {
                let unique: std::collections::BTreeSet<&str> =
                    obj.values().filter_map(|v| v.as_str()).collect();
                unique.into_iter().take(3).collect::<Vec<_>>().join(", ")
            })
            .unwrap_or_default();

        // ── 8. Derive a verdict ───────────────────────────────────────────────
        let verdict = if malicious > 5 {
            "malicious"
        } else if malicious > 0 || suspicious > 0 {
            "suspicious"
        } else {
            "clean"
        };

        let summary = format!("{malicious}/{total} malicious");
        debug!("[virustotal] {value} → {summary} (verdict: {verdict})");

        // ── 9. Build the report node using the shared helper ──────────────────
        let mut extra: Vec<(&str, String)> = vec![
            ("malicious",      malicious.to_string()),
            ("suspicious",     suspicious.to_string()),
            ("harmless",       harmless.to_string()),
            ("undetected",     undetected.to_string()),
            ("total_engines",  total.to_string()),
            ("reputation",     reputation.to_string()),
            ("verdict",        verdict.to_string()),
            ("target",         value.clone()),
        ];
        if !threat_label.is_empty() {
            extra.push(("threat_label", threat_label));
        }
        if !categories.is_empty() {
            extra.push(("categories", categories));
        }
        if last_analysis_date > 0 {
            extra.push(("last_analysis_date", last_analysis_date.to_string()));
        }

        let (report_node, report_edge) = create_result_node(
            &seed.id,
            "VirusTotalReport",
            &summary,
            extra,
            "vt-report",
        );

        info!(
            "[virustotal] {value} → verdict={verdict}, \
             malicious={malicious}/{total}, reputation={reputation}"
        );

        Ok((vec![report_node], vec![report_edge]))
    }
}

#[cfg(test)]
mod virustotal_tests {
    use super::*;

    #[test]
    fn vt_applicable_types_include_domain_ip_url() {
        let t = VirusTotalTransform::new();
        let types = t.applicable_types();
        assert!(types.iter().any(|s| s == "Domain"));
        assert!(types.iter().any(|s| s == "IP"));
        assert!(types.iter().any(|s| s == "IPv6"));
        assert!(types.iter().any(|s| s == "URL"));
    }

    #[test]
    fn vt_api_url_domain() {
        let url = VirusTotalTransform::api_url("Domain", "example.com").unwrap();
        assert_eq!(url, "https://www.virustotal.com/api/v3/domains/example.com");
    }

    #[test]
    fn vt_api_url_ipv4() {
        let url = VirusTotalTransform::api_url("IP", "8.8.8.8").unwrap();
        assert_eq!(url, "https://www.virustotal.com/api/v3/ip_addresses/8.8.8.8");
    }

    #[test]
    fn vt_api_url_ipv6() {
        let url = VirusTotalTransform::api_url("IPv6", "2001:4860:4860::8888").unwrap();
        assert!(url.contains("/api/v3/ip_addresses/"));
    }

    #[test]
    fn vt_api_url_unsupported_type_returns_err() {
        assert!(VirusTotalTransform::api_url("Hash", "abc123").is_err());
        assert!(VirusTotalTransform::api_url("Person", "Alice").is_err());
    }

    #[test]
    fn vt_api_url_percent_encodes_path_injection() {
        let url = VirusTotalTransform::api_url(
            "Domain",
            "evil.com/../../x?y=1#z",
        ).unwrap();
        // The crafted value must stay a single, encoded path segment —
        // `/`, `?` and `#` are percent-encoded so they cannot rewrite the
        // request path (`=` is a legal pchar and stays raw).
        assert!(url.starts_with("https://www.virustotal.com/api/v3/domains/"), "{url}");
        assert!(url.contains("evil.com%2F..%2F..%2Fx%3Fy=1%23z"), "{url}");
        assert!(!url.contains('?'), "query must be encoded: {url}");
        assert!(!url.contains('#'), "fragment must be encoded: {url}");
    }

    #[test]
    fn vt_no_api_key_returns_descriptive_error() {
        // When no key is configured, run() must return Err before touching
        // the network.  We fabricate a Domain node and expect the "not
        // configured" message.
        //
        // Note: this test reads from the real system keychain.  If the CI
        // environment happens to have a key stored it will proceed to a
        // network call (and likely fail with a real VT error).  Mark ignored
        // in environments without network access.
        let t = VirusTotalTransform::new();
        let mut node = Node::new("Domain");
        node.properties.insert("value".into(), "example.com".into());

        if crate::keychain::has_key(crate::keychain::VIRUSTOTAL).unwrap_or(false) {
            // Key present — skip to avoid hitting the real API in tests.
            return;
        }

        let err = t.run(&node).unwrap_err();
        assert!(
            err.to_lowercase().contains("api key") || err.to_lowercase().contains("not configured"),
            "expected 'API key not configured' message, got: {err}"
        );
    }

    #[test]
    fn rate_limiter_allows_first_request_immediately() {
        let rl = RateLimiter {
            last:         Mutex::new(None),
            min_interval: Duration::from_secs(15),
            max_wait:     Duration::from_secs(60),
        };
        // First call: no previous timestamp, should return Ok without sleeping.
        let start = Instant::now();
        assert!(rl.acquire().is_ok());
        assert!(start.elapsed() < Duration::from_millis(100));
    }

    #[test]
    fn rate_limiter_rejects_when_wait_exceeds_max() {
        // Simulate a request that happened 1 second ago — wait = ~14.5 s
        // which is less than max_wait (60 s), so it would normally sleep.
        // Use a very low max_wait to make the test fast.
        let rl = RateLimiter {
            last:         Mutex::new(Some(Instant::now())),
            min_interval: Duration::from_secs(30),
            max_wait:     Duration::from_millis(500), // deliberately tiny
        };
        let result = rl.acquire();
        assert!(result.is_err(), "should reject when wait > max_wait");
        assert!(result.unwrap_err().to_lowercase().contains("wait"));
    }
}

#[cfg(test)]
mod machine_tests {
    use super::*;

    /// Regression test: every step of every Machine must resolve to a real
    /// transform. Name drift here used to make machines silently skip steps.
    #[test]
    fn machine_steps_resolve() {
        let registry = TransformRegistry::new();
        for machine in &registry.machines.machines {
            for step in &machine.transforms {
                assert!(
                    registry.find(step).is_some(),
                    "machine '{}' references unknown transform '{}'",
                    machine.name,
                    step
                );
            }
        }
    }
}
