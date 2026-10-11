                                                                            
                                                                        

use crate::{
    sexagenary_hour_from_day_stem, EarthlyBranch, FiveElement, HeavenlyStem, JieMonthCandidate,
    LichunYearCandidate, SexagenaryDesignation, ShichenBoundary, StemBranchMetadata, YinYang,
    STEM_BRANCH_METADATA_ALGORITHM_VERSION, STEM_BRANCH_METADATA_STATUS,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const FOUR_PILLARS_CHART_ALGORITHM_VERSION: &str =
    "jiuheng.four_pillars.mechanical_lichun_jie_true_solar.v1";
pub const FOUR_PILLARS_CHART_STATUS: &str =
    "chart_candidate_only_no_fortune_rules_or_runtime_signals.v1";
pub const LATE_ZI_DUAL_CANDIDATE_ALGORITHM_VERSION: &str =
    "jiuheng.four_pillars.late_zi_parallel_day_boundaries.v1";
pub const LATE_ZI_DUAL_CANDIDATE_STATUS: &str =
    "parallel_candidates_unresolved_not_rule_eligible.v1";
pub const FOUR_PILLARS_VISIBLE_STEM_TEN_GODS_ALGORITHM_VERSION: &str =
    "jiuheng.four_pillars.visible_stem_ten_gods.day_master_relation.v1";
pub const FOUR_PILLARS_VISIBLE_STEM_TEN_GODS_STATUS: &str =
    "mechanical_visible_stem_identity_only_no_hidden_stems_strength_pattern_or_interpretation.v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FourPillarsChartCandidate {
    pub year_candidate: LichunYearCandidate,
    pub month_candidate: JieMonthCandidate,
    pub day_pillar_bjt_midnight: SexagenaryDesignation,
    pub day_pillar_true_solar_midnight: SexagenaryDesignation,
    pub true_solar_shichen: ShichenBoundary,
    pub hour_pillar_true_solar_midnight: SexagenaryDesignation,
    pub late_zi_hour_day_ambiguous: bool,
    pub true_solar_time_boundary_safe: bool,
    pub mechanical_boundaries_safe: bool,
    pub rule_eligible: bool,
    pub year_convention: String,
    pub month_convention: String,
    pub day_convention: String,
    pub hour_convention: String,
    pub algorithm_version: String,
    pub status: String,
}

