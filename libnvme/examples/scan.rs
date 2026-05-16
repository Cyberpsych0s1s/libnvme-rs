use libnvme::Root;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = Root::scan()?;
    println!("{root:?}");
    Ok(())
}
