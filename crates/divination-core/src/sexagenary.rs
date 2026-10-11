use crate::calendar::{julian_day, CalendarError, CalendarSystem, HistoricalDateTime};
use crate::{SolarTerm, SolarTermClass, SolarTermClassInterval, SolarTermEvent};
use chrono::{DateTime, Datelike, FixedOffset, Timelike};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const SEXAGENARY_DAY_ALGORITHM_VERSION: &str =
    "jiuheng.sexagenary_day.hko_2024_02_10_anchor.v1";
pub const SHICHEN_ALGORITHM_VERSION: &str = "jiuheng.shichen.true_solar.two_hour.v1";
pub const ZI_HOUR_DAY_BOUNDARY_STATUS: &str = "unresolved_tradition_difference_not_rule_eligible";
pub const PILLAR_CANDIDATE_ALGORITHM_VERSION: &str =
    "jiuheng.pillar_candidate.lichun_jie_hko_tables.v1";
pub const PILLAR_CANDIDATE_STATUS: &str = "audit_only_convention_not_selected_by_runtime_rule";
pub const STEM_BRANCH_METADATA_ALGORITHM_VERSION: &str =
    "jiuheng.stem_branch.yinyang_five_elements.huainanzi_wuxingdayi_leijingtuyi.v1";
pub const STEM_BRANCH_METADATA_STATUS: &str =
    "structural_metadata_only_not_strength_or_fortune_rule.v1";

