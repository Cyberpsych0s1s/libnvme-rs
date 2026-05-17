use libnvme::Root;

#[test]
fn scan_returns_a_root() {
    let root = Root::scan().expect("nvme_scan failed unexpectedly");
    let _ = format!("{root:?}");
}

#[test]
fn dropping_root_does_not_crash() {
    for _ in 0..3 {
        let _ = Root::scan().expect("nvme_scan failed");
    }
}

#[test]
fn iteration_does_not_panic() {
    let root = Root::scan().unwrap();
    let mut hosts = 0;
    let mut subsystems = 0;
    let mut controllers = 0;
    let mut namespaces = 0;
    for host in root.hosts() {
        hosts += 1;
        for subsys in host.subsystems() {
            subsystems += 1;
            for ctrl in subsys.controllers() {
                controllers += 1;
                for _ns in ctrl.namespaces() {
                    namespaces += 1;
                }
            }
        }
    }
    eprintln!(
        "walked: hosts={hosts} subsystems={subsystems} controllers={controllers} namespaces={namespaces}"
    );
}
