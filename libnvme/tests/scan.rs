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