impl FourPillarsChartCandidate {
    pub fn compact_zh(&self) -> String {
        format!(
            "{} {} {} {}",
            self.year_candidate.designation.zh_name(),
            self.month_candidate.designation.zh_name(),
            self.day_pillar_true_solar_midnight.zh_name(),
            self.hour_pillar_true_solar_midnight.zh_name()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FourPillarsStructuralMetadata {
    pub year: StemBranchMetadata,
    pub month: StemBranchMetadata,
    pub day_true_solar_midnight: StemBranchMetadata,
    pub hour_true_solar_midnight: StemBranchMetadata,
    pub algorithm_version: String,
    pub status: String,
}

impl FourPillarsStructuralMetadata {
    pub fn compact_zh(&self) -> String {
        format!(
            "{}  {}  {}  {}",
            self.year.compact_zh(),
            self.month.compact_zh(),
            self.day_true_solar_midnight.compact_zh(),
            self.hour_true_solar_midnight.compact_zh()
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FourPillarsPosition {
    Year,
    Month,
    Day,
    Hour,
}

impl FourPillarsPosition {
    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Year => "年干",
            Self::Month => "月干",
            Self::Day => "日干",
            Self::Hour => "时干",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum TenGodIdentity {
    Peer = 0,
    RobWealth = 1,
    EatingGod = 2,
    HurtingOfficer = 3,
    IndirectWealth = 4,
    DirectWealth = 5,
    SevenKillings = 6,
    DirectOfficer = 7,
    IndirectResource = 8,
    DirectResource = 9,
}

impl TenGodIdentity {
    pub const ALL: [Self; 10] = [
        Self::Peer,
        Self::RobWealth,
        Self::EatingGod,
        Self::HurtingOfficer,
        Self::IndirectWealth,
        Self::DirectWealth,
        Self::SevenKillings,
        Self::DirectOfficer,
        Self::IndirectResource,
        Self::DirectResource,
    ];

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Peer => "比肩",
            Self::RobWealth => "劫财",
            Self::EatingGod => "食神",
            Self::HurtingOfficer => "伤官",
            Self::IndirectWealth => "偏财",
            Self::DirectWealth => "正财",
            Self::SevenKillings => "七杀",
            Self::DirectOfficer => "正官",
            Self::IndirectResource => "偏印",
            Self::DirectResource => "正印",
        }
    }

    pub const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FourPillarsVisibleStemObservation {
    pub position: FourPillarsPosition,
    pub stem: HeavenlyStem,
    pub five_element: FiveElement,
    pub yin_yang: YinYang,
    pub is_day_master: bool,
    pub ten_god_identity: Option<TenGodIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FourPillarsVisibleStemTenGodAudit {
    pub audit_version: u16,
    pub algorithm_version: String,
    pub status: String,
    pub day_master_stem: HeavenlyStem,
    pub day_master_five_element: FiveElement,
    pub day_master_yin_yang: YinYang,
    pub observations: Vec<FourPillarsVisibleStemObservation>,
    pub visible_stem_identities_assigned: bool,
    pub hidden_stems_included: bool,
    pub hidden_stem_identities_assigned: bool,
    pub strength_assessed: bool,
    pub pattern_selected: bool,
    pub interpretation_eligible: bool,
    pub rule_eligible: bool,
    pub blocked_reasons: Vec<String>,
}

impl FourPillarsVisibleStemTenGodAudit {
    pub fn compact_zh(&self) -> String {
        self.observations
            .iter()
            .map(|item| {
                format!(
                    "{}{}·{}",
                    item.position.zh_name(),
                    item.stem.zh_name(),
                    item.ten_god_identity
                        .map(TenGodIdentity::zh_name)
                        .unwrap_or("日主")
                )
            })
            .collect::<Vec<_>>()
            .join(" / ")
    }
}

const fn five_element_generates(source: FiveElement, target: FiveElement) -> bool {
    matches!(
        (source, target),
        (FiveElement::Wood, FiveElement::Fire)
            | (FiveElement::Fire, FiveElement::Earth)
            | (FiveElement::Earth, FiveElement::Metal)
            | (FiveElement::Metal, FiveElement::Water)
            | (FiveElement::Water, FiveElement::Wood)
    )
}

const fn five_element_overcomes(source: FiveElement, target: FiveElement) -> bool {
    matches!(
        (source, target),
        (FiveElement::Wood, FiveElement::Earth)
            | (FiveElement::Earth, FiveElement::Water)
            | (FiveElement::Water, FiveElement::Fire)
            | (FiveElement::Fire, FiveElement::Metal)
            | (FiveElement::Metal, FiveElement::Wood)
    )
}

                                                                             
                                                                                
                                     
pub fn ten_god_for_visible_stems(day_master: HeavenlyStem, other: HeavenlyStem) -> TenGodIdentity {
    let day_element = day_master.five_element();
    let other_element = other.five_element();
    let same_polarity = day_master.yin_yang() as u8 == other.yin_yang() as u8;

    if day_element as u8 == other_element as u8 {
        if same_polarity {
            TenGodIdentity::Peer
        } else {
            TenGodIdentity::RobWealth
        }
    } else if five_element_generates(day_element, other_element) {
        if same_polarity {
            TenGodIdentity::EatingGod
        } else {
            TenGodIdentity::HurtingOfficer
        }
    } else if five_element_overcomes(day_element, other_element) {
        if same_polarity {
            TenGodIdentity::IndirectWealth
        } else {
            TenGodIdentity::DirectWealth
        }
    } else if five_element_overcomes(other_element, day_element) {
        if same_polarity {
            TenGodIdentity::SevenKillings
        } else {
            TenGodIdentity::DirectOfficer
        }
    } else if five_element_generates(other_element, day_element) {
        if same_polarity {
            TenGodIdentity::IndirectResource
        } else {
            TenGodIdentity::DirectResource
        }
    } else {
        unreachable!()
    }
}

pub fn four_pillars_visible_stem_ten_gods_audit(
    chart: &FourPillarsChartCandidate,
) -> FourPillarsVisibleStemTenGodAudit {
    visible_stem_ten_gods_for_day_and_hour(
        chart,
        chart.day_pillar_true_solar_midnight.stem,
        chart.hour_pillar_true_solar_midnight.stem,
    )
}

                                                                       
                                                                             
pub fn four_pillars_visible_stem_ten_gods_zi_start_audit(
    chart: &FourPillarsChartCandidate,
) -> Option<FourPillarsVisibleStemTenGodAudit> {
    let dual = late_zi_dual_candidates(chart)?;
    Some(visible_stem_ten_gods_for_day_and_hour(
        chart,
        dual.zi_start_boundary.day_pillar.stem,
        dual.zi_start_boundary.hour_pillar.stem,
    ))
}

fn visible_stem_ten_gods_for_day_and_hour(
    chart: &FourPillarsChartCandidate,
    day_master_stem: HeavenlyStem,
    hour_stem: HeavenlyStem,
) -> FourPillarsVisibleStemTenGodAudit {
    let stems = [
        (
            FourPillarsPosition::Year,
            chart.year_candidate.designation.stem,
        ),
        (
            FourPillarsPosition::Month,
            chart.month_candidate.designation.stem,
        ),
        (FourPillarsPosition::Day, day_master_stem),
        (FourPillarsPosition::Hour, hour_stem),
    ];
    let observations = stems
        .into_iter()
        .map(|(position, stem)| FourPillarsVisibleStemObservation {
            position,
            stem,
            five_element: stem.five_element(),
            yin_yang: stem.yin_yang(),
            is_day_master: position == FourPillarsPosition::Day,
            ten_god_identity: (position != FourPillarsPosition::Day)
                .then(|| ten_god_for_visible_stems(day_master_stem, stem)),
        })
        .collect();

    FourPillarsVisibleStemTenGodAudit {
        audit_version: 1,
        algorithm_version: FOUR_PILLARS_VISIBLE_STEM_TEN_GODS_ALGORITHM_VERSION.to_owned(),
        status: FOUR_PILLARS_VISIBLE_STEM_TEN_GODS_STATUS.to_owned(),
        day_master_stem,
        day_master_five_element: day_master_stem.five_element(),
        day_master_yin_yang: day_master_stem.yin_yang(),
        observations,
        visible_stem_identities_assigned: true,
        hidden_stems_included: false,
        hidden_stem_identities_assigned: false,
        strength_assessed: false,
        pattern_selected: false,
        interpretation_eligible: false,
        rule_eligible: false,
        blocked_reasons: vec![
            "facsimile_bytes_variants_and_transmission_layers_not_closed".to_owned(),
            "hidden_stem_catalog_and_weighting_not_selected".to_owned(),
            "strength_pattern_useful_god_luck_cycles_timing_and_judgment_not_reviewed".to_owned(),
        ],
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LateZiConventionCandidate {
    pub convention: String,
    pub day_pillar: SexagenaryDesignation,
    pub hour_pillar: SexagenaryDesignation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LateZiDualCandidates {
    pub midnight_boundary: LateZiConventionCandidate,
    pub zi_start_boundary: LateZiConventionCandidate,
    pub selected_convention: Option<String>,
    pub algorithm_version: String,
    pub status: String,
}

pub fn four_pillars_structural_metadata(
    chart: &FourPillarsChartCandidate,
) -> FourPillarsStructuralMetadata {
    FourPillarsStructuralMetadata {
        year: StemBranchMetadata::from_designation(chart.year_candidate.designation),
        month: StemBranchMetadata::from_designation(chart.month_candidate.designation),
        day_true_solar_midnight: StemBranchMetadata::from_designation(
            chart.day_pillar_true_solar_midnight,
        ),
        hour_true_solar_midnight: StemBranchMetadata::from_designation(
            chart.hour_pillar_true_solar_midnight,
        ),
        algorithm_version: STEM_BRANCH_METADATA_ALGORITHM_VERSION.to_owned(),
        status: STEM_BRANCH_METADATA_STATUS.to_owned(),
    }
}

                                                                            
                                                                 
pub fn late_zi_dual_candidates(chart: &FourPillarsChartCandidate) -> Option<LateZiDualCandidates> {
    if !chart.late_zi_hour_day_ambiguous {
        return None;
    }
    let zi_start_day = SexagenaryDesignation::from_cycle_index(
        i64::from(chart.day_pillar_true_solar_midnight.cycle_index) + 1,
    );
    let zi_start_hour = sexagenary_hour_from_day_stem(zi_start_day.stem, EarthlyBranch::Zi);
    Some(LateZiDualCandidates {
        midnight_boundary: LateZiConventionCandidate {
            convention: "true_solar_midnight_changes_day.v1".to_owned(),
            day_pillar: chart.day_pillar_true_solar_midnight,
            hour_pillar: chart.hour_pillar_true_solar_midnight,
        },
        zi_start_boundary: LateZiConventionCandidate {
            convention: "true_solar_zi_start_changes_day.v1".to_owned(),
            day_pillar: zi_start_day,
            hour_pillar: zi_start_hour,
        },
        selected_convention: None,
        algorithm_version: LATE_ZI_DUAL_CANDIDATE_ALGORITHM_VERSION.to_owned(),
        status: LATE_ZI_DUAL_CANDIDATE_STATUS.to_owned(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FourPillarsChartError {
    #[error("月柱候选采用的立春年干与年柱候选不一致")]
    InconsistentYearMonthStem,
    #[error("时柱候选的地支与真太阳时辰不一致")]
    InconsistentHourBranch,
    #[error("时柱候选与真太阳时零点换日的日干不一致")]
    InconsistentHourStem,
    #[error("晚子时标志只能用于子时")]
    LateZiOutsideZiHour,
}

                                                                              
                                                                            
                                                                           
                       
#[allow(clippy::too_many_arguments)]
pub fn four_pillars_chart_candidate(
    year_candidate: &LichunYearCandidate,
    month_candidate: &JieMonthCandidate,
    day_pillar_bjt_midnight: SexagenaryDesignation,
    day_pillar_true_solar_midnight: SexagenaryDesignation,
    true_solar_shichen: ShichenBoundary,
    hour_pillar_true_solar_midnight: SexagenaryDesignation,
    true_solar_time_boundary_safe: bool,
    late_zi_hour: bool,
) -> Result<FourPillarsChartCandidate, FourPillarsChartError> {
    if month_candidate.lichun_year_stem != year_candidate.designation.stem {
        return Err(FourPillarsChartError::InconsistentYearMonthStem);
    }
    if hour_pillar_true_solar_midnight.branch != true_solar_shichen.branch {
        return Err(FourPillarsChartError::InconsistentHourBranch);
    }
    let expected_hour = sexagenary_hour_from_day_stem(
        day_pillar_true_solar_midnight.stem,
        true_solar_shichen.branch,
    );
    if hour_pillar_true_solar_midnight != expected_hour {
        return Err(FourPillarsChartError::InconsistentHourStem);
    }
    if late_zi_hour && true_solar_shichen.branch != EarthlyBranch::Zi {
        return Err(FourPillarsChartError::LateZiOutsideZiHour);
    }

    let mechanical_boundaries_safe = year_candidate.boundary_safe
        && month_candidate.boundary_safe
        && true_solar_time_boundary_safe
        && true_solar_shichen.boundary_safe
        && !late_zi_hour;

    Ok(FourPillarsChartCandidate {
        year_candidate: year_candidate.clone(),
        month_candidate: month_candidate.clone(),
        day_pillar_bjt_midnight,
        day_pillar_true_solar_midnight,
        true_solar_shichen,
        hour_pillar_true_solar_midnight,
        late_zi_hour_day_ambiguous: late_zi_hour,
        true_solar_time_boundary_safe,
        mechanical_boundaries_safe,
        rule_eligible: false,
        year_convention: "precise_lichun_tt_boundary_candidate.v1".to_owned(),
        month_convention: "twelve_jie_interval_candidate.v1".to_owned(),
        day_convention: "true_solar_midnight_with_parallel_bjt_midnight_audit.v1".to_owned(),
        hour_convention: "true_solar_two_hour_branch_true_solar_day_stem.v1".to_owned(),
        algorithm_version: FOUR_PILLARS_CHART_ALGORITHM_VERSION.to_owned(),
        status: FOUR_PILLARS_CHART_STATUS.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HeavenlyStem, SolarTerm};

    fn candidates() -> (LichunYearCandidate, JieMonthCandidate) {
        let year_designation = SexagenaryDesignation::from_cycle_index(40);
        (
            LichunYearCandidate {
                civil_year: 2024,
                effective_year_label: 2024,
                designation: year_designation,
                boundary_jd_tt: 2_460_344.0,
                seconds_from_boundary: 50_000.0,
                boundary_margin_seconds: 300.0,
                boundary_safe: true,
                algorithm_version: "test.year".to_owned(),
                status: "test".to_owned(),
            },
            JieMonthCandidate {
                designation: SexagenaryDesignation::from_cycle_index(2),
                previous_jie: SolarTerm::Lichun,
                next_jie: SolarTerm::Jingzhe,
                lichun_year_stem: year_designation.stem,
                boundary_safe: true,
                algorithm_version: "test.month".to_owned(),
                status: "test".to_owned(),
            },
        )
    }

    #[test]
    fn assembles_consistent_chart_without_enabling_rules() {
        let (year, month) = candidates();
        let day = SexagenaryDesignation::from_cycle_index(0);
        let shichen = ShichenBoundary {
            branch: EarthlyBranch::Chen,
            minutes_to_boundary: 45.0,
            boundary_margin_minutes: 5.0,
            boundary_safe: true,
        };
        let hour = sexagenary_hour_from_day_stem(day.stem, shichen.branch);
        let chart =
            four_pillars_chart_candidate(&year, &month, day, day, shichen, hour, true, false)
                .unwrap();
        assert!(chart.mechanical_boundaries_safe);
        assert!(!chart.rule_eligible);
        assert_eq!(chart.compact_zh().split_whitespace().count(), 4);
        assert_eq!(
            chart.algorithm_version,
            FOUR_PILLARS_CHART_ALGORITHM_VERSION
        );
        let metadata = four_pillars_structural_metadata(&chart);
        assert_eq!(metadata.year.compact_zh(), "甲（阳木）辰（阳土）");
        assert_eq!(metadata.status, STEM_BRANCH_METADATA_STATUS);
        let ten_gods = four_pillars_visible_stem_ten_gods_audit(&chart);
        assert_eq!(ten_gods.day_master_stem, HeavenlyStem::Jia);
        assert_eq!(ten_gods.observations.len(), 4);
        assert_eq!(
            ten_gods
                .observations
                .iter()
                .map(|item| item.ten_god_identity)
                .collect::<Vec<_>>(),
            vec![
                Some(TenGodIdentity::Peer),
                Some(TenGodIdentity::EatingGod),
                None,
                Some(TenGodIdentity::IndirectWealth),
            ]
        );
        assert!(ten_gods.visible_stem_identities_assigned);
        assert!(!ten_gods.hidden_stems_included);
        assert!(!ten_gods.strength_assessed);
        assert!(!ten_gods.pattern_selected);
        assert!(!ten_gods.interpretation_eligible);
        assert!(!ten_gods.rule_eligible);
        assert_eq!(late_zi_dual_candidates(&chart), None);
    }

    #[test]
    fn all_hundred_visible_stem_pairs_assign_one_balanced_ten_god_identity() {
        let expected_for_jia = [
            TenGodIdentity::Peer,
            TenGodIdentity::RobWealth,
            TenGodIdentity::EatingGod,
            TenGodIdentity::HurtingOfficer,
            TenGodIdentity::IndirectWealth,
            TenGodIdentity::DirectWealth,
            TenGodIdentity::SevenKillings,
            TenGodIdentity::DirectOfficer,
            TenGodIdentity::IndirectResource,
            TenGodIdentity::DirectResource,
        ];
        assert_eq!(
            HeavenlyStem::ALL.map(|stem| ten_god_for_visible_stems(HeavenlyStem::Jia, stem)),
            expected_for_jia
        );

        let mut counts = [0_usize; 10];
        for day_master in HeavenlyStem::ALL {
            for other in HeavenlyStem::ALL {
                counts[ten_god_for_visible_stems(day_master, other).index()] += 1;
            }
        }
        assert_eq!(counts, [10; 10]);
    }

    #[test]
    fn rejects_inconsistent_hour_material() {
        let (year, month) = candidates();
        let day = SexagenaryDesignation::from_cycle_index(0);
        let shichen = ShichenBoundary {
            branch: EarthlyBranch::Chen,
            minutes_to_boundary: 45.0,
            boundary_margin_minutes: 5.0,
            boundary_safe: true,
        };
        let wrong_hour = SexagenaryDesignation::from_cycle_index(0);
        assert_eq!(
            four_pillars_chart_candidate(&year, &month, day, day, shichen, wrong_hour, true, false,),
            Err(FourPillarsChartError::InconsistentHourBranch)
        );
    }

    #[test]
    fn late_zi_ambiguity_blocks_aggregate_safety() {
        let (year, month) = candidates();
        let day = SexagenaryDesignation::from_cycle_index(0);
        let shichen = ShichenBoundary {
            branch: EarthlyBranch::Zi,
            minutes_to_boundary: 30.0,
            boundary_margin_minutes: 5.0,
            boundary_safe: true,
        };
        let hour = sexagenary_hour_from_day_stem(day.stem, shichen.branch);
        let chart =
            four_pillars_chart_candidate(&year, &month, day, day, shichen, hour, true, true)
                .unwrap();
        assert!(chart.late_zi_hour_day_ambiguous);
        assert!(!chart.mechanical_boundaries_safe);
        assert!(!chart.rule_eligible);
        let dual = late_zi_dual_candidates(&chart).expect("late Zi keeps both candidates");
        assert_eq!(dual.midnight_boundary.day_pillar.zh_name(), "甲子");
        assert_eq!(dual.midnight_boundary.hour_pillar.zh_name(), "甲子");
        assert_eq!(dual.zi_start_boundary.day_pillar.zh_name(), "乙丑");
        assert_eq!(dual.zi_start_boundary.hour_pillar.zh_name(), "丙子");
        assert_eq!(dual.selected_convention, None);
        assert_eq!(dual.status, LATE_ZI_DUAL_CANDIDATE_STATUS);
        let midnight_ten_gods = four_pillars_visible_stem_ten_gods_audit(&chart);
        let zi_start_ten_gods =
            four_pillars_visible_stem_ten_gods_zi_start_audit(&chart).expect("Zi-start identities");
        assert_eq!(midnight_ten_gods.day_master_stem.zh_name(), "甲");
        assert_eq!(zi_start_ten_gods.day_master_stem.zh_name(), "乙");
        assert_eq!(midnight_ten_gods.observations[3].stem.zh_name(), "甲");
        assert_eq!(zi_start_ten_gods.observations[3].stem.zh_name(), "丙");
        assert_eq!(
            midnight_ten_gods.observations[0].ten_god_identity,
            Some(TenGodIdentity::Peer)
        );
        assert_eq!(
            zi_start_ten_gods.observations[0].ten_god_identity,
            Some(TenGodIdentity::RobWealth)
        );
        assert!(!midnight_ten_gods.rule_eligible);
        assert!(!zi_start_ten_gods.rule_eligible);

        let non_zi = ShichenBoundary {
            branch: EarthlyBranch::Chen,
            ..shichen
        };
        let non_zi_hour = sexagenary_hour_from_day_stem(day.stem, non_zi.branch);
        assert_eq!(
            four_pillars_chart_candidate(&year, &month, day, day, non_zi, non_zi_hour, true, true,),
            Err(FourPillarsChartError::LateZiOutsideZiHour)
        );
    }

    #[test]
    fn rejects_month_stem_from_a_different_year_candidate() {
        let (year, mut month) = candidates();
        month.lichun_year_stem = HeavenlyStem::Yi;
        let day = SexagenaryDesignation::from_cycle_index(0);
        let shichen = ShichenBoundary {
            branch: EarthlyBranch::Chen,
            minutes_to_boundary: 45.0,
            boundary_margin_minutes: 5.0,
            boundary_safe: true,
        };
        let hour = sexagenary_hour_from_day_stem(day.stem, shichen.branch);
        assert_eq!(
            four_pillars_chart_candidate(&year, &month, day, day, shichen, hour, true, false,),
            Err(FourPillarsChartError::InconsistentYearMonthStem)
        );
    }
}
