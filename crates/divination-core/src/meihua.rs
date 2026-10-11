                                                                          
                                                                        
                                                                               

use crate::{
    sexagenary_year_for_astronomical_year_label, EarthlyBranch, LunisolarDateCandidate,
    ShichenBoundary, Trigram,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const MEIHUA_TIME_CAST_ALGORITHM_VERSION: &str =
    "jiuheng.meihua.received_text.year_month_day_hour.v1";
pub const MEIHUA_TIME_CAST_STATUS: &str =
    "mechanical_textual_candidate_no_interpretation_or_runtime_signals.v1";
pub const MEIHUA_TIME_CASTING_AUDIT_VERSION: u16 = 1;
pub const MEIHUA_MUTUAL_RAW_ALGORITHM_VERSION: &str =
    "jiuheng.meihua.received_text.inner_trigram_observations.v1";

                                                                              
                                                                       
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeihuaMutualRawAudit {
    pub primary_lower_from_lines_2_3_4: Trigram,
    pub primary_upper_from_lines_3_4_5: Trigram,
    pub changed_lower_from_lines_2_3_4: Trigram,
    pub changed_upper_from_lines_3_4_5: Trigram,
    pub primary_is_pure_qian_or_kun: bool,
    pub selected_received_basis: Option<String>,
    pub algorithm_version: String,
    pub rule_eligible: bool,
}

impl MeihuaMutualRawAudit {
    pub fn compact_zh(&self) -> String {
        format!(
            "本卦中爻下{}上{}；变卦中爻下{}上{}{}（未选择互卦口径）",
            self.primary_lower_from_lines_2_3_4.zh_name(),
            self.primary_upper_from_lines_3_4_5.zh_name(),
            self.changed_lower_from_lines_2_3_4.zh_name(),
            self.changed_upper_from_lines_3_4_5.zh_name(),
            if self.primary_is_pure_qian_or_kun {
                "；乾坤无互例句待校勘"
            } else {
                ""
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MeihuaMutualRawError {
    #[error("六爻位图必须在 0 至 63 之间")]
    InvalidHexagramBits,
    #[error("变卦必须与本卦恰有一爻不同")]
    NotSingleMovingLine,
}

                                                                   
                                                  
pub fn meihua_mutual_raw_audit(
    primary_bits: u8,
    changed_bits: u8,
) -> Result<MeihuaMutualRawAudit, MeihuaMutualRawError> {
    if primary_bits > 0b11_1111 || changed_bits > 0b11_1111 {
        return Err(MeihuaMutualRawError::InvalidHexagramBits);
    }
    if (primary_bits ^ changed_bits).count_ones() != 1 {
        return Err(MeihuaMutualRawError::NotSingleMovingLine);
    }
    let middle = |bits: u8, offset| {
        Trigram::from_bits_bottom_up((bits >> offset) & 0b111_u8)
            .expect("three masked lines always identify one trigram")
    };
    Ok(MeihuaMutualRawAudit {
        primary_lower_from_lines_2_3_4: middle(primary_bits, 1),
        primary_upper_from_lines_3_4_5: middle(primary_bits, 2),
        changed_lower_from_lines_2_3_4: middle(changed_bits, 1),
        changed_upper_from_lines_3_4_5: middle(changed_bits, 2),
        primary_is_pure_qian_or_kun: matches!(primary_bits, 0 | 0b11_1111),
        selected_received_basis: None,
        algorithm_version: MEIHUA_MUTUAL_RAW_ALGORITHM_VERSION.to_owned(),
        rule_eligible: false,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeihuaTimeInput {
    pub year_branch_number: u8,
    pub lunar_month_number: u8,
    pub lunar_day_number: u8,
    pub hour_branch_number: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeihuaTimeCast {
    pub input: MeihuaTimeInput,
    pub year_month_day_sum: u16,
    pub year_month_day_hour_sum: u16,
    pub upper_trigram_number: u8,
    pub lower_trigram_number: u8,
    pub moving_line_bottom_up: u8,
    pub upper_trigram: Trigram,
    pub lower_trigram: Trigram,
                                                    
    pub primary_bits: u8,
                                                                            
    pub changed_bits: u8,
    pub algorithm_version: String,
    pub status: String,
    pub rule_eligible: bool,
}

impl MeihuaTimeCast {
    pub fn compact_zh(&self) -> String {
        format!(
            "上{} 下{} · {}爻动 · {:06b}→{:06b}",
            self.upper_trigram.zh_name(),
            self.lower_trigram.zh_name(),
            self.moving_line_bottom_up,
            self.primary_bits,
            self.changed_bits
        )
    }

                                                                        
                                                                             
    pub fn mutual_raw_audit(&self) -> Result<MeihuaMutualRawAudit, MeihuaMutualRawError> {
        meihua_mutual_raw_audit(self.primary_bits, self.changed_bits)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeihuaHourConventionCandidate {
    pub hour_convention: String,
    pub shichen: ShichenBoundary,
    pub mechanical_boundaries_safe: bool,
    pub cast: MeihuaTimeCast,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeihuaTimeCastingAudit {
    pub audit_version: u16,
    pub lunisolar_date_candidate: LunisolarDateCandidate,
    pub year_branch: EarthlyBranch,
    pub calendar_convention: String,
    pub beijing_time_candidate: Option<MeihuaHourConventionCandidate>,
    pub true_solar_time_candidate: Option<MeihuaHourConventionCandidate>,
    pub candidates_identical: bool,
    pub selected_hour_convention: Option<String>,
    pub blocked_reasons: Vec<String>,
    pub rule_eligible: bool,
    pub algorithm_version: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MeihuaTimeCastError {
    #[error("梅花时间起卦年支数必须为 1 至 12")]
    InvalidYearBranchNumber,
    #[error("梅花时间起卦月份数必须为 1 至 12")]
    InvalidMonthNumber,
    #[error("梅花时间起卦日期数必须为 1 至 30")]
    InvalidDayNumber,
    #[error("梅花时间起卦时支数必须为 1 至 12")]
    InvalidHourBranchNumber,
}

pub fn meihua_time_cast(input: MeihuaTimeInput) -> Result<MeihuaTimeCast, MeihuaTimeCastError> {
    if !(1..=12).contains(&input.year_branch_number) {
        return Err(MeihuaTimeCastError::InvalidYearBranchNumber);
    }
    if !(1..=12).contains(&input.lunar_month_number) {
        return Err(MeihuaTimeCastError::InvalidMonthNumber);
    }
    if !(1..=30).contains(&input.lunar_day_number) {
        return Err(MeihuaTimeCastError::InvalidDayNumber);
    }
    if !(1..=12).contains(&input.hour_branch_number) {
        return Err(MeihuaTimeCastError::InvalidHourBranchNumber);
    }

    let year_month_day_sum = u16::from(input.year_branch_number)
        + u16::from(input.lunar_month_number)
        + u16::from(input.lunar_day_number);
    let year_month_day_hour_sum = year_month_day_sum + u16::from(input.hour_branch_number);
    let upper_trigram_number = received_remainder(year_month_day_sum, 8);
    let lower_trigram_number = received_remainder(year_month_day_hour_sum, 8);
    let moving_line_bottom_up = received_remainder(year_month_day_hour_sum, 6);
    let upper_trigram = Trigram::from_received_number(upper_trigram_number)
        .expect("received remainder is always one through eight");
    let lower_trigram = Trigram::from_received_number(lower_trigram_number)
        .expect("received remainder is always one through eight");
    let primary_bits = lower_trigram.bits_bottom_up() | (upper_trigram.bits_bottom_up() << 3);
    let changed_bits = primary_bits ^ (1 << (moving_line_bottom_up - 1));

    Ok(MeihuaTimeCast {
        input,
        year_month_day_sum,
        year_month_day_hour_sum,
        upper_trigram_number,
        lower_trigram_number,
        moving_line_bottom_up,
        upper_trigram,
        lower_trigram,
        primary_bits,
        changed_bits,
        algorithm_version: MEIHUA_TIME_CAST_ALGORITHM_VERSION.to_owned(),
        status: MEIHUA_TIME_CAST_STATUS.to_owned(),
        rule_eligible: false,
    })
}

pub fn meihua_time_casting_audit(
    lunisolar_date: &LunisolarDateCandidate,
    beijing_shichen: ShichenBoundary,
    true_solar_shichen: ShichenBoundary,
) -> Result<MeihuaTimeCastingAudit, MeihuaTimeCastError> {
    let year_branch =
        sexagenary_year_for_astronomical_year_label(lunisolar_date.nominal_lunar_year).branch;
    let mut blocked_reasons = vec![
        "received_text_facsimile_and_attribution_review_incomplete".to_owned(),
        "beijing_vs_true_solar_hour_basis_not_selected".to_owned(),
    ];
    if !lunisolar_date.date_assignment_safe {
        blocked_reasons.push("lunisolar_date_assignment_boundary_not_safe".to_owned());
    }

    let (beijing_time_candidate, true_solar_time_candidate, candidates_identical) =
        if lunisolar_date.is_leap_month {
            blocked_reasons.push("leap_month_numbering_not_defined_by_cited_passage".to_owned());
            (None, None, false)
        } else {
            let common = MeihuaTimeInput {
                year_branch_number: year_branch.index() + 1,
                lunar_month_number: lunisolar_date.month_number,
                lunar_day_number: lunisolar_date.day_number,
                hour_branch_number: beijing_shichen.branch.index() + 1,
            };
            let beijing_cast = meihua_time_cast(common)?;
            let true_solar_cast = meihua_time_cast(MeihuaTimeInput {
                hour_branch_number: true_solar_shichen.branch.index() + 1,
                ..common
            })?;
            let identical = beijing_cast == true_solar_cast;
            (
                Some(MeihuaHourConventionCandidate {
                    hour_convention: "beijing_standard_two_hour_branch.v1".to_owned(),
                    shichen: beijing_shichen,
                    mechanical_boundaries_safe: lunisolar_date.date_assignment_safe
                        && beijing_shichen.boundary_safe,
                    cast: beijing_cast,
                }),
                Some(MeihuaHourConventionCandidate {
                    hour_convention: "local_true_solar_two_hour_branch.v1".to_owned(),
                    shichen: true_solar_shichen,
                    mechanical_boundaries_safe: lunisolar_date.date_assignment_safe
                        && true_solar_shichen.boundary_safe,
                    cast: true_solar_cast,
                }),
                identical,
            )
        };

    Ok(MeihuaTimeCastingAudit {
        audit_version: MEIHUA_TIME_CASTING_AUDIT_VERSION,
        lunisolar_date_candidate: lunisolar_date.clone(),
        year_branch,
        calendar_convention: "modern_official_lunisolar_candidate_not_historical_calendar.v1"
            .to_owned(),
        beijing_time_candidate,
        true_solar_time_candidate,
        candidates_identical,
        selected_hour_convention: None,
        blocked_reasons,
        rule_eligible: false,
        algorithm_version: MEIHUA_TIME_CAST_ALGORITHM_VERSION.to_owned(),
        status: MEIHUA_TIME_CAST_STATUS.to_owned(),
    })
}

const fn received_remainder(value: u16, divisor: u16) -> u8 {
    let remainder = value % divisor;
    if remainder == 0 {
        divisor as u8
    } else {
        remainder as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HistoricalDateTime;

    fn date_candidate(is_leap_month: bool) -> LunisolarDateCandidate {
        LunisolarDateCandidate {
            target_bjt: HistoricalDateTime {
                year: 2026,
                month: 8,
                day: 30,
                hour: 12,
                minute: 45,
                second: 0,
            },
            target_day_number: 2_461_283,
            structure_winter_solstice_year: 2025,
            nominal_lunar_year: 2026,
            month_sequence_index: 9,
            month_number: 7,
            is_leap_month,
            day_number: 18,
            month_length_days: 30,
            month_start_bjt: HistoricalDateTime {
                year: 2026,
                month: 8,
                day: 13,
                hour: 0,
                minute: 0,
                second: 0,
            },
            next_month_start_bjt: HistoricalDateTime {
                year: 2026,
                month: 9,
                day: 12,
                hour: 0,
                minute: 0,
                second: 0,
            },
            selected_month_boundaries_safe: true,
            structure_boundaries_safe: true,
            date_assignment_safe: true,
            algorithm_version: "test.lunisolar".to_owned(),
            status: "test".to_owned(),
        }
    }

    fn shichen(branch: EarthlyBranch) -> ShichenBoundary {
        ShichenBoundary {
            branch,
            minutes_to_boundary: 30.0,
            boundary_margin_minutes: 5.0,
            boundary_safe: true,
        }
    }

    #[test]
    fn received_trigram_numbers_and_bottom_up_bits_are_complete() {
        let expected = [
            (Trigram::Qian, 1, 0b111),
            (Trigram::Dui, 2, 0b011),
            (Trigram::Li, 3, 0b101),
            (Trigram::Zhen, 4, 0b001),
            (Trigram::Xun, 5, 0b110),
            (Trigram::Kan, 6, 0b010),
            (Trigram::Gen, 7, 0b100),
            (Trigram::Kun, 8, 0b000),
        ];
        for (trigram, number, bits) in expected {
            assert_eq!(trigram.received_number(), number);
            assert_eq!(trigram.bits_bottom_up(), bits);
            assert_eq!(Trigram::from_received_number(number), Some(trigram));
        }
    }

    #[test]
    fn reproduces_the_received_chen_year_observing_plum_example_structure() {
        let cast = meihua_time_cast(MeihuaTimeInput {
            year_branch_number: 5,
            lunar_month_number: 12,
            lunar_day_number: 17,
            hour_branch_number: 9,
        })
        .unwrap();
        assert_eq!(cast.year_month_day_sum, 34);
        assert_eq!(cast.year_month_day_hour_sum, 43);
        assert_eq!(cast.upper_trigram, Trigram::Dui);
        assert_eq!(cast.lower_trigram, Trigram::Li);
        assert_eq!(cast.moving_line_bottom_up, 1);
        assert_eq!(cast.primary_bits, 0b011101);
        assert_eq!(cast.changed_bits, 0b011100);
        assert!(!cast.rule_eligible);
    }

    #[test]
    fn exact_division_maps_to_kun_and_the_sixth_line() {
        let cast = meihua_time_cast(MeihuaTimeInput {
            year_branch_number: 1,
            lunar_month_number: 1,
            lunar_day_number: 14,
            hour_branch_number: 8,
        })
        .unwrap();
        assert_eq!(cast.upper_trigram, Trigram::Kun);
        assert_eq!(cast.lower_trigram, Trigram::Kun);
        assert_eq!(cast.moving_line_bottom_up, 6);
        assert_eq!(cast.primary_bits, 0);
        assert_eq!(cast.changed_bits, 0b100000);
    }

    #[test]
    fn rejects_every_out_of_range_component() {
        let valid = MeihuaTimeInput {
            year_branch_number: 1,
            lunar_month_number: 1,
            lunar_day_number: 1,
            hour_branch_number: 1,
        };
        assert_eq!(
            meihua_time_cast(MeihuaTimeInput {
                year_branch_number: 0,
                ..valid
            }),
            Err(MeihuaTimeCastError::InvalidYearBranchNumber)
        );
        assert_eq!(
            meihua_time_cast(MeihuaTimeInput {
                lunar_month_number: 13,
                ..valid
            }),
            Err(MeihuaTimeCastError::InvalidMonthNumber)
        );
        assert_eq!(
            meihua_time_cast(MeihuaTimeInput {
                lunar_day_number: 31,
                ..valid
            }),
            Err(MeihuaTimeCastError::InvalidDayNumber)
        );
        assert_eq!(
            meihua_time_cast(MeihuaTimeInput {
                hour_branch_number: 0,
                ..valid
            }),
            Err(MeihuaTimeCastError::InvalidHourBranchNumber)
        );
    }

    #[test]
    fn audit_keeps_beijing_and_true_solar_hour_candidates_parallel() {
        let audit = meihua_time_casting_audit(
            &date_candidate(false),
            shichen(EarthlyBranch::Wu),
            shichen(EarthlyBranch::Si),
        )
        .unwrap();
        assert_eq!(audit.year_branch, EarthlyBranch::Wu);
        assert!(!audit.candidates_identical);
        assert_eq!(audit.selected_hour_convention, None);
        assert_eq!(
            audit
                .beijing_time_candidate
                .as_ref()
                .unwrap()
                .cast
                .input
                .hour_branch_number,
            7
        );
        assert_eq!(
            audit
                .true_solar_time_candidate
                .as_ref()
                .unwrap()
                .cast
                .input
                .hour_branch_number,
            6
        );
        assert!(!audit.rule_eligible);
    }

    #[test]
    fn leap_month_is_explicitly_blocked_instead_of_reusing_the_month_number() {
        let audit = meihua_time_casting_audit(
            &date_candidate(true),
            shichen(EarthlyBranch::Wu),
            shichen(EarthlyBranch::Wu),
        )
        .unwrap();
        assert_eq!(audit.beijing_time_candidate, None);
        assert_eq!(audit.true_solar_time_candidate, None);
        assert!(audit
            .blocked_reasons
            .contains(&"leap_month_numbering_not_defined_by_cited_passage".to_owned()));
    }

    #[test]
    fn mutual_raw_observations_cover_all_six_bit_structures_and_moving_lines() {
        for primary in 0..64_u8 {
            for moving_line in 0..6 {
                let changed = primary ^ (1 << moving_line);
                let audit = meihua_mutual_raw_audit(primary, changed).unwrap();
                assert_eq!(
                    audit.primary_lower_from_lines_2_3_4.bits_bottom_up(),
                    (primary >> 1) & 0b111
                );
                assert_eq!(
                    audit.primary_upper_from_lines_3_4_5.bits_bottom_up(),
                    (primary >> 2) & 0b111
                );
                assert_eq!(
                    audit.changed_lower_from_lines_2_3_4.bits_bottom_up(),
                    (changed >> 1) & 0b111
                );
                assert_eq!(
                    audit.changed_upper_from_lines_3_4_5.bits_bottom_up(),
                    (changed >> 2) & 0b111
                );
                assert_eq!(audit.primary_is_pure_qian_or_kun, matches!(primary, 0 | 63));
                assert_eq!(audit.selected_received_basis, None);
                assert!(!audit.rule_eligible);
            }
        }
    }

    #[test]
    fn pure_qian_kun_exception_is_flagged_without_choosing_an_interpretation() {
        let kun = meihua_mutual_raw_audit(0, 0b000100).unwrap();
        assert!(kun.primary_is_pure_qian_or_kun);
        assert_eq!(kun.primary_lower_from_lines_2_3_4, Trigram::Kun);
        assert_eq!(kun.primary_upper_from_lines_3_4_5, Trigram::Kun);
        assert_eq!(kun.changed_lower_from_lines_2_3_4, Trigram::Kan);
        assert_eq!(kun.changed_upper_from_lines_3_4_5, Trigram::Zhen);
        assert_eq!(kun.selected_received_basis, None);

        let qian = meihua_mutual_raw_audit(63, 59).unwrap();
        assert!(qian.primary_is_pure_qian_or_kun);
        assert_eq!(qian.changed_lower_from_lines_2_3_4, Trigram::Li);
        assert_eq!(qian.changed_upper_from_lines_3_4_5, Trigram::Xun);
        assert_eq!(qian.selected_received_basis, None);
    }

    #[test]
    fn mutual_raw_rejects_out_of_range_and_non_single_line_changes() {
        assert_eq!(
            meihua_mutual_raw_audit(64, 0),
            Err(MeihuaMutualRawError::InvalidHexagramBits)
        );
        assert_eq!(
            meihua_mutual_raw_audit(0, 64),
            Err(MeihuaMutualRawError::InvalidHexagramBits)
        );
        assert_eq!(
            meihua_mutual_raw_audit(0, 0),
            Err(MeihuaMutualRawError::NotSingleMovingLine)
        );
        assert_eq!(
            meihua_mutual_raw_audit(0, 0b000011),
            Err(MeihuaMutualRawError::NotSingleMovingLine)
        );
    }
}
