use super::client::MikrotikClient;
use super::models::IpBinding;
use mikrotik_rs::CommandBuilder;

impl MikrotikClient {
    pub async fn get_ip_bindings(&mut self) -> Result<Vec<IpBinding>, String> {
        println!("Fetching IP bindings...");
        let cmd = CommandBuilder::new().command("/ip/hotspot/ip-binding/print").build();
        
        let mut rx = self.device.send_command(cmd).await.map_err(|e| e.to_string())?;
        
        let mut bindings = Vec::new();
        while let Some(event) = rx.recv().await {
            match event {
                mikrotik_rs::Event::Reply { response, .. } => {
                    bindings.push(IpBinding {
                        id: response.attributes.get(".id").cloned().flatten().unwrap_or_default(),
                        address: response.attributes.get("address").cloned().flatten().unwrap_or_default(),
                        mac_address: response.attributes.get("mac-address").cloned().flatten().unwrap_or_default(),
                        binding_type: response.attributes.get("type").cloned().flatten().unwrap_or_default(),
                        comment: response.attributes.get("comment").cloned().flatten().unwrap_or_default(),
                        server: response.attributes.get("server").cloned().flatten().unwrap_or_default(),
                        disabled: response.attributes.get("disabled").cloned().flatten().unwrap_or_default() == "true",
                    });
                }
                mikrotik_rs::Event::Done { .. } => break,
                _ => {}
            }
        }
        
        println!("Fetched {} IP bindings from router", bindings.len());
        Ok(bindings)
    }
    
    pub async fn get_hotspot_servers(&mut self) -> Result<Vec<String>, String> {
        println!("Fetching hotspot servers...");
        let cmd = CommandBuilder::new().command("/ip/hotspot/print").build();
        
        let mut rx = self.device.send_command(cmd).await.map_err(|e| e.to_string())?;
        
        let mut servers = vec!["all".to_string()];
        while let Some(event) = rx.recv().await {
            match event {
                mikrotik_rs::Event::Reply { response, .. } => {
                    if let Some(Some(name)) = response.attributes.get("name") {
                        servers.push(name.clone());
                    }
                }
                mikrotik_rs::Event::Done { .. } => break,
                _ => {}
            }
        }
        Ok(servers)
    }

    #[allow(dead_code)]
    pub async fn add_binding(&mut self, mac: &str, server: &str, btype: &str, comment: &str, disabled: bool) -> Result<(), String> {
        println!("Adding binding: {} ({})", mac, comment);
        
        let disabled_str = if disabled { "yes" } else { "no" };
        
        let cmd = CommandBuilder::new()
            .command("/ip/hotspot/ip-binding/add")
            .attribute("mac-address", Some(mac))
            .attribute("type", Some(btype))
            .attribute("comment", Some(comment))
            .attribute("server", Some(server))
            .attribute("disabled", Some(disabled_str))
            .build();
            
        let mut rx = self.device.send_command(cmd).await.map_err(|e| e.to_string())?;
        while let Some(event) = rx.recv().await {
            match event {
                mikrotik_rs::Event::Trap { response, .. } => {
                    return Err(format!("API Error: {}", response.message));
                }
                mikrotik_rs::Event::Done { .. } => break,
                _ => {}
            }
        }
        Ok(())
    }
    
    pub async fn set_binding(&mut self, id: &str, mac: &str, server: &str, btype: &str, comment: &str, disabled: bool) -> Result<(), String> {
        println!("Setting binding: {} ({})", mac, comment);

        let disabled_str = if disabled { "yes" } else { "no" };

        let cmd = CommandBuilder::new()
            .command("/ip/hotspot/ip-binding/set")
            .attribute(".id", Some(id))
            .attribute("mac-address", Some(mac))
            .attribute("type", Some(btype))
            .attribute("comment", Some(comment))
            .attribute("server", Some(server))
            .attribute("disabled", Some(disabled_str))
            .build();

        let mut rx = self.device.send_command(cmd).await.map_err(|e| e.to_string())?;
        while let Some(event) = rx.recv().await {
            match event {
                mikrotik_rs::Event::Trap { response, .. } => {
                    return Err(format!("API Error: {}", response.message));
                }
                mikrotik_rs::Event::Done { .. } => break,
                _ => {}
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn remove_binding(&mut self, id: &str) -> Result<(), String> {
        println!("Removing binding: {}", id);
        let cmd = CommandBuilder::new()
            .command("/ip/hotspot/ip-binding/remove")
            .attribute(".id", Some(id))
            .build();
            
        let mut rx = self.device.send_command(cmd).await.map_err(|e| e.to_string())?;
        while let Some(event) = rx.recv().await {
            if let mikrotik_rs::Event::Done { .. } = event {
                break;
            }
        }
        Ok(())
    }
    
    #[allow(dead_code)]
    pub async fn sync_ip_binding(&mut self, id: &str, mac: &str) -> Result<String, String> {
        println!("Syncing binding {} with MAC {}", id, mac);
        let cmd = CommandBuilder::new()
            .command("/ip/arp/print")
            .query_equal("mac-address", mac)
            .build();
            
        let mut rx = self.device.send_command(cmd).await.map_err(|e| e.to_string())?;
        let mut ip = String::new();
        while let Some(event) = rx.recv().await {
            match event {
                mikrotik_rs::Event::Reply { response, .. } => {
                    if let Some(Some(addr)) = response.attributes.get("address") {
                        ip = addr.clone();
                    }
                }
                mikrotik_rs::Event::Done { .. } => break,
                _ => {}
            }
        }
        
        if ip.is_empty() {
            return Err("MAC not found in ARP table".to_string());
        }
        
        let set_cmd = CommandBuilder::new()
            .command("/ip/hotspot/ip-binding/set")
            .attribute(".id", Some(id))
            .attribute("address", Some(&ip))
            .build();
            
        let mut set_rx = self.device.send_command(set_cmd).await.map_err(|e| e.to_string())?;
        while let Some(event) = set_rx.recv().await {
            if let mikrotik_rs::Event::Done { .. } = event {
                break;
            }
        }
        
        Ok(ip)
    }
}
