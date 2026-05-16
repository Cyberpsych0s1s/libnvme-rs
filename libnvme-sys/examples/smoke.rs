use libnvme_sys::{nvme_free_tree, nvme_scan};

fn main() {
    let root = unsafe { nvme_scan(std::ptr::null()) };
    if root.is_null() {
        eprintln!("nvme_scan returned NULL");
        std::process::exit(1);
    }
    println!("nvme_scan returned root={root:p}");
    unsafe { nvme_free_tree(root) };
}
