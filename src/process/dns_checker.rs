use std::process::Command;
use std::net::Ipv4Addr;

/// Ejecuta `nslookup domain` y devuelve la IP encontrada (si existe)
pub fn check_dns_ip(domain: &str) -> Option<String> {
    let output = Command::new("nslookup")
        .arg(domain)
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Buscar algo tipo: "Address: 203.0.113.5"
    for line in stdout.lines() {
        if line.trim_start().starts_with("Address:") {
            if let Some(ip_str) = line.split_whitespace().nth(1) {
                // Validar formato IP
                if ip_str.parse::<Ipv4Addr>().is_ok() {
                    return Some(ip_str.to_string());
                }
            }
        }
    }
    None
}
