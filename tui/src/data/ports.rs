use std::process::Command;

#[derive(Debug, Clone)]
pub struct DevPort {
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
    pub memory_str: String,
    pub project_name: Option<String>,
}

pub fn scan_dev_ports() -> Vec<DevPort> {
    let dev_ports = [3000, 3001, 3002, 5173, 5174, 8000, 8080, 4321, 5000, 8081, 4200, 9000, 3333];
    let mut results = Vec::new();

    let output = match Command::new("netstat").args(["-ano", "-p", "tcp"]).output() {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
        Err(_) => return Vec::new(),
    };

    for line in output.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("TCP") || !trimmed.contains("LISTENING") {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 5 {
            continue;
        }

        let local_addr = parts[1];
        let pid_str = parts[4];

        if let Some(colon_idx) = local_addr.rfind(':') {
            let port_str = &local_addr[colon_idx + 1..];
            if let Ok(port) = port_str.parse::<u16>() {
                if dev_ports.contains(&port) {
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        // Avoid duplicates
                        if !results.iter().any(|p: &DevPort| p.port == port && p.pid == pid) {
                            let (pname, pmem) = get_process_info(pid);
                            results.push(DevPort {
                                port,
                                pid,
                                process_name: pname,
                                memory_str: pmem,
                                project_name: None,
                            });
                        }
                    }
                }
            }
        }
    }

    results.sort_by_key(|p| p.port);
    results
}

pub fn kill_port_process(pid: u32) -> bool {
    Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn get_process_info(pid: u32) -> (String, String) {
    let output = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
        .output();

    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            let fields: Vec<String> = line
                .split(',')
                .map(|s| s.trim_matches('"').trim().to_string())
                .collect();
            if fields.len() >= 5 {
                let name = fields[0].clone();
                let mem = fields[4].clone();
                return (name, mem);
            }
        }
    }
    ("unknown".into(), "-".into())
}
