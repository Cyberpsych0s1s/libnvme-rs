//! Rust port of `nvme list` — walks the libnvme tree and prints a table.

use libnvme::{Controller, Namespace, Root};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = Root::scan()?;

    let mut rows: Vec<Row> = Vec::new();
    for host in root.hosts() {
        for subsys in host.subsystems() {
            for ctrl in subsys.controllers() {
                let mut had_namespace = false;
                for ns in ctrl.namespaces() {
                    rows.push(Row::from(&ctrl, Some(&ns)));
                    had_namespace = true;
                }
                if !had_namespace {
                    rows.push(Row::from(&ctrl, None));
                }
            }
        }
    }

    if rows.is_empty() {
        println!("(no NVMe devices found)");
        return Ok(());
    }

    print_table(&rows);
    Ok(())
}

struct Row {
    node: String,
    model: String,
    serial: String,
    firmware: String,
    transport: String,
    address: String,
    nsid: String,
    size: String,
}

impl Row {
    fn from(ctrl: &Controller<'_>, ns: Option<&Namespace<'_>>) -> Self {
        Row {
            node: ns
                .and_then(|n| n.name().ok().map(|s| format!("/dev/{s}")))
                .unwrap_or_else(|| {
                    ctrl.name()
                        .ok()
                        .map(|s| format!("/dev/{s}"))
                        .unwrap_or_default()
                }),
            model: ctrl.model().unwrap_or("").to_string(),
            serial: ctrl.serial().unwrap_or("").to_string(),
            firmware: ctrl.firmware().unwrap_or("").to_string(),
            transport: ctrl.transport().unwrap_or("").to_string(),
            address: ctrl.address().unwrap_or("").to_string(),
            nsid: ns.map(|n| n.nsid().to_string()).unwrap_or_default(),
            size: ns.map(|n| format_bytes(n.size_bytes())).unwrap_or_default(),
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    let mut value = bytes as f64;
    let mut unit_idx = 0;
    while value >= 1000.0 && unit_idx + 1 < UNITS.len() {
        value /= 1000.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        format!("{bytes} {}", UNITS[0])
    } else {
        format!("{value:.2} {}", UNITS[unit_idx])
    }
}

fn print_table(rows: &[Row]) {
    let headers = [
        "Node",
        "NSID",
        "Model",
        "Serial",
        "FW",
        "Size",
        "Transport",
        "Address",
    ];
    let widths: Vec<usize> = (0..headers.len())
        .map(|i| {
            let max_data = rows.iter().map(|r| field(r, i).len()).max().unwrap_or(0);
            std::cmp::max(headers[i].len(), max_data)
        })
        .collect();

    for (i, header) in headers.iter().enumerate() {
        print!("{:<width$}  ", header, width = widths[i]);
    }
    println!();
    for w in &widths {
        print!("{:-<width$}  ", "", width = *w);
    }
    println!();

    for row in rows {
        for (i, w) in widths.iter().enumerate() {
            print!("{:<width$}  ", field(row, i), width = *w);
        }
        println!();
    }
}

fn field(row: &Row, idx: usize) -> &str {
    match idx {
        0 => &row.node,
        1 => &row.nsid,
        2 => &row.model,
        3 => &row.serial,
        4 => &row.firmware,
        5 => &row.size,
        6 => &row.transport,
        7 => &row.address,
        _ => "",
    }
}
