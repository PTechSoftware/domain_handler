use crate::process::logger::{entry_for_log, purge_log};
use super::{domains::list_domains, duck_communicate::{get_public_ip, send_update}, logger::entry_for_errorlog};


#[allow(unused)]
pub async fn run_loop() {
    let mut previous_ip = String::new();
    let mut had_previous_errors = false;
    let mut dms = String::new();

    loop {

        let _ = purge_log();
        let domains = list_domains();
        let dms_cal = domains.join("");

        match get_public_ip() {
            Ok(current_ip) => {
                let ip_changed = current_ip != previous_ip;

                if !ip_changed && !had_previous_errors {
                    // Nada que hacer
                    continue;
                }

                if ip_changed {
                    println!("Detected IP change: {} -> {}", previous_ip, current_ip);
                    let _ = entry_for_log(&format!(
                        "[INFO] Detected IP change: {} -> {}",
                        previous_ip, current_ip
                    ), false);
                } else if had_previous_errors {
                    println!("Retrying due to previous errors");
                    let _ = entry_for_log("[INFO] Retrying updates due to previous errors", false);
                }

                previous_ip = current_ip.clone();
                had_previous_errors = false;

                for domain in domains.into_iter().filter(|d| d.activated) {
                    match send_update(&domain.name, &current_ip, &domain.token, domain.txt.clone()).await {
                        Ok(res) => {
                            println!("Updated {}: {:?}", domain.name, res.status());
                            let _ = entry_for_log(&format!(
                                "[SUCCESS] Updated {}: {}",
                                domain.name,
                                res.status()
                            ), false);
                            //override if it s ok
                            entry_for_errorlog("",true);
                        }
                        Err(err) => {
                            had_previous_errors = true;
                            let _ = entry_for_errorlog(&format!(
                                "[ERROR] Failed to update {}: {}",
                                domain.name, err
                            ), false);
                        }
                    }
                }
            }
            Err(err) => {
                had_previous_errors = true;
                let _ = entry_for_log(&format!("[ERROR] Could not get public IP: {}", err), had_previous_errors);
            }
        }
    }
}
