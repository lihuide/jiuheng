                                                                        

use divination_core::{taiyi_annual_frame_from_traditional_year_candidate, TaiyiYearEpochProfile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let year: i32 = args
        .next()
        .ok_or("supply an astronomical traditional year label")?
        .parse()?;
    let candidate = taiyi_annual_frame_from_traditional_year_candidate(
        year,
        TaiyiYearEpochProfile::ReceivedKaiyuan12Year724Count1937281,
    )?;
    let rendered = serde_json::to_string_pretty(&candidate)? + "\n";
    if let Some(path) = args.next() {
        std::fs::write(path, rendered)?;
    } else {
        print!("{rendered}");
    }
    Ok(())
}
