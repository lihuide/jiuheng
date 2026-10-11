                                                                           
                                                                         
                                               

use crate::{taiyi_annual_frame_from_accumulated_count, TaiyiAnnualFrame};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const TAIYI_TRADITIONAL_YEAR_EPOCH_ALGORITHM_VERSION: &str =
    "jiuheng.taiyi.received_kaiyuan12_traditional_year_epoch_candidate.v1";
pub const TAIYI_TRADITIONAL_YEAR_EPOCH_STATUS: &str =
    "explicit_traditional_year_label_to_received_accumulated_year_and_annual_frame_no_civil_date_boundary_runtime_epoch_or_interpretation.v1";

                                                                          
                                                                               
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaiyiYearEpochProfile {
    ReceivedKaiyuan12Year724Count1937281,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaiyiTraditionalYearEpochCandidate {
    pub algorithm_version: String,
    pub status: String,
    pub requested_profile: TaiyiYearEpochProfile,
                                                             
                                                                               
    pub supplied_traditional_year_label: i32,
    pub anchor_astronomical_year: i32,
    pub anchor_accumulated_year_count: u64,
    pub signed_year_delta: i64,
    pub accumulated_year_count: u64,
    pub source_fragment_id: String,
    pub annual_frame: TaiyiAnnualFrame,
    pub civil_date_converted: bool,
    pub traditional_year_boundary_selected: bool,
    pub runtime_epoch_selection_authorized: bool,
    pub source_facsimile_verified: bool,
    pub interpretation_eligible: bool,
    pub signal_eligible: bool,
    pub rule_eligible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TaiyiYearEpochError {
    #[error("传统年标签研究范围为天文年 -999 至 3000；不能以超出范围的年或现代日期替代")]
    UnsupportedYearLabel,
}

                                                                            
                                                                             
                                                                              
                                                        
   
                                                                          
                                                                           
                                                                               
                                                                               
pub fn taiyi_annual_frame_from_traditional_year_candidate(
    supplied_traditional_year_label: i32,
    requested_profile: TaiyiYearEpochProfile,
) -> Result<TaiyiTraditionalYearEpochCandidate, TaiyiYearEpochError> {
    if !(-999..=3000).contains(&supplied_traditional_year_label) {
        return Err(TaiyiYearEpochError::UnsupportedYearLabel);
    }
    let (anchor_astronomical_year, anchor_accumulated_year_count) = match requested_profile {
        TaiyiYearEpochProfile::ReceivedKaiyuan12Year724Count1937281 => (724, 1_937_281_u64),
    };
    let signed_year_delta =
        i64::from(supplied_traditional_year_label) - i64::from(anchor_astronomical_year);
                                                                               
    let accumulated_year_count = (anchor_accumulated_year_count as i64 + signed_year_delta) as u64;
    let annual_frame = taiyi_annual_frame_from_accumulated_count(accumulated_year_count)
        .expect("validated traditional-year range gives a positive accumulated count");
    Ok(TaiyiTraditionalYearEpochCandidate {
        algorithm_version: TAIYI_TRADITIONAL_YEAR_EPOCH_ALGORITHM_VERSION.to_owned(),
        status: TAIYI_TRADITIONAL_YEAR_EPOCH_STATUS.to_owned(),
        requested_profile,
        supplied_traditional_year_label,
        anchor_astronomical_year,
        anchor_accumulated_year_count,
        signed_year_delta,
        accumulated_year_count,
        source_fragment_id: "TY-JJSJ-WYG-J1-ACCUMULATED-EPOCHS-CANDIDATE".to_owned(),
        annual_frame,
        civil_date_converted: false,
        traditional_year_boundary_selected: false,
        runtime_epoch_selection_authorized: false,
        source_facsimile_verified: false,
        interpretation_eligible: false,
        signal_eligible: false,
        rule_eligible: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sexagenary_year_for_astronomical_year_label;

    const PROFILE: TaiyiYearEpochProfile =
        TaiyiYearEpochProfile::ReceivedKaiyuan12Year724Count1937281;

    #[test]
    fn received_anchor_and_adjacent_years_are_exact() {
        for (year, count, name) in [
            (723, 1_937_280, "癸亥"),
            (724, 1_937_281, "甲子"),
            (725, 1_937_282, "乙丑"),
            (2026, 1_938_583, "丙午"),
        ] {
            let candidate =
                taiyi_annual_frame_from_traditional_year_candidate(year, PROFILE).unwrap();
            assert_eq!(candidate.accumulated_year_count, count);
            assert_eq!(candidate.annual_frame.tai_sui_designation.zh_name(), name);
            assert_eq!(candidate.anchor_astronomical_year, 724);
            assert_eq!(candidate.anchor_accumulated_year_count, 1_937_281);
            assert_eq!(candidate.signed_year_delta, i64::from(year) - 724);
        }
    }

    #[test]
    fn astronomical_zero_does_not_skip_or_duplicate_a_year() {
        for (year, count) in [(-1, 1_936_556), (0, 1_936_557), (1, 1_936_558)] {
            let candidate =
                taiyi_annual_frame_from_traditional_year_candidate(year, PROFILE).unwrap();
            assert_eq!(candidate.accumulated_year_count, count);
        }
    }

    #[test]
    fn all_supported_labels_keep_count_and_sexagenary_identity_consistent() {
        for year in -999..=3000 {
            let candidate =
                taiyi_annual_frame_from_traditional_year_candidate(year, PROFILE).unwrap();
            assert_eq!(
                candidate.annual_frame.tai_sui_designation,
                sexagenary_year_for_astronomical_year_label(year)
            );
            assert_eq!(
                candidate.annual_frame,
                taiyi_annual_frame_from_accumulated_count(candidate.accumulated_year_count)
                    .unwrap()
            );
            assert!(!candidate.civil_date_converted);
            assert!(!candidate.traditional_year_boundary_selected);
            assert!(!candidate.runtime_epoch_selection_authorized);
            assert!(!candidate.source_facsimile_verified);
            assert!(!candidate.interpretation_eligible);
            assert!(!candidate.signal_eligible);
            assert!(!candidate.rule_eligible);
            assert!(!candidate.annual_frame.calendar_epoch_selected);
            assert!(!candidate.annual_frame.derived_from_civil_date);
            assert!(!candidate.annual_frame.rule_eligible);
        }
    }

    #[test]
    fn explicit_range_rejects_extremes_before_arithmetic() {
        for year in [i32::MIN, -1000, 3001, i32::MAX] {
            assert_eq!(
                taiyi_annual_frame_from_traditional_year_candidate(year, PROFILE),
                Err(TaiyiYearEpochError::UnsupportedYearLabel)
            );
        }
        for (year, count) in [(-999, 1_935_558), (3000, 1_939_557)] {
            assert_eq!(
                taiyi_annual_frame_from_traditional_year_candidate(year, PROFILE)
                    .unwrap()
                    .accumulated_year_count,
                count
            );
        }
    }

    #[test]
    fn candidate_serialization_is_deterministic_and_does_not_release_the_profile() {
        let first = taiyi_annual_frame_from_traditional_year_candidate(2026, PROFILE).unwrap();
        let bytes = serde_json::to_vec(&first).unwrap();
        let second = taiyi_annual_frame_from_traditional_year_candidate(2026, PROFILE).unwrap();
        assert_eq!(serde_json::to_vec(&second).unwrap(), bytes);
        assert_eq!(
            serde_json::from_slice::<TaiyiTraditionalYearEpochCandidate>(&bytes).unwrap(),
            first
        );
        assert!(
            serde_json::from_str::<TaiyiYearEpochProfile>("\"received_kaiyuan15_count40801\"")
                .is_err()
        );
        assert!(
            serde_json::from_str::<TaiyiYearEpochProfile>("\"modern_common_10153917\"").is_err()
        );
    }
}
