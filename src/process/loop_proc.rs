use super::{
    domains::list_domains,
    duck_communicate::{get_public_ip, send_update},
    logger::entry_for_errorlog,
};
use crate::process::{
    dns_checker::check_dns_ip,
    logger::{entry_for_log, purge_log}, notifier::{send_email_alert, MailConfig},
};

#[allow(unused)]
pub async fn run_loop() {
    // En tu run_loop, después de detectar un nslookup incorrecto:
    let mail_cfg = MailConfig {
        smtp_server: "smtp.gmail.com".into(),
        smtp_port: 587,
        sender: "ptechsoftware.correo@gmail.com".into(),
        password: "gpoo gqqz cbjq jqzc".into(),
        recipient: "nachopp98@gmail.com".into(),
    };
    let mut previous_ip = String::new();
    let mut had_previous_errors = false;
    let mut flag = false;
    let mut dms = String::new();

    loop {
        //Purgar el log si es necesario
        let _ = purge_log();
        //Listar dominios
        let domains = list_domains();
        //Verificar si hubo cambios en los dominios
        let mut calc = String::new();
        for el in domains.iter() {
            calc.push_str(&el.name);
        }
        if dms != calc {
            dms = calc.clone();
            flag = true;
        }
        //Obtener IP pública
        match get_public_ip() {
            Ok(current_ip) => {
                let ip_changed = current_ip != previous_ip;

                if !ip_changed && !had_previous_errors && !flag {
                    // Nada que hacer
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    continue;
                }

                if ip_changed {
                    println!("Detected IP change: {} -> {}", previous_ip, current_ip);
                    let _ = entry_for_log(
                        &format!(
                            "[INFO] Detected IP change: {} -> {}",
                            previous_ip, current_ip
                        ),
                        false,
                    );
                } else if had_previous_errors {
                    println!("Retrying due to previous errors");
                    let _ = entry_for_log("[INFO] Retrying updates due to previous errors", false);
                }

                previous_ip = current_ip.clone();
                had_previous_errors = false;
                flag = false;
                let mut err_ctr = 0;
                for domain in domains.into_iter().filter(|d| d.activated) {
                    match send_update(&domain.name, &current_ip, &domain.token, domain.txt.clone())
                        .await
                    {
                        Ok(res) => {
                            println!("Updated {}: {:?}", domain.name, res.status());
                            let _ = entry_for_log(
                                &format!("[SUCCESS] Updated {}: {}", domain.name, res.status()),
                                false,
                            );

                            // 🔹 Verificar si el DNS refleja la IP nueva
                            if let Some(dns_ip) = check_dns_ip(&domain.name) {
                                if dns_ip != current_ip {
                                    let msg = format!(
                                        "[WARN] Domain {} still resolves to {} instead of {}",
                                        domain.name, dns_ip, current_ip
                                    );
                                    println!("{}", msg);
                                    let _ = entry_for_errorlog(&msg, false);
                                    had_previous_errors = true;
                                    err_ctr += 1;
                                } else {
                                    let _ = entry_for_log(
                                        &format!(
                                            "[OK] DNS {} resolved correctly to {}",
                                            domain.name, dns_ip
                                        ),
                                        false,
                                    );
                                }
                            } else {
                                let msg = format!(
                                    "[ERROR] Could not resolve domain {} via nslookup.",
                                    domain.name
                                );
                                println!("{}", msg);
                                let _ = entry_for_errorlog(&msg, false);
                                had_previous_errors = true;
                                err_ctr += 1;
                            }
                        }
                        Err(err) => {
                            let subject = format!("⚠️ DNS desincronizado para {}", domain.name);
                            let body = format!(
                                "El dominio {} aún apunta a {} en lugar de {}.\nHora: {}",
                                domain.name,
                                previous_ip,
                                current_ip,
                                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
                            );
                            let _ = send_email_alert(&mail_cfg, &subject, &body).await;
                            had_previous_errors = true;
                            let _ = entry_for_errorlog(
                                &format!("[ERROR] Failed to update {}: {}", domain.name, err),
                                false,
                            );
                            err_ctr += 1;
                        }
                    }
                }
                if err_ctr > 0 {
                    flag = true;
                } else {
                    flag = false;
                }
            }
            Err(err) => {
                had_previous_errors = true;
                let subject = format!("⚠️ Error obteniendo IP pública");
                let body = format!(
                    "No se pudo obtener la IP pública.\nError: {}\nHora: {}",
                    err,
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
                );
                let _ = send_email_alert(&mail_cfg, &subject, &body).await;
                let _ = entry_for_log(
                    &format!("[ERROR] Could not get public IP: {}", err),
                    had_previous_errors,
                );
            }
        }
    }
}
