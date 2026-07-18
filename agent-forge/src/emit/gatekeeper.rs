//! Mechanical pre-write screening. agent-forge cannot invoke a semantic
//! gatekeeper agent, so this module catches the categories a regex can catch
//! and the caller is told when a semantic pass is still required.

use crate::tools::ToolError;
use std::path::Path;
use std::process::Command;

/// Whether a dotted-quad token is an RFC1918 private address. Parsing beats a
/// regex here because the range checks are exact rather than approximated.
fn is_private_ipv4(token: &str) -> bool {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    let octets: Vec<u8> = match parts.iter().map(|p| p.parse::<u8>()).collect() {
        Ok(v) => v,
        Err(_) => return false,
    };
    match octets[0] {
        10 => true,
        192 => octets[1] == 168,
        172 => (16..=31).contains(&octets[1]),
        _ => false,
    }
}

/// Strip punctuation that abuts a token in prose so it can be parsed on its own.
/// Interior dots survive because they separate the octets; leading and trailing
/// ones do not, so a sentence-final "10.0.0.1." still parses as an address.
fn trim_token(token: &str) -> &str {
    token.trim_matches(|c: char| !c.is_ascii_alphanumeric())
}

/// Scan emitted content for material that must never reach a public repository.
/// Returns one human-readable finding per detection. An empty result means the
/// mechanical checks passed; it does not mean the content is safe, which is why
/// callers still require a semantic pass on public repositories.
pub fn scan_for_leaks(content: &str) -> Vec<String> {
    let mut findings = Vec::new();

    for raw in content.split_whitespace() {
        let token = trim_token(raw);
        if is_private_ipv4(token) {
            findings.push(format!("private address: {}", token));
        }
    }

    if content.contains("PRIVATE KEY-----") {
        findings.push("private key block".to_string());
    }

    let lowered = content.to_lowercase();
    for marker in ["api_key", "apikey", "password", "secret_key", "token"] {
        for (idx, _) in lowered.match_indices(marker) {
            let rest = &lowered[idx + marker.len()..];
            let rest = rest.trim_start();
            if rest.starts_with('=') || rest.starts_with(':') {
                let value = rest.trim_start_matches(['=', ':']).trim_start();
                // A bare mention such as "password: none recorded" is prose, not
                // a credential. Require a value with some length and no spaces.
                let candidate: String = value.chars().take_while(|c| !c.is_whitespace()).collect();
                if candidate.len() >= 6 {
                    findings.push(format!("credential-shaped assignment: {}", marker));
                    break;
                }
            }
        }
    }

    findings.sort();
    findings.dedup();
    findings
}

/// Refuse to emit content that trips the leak scan, naming the findings in the
/// error. Every emission path shares this so the decision to refuse, and the
/// wording of the refusal, live in exactly one place.
pub fn guard_no_leaks(content: &str) -> Result<(), ToolError> {
    let findings = scan_for_leaks(content);
    if findings.is_empty() {
        return Ok(());
    }
    Err(ToolError::InvalidValue(format!(
        "refusing to emit: leak scan found {}",
        findings.join(", ")
    )))
}

/// Best-effort check of whether the repository at `repo_root` is public.
/// Returns true when visibility cannot be determined, so an unknown repository
/// is screened as if it were public. Failing safe is the whole point.
pub fn is_public_repo(repo_root: &Path) -> bool {
    let output = Command::new("gh")
        .args(["repo", "view", "--json", "visibility", "-q", ".visibility"])
        .current_dir(repo_root)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let visibility = String::from_utf8_lossy(&o.stdout).trim().to_uppercase();
            visibility != "PRIVATE"
        }
        _ => true,
    }
}

#[cfg(test)]
/// Tests for the mechanical leak scan.
mod tests {
    use super::*;

    /// Private RFC1918 addresses are flagged in all three ranges.
    #[test]
    fn flags_private_ipv4_ranges() {
        assert!(!scan_for_leaks("host at 10.0.0.1 today").is_empty());
        assert!(!scan_for_leaks("host at 192.168.1.1 today").is_empty());
        assert!(!scan_for_leaks("host at 172.16.0.5 today").is_empty());
    }

    /// A public address is not flagged, so ordinary prose stays publishable.
    #[test]
    fn ignores_public_ipv4() {
        assert!(scan_for_leaks("resolved 8.8.8.8 fine").is_empty());
    }

    /// Private key material is flagged.
    #[test]
    fn flags_private_key_blocks() {
        assert!(!scan_for_leaks("-----BEGIN OPENSSH PRIVATE KEY-----").is_empty());
    }

    /// Assignments that look like credentials are flagged.
    #[test]
    fn flags_credential_assignments() {
        assert!(!scan_for_leaks("api_key=abc123def").is_empty());
        assert!(!scan_for_leaks("password = hunter2").is_empty());
    }

    /// Clean technical prose produces no findings.
    #[test]
    fn clean_content_produces_no_findings() {
        let md = "# Record: Add a thing\n\n- **why:** it was simpler.\n";
        assert!(scan_for_leaks(md).is_empty());
    }

    /// An address abutted by sentence punctuation is still detected. Prose ends
    /// sentences with a period, and a trailing dot must not hide a leak.
    #[test]
    fn flags_addresses_abutted_by_punctuation() {
        assert!(!scan_for_leaks("it talks to 10.0.0.1.").is_empty());
        assert!(!scan_for_leaks("see (192.168.1.1), the host").is_empty());
    }

    /// The shared guard passes clean content through.
    #[test]
    fn guard_allows_clean_content() {
        assert!(guard_no_leaks("nothing to see here").is_ok());
    }

    /// The shared guard refuses leaking content and names the findings.
    #[test]
    fn guard_refuses_leaking_content() {
        let err = guard_no_leaks("host 10.0.0.1").unwrap_err();
        assert!(err.to_string().contains("refusing to emit"));
    }
}
