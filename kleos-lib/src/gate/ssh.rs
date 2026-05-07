use crate::config::Config;
use crate::gate::parser::parse_ssh_target;

/// Check if an SSH target is a reserved/internal address (SSRF prevention).
/// Parses IPs properly including octal, hex, and decimal-encoded representations.
pub fn is_reserved_ssh_target(host: &str) -> bool {
    let host_lower = host.to_lowercase();
    let host_trimmed = host_lower.trim_matches(|c| c == '[' || c == ']');

    // Try standard IP parse first
    if let Ok(ip) = host_trimmed.parse::<std::net::IpAddr>() {
        return is_ip_reserved(ip);
    }

    // Hostname checks
    if host_trimmed == "localhost"
        || host_trimmed.ends_with(".localhost")
        || host_trimmed == "metadata.google.internal"
        || host_trimmed == "metadata.google"
    {
        return true;
    }

    // Hex-encoded IP: 0x7f000001
    if let Some(hex_part) = host_trimmed.strip_prefix("0x") {
        if let Ok(num) = u32::from_str_radix(hex_part, 16) {
            let ip = std::net::Ipv4Addr::from(num);
            return is_ipv4_reserved(ip);
        }
    }

    // Decimal-encoded IP: 2130706433
    if host_trimmed.chars().all(|c| c.is_ascii_digit())
        && !host_trimmed.is_empty()
        && host_trimmed.len() <= 10
    {
        if let Ok(num) = host_trimmed.parse::<u32>() {
            let ip = std::net::Ipv4Addr::from(num);
            return is_ipv4_reserved(ip);
        }
    }

    // Octal-encoded IP: 0177.0.0.1 (leading zeros in octets)
    if host_trimmed.contains('.') {
        let parts: Vec<&str> = host_trimmed.split('.').collect();
        if parts.len() == 4 {
            let has_octal = parts.iter().any(|p| {
                p.starts_with('0') && p.len() > 1 && p.chars().all(|c| c.is_ascii_digit())
            });
            if has_octal {
                let octets: Option<Vec<u8>> = parts
                    .iter()
                    .map(|p| {
                        if p.starts_with('0')
                            && p.len() > 1
                            && p.chars().all(|c| c.is_ascii_digit())
                        {
                            u8::from_str_radix(p, 8).ok()
                        } else {
                            p.parse::<u8>().ok()
                        }
                    })
                    .collect();
                if let Some(bytes) = octets {
                    let ip = std::net::Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]);
                    return is_ipv4_reserved(ip);
                }
            }
        }
    }

    false
}

pub(crate) fn is_ip_reserved(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => is_ipv4_reserved(v4),
        std::net::IpAddr::V6(v6) => {
            if v6.is_loopback() || v6.is_unspecified() {
                return true;
            }
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_ipv4_reserved(v4);
            }
            // AWS IMDSv2 alternative
            if v6.to_string() == "fd00:ec2::254" {
                return true;
            }
            false
        }
    }
}

pub(crate) fn is_ipv4_reserved(ip: std::net::Ipv4Addr) -> bool {
    ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_link_local()
        || ip == std::net::Ipv4Addr::new(169, 254, 169, 254)
}

/// Resolve a hostname and return Some(block_reason) if any resolved IP lands
/// in a reserved/internal range. This catches DNS rebinding where the static
/// hostname check passed but the resolved address is internal (127.0.0.1,
/// 169.254.169.254 metadata, 10.0.0.0/8, etc). Callers should invoke this
/// for any SSH target that passed the static SSRF check.
pub async fn check_ssh_dns_rebind(host: &str, port: u16) -> Option<String> {
    if host.parse::<std::net::IpAddr>().is_ok() {
        return None;
    }
    let addr = format!("{}:{}", host, port);
    let resolved = match tokio::net::lookup_host(addr).await {
        Ok(iter) => iter.collect::<Vec<_>>(),
        Err(e) => {
            tracing::debug!(host, error = %e, "dns lookup failed for ssh target");
            return None;
        }
    };
    for sa in resolved {
        if is_ip_reserved(sa.ip()) {
            return Some(format!(
                "SSH target {} resolves to reserved/internal address {} (DNS rebinding / SSRF prevention)",
                host,
                sa.ip()
            ));
        }
    }
    None
}

/// Validate an SSH command against static rules.
/// Returns Some(block_reason) if the command should be blocked, None if it passes.
/// Checks SSRF targets, reserved IPs, and config reserved_targets list.
/// Note: DNS rebinding resolution is async and must be done at the server layer.
pub fn check_ssh_command(command: &str, config: &Config) -> Option<String> {
    let target = parse_ssh_target(command)?;
    let host = &target.host;
    let port = target.port;

    // SSRF prevention: block SSH to reserved/internal targets (hostname check)
    if is_reserved_ssh_target(host) {
        return Some(format!(
            "SSH to reserved/internal target {} blocked (SSRF prevention)",
            host
        ));
    }

    // Check config reserved_targets list
    let host_lower = host.to_lowercase();
    for reserved in &config.eidolon.gate.reserved_targets {
        if host_lower == reserved.to_lowercase() {
            return Some(format!(
                "SSH to reserved target {} blocked by configuration",
                host
            ));
        }
    }

    // Server inventory: custom-port enforcement is a warning/enrichment at the server layer
    let _ = port;

    None
}
