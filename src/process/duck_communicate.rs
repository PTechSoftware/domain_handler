use reqwest::{Client, Response};
use std::{error::Error, process::Command};

use crate::process::logger::entry_for_log;

#[allow(unused)]
pub fn get_public_ip() -> Result<String, Box<dyn Error>> {
    let output = if cfg!(target_os = "windows") {
        Command::new("nslookup")
            .arg("myip.opendns.com")
            .arg("resolver1.opendns.com")
            .output()?
    } else {
        Command::new("dig")
            .arg("+short")
            .arg("myip.opendns.com")
            .arg("@resolver1.opendns.com")
            .output()?
    };

    if output.status.success() {
        let ip = String::from_utf8(output.stdout)?.trim().to_string();
        Ok(ip)
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to get public IP",
        )))
    }
}

#[allow(dead_code)]
pub async fn send_update(domain: &str, ip : &str, token: &str,txt :Option<String>) -> Result<Response, Box<dyn Error>> {
    // Initialize the HTTP client
    let client = Client::new();

    let url = match txt {
        Some(reg) => {
            format!("https://www.duckdns.org/update?domains={}&token={}&txt={}&ip={}&verbose=true",domain,token,reg,ip)
        },
        None =>{
            format!("https://www.duckdns.org/update?domains={}&token={}&ip={}&verbose=true",domain,token,ip)
        }
    };

    println!("{}",&url);
    // Send the POST request
    let res = client
        .get(&url)
        .send()
        .await?;

    // Handle the response
    if res.status().is_success() == false  {
        let _ = entry_for_log(&format!(r#"**************************************************
Error al actualizar Dominio {}. 
Detalles:
{:?}
**************************************************"#,domain,res));
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Request failed with status: {}", res.status()),
        )))
    }
    Ok(res)
}

#[allow(dead_code)]
pub async fn send_update_no_ip(domain: &str, token: &str,txt :Option<String>) -> Result<Response, Box<dyn Error>> {
    // Initialize the HTTP client
    let client = Client::new();

    let url = match txt {
        Some(reg) => {
            format!("https://www.duckdns.org/update?domains={}&token={}&txt={}&verbose=true",domain,token,reg)
        },
        None =>{
            format!("https://www.duckdns.org/update?domains={}&token={}&verbose=true",domain,token)
        }
    };

    println!("{}",&url);
    // Send the POST request
    let res = client
        .get(&url)
        .send()
        .await?;

    // Handle the response
    if res.status().is_success() == false  {
        let _ = entry_for_log(&format!(r#"**************************************************
Error al actualizar Dominio {}. 
Detalles:
{:?}
**************************************************"#,domain,res));
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Request failed with status: {}", res.status()),
        )))
    }
    Ok(res)
}

#[cfg(test)]
mod test {
    use super::send_update;
    #[tokio::test]
    async fn test_update(){
        let ip = super::get_public_ip().unwrap();
        send_update("ptechcloud",&ip, "b2f08b23-ebf1-4d4b-8132-327b9def1bf1" ,Some("google-site-verification=8ITNQvEh75v-MRsYr-sq3409kPQxeeus80vv8isjkms".to_string()) ).await.unwrap();

    }

    #[test]
    fn test_get_ip(){
        let ip = super::get_public_ip().unwrap();
        println!("{}",ip);
    }
}