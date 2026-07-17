use chrono::Local;

pub fn log_info(message: &str) {
    let now = Local::now();
    println!("[{}] INFO: {}", now.format("%Y-%m-%d %H:%M:%S"), message);
}

pub fn log_error(message: &str) {
    let now = Local::now();
    eprintln!("[{}] ERROR: {}", now.format("%Y-%m-%d %H:%M:%S"), message);
}