#[derive(Debug, Clone, PartialEq, Error)]
pub enum PillarCandidateError {
    #[error("年柱候选输入必须为有限数值")]
    NonFiniteInput,
    #[error("年柱候选边界必须是立春")]
    BoundaryIsNotLichun,
    #[error("月柱候选必须使用节区间")]
    IntervalIsNotJie,
    #[error("月柱候选遇到未映射的节：{0}")]
    UnsupportedJie(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeavenlyStem {
    Jia,
    Yi,
    Bing,
    Ding,
    Wu,
    Ji,
    Geng,
    Xin,
    Ren,
    Gui,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum YinYang {
    Yang,
    Yin,
}

impl YinYang {
    pub fn zh_name(self) -> &'static str {
        match self {
            Self::Yang => "阳",
            Self::Yin => "阴",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FiveElement {
    Wood,
    Fire,
    Earth,
    Metal,
    Water,
}

impl FiveElement {
    pub fn zh_name(self) -> &'static str {
        match self {
            Self::Wood => "木",
            Self::Fire => "火",
            Self::Earth => "土",
            Self::Metal => "金",
            Self::Water => "水",
        }
    }
}

impl HeavenlyStem {
    pub const ALL: [Self; 10] = [
        Self::Jia,
        Self::Yi,
        Self::Bing,
        Self::Ding,
        Self::Wu,
        Self::Ji,
        Self::Geng,
        Self::Xin,
        Self::Ren,
        Self::Gui,
    ];

    pub fn index(self) -> u8 {
        Self::ALL
            .iter()
            .position(|value| *value == self)
            .expect("stem is in the complete table") as u8
    }

    pub fn zh_name(self) -> &'static str {
        ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"][self.index() as usize]
    }

    pub fn yin_yang(self) -> YinYang {
        match self {
            Self::Jia | Self::Bing | Self::Wu | Self::Geng | Self::Ren => YinYang::Yang,
            Self::Yi | Self::Ding | Self::Ji | Self::Xin | Self::Gui => YinYang::Yin,
        }
    }

    pub fn five_element(self) -> FiveElement {
        match self {
            Self::Jia | Self::Yi => FiveElement::Wood,
            Self::Bing | Self::Ding => FiveElement::Fire,
            Self::Wu | Self::Ji => FiveElement::Earth,
            Self::Geng | Self::Xin => FiveElement::Metal,
            Self::Ren | Self::Gui => FiveElement::Water,
        }
    }

    fn from_index(index: u8) -> Self {
        Self::ALL[index as usize % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EarthlyBranch {
    Zi,
    Chou,
    Yin,
    Mao,
    Chen,
    Si,
    Wu,
    Wei,
    Shen,
    You,
    Xu,
    Hai,
}

impl EarthlyBranch {
    pub const ALL: [Self; 12] = [
        Self::Zi,
        Self::Chou,
        Self::Yin,
        Self::Mao,
        Self::Chen,
        Self::Si,
        Self::Wu,
        Self::Wei,
        Self::Shen,
        Self::You,
        Self::Xu,
        Self::Hai,
    ];

    pub fn index(self) -> u8 {
        Self::ALL
            .iter()
            .position(|value| *value == self)
            .expect("branch is in the complete table") as u8
    }

    pub fn zh_name(self) -> &'static str {
        [
            "子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥",
        ][self.index() as usize]
    }

    pub fn yin_yang(self) -> YinYang {
        match self {
            Self::Zi | Self::Yin | Self::Chen | Self::Wu | Self::Shen | Self::Xu => YinYang::Yang,
            Self::Chou | Self::Mao | Self::Si | Self::Wei | Self::You | Self::Hai => YinYang::Yin,
        }
    }

    pub fn five_element(self) -> FiveElement {
        match self {
            Self::Yin | Self::Mao => FiveElement::Wood,
            Self::Si | Self::Wu => FiveElement::Fire,
            Self::Chou | Self::Chen | Self::Wei | Self::Xu => FiveElement::Earth,
            Self::Shen | Self::You => FiveElement::Metal,
            Self::Zi | Self::Hai => FiveElement::Water,
        }
    }

    fn from_index(index: u8) -> Self {
        Self::ALL[index as usize % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SexagenaryDesignation {
                                                  
    pub cycle_index: u8,
    pub stem: HeavenlyStem,
    pub branch: EarthlyBranch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemBranchMetadata {
    pub designation: SexagenaryDesignation,
    pub stem_yin_yang: YinYang,
    pub stem_element: FiveElement,
    pub branch_yin_yang: YinYang,
    pub branch_element: FiveElement,
}

impl StemBranchMetadata {
    pub fn from_designation(designation: SexagenaryDesignation) -> Self {
        Self {
            designation,
            stem_yin_yang: designation.stem.yin_yang(),
            stem_element: designation.stem.five_element(),
            branch_yin_yang: designation.branch.yin_yang(),
            branch_element: designation.branch.five_element(),
        }
    }

    pub fn compact_zh(self) -> String {
        format!(
            "{}（{}{}）{}（{}{}）",
            self.designation.stem.zh_name(),
            self.stem_yin_yang.zh_name(),
            self.stem_element.zh_name(),
            self.designation.branch.zh_name(),
            self.branch_yin_yang.zh_name(),
            self.branch_element.zh_name()
        )
    }
}

impl SexagenaryDesignation {
    pub fn from_cycle_index(index: i64) -> Self {
        let normalized = index.rem_euclid(60) as u8;
        Self {
            cycle_index: normalized,
            stem: HeavenlyStem::from_index(normalized % 10),
            branch: EarthlyBranch::from_index(normalized % 12),
        }
    }

    pub fn zh_name(self) -> String {
        format!("{}{}", self.stem.zh_name(), self.branch.zh_name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ShichenBoundary {
    pub branch: EarthlyBranch,
    pub minutes_to_boundary: f64,
    pub boundary_margin_minutes: f64,
    pub boundary_safe: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LichunYearCandidate {
    pub civil_year: i32,
    pub effective_year_label: i32,
    pub designation: SexagenaryDesignation,
    pub boundary_jd_tt: f64,
    pub seconds_from_boundary: f64,
    pub boundary_margin_seconds: f64,
    pub boundary_safe: bool,
    pub algorithm_version: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JieMonthCandidate {
    pub designation: SexagenaryDesignation,
    pub previous_jie: SolarTerm,
    pub next_jie: SolarTerm,
    pub lichun_year_stem: HeavenlyStem,
    pub boundary_safe: bool,
    pub algorithm_version: String,
    pub status: String,
}

                                                                              
                                           
pub fn sexagenary_year_for_astronomical_year_label(year: i32) -> SexagenaryDesignation {
    SexagenaryDesignation::from_cycle_index(i64::from(year) - 4)
}

                                                                          
                                                                              
pub fn lichun_year_candidate(
    civil_year: i32,
    jd_tt: f64,
    lichun: &SolarTermEvent,
    boundary_margin_seconds: f64,
) -> Result<LichunYearCandidate, PillarCandidateError> {
    if !jd_tt.is_finite()
        || !lichun.jd_tt.is_finite()
        || !boundary_margin_seconds.is_finite()
        || boundary_margin_seconds < 0.0
    {
        return Err(PillarCandidateError::NonFiniteInput);
    }
    if lichun.term != SolarTerm::Lichun {
        return Err(PillarCandidateError::BoundaryIsNotLichun);
    }
    let seconds_from_boundary = (jd_tt - lichun.jd_tt) * 86_400.0;
    let effective_year_label = if seconds_from_boundary >= 0.0 {
        civil_year
    } else {
        civil_year - 1
    };
    Ok(LichunYearCandidate {
        civil_year,
        effective_year_label,
        designation: sexagenary_year_for_astronomical_year_label(effective_year_label),
        boundary_jd_tt: lichun.jd_tt,
        seconds_from_boundary,
        boundary_margin_seconds,
        boundary_safe: seconds_from_boundary.abs()
            > lichun.combined_uncertainty_seconds + boundary_margin_seconds,
        algorithm_version: PILLAR_CANDIDATE_ALGORITHM_VERSION.to_owned(),
        status: PILLAR_CANDIDATE_STATUS.to_owned(),
    })
}

                                                                             
                                                                          
pub fn jie_month_candidate(
    lichun_year_stem: HeavenlyStem,
    interval: &SolarTermClassInterval,
) -> Result<JieMonthCandidate, PillarCandidateError> {
    if interval.class != SolarTermClass::Jie {
        return Err(PillarCandidateError::IntervalIsNotJie);
    }
    let (branch, month_offset) = jie_month_branch(interval.previous.term)?;
    let yin_stem_index = (lichun_year_stem.index() % 5) * 2 + 2;
    let month_stem = HeavenlyStem::from_index(yin_stem_index + month_offset);
    let designation = (0..60)
        .map(SexagenaryDesignation::from_cycle_index)
        .find(|candidate| candidate.stem == month_stem && candidate.branch == branch)
        .expect("the month table always forms a valid stem-branch pair");
    Ok(JieMonthCandidate {
        designation,
        previous_jie: interval.previous.term,
        next_jie: interval.next.term,
        lichun_year_stem,
        boundary_safe: interval.boundary_safe,
        algorithm_version: PILLAR_CANDIDATE_ALGORITHM_VERSION.to_owned(),
        status: PILLAR_CANDIDATE_STATUS.to_owned(),
    })
}

fn jie_month_branch(term: SolarTerm) -> Result<(EarthlyBranch, u8), PillarCandidateError> {
    let value = match term {
        SolarTerm::Lichun => (EarthlyBranch::Yin, 0),
        SolarTerm::Jingzhe => (EarthlyBranch::Mao, 1),
        SolarTerm::Qingming => (EarthlyBranch::Chen, 2),
        SolarTerm::Lixia => (EarthlyBranch::Si, 3),
        SolarTerm::Mangzhong => (EarthlyBranch::Wu, 4),
        SolarTerm::Xiaoshu => (EarthlyBranch::Wei, 5),
        SolarTerm::Liqiu => (EarthlyBranch::Shen, 6),
        SolarTerm::Bailu => (EarthlyBranch::You, 7),
        SolarTerm::Hanlu => (EarthlyBranch::Xu, 8),
        SolarTerm::Lidong => (EarthlyBranch::Hai, 9),
        SolarTerm::Daxue => (EarthlyBranch::Zi, 10),
        SolarTerm::Xiaohan => (EarthlyBranch::Chou, 11),
        _ => {
            return Err(PillarCandidateError::UnsupportedJie(
                term.zh_name().to_owned(),
            ));
        }
    };
    Ok(value)
}

                                                                          
   
                                                                      
                                                                           
                                                                            
pub fn sexagenary_day(
    date_time: HistoricalDateTime,
    system: CalendarSystem,
) -> Result<SexagenaryDesignation, CalendarError> {
    let midnight = HistoricalDateTime {
        hour: 0,
        minute: 0,
        second: 0,
        ..date_time
    };
    let jd_midnight = julian_day(midnight, system)?;
    let julian_day_number = (jd_midnight + 0.5).floor() as i64;
    Ok(SexagenaryDesignation::from_cycle_index(
        julian_day_number + 49,
    ))
}

pub fn sexagenary_day_for_fixed_date(
    date_time: DateTime<FixedOffset>,
    system: CalendarSystem,
) -> Result<SexagenaryDesignation, CalendarError> {
    sexagenary_day(
        HistoricalDateTime {
            year: date_time.year(),
            month: date_time.month() as u8,
            day: date_time.day() as u8,
            hour: 0,
            minute: 0,
            second: 0,
        },
        system,
    )
}

                                                                    
pub fn shichen_from_true_solar_time(
    true_solar_time: DateTime<FixedOffset>,
    boundary_margin_minutes: f64,
) -> Result<ShichenBoundary, CalendarError> {
    if !boundary_margin_minutes.is_finite() {
        return Err(CalendarError::NonFiniteSolarTimeInput);
    }
    if boundary_margin_minutes < 0.0 {
        return Err(CalendarError::NegativeBoundaryMargin);
    }

    let minute_of_day = true_solar_time.hour() as f64 * 60.0
        + true_solar_time.minute() as f64
        + true_solar_time.second() as f64 / 60.0
        + true_solar_time.nanosecond() as f64 / 60_000_000_000.0;
    let shifted = (minute_of_day + 60.0).rem_euclid(1_440.0);
    let branch_index = (shifted / 120.0).floor() as u8;
    let minutes_inside = shifted.rem_euclid(120.0);
    let minutes_to_boundary = minutes_inside.min(120.0 - minutes_inside);

    Ok(ShichenBoundary {
        branch: EarthlyBranch::from_index(branch_index),
        minutes_to_boundary,
        boundary_margin_minutes,
        boundary_safe: minutes_to_boundary >= boundary_margin_minutes,
    })
}

                                                                            
                                                                           
pub fn sexagenary_hour_from_day_stem(
    day_stem: HeavenlyStem,
    hour_branch: EarthlyBranch,
) -> SexagenaryDesignation {
    let zi_stem_index = (day_stem.index() % 5) * 2;
    let hour_stem = HeavenlyStem::from_index(zi_stem_index + hour_branch.index());
    let cycle_index = (0..60)
        .find(|index| {
            index % 10 == hour_stem.index() as i32 && index % 12 == hour_branch.index() as i32
        })
        .expect("the hour table always forms a valid stem-branch pair");
    SexagenaryDesignation::from_cycle_index(cycle_index as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AstronomyPrecision;
    use chrono::TimeZone;

    fn term_event(term: SolarTerm, jd_tt: f64) -> SolarTermEvent {
        SolarTermEvent {
            term,
            jd_tt,
            final_bracket_seconds: 0.1,
            iterations: 20,
            solver_uncertainty_seconds: 0.05,
            ephemeris_time_uncertainty_seconds: 100.0,
            combined_uncertainty_seconds: 100.05,
            precision: AstronomyPrecision::SpaReferenceVectorMatched,
            algorithm_version: "test.solar".to_owned(),
        }
    }

    fn jie_interval(previous: SolarTerm, next: SolarTerm) -> SolarTermClassInterval {
        SolarTermClassInterval {
            class: SolarTermClass::Jie,
            previous: term_event(previous, 2_460_000.0),
            next: term_event(next, 2_460_030.0),
            seconds_since_previous: 1_000.0,
            seconds_until_next: 2_000.0,
            boundary_margin_seconds: 300.0,
            boundary_safe: true,
            algorithm_version: "test.jie".to_owned(),
        }
    }

    fn historical_date(year: i32, month: u8, day: u8) -> HistoricalDateTime {
        HistoricalDateTime {
            year,
            month,
            day,
            hour: 12,
            minute: 0,
            second: 0,
        }
    }

    #[test]
    fn cycle_starts_with_jia_zi_and_ends_with_gui_hai() {
        assert_eq!(SexagenaryDesignation::from_cycle_index(0).zh_name(), "甲子");
        assert_eq!(
            SexagenaryDesignation::from_cycle_index(59).zh_name(),
            "癸亥"
        );
        assert_eq!(
            SexagenaryDesignation::from_cycle_index(60).zh_name(),
            "甲子"
        );
        assert_eq!(
            SexagenaryDesignation::from_cycle_index(-1).zh_name(),
            "癸亥"
        );
    }

    #[test]
    fn maps_all_stems_to_audited_yin_yang_and_five_elements() {
        let expected = [
            (HeavenlyStem::Jia, YinYang::Yang, FiveElement::Wood),
            (HeavenlyStem::Yi, YinYang::Yin, FiveElement::Wood),
            (HeavenlyStem::Bing, YinYang::Yang, FiveElement::Fire),
            (HeavenlyStem::Ding, YinYang::Yin, FiveElement::Fire),
            (HeavenlyStem::Wu, YinYang::Yang, FiveElement::Earth),
            (HeavenlyStem::Ji, YinYang::Yin, FiveElement::Earth),
            (HeavenlyStem::Geng, YinYang::Yang, FiveElement::Metal),
            (HeavenlyStem::Xin, YinYang::Yin, FiveElement::Metal),
            (HeavenlyStem::Ren, YinYang::Yang, FiveElement::Water),
            (HeavenlyStem::Gui, YinYang::Yin, FiveElement::Water),
        ];
        for (stem, yin_yang, element) in expected {
            assert_eq!(stem.yin_yang(), yin_yang);
            assert_eq!(stem.five_element(), element);
        }
    }

    #[test]
    fn maps_all_branches_to_audited_yin_yang_and_five_elements() {
        let expected = [
            (EarthlyBranch::Zi, YinYang::Yang, FiveElement::Water),
            (EarthlyBranch::Chou, YinYang::Yin, FiveElement::Earth),
            (EarthlyBranch::Yin, YinYang::Yang, FiveElement::Wood),
            (EarthlyBranch::Mao, YinYang::Yin, FiveElement::Wood),
            (EarthlyBranch::Chen, YinYang::Yang, FiveElement::Earth),
            (EarthlyBranch::Si, YinYang::Yin, FiveElement::Fire),
            (EarthlyBranch::Wu, YinYang::Yang, FiveElement::Fire),
            (EarthlyBranch::Wei, YinYang::Yin, FiveElement::Earth),
            (EarthlyBranch::Shen, YinYang::Yang, FiveElement::Metal),
            (EarthlyBranch::You, YinYang::Yin, FiveElement::Metal),
            (EarthlyBranch::Xu, YinYang::Yang, FiveElement::Earth),
            (EarthlyBranch::Hai, YinYang::Yin, FiveElement::Water),
        ];
        for (branch, yin_yang, element) in expected {
            assert_eq!(branch.yin_yang(), yin_yang);
            assert_eq!(branch.five_element(), element);
        }
    }

    #[test]
    fn metadata_preserves_designation_and_adds_no_interpretation() {
        let metadata =
            StemBranchMetadata::from_designation(SexagenaryDesignation::from_cycle_index(0));
        assert_eq!(metadata.compact_zh(), "甲（阳木）子（阳水）");
        assert_eq!(metadata.designation.zh_name(), "甲子");
        assert_eq!(
            STEM_BRANCH_METADATA_STATUS,
            "structural_metadata_only_not_strength_or_fortune_rule.v1"
        );
    }

    #[test]
    fn matches_hong_kong_observatory_2024_daily_anchors() {
        let february_first =
            sexagenary_day(historical_date(2024, 2, 1), CalendarSystem::Gregorian).unwrap();
        let lunar_new_year =
            sexagenary_day(historical_date(2024, 2, 10), CalendarSystem::Gregorian).unwrap();

        assert_eq!(february_first.zh_name(), "乙未");
        assert_eq!(february_first.cycle_index, 31);
        assert_eq!(lunar_new_year.zh_name(), "甲辰");
        assert_eq!(lunar_new_year.cycle_index, 40);
    }

    #[test]
    fn historical_cutover_advances_the_cycle_by_one_day() {
        let before = sexagenary_day(
            historical_date(1582, 10, 4),
            CalendarSystem::HistoricalCutover,
        )
        .unwrap();
        let after = sexagenary_day(
            historical_date(1582, 10, 15),
            CalendarSystem::HistoricalCutover,
        )
        .unwrap();
        assert_eq!((before.cycle_index + 1) % 60, after.cycle_index);
    }

    #[test]
    fn maps_true_solar_time_to_hko_two_hour_branches() {
        let zone = FixedOffset::east_opt(8 * 3_600).unwrap();
        let zi_start = zone.with_ymd_and_hms(2026, 8, 31, 23, 0, 0).unwrap();
        let zi_middle = zone.with_ymd_and_hms(2026, 8, 31, 0, 0, 0).unwrap();
        let chou_start = zone.with_ymd_and_hms(2026, 8, 31, 1, 0, 0).unwrap();

        let start = shichen_from_true_solar_time(zi_start, 5.0).unwrap();
        let middle = shichen_from_true_solar_time(zi_middle, 5.0).unwrap();
        let chou = shichen_from_true_solar_time(chou_start, 5.0).unwrap();
        assert_eq!(start.branch, EarthlyBranch::Zi);
        assert_eq!(start.minutes_to_boundary, 0.0);
        assert!(!start.boundary_safe);
        assert_eq!(middle.branch, EarthlyBranch::Zi);
        assert_eq!(middle.minutes_to_boundary, 60.0);
        assert!(middle.boundary_safe);
        assert_eq!(chou.branch, EarthlyBranch::Chou);
    }

    #[test]
    fn matches_hko_day_stem_to_hour_stem_table() {
        assert_eq!(
            sexagenary_hour_from_day_stem(HeavenlyStem::Jia, EarthlyBranch::Zi).zh_name(),
            "甲子"
        );
        assert_eq!(
            sexagenary_hour_from_day_stem(HeavenlyStem::Jia, EarthlyBranch::Hai).zh_name(),
            "乙亥"
        );
        assert_eq!(
            sexagenary_hour_from_day_stem(HeavenlyStem::Yi, EarthlyBranch::Zi).zh_name(),
            "丙子"
        );
        assert_eq!(
            sexagenary_hour_from_day_stem(HeavenlyStem::Gui, EarthlyBranch::Hai).zh_name(),
            "癸亥"
        );
    }

    #[test]
    fn labels_modern_astronomical_years_in_the_sixty_cycle() {
        assert_eq!(
            sexagenary_year_for_astronomical_year_label(1984).zh_name(),
            "甲子"
        );
        assert_eq!(
            sexagenary_year_for_astronomical_year_label(2023).zh_name(),
            "癸卯"
        );
        assert_eq!(
            sexagenary_year_for_astronomical_year_label(2024).zh_name(),
            "甲辰"
        );
    }

    #[test]
    fn lichun_candidate_changes_year_but_blocks_its_uncertainty_band() {
        let lichun = term_event(SolarTerm::Lichun, 2_460_000.0);
        let before = lichun_year_candidate(2024, 2_459_999.0, &lichun, 300.0).unwrap();
        let after = lichun_year_candidate(2024, 2_460_001.0, &lichun, 300.0).unwrap();
        let boundary = lichun_year_candidate(2024, 2_460_000.0, &lichun, 300.0).unwrap();

        assert_eq!(before.effective_year_label, 2023);
        assert_eq!(before.designation.zh_name(), "癸卯");
        assert!(before.boundary_safe);
        assert_eq!(after.effective_year_label, 2024);
        assert_eq!(after.designation.zh_name(), "甲辰");
        assert!(after.boundary_safe);
        assert!(!boundary.boundary_safe);
        assert_eq!(boundary.status, PILLAR_CANDIDATE_STATUS);
    }

    #[test]
    fn matches_hko_year_stem_to_month_stem_table_for_all_five_groups() {
        let lichun_to_jingzhe = jie_interval(SolarTerm::Lichun, SolarTerm::Jingzhe);
        let expected = [
            (HeavenlyStem::Jia, "丙寅"),
            (HeavenlyStem::Yi, "戊寅"),
            (HeavenlyStem::Bing, "庚寅"),
            (HeavenlyStem::Ding, "壬寅"),
            (HeavenlyStem::Wu, "甲寅"),
            (HeavenlyStem::Ji, "丙寅"),
            (HeavenlyStem::Geng, "戊寅"),
            (HeavenlyStem::Xin, "庚寅"),
            (HeavenlyStem::Ren, "壬寅"),
            (HeavenlyStem::Gui, "甲寅"),
        ];
        for (year_stem, expected_name) in expected {
            let candidate = jie_month_candidate(year_stem, &lichun_to_jingzhe).unwrap();
            assert_eq!(candidate.designation.zh_name(), expected_name);
            assert_eq!(candidate.status, PILLAR_CANDIDATE_STATUS);
        }

        let xiaohan_to_lichun = jie_interval(SolarTerm::Xiaohan, SolarTerm::Lichun);
        assert_eq!(
            jie_month_candidate(HeavenlyStem::Jia, &xiaohan_to_lichun)
                .unwrap()
                .designation
                .zh_name(),
            "丁丑"
        );
    }

    #[test]
    fn month_candidate_rejects_a_zhongqi_interval() {
        let mut interval = jie_interval(SolarTerm::Lichun, SolarTerm::Jingzhe);
        interval.class = SolarTermClass::Zhongqi;
        assert!(matches!(
            jie_month_candidate(HeavenlyStem::Jia, &interval),
            Err(PillarCandidateError::IntervalIsNotJie)
        ));
    }
}
