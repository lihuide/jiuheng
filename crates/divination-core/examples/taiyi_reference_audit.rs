                                                                        

#[path = "../tests/support/taiyi_reference.rs"]
mod taiyi_reference;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let report = taiyi_reference::compare(&taiyi_reference::fixture());
    let rendered = serde_json::to_string_pretty(&report)? + "\n";
    if let Some(path) = std::env::args_os().nth(1) {
        std::fs::write(path, rendered)?;
    } else {
        print!("{rendered}");
    }
    Ok(())
}
