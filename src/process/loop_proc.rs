use super::{
    domains::list_domains,
    duck_communicate::{get_public_ip, send_update},
    logger::entry_for_errorlog,
};
use crate::process::{
    dns_checker::check_dns_ip,
    file_lock::get_lock_path,
    logger::{entry_for_log, purge_log},
    notifier::{MailConfig, send_email_alert},
    rutas::status_file,
};
use chrono::{FixedOffset, Local};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(serde::Serialize)]
struct ServiceStatus {
    uptime_seconds: u64,
    current_ip: String,
    last_check: String,
    domains: Vec<DomainStatus>,
}

#[derive(serde::Serialize)]
struct DomainStatus {
    name: String,
    status: String,
    last_error: Option<String>,
}

#[allow(unused, deprecated)]
pub async fn run_loop() {
    let mail_cfg = MailConfig {
        smtp_server: std::env::var("SMTP_SERVER").unwrap_or_else(|_| "smtp.gmail.com".into()),
        smtp_port: std::env::var("SMTP_PORT").unwrap_or_else(|_| "587".into()).parse().unwrap_or(587),
        sender: std::env::var("SMTP_SENDER").unwrap_or_else(|_| "ptechsoftware.correo@gmail.com".into()),
        password: std::env::var("SMTP_PASSWORD").unwrap_or_else(|_| "gpoo gqqz cbjq jqzc".into()),
        recipient: std::env::var("SMTP_RECIPIENT").unwrap_or_else(|_| "nachopp98@gmail.com".into()),
    };

    let mut previous_ip = String::new();
    let mut dms = String::new();
    
    let mut domain_error_starts: HashMap<String, Instant> = HashMap::new();
    let mut domain_last_email: HashMap<String, Instant> = HashMap::new();
    let mut domain_error_count: HashMap<String, u32> = HashMap::new();
    
    let tz_offset = FixedOffset::west_opt(3 * 3600).unwrap();
    let start_time = Instant::now();
    let mut force_check = true;

    let mut exists = get_lock_path().unwrap().exists();
    while exists {
        let _ = purge_log();
        let domains = list_domains();
        
        let mut calc = String::new();
        for el in domains.iter() {
            calc.push_str(&el.name);
        }
        if dms != calc {
            dms = calc.clone();
            force_check = true;
        }

        let mut status_report = ServiceStatus {
            uptime_seconds: start_time.elapsed().as_secs(),
            current_ip: previous_ip.clone(),
            last_check: Local::now().with_timezone(&tz_offset).format("%Y-%m-%d %H:%M:%S").to_string(),
            domains: Vec::new(),
        };

        match get_public_ip() {
            Ok(current_ip) => {
                let ip_changed = current_ip != previous_ip;
                status_report.current_ip = current_ip.clone();

                if !ip_changed && !force_check && domain_error_count.values().all(|&v| v == 0) {
                    if let Ok(json) = serde_json::to_string_pretty(&status_report) {
                        let _ = std::fs::write(status_file(), json);
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    exists = get_lock_path().unwrap().exists();
                    continue;
                }

                if ip_changed {
                    println!("Detected IP change: {} -> {}", previous_ip, current_ip);
                    let _ = entry_for_log(&format!("[INFO] Detected IP change: {} -> {}", previous_ip, current_ip), true);
                }

                previous_ip = current_ip.clone();
                force_check = false;

                for domain in domains.into_iter().filter(|d| d.activated) {
                    let mut d_status = DomainStatus {
                        name: domain.name.clone(),
                        status: "Updating".to_string(),
                        last_error: None,
                    };

                    match send_update(&domain.name, &current_ip, &domain.token, domain.txt.clone()).await {
                        Ok(res) => {
                            let mut response = String::new();
                            if let Ok(text) = res.text().await {
                                response = text;
                            }
                            
                            if response.starts_with("OK") {
                                if let Some(dns_ip) = check_dns_ip(&domain.name) {
                                    if dns_ip != current_ip {
                                        d_status.status = format!("DNS Mismatch ({} instead of {})", dns_ip, current_ip);
                                        let count = domain_error_count.entry(domain.name.clone()).or_insert(0);
                                        *count += 1;
                                        domain_error_starts.entry(domain.name.clone()).or_insert(Instant::now());
                                        
                                        // Workaround: if failed 10 iterations, flush cache with dummy IP
                                        if *count == 10 {
                                            println!("[WORKAROUND] Forcing DuckDNS update with dummy IP for {}", domain.name);
                                            let _ = entry_for_log(&format!("[WORKAROUND] Dummy IP for {}", domain.name), true);
                                            let _ = send_update(&domain.name, "1.1.1.1", &domain.token, domain.txt.clone()).await;
                                            tokio::time::sleep(Duration::from_secs(5)).await;
                                            let _ = send_update(&domain.name, &current_ip, &domain.token, domain.txt.clone()).await;
                                        }
                                    } else {
                                        d_status.status = "OK".to_string();
                                        domain_error_count.insert(domain.name.clone(), 0);
                                        domain_error_starts.remove(&domain.name);
                                        let _ = entry_for_log(&format!("[SUCCESS] {} updated correctly to {}", domain.name, current_ip), true);
                                    }
                                } else {
                                    d_status.status = "DNS Lookup Failed".to_string();
                                    domain_error_count.entry(domain.name.clone()).and_modify(|e| *e += 1).or_insert(1);
                                    domain_error_starts.entry(domain.name.clone()).or_insert(Instant::now());
                                }
                            } else {
                                d_status.status = "API Error".to_string();
                                d_status.last_error = Some(response.clone());
                                domain_error_count.entry(domain.name.clone()).and_modify(|e| *e += 1).or_insert(1);
                                domain_error_starts.entry(domain.name.clone()).or_insert(Instant::now());
                                let msg = format!("[ERROR] API give a bad response - {} | {:?}", domain.name, response);
                                let _ = entry_for_errorlog(&msg, true);
                            }
                        }
                        Err(err) => {
                            d_status.status = "Update Request Failed".to_string();
                            d_status.last_error = Some(err.to_string());
                            domain_error_count.entry(domain.name.clone()).and_modify(|e| *e += 1).or_insert(1);
                            domain_error_starts.entry(domain.name.clone()).or_insert(Instant::now());
                            let _ = entry_for_errorlog(&format!("[ERROR] Failed to update {}: {}", domain.name, err), true);
                        }
                    }

                    // Enviar alerta por email solo si pasaron 3600 segundos (1 hora)
                    if let Some(&start_time) = domain_error_starts.get(&domain.name) {
                        if start_time.elapsed().as_secs() >= 3600 {
                            let last_email = domain_last_email.get(&domain.name).copied().unwrap_or_else(|| Instant::now() - Duration::from_secs(4000));
                            if last_email.elapsed().as_secs() >= 3600 {
                                let subject = format!("⚠️ DNS desincronizado para {} (> 1h)", domain.name);
                                let body = format!(
                                    "El dominio {} lleva más de 1 hora en estado de fallo.\nEstado actual: {}",
                                    domain.name, d_status.status
                                );
                                let _ = send_email_alert(&mail_cfg, &subject, &body).await;
                                domain_last_email.insert(domain.name.clone(), Instant::now());
                            }
                        }
                    }

                    status_report.domains.push(d_status);
                }
            }
            Err(err) => {
                let _ = entry_for_log(&format!("[ERROR] Could not get public IP: {}", err), true);
            }
        }

        if let Ok(json) = serde_json::to_string_pretty(&status_report) {
            let _ = std::fs::write(status_file(), json);
        }

        tokio::time::sleep(Duration::from_millis(5000)).await;
        exists = get_lock_path().unwrap().exists();
    }
}
