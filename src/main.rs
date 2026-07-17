slint::include_modules!();

use std::sync::Arc;
use tokio::sync::Mutex;
mod mikrotik;
mod utils;

use mikrotik::client::MikrotikClient;
use slint::{ModelRc, VecModel, StandardListViewItem, SharedString};

fn build_bindings_model(bindings: Vec<mikrotik::models::IpBinding>) -> ModelRc<ModelRc<StandardListViewItem>> {
    let row_models: Vec<ModelRc<StandardListViewItem>> = bindings.into_iter().map(|b| {
        let type_with_emoji = match b.binding_type.as_str() {
            "regular" => format!("🔵 {}", b.binding_type),
            "bypassed" => format!("🟢 {}", b.binding_type),
            "blocked" => format!("🔴 {}", b.binding_type),
            _ => b.binding_type.clone(),
        };
        let items = vec![
            StandardListViewItem::from(SharedString::from(b.comment)),
            StandardListViewItem::from(SharedString::from(b.mac_address)),
            StandardListViewItem::from(SharedString::from(b.address)),
            StandardListViewItem::from(SharedString::from(b.server)),
            StandardListViewItem::from(SharedString::from(type_with_emoji)),
        ];
        ModelRc::from(std::rc::Rc::new(VecModel::from(items)))
    }).collect();
    
    ModelRc::from(std::rc::Rc::new(VecModel::from(row_models)))
}

fn update_pagination_ui(
    ui_handle: slint::Weak<AppWindow>,
    mut bindings: Vec<mikrotik::models::IpBinding>,
    current_page: i32,
    page_size: i32,
    sort_column: i32,
    sort_asc: bool,
) {
    bindings.sort_by(|a, b| {
        let cmp = match sort_column {
            0 => a.comment.to_lowercase().cmp(&b.comment.to_lowercase()),
            1 => a.mac_address.to_lowercase().cmp(&b.mac_address.to_lowercase()),
            2 => {
                let ip_a: std::net::Ipv4Addr = a.address.parse().unwrap_or(std::net::Ipv4Addr::new(0,0,0,0));
                let ip_b: std::net::Ipv4Addr = b.address.parse().unwrap_or(std::net::Ipv4Addr::new(0,0,0,0));
                ip_a.cmp(&ip_b)
            },
            3 => a.server.to_lowercase().cmp(&b.server.to_lowercase()),
            4 => a.binding_type.to_lowercase().cmp(&b.binding_type.to_lowercase()),
            _ => std::cmp::Ordering::Equal,
        };
        if sort_asc { cmp } else { cmp.reverse() }
    });

    let total_items = bindings.len() as i32;
    let mut total_pages = total_items / page_size;
    if total_items % page_size != 0 || total_pages == 0 {
        total_pages += 1;
    }
    
    let safe_current_page = current_page.max(1).min(total_pages);
    
    let start_idx = ((safe_current_page - 1) * page_size) as usize;
    let end_idx = (start_idx + page_size as usize).min(bindings.len());
    
    let slice = if start_idx < bindings.len() {
        bindings[start_idx..end_idx].to_vec()
    } else {
        Vec::new()
    };
    
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_handle.upgrade() {
            ui.set_total_items(total_items);
            ui.set_total_pages(total_pages);
            ui.set_current_page(safe_current_page);
            ui.set_bindings(build_bindings_model(slice));
        }
    });
}

