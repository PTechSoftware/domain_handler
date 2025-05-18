use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use crate::process::rutas::{bin_dir, bin_path, config_dir, config_file, service_path, systemd_user_dir};
pub fn install_service() -> anyhow::Result<()> {
    match fs::create_dir_all(bin_dir()) {
        Ok(_) => println!("[OK] Created bin dir"),
        Err(e) => eprintln!("[ERR] Creating bin dir: {e}"),
    }

    match fs::create_dir_all(config_dir()) {
        Ok(_) => println!("[OK] Created config dir"),
        Err(e) => eprintln!("[ERR] Creating config dir: {e}"),
    }

    match fs::create_dir_all(systemd_user_dir()) {
        Ok(_) => println!("[OK] Created systemd user dir"),
        Err(e) => eprintln!("[ERR] Creating systemd user dir: {e}"),
    }

    // 🔽 Copiar binario desde su propia ubicación
    let this_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[ERR] Getting current exe path: {e}");
            return Err(e.into());
        }
    };

    if this_exe.is_file() {
        match fs::copy(&this_exe, &bin_path()) {
            Ok(_) => println!("[OK] Copied binary"),
            Err(e) => eprintln!("[ERR] Copying binary: {e}"),
        }

        match fs::set_permissions(&bin_path(), fs::Permissions::from_mode(0o755)) {
            Ok(_) => println!("[OK] Set permissions on binary"),
            Err(e) => eprintln!("[ERR] Setting binary permissions: {e}"),
        }
    } else {
        eprintln!(
            "[ERR] Source binary is not a regular file: {}",
            this_exe.display()
        );
    }

    if Path::new("domainhdlr.json").exists() {
        match fs::copy("domainhdlr.json", &config_file()) {
            Ok(_) => println!("[OK] Copied config file"),
            Err(e) => eprintln!("[ERR] Copying config file: {e}"),
        }

        match fs::set_permissions(&config_file(), fs::Permissions::from_mode(0o644)) {
            Ok(_) => println!("[OK] Set permissions on config file"),
            Err(e) => eprintln!("[ERR] Setting config permissions: {e}"),
        }
    } else {
        println!("[INFO] Config file not found, skipping");
    }

    let bashrc_path = dirs::home_dir().unwrap().join(".bashrc");
    let export_line = r#"export PATH="$HOME/.local/bin:$PATH""#;
    let mut added_to_bashrc = false;

    if bashrc_path.exists() {
        match fs::read_to_string(&bashrc_path) {
            Ok(content) => {
                if !content.contains(export_line) {
                    match fs::OpenOptions::new().append(true).open(&bashrc_path) {
                        Ok(mut file) => {
                            let _ = writeln!(file, "\n{}", export_line);
                            println!("[OK] Added export to .bashrc");
                            added_to_bashrc = true;
                        }
                        Err(e) => eprintln!("[ERR] Appending to .bashrc: {e}"),
                    }
                }
            }
            Err(e) => eprintln!("[ERR] Reading .bashrc: {e}"),
        }
    } else {
        match fs::File::create(&bashrc_path) {
            Ok(mut file) => {
                let _ = writeln!(file, "{}", export_line);
                println!("[OK] Created .bashrc with export line");
                added_to_bashrc = true;
            }
            Err(e) => eprintln!("[ERR] Creating .bashrc: {e}"),
        }
    }

    let service_content = format!(
        r#"[Unit]
Description=Domain Handler Service for DuckDNS
After=network.target

[Service]
ExecStart={} start --detach=false
Restart=always
User={}
WorkingDirectory={}

[Install]
WantedBy=default.target
"#,
        bin_path().to_string_lossy(),
        whoami::username(),
        bin_dir().to_string_lossy()
    );

    match fs::write(service_path(), service_content) {
        Ok(_) => println!("[OK] Wrote systemd service file"),
        Err(e) => eprintln!("[ERR] Writing service file: {e}"),
    }

    match fs::set_permissions(service_path(), fs::Permissions::from_mode(0o644)) {
        Ok(_) => println!("[OK] Set permissions on service file"),
        Err(e) => eprintln!("[ERR] Setting permissions on service file: {e}"),
    }

    match Command::new("systemctl").args(["--user", "daemon-reload"]).status() {
        Ok(status) if status.success() => println!("[OK] Reloaded systemd user daemon"),
        Ok(status) => eprintln!("[ERR] systemctl reload failed with code {:?}", status.code()),
        Err(e) => eprintln!("[ERR] Running systemctl reload: {e}"),
    }

    println!("✅ Install complete");
    if added_to_bashrc {
        println!("➡️  Please run `source ~/.bashrc` or reopen terminal");
        println!("👉 Or run:\n   source <(domainhdlr install)");
        println!("{}", export_line);
    }

    Ok(())
}

pub fn uninstall_service() -> anyhow::Result<()> {
    match fs::remove_file(service_path()) {
        Ok(_) => println!("[OK] Removed service file"),
        Err(e) => eprintln!("[ERR] Removing service file: {e}"),
    }

    match fs::remove_file(bin_path()) {
        Ok(_) => println!("[OK] Removed binary"),
        Err(e) => eprintln!("[ERR] Removing binary: {e}"),
    }

    match fs::remove_file(config_file()) {
        Ok(_) => println!("[OK] Removed config file"),
        Err(e) => eprintln!("[ERR] Removing config file: {e}"),
    }

    if fs::read_dir(config_dir()).map_or(false, |mut d| d.next().is_none()) {
        match fs::remove_dir(config_dir()) {
            Ok(_) => println!("[OK] Removed config dir"),
            Err(e) => eprintln!("[ERR] Removing config dir: {e}"),
        }
    }

    if fs::read_dir(bin_dir()).map_or(false, |mut d| d.next().is_none()) {
        match fs::remove_dir(bin_dir()) {
            Ok(_) => println!("[OK] Removed bin dir"),
            Err(e) => eprintln!("[ERR] Removing bin dir: {e}"),
        }
    }

    match Command::new("systemctl").args(["daemon-reload"]).status() {
        Ok(status) if status.success() => println!("[OK] Reloaded systemd"),
        Ok(status) => eprintln!("[ERR] systemctl reload failed with code {:?}", status.code()),
        Err(e) => eprintln!("[ERR] Running systemctl reload: {e}"),
    }

    println!("🗑️ Service uninstalled");
    Ok(())
}

pub fn set_enable_on_boot(enable: bool) -> anyhow::Result<()> {
    let action = if enable { "enable" } else { "disable" };
    println!("[INFO] systemctl {} domainhdlr.service", action);

    match Command::new("systemctl")
        .args([action, "domainhdlr.service"])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                println!("[OK] Service {}d on boot", action);
            } else {
                eprintln!("[ERR] systemctl {} failed", action);
                if !output.stderr.is_empty() {
                    eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
                }
                if !output.stdout.is_empty() {
                    eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
                }
            }
        }
        Err(e) => {
            eprintln!("[ERR] Failed to run systemctl: {e}");
        }
    }

    Ok(())
}
