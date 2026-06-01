//! CUPS printer management via CLI wrappers.
//! Uses lpstat, lpadmin, cancel, lp — standard on all distros with CUPS installed.

use crate::protocol::{PrintJob, PrintPrinter};

/// List all printers (lpstat -v + lpstat -d).
pub fn print_list() -> anyhow::Result<Vec<PrintPrinter>> {
    let output = std::process::Command::new("lpstat").args(["-v"]).output()?;
    let printers_raw = String::from_utf8_lossy(&output.stdout);

    let default_output = std::process::Command::new("lpstat").args(["-d"]).output()?;
    let default_raw = String::from_utf8_lossy(&default_output.stdout);
    let default_name = default_raw
        .strip_prefix("system default destination: ")
        .map(|s| s.trim().to_string());

    let mut printers = Vec::new();
    for line in printers_raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Format: "device for PrinterName: uri"
        let (name, uri) = if let Some(rest) = line.strip_prefix("device for ") {
            if let Some((n, u)) = rest.split_once(':') {
                (n.trim().to_string(), Some(u.trim().to_string()))
            } else {
                (rest.trim().to_string(), None)
            }
        } else {
            continue;
        };

        // Get printer status
        let status = get_printer_status(&name);

        printers.push(PrintPrinter {
            is_default: default_name.as_ref() == Some(&name),
            location: String::new(),
            status,
            name,
            uri,
        });
    }
    Ok(printers)
}

fn get_printer_status(name: &str) -> String {
    let output = std::process::Command::new("lpstat")
        .args(["-p", name])
        .output()
        .ok();
    if let Some(out) = output {
        let s = String::from_utf8_lossy(&out.stdout);
        let first_line = s.lines().next().unwrap_or("");
        // "printer PrinterName is idle." or "printer PrinterName disabled since ..."
        if first_line.contains("is idle") {
            "idle".into()
        } else if first_line.contains("disabled") {
            "disabled".into()
        } else if first_line.contains("now printing") {
            "printing".into()
        } else if first_line.contains("is ready") {
            "ready".into()
        } else {
            "unknown".into()
        }
    } else {
        "unknown".into()
    }
}

/// Get or set default printer.
pub fn print_default(printer: Option<&str>) -> anyhow::Result<PrintPrinter> {
    if let Some(name) = printer {
        // Set default printer
        let status = std::process::Command::new("lpadmin")
            .args(["-d", name])
            .status()?;
        if !status.success() {
            anyhow::bail!("lpadmin -d {name} failed");
        }
    }

    // Get current default
    let output = std::process::Command::new("lpstat").args(["-d"]).output()?;
    let raw = String::from_utf8_lossy(&output.stdout);
    let default_name = raw
        .strip_prefix("system default destination: ")
        .unwrap_or("")
        .trim()
        .to_string();

    let printers = print_list()?;
    printers
        .into_iter()
        .find(|p| p.name == default_name)
        .ok_or_else(|| {
            anyhow::anyhow!("default printer '{default_name}' not found in printer list")
        })
}

/// List all print jobs (lpstat -o).
pub fn print_jobs() -> anyhow::Result<Vec<PrintJob>> {
    let output = std::process::Command::new("lpstat").args(["-o"]).output()?;
    let raw = String::from_utf8_lossy(&output.stdout);

    let mut jobs = Vec::new();
    for line in raw.lines().filter(|l| !l.trim().is_empty()) {
        // Format: "PrinterName-JobID  user  size  submitted  status"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        // First part is "PrinterName-JobID"
        let (printer, job_id) = if let Some((p, j)) = parts[0].rsplit_once('-') {
            (p.to_string(), j.to_string())
        } else {
            continue;
        };
        let user = parts[1].to_string();
        let size = if parts.len() > 3 {
            Some(parts[2].to_string())
        } else {
            None
        };
        let status = if parts.len() > 4 {
            parts[4..].join(" ")
        } else {
            parts.get(3).map(|s| s.to_string()).unwrap_or_default()
        };

        jobs.push(PrintJob {
            id: job_id,
            printer,
            user,
            name: String::new(),
            size,
            status,
            submitted: None,
        });
    }
    Ok(jobs)
}

/// Cancel a print job.
pub fn print_job_cancel(job_id: &str) -> anyhow::Result<()> {
    let status = std::process::Command::new("cancel").arg(job_id).status()?;
    if !status.success() {
        anyhow::bail!("cancel {job_id} failed");
    }
    Ok(())
}

/// Pause a print job.
pub fn print_job_pause(job_id: &str) -> anyhow::Result<()> {
    let status = std::process::Command::new("lp")
        .args(["-i", job_id, "-H", "hold"])
        .status()?;
    if !status.success() {
        anyhow::bail!("lp -i {job_id} -H hold failed");
    }
    Ok(())
}

/// Resume a print job.
pub fn print_job_resume(job_id: &str) -> anyhow::Result<()> {
    let status = std::process::Command::new("lp")
        .args(["-i", job_id, "-H", "resume"])
        .status()?;
    if !status.success() {
        anyhow::bail!("lp -i {job_id} -H resume failed");
    }
    Ok(())
}