fn get_binding_at_index(mut bindings: Vec<mikrotik::models::IpBinding>, cp: i32, ps: i32, sc: i32, sa: bool, idx: i32) -> Option<mikrotik::models::IpBinding> {
    bindings.sort_by(|a, b| {
        let cmp = match sc {
            0 => a.comment.to_lowercase().cmp(&b.comment.to_lowercase()),
            1 => a.mac_address.to_lowercase().cmp(&b.mac_address.to_lowercase()),
            2 => {
                let ip_a: std::net::Ipv4Addr = a.address.parse().unwrap_or(std::net::Ipv4Addr::new(0,0,0,0));
                let ip_b: std::net::Ipv4Addr = b.address.parse().unwrap_or(std::net::Ipv4Addr::new(0,0,0,0));
                ip_a.cmp(&ip_b)
            },
            3 => a.server.to_lowercase().cmp(&b.server.to_lowercase()),
            4 => a.binding_type.to_lowercase().cmp(&b.binding_type.to_lowercase()),
            _ => std::cmp::Ordering::Equal,
        };
        if sa { cmp } else { cmp.reverse() }
    });
    
    let safe_cp = cp.max(1);
    let start_idx = ((safe_cp - 1) * ps) as usize;
    let target_idx = start_idx + idx as usize;
    
    if target_idx < bindings.len() {
        Some(bindings[target_idx].clone())
    } else {
        None
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    
    let config = utils::config::load_config();
    
    let ui = AppWindow::new()?;
    
    if let Some(ip) = &config.ip { ui.set_init_ip(slint::SharedString::from(ip)); }
    if let Some(user) = &config.user { ui.set_init_user(slint::SharedString::from(user)); }
    if let Some(pass) = config.get_password() { ui.set_init_pass(slint::SharedString::from(pass)); }
    if let Some(secure) = config.secure { ui.set_init_secure(secure); }
    if let Some(save) = config.save_credentials { ui.set_init_save(save); }
    
    let ui_handle = ui.as_weak();
    
    let client_state: Arc<Mutex<Option<MikrotikClient>>> = Arc::new(Mutex::new(None));
    let last_login_state = Arc::new(tokio::sync::Mutex::new((String::new(), String::new(), String::new(), false, false)));
    
    let all_bindings: Arc<tokio::sync::Mutex<Vec<mikrotik::models::IpBinding>>> = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let pagination_state = Arc::new(tokio::sync::Mutex::new((1_i32, 25_i32, 0_i32, true))); // (current_page, page_size, sort_column, sort_asc)
    
    ui.on_login({
        let ui_handle = ui_handle.clone();
        let client_state = client_state.clone();
        let last_login_state = last_login_state.clone();
        let all_bindings = all_bindings.clone();
        let pagination_state = pagination_state.clone();
        move |ip, user, pass, secure, save_credentials| {
            let ui_handle = ui_handle.clone();
            let client_state = client_state.clone();
            let last_login_state = last_login_state.clone();
            let all_bindings = all_bindings.clone();
            let pagination_state = pagination_state.clone();
            let ip = ip.to_string();
            let _user = user.to_string();
            let _pass = pass.to_string();
            
            if save_credentials {
                let mut config = utils::config::AppConfig {
                    ip: Some(ip.clone()),
                    user: Some(_user.clone()),
                    secure: Some(secure),
                    save_credentials: Some(true),
                    ..Default::default()
                };
                config.set_password(&_pass);
                utils::config::save_config(&config);
            } else {
                let config = utils::config::AppConfig {
                    save_credentials: Some(false),
                    ..Default::default()
                };
                utils::config::save_config(&config);
            }
            
            let ui_handle_loading = ui_handle.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_handle_loading.upgrade() {
                    ui.set_is_loading(true);
                    ui.set_loading_text(slint::SharedString::from("Connecting and fetching data..."));
                }
            });
            
            tokio::spawn(async move {
                *last_login_state.lock().await = (ip.clone(), _user.clone(), _pass.clone(), secure, save_credentials);
                let rejected_cert_hash = Arc::new(std::sync::Mutex::new(None));
                utils::logger::log_info(&format!("Attempting login to {} (Secure: {})", ip, secure));
                match MikrotikClient::connect(&ip, &_user, &_pass, secure, rejected_cert_hash).await {
                    Ok(mut client) => {
                        let servers = client.get_hotspot_servers().await.unwrap_or_else(|_| vec!["all".to_string()]);
                        
                        utils::logger::log_info("Login success! Fetching bindings...");
                        let bindings_res = client.get_ip_bindings().await;
                        *client_state.lock().await = Some(client);
                        
                        let ui_handle2 = ui_handle.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_handle2.upgrade() {
                                let slint_servers = slint::ModelRc::new(slint::VecModel::from(
                                    servers.into_iter().map(slint::SharedString::from).collect::<Vec<_>>()
                                ));
                                ui.set_server_list(slint_servers);
                                ui.set_is_logged_in(true);
                                ui.set_is_loading(false);
                            }
                        });
                        
                        if let Ok(bindings) = bindings_res {
                            *all_bindings.lock().await = bindings.clone();
                            let (_, ps, sc, sa) = *pagination_state.lock().await;
                            let cp = 1;
                            *pagination_state.lock().await = (cp, ps, sc, sa);
                            update_pagination_ui(ui_handle, bindings, cp, ps, sc, sa);
                        } else if let Err(e) = bindings_res {
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_handle.upgrade() {
                                    ui.set_error_message(slint::SharedString::from(e));
                                    ui.set_show_error(true);
                                }
                            });
                        }
                    }
                    Err(e) => {
                        if e.starts_with("UNTRUSTED_CERT:") {
                            let hash = e.split(':').nth(1).unwrap_or("").to_string();
                            utils::logger::log_error(&format!("Untrusted certificate detected: {}", hash));
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_handle.upgrade() {
                                    ui.set_is_loading(false);
                                    ui.set_untrusted_cert_fingerprint(slint::SharedString::from(hash));
                                    ui.set_show_cert_prompt(true);
                                }
                            });
                        } else {
                            utils::logger::log_error(&format!("Login failed: {}", e));
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_handle.upgrade() {
                                    ui.set_is_loading(false);
                                    ui.set_error_message(slint::SharedString::from(format!("Login failed: {}", e)));
                                    ui.set_show_error(true);
                                }
                            });
                        }
                    }
                }
            });
        }
    });

    ui.on_accept_certificate({
        let ui_handle = ui_handle.clone();
        let last_login_state = last_login_state.clone();
        move |fingerprint| {
            let ui_handle = ui_handle.clone();
            let last_login_state = last_login_state.clone();
            
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_show_cert_prompt(false);
            }
            
            let fingerprint = fingerprint.to_string();
            
            tokio::spawn(async move {
                mikrotik::tls::save_known_cert(fingerprint);
                utils::logger::log_info("Certificate accepted and saved! Retrying login...");
                
                let (ip, user, pass, secure, save) = last_login_state.lock().await.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_handle.upgrade() {
                        ui.invoke_login(slint::SharedString::from(ip), slint::SharedString::from(user), slint::SharedString::from(pass), secure, save);
                    }
                });
            });
        }
    });

    ui.on_reject_certificate({
        let ui_handle = ui_handle.clone();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_show_cert_prompt(false);
                utils::logger::log_info("Certificate rejected by user.");
            }
        }
    });

    ui.on_close_error({
        let ui_handle = ui_handle.clone();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_show_error(false);
            }
        }
    });

    ui.on_add_device({
        let ui_handle = ui_handle.clone();
        let client_state = client_state.clone();
        let all_bindings = all_bindings.clone();
        let pagination_state = pagination_state.clone();
        move |mac, server, policy, comment, enabled| {
            let ui_handle = ui_handle.clone();
            let client_state = client_state.clone();
            let all_bindings = all_bindings.clone();
            let pagination_state = pagination_state.clone();
            let mac = mac.to_string();
            let server = server.to_string();
            let policy = policy.to_string();
            let comment = comment.to_string();
            let disabled = !enabled;
            
            let ui_handle_loading = ui_handle.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_handle_loading.upgrade() {
                    ui.set_is_loading(true);
                    ui.set_loading_text(slint::SharedString::from("Adding device..."));
                }
            });
            
            tokio::spawn(async move {
                if mac.split(':').count() != 6 || mac.len() != 17 {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_is_loading(false);
                            ui.set_error_message(slint::SharedString::from("Invalid MAC Address format. Use XX:XX:XX:XX:XX:XX"));
                            ui.set_show_error(true);
                        }
                    });
                    return;
                }
                
                let mut client_opt = client_state.lock().await;
                let mut ui_err = None;
                if let Some(client) = client_opt.as_mut() {
                    match client.add_binding(&mac, &server, &policy, &comment, disabled).await {
                        Ok(_) => {
                            utils::logger::log_info("Device added successfully.");
                            let bindings_res = client.get_ip_bindings().await;
                            if let Ok(bindings) = bindings_res {
                                *all_bindings.lock().await = bindings.clone();
                                let (cp, ps, sc, sa) = *pagination_state.lock().await;
                                update_pagination_ui(ui_handle.clone(), bindings, cp, ps, sc, sa);
                            }
                        }
                        Err(e) => {
                            utils::logger::log_error(&format!("Failed to add device: {}", e));
                            ui_err = Some(e);
                        }
                    }
                } else {
                    ui_err = Some("Not connected".to_string());
                }
                
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_handle.upgrade() {
                        ui.set_is_loading(false);
                        if let Some(e) = ui_err {
                            ui.set_error_message(slint::SharedString::from(format!("Failed to add device: {}", e)));
                            ui.set_show_error(true);
                        }
                    }
                });
            });
        }
    });

    ui.on_remove_device({
        let ui_handle = ui_handle.clone();
        let client_state = client_state.clone();
        let all_bindings = all_bindings.clone();
        let pagination_state = pagination_state.clone();
        move |idx| {
            let ui_handle = ui_handle.clone();
            let client_state = client_state.clone();
            let all_bindings = all_bindings.clone();
            let pagination_state = pagination_state.clone();
            
            tokio::spawn(async move {
                let id_to_remove = {
                    let bindings = all_bindings.lock().await.clone();
                    let (cp, ps, sc, sa) = *pagination_state.lock().await;
                    get_binding_at_index(bindings, cp, ps, sc, sa, idx).map(|b| b.id)
                };
                
                if let Some(id) = id_to_remove {
                    let ui_handle_loading = ui_handle.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_handle_loading.upgrade() {
                            ui.set_is_loading(true);
                            ui.set_loading_text(slint::SharedString::from("Removing device..."));
                        }
                    });
                    
                    let mut client_opt = client_state.lock().await;
                    let mut ui_err = None;
                    if let Some(client) = client_opt.as_mut() {
                        match client.remove_binding(&id).await {
                            Ok(_) => {
                                utils::logger::log_info("Device removed successfully.");
                                if let Ok(bindings) = client.get_ip_bindings().await {
                                    *all_bindings.lock().await = bindings.clone();
                                    let (cp, ps, sc, sa) = *pagination_state.lock().await;
                                    update_pagination_ui(ui_handle.clone(), bindings, cp, ps, sc, sa);
                                }
                            }
                            Err(e) => {
                                ui_err = Some(e);
                            }
                        }
                    } else {
                        ui_err = Some("Not connected".to_string());
                    }
                    
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_is_loading(false);
                            if let Some(e) = ui_err {
                                ui.set_error_message(slint::SharedString::from(format!("Failed to remove device: {}", e)));
                                ui.set_show_error(true);
                            }
                        }
                    });
                }
            });
        }
    });

    ui.on_sync_selected({
        let ui_handle = ui_handle.clone();
        let client_state = client_state.clone();
        let all_bindings = all_bindings.clone();
        let pagination_state = pagination_state.clone();
        move |idx| {
            let ui_handle = ui_handle.clone();
            let client_state = client_state.clone();
            let all_bindings = all_bindings.clone();
            let pagination_state = pagination_state.clone();
            
            tokio::spawn(async move {
                let binding_opt = {
                    let bindings = all_bindings.lock().await.clone();
                    let (cp, ps, sc, sa) = *pagination_state.lock().await;
                    get_binding_at_index(bindings, cp, ps, sc, sa, idx)
                };
                
                if let Some(b) = binding_opt {
                    if b.mac_address.is_empty() { return; }
                    
                    let ui_handle_loading = ui_handle.clone();
                    let mac_clone = b.mac_address.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_handle_loading.upgrade() {
                            ui.set_is_loading(true);
                            ui.set_loading_text(slint::SharedString::from(format!("Syncing IP for {}...", mac_clone)));
                        }
                    });
                    
                    let mut client_opt = client_state.lock().await;
                    let mut ui_err = None;
                    if let Some(client) = client_opt.as_mut() {
                        match client.sync_ip_binding(&b.id, &b.mac_address).await {
                            Ok(_) => {
                                utils::logger::log_info(&format!("Successfully synced {}", b.mac_address));
                                if let Ok(bindings) = client.get_ip_bindings().await {
                                    *all_bindings.lock().await = bindings.clone();
                                    let (cp, ps, sc, sa) = *pagination_state.lock().await;
                                    update_pagination_ui(ui_handle.clone(), bindings, cp, ps, sc, sa);
                                }
                            }
                            Err(e) => { ui_err = Some(e); }
                        }
                    } else {
                        ui_err = Some("Not connected".to_string());
                    }
                    
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_is_loading(false);
                            if let Some(e) = ui_err {
                                ui.set_error_message(slint::SharedString::from(format!("Sync failed: {}", e)));
                                ui.set_show_error(true);
                            }
                        }
                    });
                }
            });
        }
    });
    
    ui.on_request_edit_device({
        let ui_handle = ui_handle.clone();
        let all_bindings = all_bindings.clone();
        let pagination_state = pagination_state.clone();
        move |idx| {
            let ui_handle = ui_handle.clone();
            let all_bindings = all_bindings.clone();
            let pagination_state = pagination_state.clone();
            
            tokio::spawn(async move {
                let binding_opt = {
                    let bindings = all_bindings.lock().await.clone();
                    let (cp, ps, sc, sa) = *pagination_state.lock().await;
                    get_binding_at_index(bindings, cp, ps, sc, sa, idx)
                };
                
                if let Some(b) = binding_opt {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_is_editing(true);
                            ui.set_edit_id(slint::SharedString::from(&b.id));
                            ui.set_form_name(slint::SharedString::from(&b.comment));
                            ui.set_form_mac(slint::SharedString::from(&b.mac_address));
                            ui.set_form_server(slint::SharedString::from(&b.server));
                            ui.set_form_policy(slint::SharedString::from(&b.binding_type));
                            ui.set_form_enabled(!b.disabled);
                            ui.set_show_add_dialog(true);
                        }
                    });
                }
            });
        }
    });
    
    ui.on_save_edit_device({
        let ui_handle = ui_handle.clone();
        let client_state = client_state.clone();
        let all_bindings = all_bindings.clone();
        let pagination_state = pagination_state.clone();
        move |id, mac, server, policy, comment, enabled| {
            let ui_handle = ui_handle.clone();
            let client_state = client_state.clone();
            let all_bindings = all_bindings.clone();
            let pagination_state = pagination_state.clone();
            
            let id = id.to_string();
            let mac = mac.to_string();
            let server = server.to_string();
            let policy = policy.to_string();
            let comment = comment.to_string();
            let disabled = !enabled;
            
            let ui_handle_loading = ui_handle.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_handle_loading.upgrade() {
                    ui.set_is_loading(true);
                    ui.set_loading_text(slint::SharedString::from("Saving device..."));
                }
            });
            
            tokio::spawn(async move {
                if mac.split(':').count() != 6 || mac.len() != 17 {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_is_loading(false);
                            ui.set_error_message(slint::SharedString::from("Invalid MAC Address format. Use XX:XX:XX:XX:XX:XX"));
                            ui.set_show_error(true);
                        }
                    });
                    return;
                }
                
                let mut client_opt = client_state.lock().await;
                let mut ui_err = None;
                if let Some(client) = client_opt.as_mut() {
                    match client.set_binding(&id, &mac, &server, &policy, &comment, disabled).await {
                        Ok(_) => {
                            utils::logger::log_info("Device edited successfully.");
                            if let Ok(bindings) = client.get_ip_bindings().await {
                                *all_bindings.lock().await = bindings.clone();
                                let (cp, ps, sc, sa) = *pagination_state.lock().await;
                                update_pagination_ui(ui_handle.clone(), bindings, cp, ps, sc, sa);
                            }
                        }
                        Err(e) => { ui_err = Some(e); }
                    }
                } else {
                    ui_err = Some("Not connected".to_string());
                }
                
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_handle.upgrade() {
                        ui.set_is_loading(false);
                        if let Some(e) = ui_err {
                            ui.set_error_message(slint::SharedString::from(format!("Failed to edit device: {}", e)));
                            ui.set_show_error(true);
                        }
                    }
                });
            });
        }
    });
    
    ui.on_format_mac({
        let ui_handle = ui_handle.clone();
        move |mac| {
            let mut formatted = String::new();
            let mut char_count = 0;
            
            for c in mac.chars() {
                if c.is_ascii_hexdigit() {
                    if char_count > 0 && char_count % 2 == 0 && char_count < 12 {
                        formatted.push(':');
                    }
                    formatted.push(c.to_ascii_uppercase());
                    char_count += 1;
                }
            }
            if formatted.len() > 17 {
                formatted.truncate(17);
            }
            
            let _ = slint::invoke_from_event_loop({
                let ui_handle = ui_handle.clone();
                move || {
                    if let Some(ui) = ui_handle.upgrade() {
                        ui.set_form_mac(slint::SharedString::from(formatted));
                    }
                }
            });
        }
    });

    ui.on_refresh({
        let ui_handle = ui_handle.clone();
        let client_state = client_state.clone();
        let all_bindings = all_bindings.clone();
        let pagination_state = pagination_state.clone();
        move || {
            let ui_handle = ui_handle.clone();
            let client_state = client_state.clone();
            let all_bindings = all_bindings.clone();
            let pagination_state = pagination_state.clone();
            let ui_handle_loading = ui_handle.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_handle_loading.upgrade() {
                    ui.set_is_loading(true);
                    ui.set_loading_text(slint::SharedString::from("Refreshing data..."));
                }
            });
            
            tokio::spawn(async move {
                utils::logger::log_info("Refreshing data...");
                let mut client_opt = client_state.lock().await;
                let mut ui_err = None;
                if let Some(client) = client_opt.as_mut() {
                    match client.get_ip_bindings().await {
                        Ok(bindings) => {
                            *all_bindings.lock().await = bindings.clone();
                            let (cp, ps, sc, sa) = *pagination_state.lock().await;
                            update_pagination_ui(ui_handle.clone(), bindings, cp, ps, sc, sa);
                        }
                        Err(e) => {
                            ui_err = Some(e);
                        }
                    }
                } else {
                    ui_err = Some("Not connected".to_string());
                }
                
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_handle.upgrade() {
                        ui.set_is_loading(false);
                        if let Some(e) = ui_err {
                            ui.set_error_message(slint::SharedString::from(format!("Refresh failed: {}", e)));
                            ui.set_show_error(true);
                        }
                    }
                });
            });
        }
    });
    
    ui.on_sync_all({
        let ui_handle = ui_handle.clone();
        let client_state = client_state.clone();
        let all_bindings = all_bindings.clone();
        let pagination_state = pagination_state.clone();
        move || {
            let ui_handle = ui_handle.clone();
            let client_state = client_state.clone();
            let all_bindings = all_bindings.clone();
            let pagination_state = pagination_state.clone();
            
            let ui_handle_loading = ui_handle.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_handle_loading.upgrade() {
                    ui.set_is_loading(true);
                    ui.set_loading_text(slint::SharedString::from("Syncing missing IPs from ARP..."));
                }
            });
            
            tokio::spawn(async move {
                utils::logger::log_info("Syncing all...");
                let mut client_opt = client_state.lock().await;
                let mut ui_err = None;
                let mut count = 0;
                
                if let Some(client) = client_opt.as_mut() {
                    let mut ids_to_sync = Vec::new();
                    if let Ok(bindings) = client.get_ip_bindings().await {
                        for b in bindings {
                            if b.address.is_empty() || b.address == "0.0.0.0" {
                                ids_to_sync.push((b.id.clone(), b.mac_address.clone()));
                            }
                        }
                    }
                    
                    for (id, mac) in ids_to_sync {
                        if mac.is_empty() { continue; }
                        if let Ok(_) = client.sync_ip_binding(&id, &mac).await {
                            count += 1;
                        }
                    }
                    
                    match client.get_ip_bindings().await {
                        Ok(bindings) => {
                            *all_bindings.lock().await = bindings.clone();
                            let (cp, ps, sc, sa) = *pagination_state.lock().await;
                            update_pagination_ui(ui_handle.clone(), bindings, cp, ps, sc, sa);
                        }
                        Err(e) => {
                            ui_err = Some(e);
                        }
                    }
                } else {
                    ui_err = Some("Not connected".to_string());
                }
                
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_handle.upgrade() {
                        ui.set_is_loading(false);
                        if let Some(e) = ui_err {
                            ui.set_error_message(slint::SharedString::from(format!("Sync failed: {}", e)));
                            ui.set_show_error(true);
                        } else {
                            utils::logger::log_info(&format!("Synced {} IPs from ARP.", count));
                        }
                    }
                });
            });
        }
    });
    
    ui.on_next_page({
        let ui_handle = ui_handle.clone();
        let all_bindings = all_bindings.clone();
        let pagination_state = pagination_state.clone();
        move || {
            let ui_handle = ui_handle.clone();
            let all_bindings = all_bindings.clone();
            let pagination_state = pagination_state.clone();
            tokio::spawn(async move {
                let mut state = pagination_state.lock().await;
                state.0 += 1; // current_page
                let (cp, ps, sc, sa) = *state;
                let bindings = all_bindings.lock().await.clone();
                update_pagination_ui(ui_handle, bindings, cp, ps, sc, sa);
            });
        }
    });
    
    ui.on_prev_page({
        let ui_handle = ui_handle.clone();
        let all_bindings = all_bindings.clone();
        let pagination_state = pagination_state.clone();
        move || {
            let ui_handle = ui_handle.clone();
            let all_bindings = all_bindings.clone();
            let pagination_state = pagination_state.clone();
            tokio::spawn(async move {
                let mut state = pagination_state.lock().await;
                if state.0 > 1 {
                    state.0 -= 1; // current_page
                }
                let (cp, ps, sc, sa) = *state;
                let bindings = all_bindings.lock().await.clone();
                update_pagination_ui(ui_handle, bindings, cp, ps, sc, sa);
            });
        }
    });
    
    ui.on_change_page_size({
        let ui_handle = ui_handle.clone();
        let all_bindings = all_bindings.clone();
        let pagination_state = pagination_state.clone();
        move |size_str| {
            let ui_handle = ui_handle.clone();
            let all_bindings = all_bindings.clone();
            let pagination_state = pagination_state.clone();
            let size: i32 = size_str.parse().unwrap_or(25);
            tokio::spawn(async move {
                let mut state = pagination_state.lock().await;
                state.0 = 1; // reset to page 1
                state.1 = size; // page_size
                let (cp, ps, sc, sa) = *state;
                let bindings = all_bindings.lock().await.clone();
                update_pagination_ui(ui_handle, bindings, cp, ps, sc, sa);
            });
        }
    });
    
    ui.on_sort_table({
        let ui_handle = ui_handle.clone();
        let all_bindings = all_bindings.clone();
        let pagination_state = pagination_state.clone();
        move |sc, sa| {
            let ui_handle = ui_handle.clone();
            let all_bindings = all_bindings.clone();
            let pagination_state = pagination_state.clone();
            tokio::spawn(async move {
                let mut state = pagination_state.lock().await;
                state.2 = sc; // sort_column
                state.3 = sa; // sort_asc
                let (cp, ps, sc, sa) = *state;
                let bindings = all_bindings.lock().await.clone();
                update_pagination_ui(ui_handle, bindings, cp, ps, sc, sa);
            });
        }
    });

    ui.run()?;
    Ok(())
}
