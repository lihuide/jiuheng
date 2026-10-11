                                                                      
                                                                         
                                                          

use crate::{
    estimate_delta_t, historical_datetime_from_julian_day, julian_day, lunation_interval,
    solar_term_class_interval, solve_solar_term_near_bjt_date, true_lunar_phase, AstronomyError,
    CalendarError, CalendarSystem, EphemerisProvider, HistoricalDateTime, LunarPhase,
    LunarPhaseError, LunarPhaseEvent, SolarTerm, SolarTermClass, SolarTermEvent,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const LUNISOLAR_MONTH_STRUCTURE_ALGORITHM_VERSION: &str =
    "jiuheng.lunisolar_month_structure.solstice_zhongqi_civil_day.v1";
pub const LUNISOLAR_MONTH_STRUCTURE_STATUS: &str =
    "audit_candidate_not_complete_chinese_calendar.v1";
pub const LUNISOLAR_DATE_CANDIDATE_ALGORITHM_VERSION: &str =
    "jiuheng.lunisolar_date.modern_official_civil_day.v1";
pub const LUNISOLAR_DATE_CANDIDATE_STATUS: &str =
    "modern_official_candidate_not_historical_calendar_or_divination_rule.v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BeijingCivilDayAudit {
    pub day_number: i64,
    pub local_date_time: HistoricalDateTime,
    pub seconds_from_midnight: f64,
    pub event_uncertainty_seconds: f64,
    pub delta_t_uncertainty_seconds: f64,
    pub boundary_margin_seconds: f64,
    pub midnight_boundary_safe: bool,
    pub conversion_status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LunisolarMonthCandidate {
    pub sequence_index: u8,
    pub month_number: u8,
    pub is_leap: bool,
    pub start_new_moon: LunarPhaseEvent,
    pub start_bjt: BeijingCivilDayAudit,
    pub end_new_moon: LunarPhaseEvent,
    pub end_bjt: BeijingCivilDayAudit,
    pub major_solar_term: Option<SolarTermEvent>,
    pub major_solar_term_bjt: Option<BeijingCivilDayAudit>,
    pub contains_major_solar_term: bool,
    pub term_classification_safe: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LunisolarMonthStructure {
                                                                             
    pub winter_solstice_year: i32,
    pub previous_winter_solstice: SolarTermEvent,
    pub previous_winter_solstice_bjt: BeijingCivilDayAudit,
    pub next_winter_solstice: SolarTermEvent,
    pub next_winter_solstice_bjt: BeijingCivilDayAudit,
    pub month_count: u8,
    pub leap_month_number: Option<u8>,
    pub all_boundaries_safe: bool,
    pub months: Vec<LunisolarMonthCandidate>,
    pub algorithm_version: String,
    pub status: String,
}

                                                                             
   
                                                                              
                                                                     
                                              
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LunisolarDateCandidate {
    pub target_bjt: HistoricalDateTime,
    pub target_day_number: i64,
    pub structure_winter_solstice_year: i32,
    pub nominal_lunar_year: i32,
    pub month_sequence_index: u8,
    pub month_number: u8,
    pub is_leap_month: bool,
    pub day_number: u8,
    pub month_length_days: u8,
    pub month_start_bjt: HistoricalDateTime,
    pub next_month_start_bjt: HistoricalDateTime,
    pub selected_month_boundaries_safe: bool,
    pub structure_boundaries_safe: bool,
    pub date_assignment_safe: bool,
    pub algorithm_version: String,
    pub status: String,
}

#[derive(Debug, Error)]
pub enum LunisolarError {
    #[error("节气计算失败：{0}")]
    Astronomy(#[from] AstronomyError),
    #[error("月相计算失败：{0}")]
    LunarPhase(#[from] LunarPhaseError),
    #[error("民用历法换算失败：{0}")]
    Calendar(#[from] CalendarError),
    #[error("农历月结构安全边距必须是非负有限秒数")]
    InvalidBoundaryMargin,
    #[error("两个冬至月之间得到 {month_count} 个朔望月，超出 12 或 13 月结构")]
    InvalidMonthCount { month_count: i64 },
    #[error("十三月结构中没有唯一的无中气月")]
    LeapMonthNotUnique,
    #[error("十二月结构出现无中气月，拒绝生成月序")]
    TwelveMonthCycleMissingMajorTerm,
    #[error("无法找到民用日包含目标冬至的朔望月")]
    SolsticeMonthNotFound,
    #[error("一个朔望月候选包含多个中气，当前结构拒绝继续")]
    MultipleMajorTerms,
    #[error("目标北京时间不在所计算的冬至月结构中")]
    TargetDateNotCovered,
    #[error("月结构中没有可用于标记年份的正月")]
    FirstMonthNotFound,
    #[error("朔望月民用日长度为 {days} 天，不是 29 或 30 天，拒绝生成日期")]
    InvalidCivilMonthLength { days: i64 },
}

                                                                            
                                                                             
                                  
   
                                                                           
                                                                             
                                                                              
pub fn lunisolar_month_structure<P: EphemerisProvider + ?Sized>(
    provider: &P,
    winter_solstice_year: i32,
    boundary_margin_seconds: f64,
) -> Result<LunisolarMonthStructure, LunisolarError> {
    if !boundary_margin_seconds.is_finite() || boundary_margin_seconds < 0.0 {
        return Err(LunisolarError::InvalidBoundaryMargin);
    }

    let previous_solstice = solve_solar_term_near_bjt_date(
        provider,
        SolarTerm::Dongzhi,
        winter_solstice_year,
        12,
        21,
        7.0,
    )?;
    let next_solstice = solve_solar_term_near_bjt_date(
        provider,
        SolarTerm::Dongzhi,
        winter_solstice_year + 1,
        12,
        21,
        7.0,
    )?;
    let previous_solstice_bjt = beijing_civil_day_audit(
        previous_solstice.jd_tt,
        previous_solstice.combined_uncertainty_seconds,
        boundary_margin_seconds,
    )?;
    let next_solstice_bjt = beijing_civil_day_audit(
        next_solstice.jd_tt,
        next_solstice.combined_uncertainty_seconds,
        boundary_margin_seconds,
    )?;

    let start_index = lunation_index_containing_civil_day(
        previous_solstice.jd_tt,
        previous_solstice_bjt.day_number,
        boundary_margin_seconds,
    )?;
    let end_index = lunation_index_containing_civil_day(
        next_solstice.jd_tt,
        next_solstice_bjt.day_number,
        boundary_margin_seconds,
    )?;
    let month_count = end_index - start_index;
    if !matches!(month_count, 12 | 13) {
        return Err(LunisolarError::InvalidMonthCount { month_count });
    }

    let mut raw_months = Vec::with_capacity(month_count as usize);
    for offset in 0..month_count {
        let mut month = build_raw_month(provider, start_index + offset, boundary_margin_seconds)?;
        month.sequence_index = offset as u8;
        raw_months.push(month);
    }

    let leap_index = if month_count == 13 {
        let without_term = raw_months
            .iter()
            .enumerate()
            .filter(|(_, month)| !month.contains_major_solar_term)
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if without_term.len() != 1 || without_term[0] == 0 {
            return Err(LunisolarError::LeapMonthNotUnique);
        }
        Some(without_term[0])
    } else {
        if raw_months
            .iter()
            .any(|month| !month.contains_major_solar_term)
        {
            return Err(LunisolarError::TwelveMonthCycleMissingMajorTerm);
        }
        None
    };

    let mut current_number = 11_u8;
    for (index, month) in raw_months.iter_mut().enumerate() {
        if index == 0 {
            month.month_number = 11;
            continue;
        }
        if Some(index) == leap_index {
            month.month_number = current_number;
            month.is_leap = true;
        } else {
            current_number = if current_number == 12 {
                1
            } else {
                current_number + 1
            };
            month.month_number = current_number;
        }
    }
    let leap_month_number = leap_index.map(|index| raw_months[index].month_number);
    let all_boundaries_safe = previous_solstice_bjt.midnight_boundary_safe
        && next_solstice_bjt.midnight_boundary_safe
        && raw_months
            .iter()
            .all(|month| month.term_classification_safe);

    Ok(LunisolarMonthStructure {
        winter_solstice_year,
        previous_winter_solstice: previous_solstice,
        previous_winter_solstice_bjt: previous_solstice_bjt,
        next_winter_solstice: next_solstice,
        next_winter_solstice_bjt: next_solstice_bjt,
        month_count: month_count as u8,
        leap_month_number,
        all_boundaries_safe,
        months: raw_months,
        algorithm_version: LUNISOLAR_MONTH_STRUCTURE_ALGORITHM_VERSION.to_owned(),
        status: LUNISOLAR_MONTH_STRUCTURE_STATUS.to_owned(),
    })
}

                                                                           
                                                                             
                                     
pub fn lunisolar_month_structure_for_civil_date<P: EphemerisProvider + ?Sized>(
    provider: &P,
    target_bjt: HistoricalDateTime,
    boundary_margin_seconds: f64,
) -> Result<LunisolarMonthStructure, LunisolarError> {
    if !boundary_margin_seconds.is_finite() || boundary_margin_seconds < 0.0 {
        return Err(LunisolarError::InvalidBoundaryMargin);
    }
    let target_jd_bjt = julian_day(target_bjt, CalendarSystem::Gregorian)?;
    let target_day_number = (target_jd_bjt + 0.5).floor() as i64;
    let mut first_error = None;

                                                                             
                                                                           
                                                                 
    for cycle_year in [target_bjt.year, target_bjt.year - 1] {
        match lunisolar_month_structure(provider, cycle_year, boundary_margin_seconds) {
            Ok(structure) => {
                if structure.months.iter().any(|month| {
                    civil_day_is_in_month(target_day_number, &month.start_bjt, &month.end_bjt)
                }) {
                    return Ok(structure);
                }
            }
            Err(error) => {
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
    }

    Err(first_error.unwrap_or(LunisolarError::TargetDateNotCovered))
}

                                                                            
                                                                          
                                                                          
pub fn lunisolar_date_candidate<P: EphemerisProvider + ?Sized>(
    provider: &P,
    target_bjt: HistoricalDateTime,
    boundary_margin_seconds: f64,
) -> Result<LunisolarDateCandidate, LunisolarError> {
    let structure =
        lunisolar_month_structure_for_civil_date(provider, target_bjt, boundary_margin_seconds)?;
    lunisolar_date_candidate_from_structure(&structure, target_bjt)
}

                                                                            
                                                                              
                                   
pub fn lunisolar_date_candidate_from_structure(
    structure: &LunisolarMonthStructure,
    target_bjt: HistoricalDateTime,
) -> Result<LunisolarDateCandidate, LunisolarError> {
    let target_jd_bjt = julian_day(target_bjt, CalendarSystem::Gregorian)?;
    let target_day_number = (target_jd_bjt + 0.5).floor() as i64;
    let month = structure
        .months
        .iter()
        .find(|month| civil_day_is_in_month(target_day_number, &month.start_bjt, &month.end_bjt))
        .ok_or(LunisolarError::TargetDateNotCovered)?;
    let month_length = month.end_bjt.day_number - month.start_bjt.day_number;
    if !matches!(month_length, 29 | 30) {
        return Err(LunisolarError::InvalidCivilMonthLength { days: month_length });
    }
    let day_number = target_day_number - month.start_bjt.day_number + 1;
    if !(1..=month_length).contains(&day_number) {
        return Err(LunisolarError::TargetDateNotCovered);
    }
    let first_month = structure
        .months
        .iter()
        .find(|candidate| candidate.month_number == 1 && !candidate.is_leap)
        .ok_or(LunisolarError::FirstMonthNotFound)?;
    let nominal_lunar_year = if month.sequence_index < first_month.sequence_index {
        first_month.start_bjt.local_date_time.year - 1
    } else {
        first_month.start_bjt.local_date_time.year
    };
    let selected_month_boundaries_safe =
        month.start_bjt.midnight_boundary_safe && month.end_bjt.midnight_boundary_safe;
    let structure_boundaries_safe = structure.all_boundaries_safe;

    Ok(LunisolarDateCandidate {
        target_bjt,
        target_day_number,
        structure_winter_solstice_year: structure.winter_solstice_year,
        nominal_lunar_year,
        month_sequence_index: month.sequence_index,
        month_number: month.month_number,
        is_leap_month: month.is_leap,
        day_number: day_number as u8,
        month_length_days: month_length as u8,
        month_start_bjt: month.start_bjt.local_date_time,
        next_month_start_bjt: month.end_bjt.local_date_time,
        selected_month_boundaries_safe,
        structure_boundaries_safe,
        date_assignment_safe: selected_month_boundaries_safe && structure_boundaries_safe,
        algorithm_version: LUNISOLAR_DATE_CANDIDATE_ALGORITHM_VERSION.to_owned(),
        status: LUNISOLAR_DATE_CANDIDATE_STATUS.to_owned(),
    })
}

fn build_raw_month<P: EphemerisProvider + ?Sized>(
    provider: &P,
    lunation_index: i64,
    boundary_margin_seconds: f64,
) -> Result<LunisolarMonthCandidate, LunisolarError> {
    let start = true_lunar_phase(lunation_index, LunarPhase::NewMoon)?;
    let end = true_lunar_phase(lunation_index + 1, LunarPhase::NewMoon)?;
    let start_bjt = beijing_civil_day_audit(
        start.jd_tt,
        start.estimated_formula_uncertainty_seconds,
        boundary_margin_seconds,
    )?;
    let end_bjt = beijing_civil_day_audit(
        end.jd_tt,
        end.estimated_formula_uncertainty_seconds,
        boundary_margin_seconds,
    )?;
    let term_interval = solar_term_class_interval(
        provider,
        start.jd_tt,
        SolarTermClass::Zhongqi,
        boundary_margin_seconds,
    )?;
    let previous_term_bjt = beijing_civil_day_audit(
        term_interval.previous.jd_tt,
        term_interval.previous.combined_uncertainty_seconds,
        boundary_margin_seconds,
    )?;
    let next_term_bjt = beijing_civil_day_audit(
        term_interval.next.jd_tt,
        term_interval.next.combined_uncertainty_seconds,
        boundary_margin_seconds,
    )?;

    let mut contained = Vec::new();
    if civil_day_is_in_month(previous_term_bjt.day_number, &start_bjt, &end_bjt) {
        contained.push((term_interval.previous.clone(), previous_term_bjt.clone()));
    }
    if civil_day_is_in_month(next_term_bjt.day_number, &start_bjt, &end_bjt) {
        contained.push((term_interval.next.clone(), next_term_bjt.clone()));
    }
    if contained.len() > 1 {
        return Err(LunisolarError::MultipleMajorTerms);
    }
    let (major_solar_term, major_solar_term_bjt) = contained
        .into_iter()
        .next()
        .map_or((None, None), |(term, day)| (Some(term), Some(day)));
    let term_classification_safe = start_bjt.midnight_boundary_safe
        && end_bjt.midnight_boundary_safe
        && previous_term_bjt.midnight_boundary_safe
        && next_term_bjt.midnight_boundary_safe;

    Ok(LunisolarMonthCandidate {
        sequence_index: 0,
        month_number: 0,
        is_leap: false,
        start_new_moon: start,
        start_bjt,
        end_new_moon: end,
        end_bjt,
        contains_major_solar_term: major_solar_term.is_some(),
        major_solar_term,
        major_solar_term_bjt,
        term_classification_safe,
    })
}

fn lunation_index_containing_civil_day(
    reference_jd_tt: f64,
    target_day_number: i64,
    boundary_margin_seconds: f64,
) -> Result<i64, LunisolarError> {
    let interval = lunation_interval(reference_jd_tt, boundary_margin_seconds)?;
    let estimate = interval.previous_new_moon.lunation_index;
    for index in (estimate - 1)..=(estimate + 1) {
        let start = true_lunar_phase(index, LunarPhase::NewMoon)?;
        let end = true_lunar_phase(index + 1, LunarPhase::NewMoon)?;
        let start_bjt = beijing_civil_day_audit(
            start.jd_tt,
            start.estimated_formula_uncertainty_seconds,
            boundary_margin_seconds,
        )?;
        let end_bjt = beijing_civil_day_audit(
            end.jd_tt,
            end.estimated_formula_uncertainty_seconds,
            boundary_margin_seconds,
        )?;
        if civil_day_is_in_month(target_day_number, &start_bjt, &end_bjt) {
            return Ok(index);
        }
    }
    Err(LunisolarError::SolsticeMonthNotFound)
}

fn civil_day_is_in_month(
    day_number: i64,
    start: &BeijingCivilDayAudit,
    end: &BeijingCivilDayAudit,
) -> bool {
    start.day_number <= day_number && day_number < end.day_number
}

fn beijing_civil_day_audit(
    jd_tt: f64,
    event_uncertainty_seconds: f64,
    boundary_margin_seconds: f64,
) -> Result<BeijingCivilDayAudit, LunisolarError> {
    if !jd_tt.is_finite()
        || !event_uncertainty_seconds.is_finite()
        || event_uncertainty_seconds < 0.0
        || !boundary_margin_seconds.is_finite()
        || boundary_margin_seconds < 0.0
    {
        return Err(LunisolarError::InvalidBoundaryMargin);
    }
    let approximate_tt = historical_datetime_from_julian_day(jd_tt, CalendarSystem::Gregorian)?;
    let first_delta_t = estimate_delta_t(approximate_tt.year, approximate_tt.month)?;
    let first_jd_bjt = jd_tt - first_delta_t.seconds / 86_400.0 + 8.0 / 24.0;
    let first_bjt = historical_datetime_from_julian_day(first_jd_bjt, CalendarSystem::Gregorian)?;
    let delta_t = estimate_delta_t(first_bjt.year, first_bjt.month)?;
    let jd_bjt = jd_tt - delta_t.seconds / 86_400.0 + 8.0 / 24.0;
    let local_date_time = historical_datetime_from_julian_day(jd_bjt, CalendarSystem::Gregorian)?;
    let seconds_from_midnight = f64::from(local_date_time.hour) * 3_600.0
        + f64::from(local_date_time.minute) * 60.0
        + f64::from(local_date_time.second);
    let distance_to_midnight = seconds_from_midnight.min(86_400.0 - seconds_from_midnight);
    let combined_guard = event_uncertainty_seconds
        + delta_t.estimated_uncertainty_seconds
        + boundary_margin_seconds
        + 0.5;

    Ok(BeijingCivilDayAudit {
        day_number: (jd_bjt + 0.5).floor() as i64,
        local_date_time,
        seconds_from_midnight,
        event_uncertainty_seconds,
        delta_t_uncertainty_seconds: delta_t.estimated_uncertainty_seconds,
        boundary_margin_seconds,
        midnight_boundary_safe: distance_to_midnight > combined_guard,
        conversion_status: "tt_to_ut1_approx_then_utc8_civil_day.v1".to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SpaSolarEphemeris;

    fn leap_month(structure: &LunisolarMonthStructure) -> &LunisolarMonthCandidate {
        structure.months.iter().find(|month| month.is_leap).unwrap()
    }

    #[test]
    fn matches_hko_2023_leap_second_month_start() {
        let structure = lunisolar_month_structure(&SpaSolarEphemeris, 2022, 300.0).unwrap();
        assert_eq!(structure.month_count, 13);
        assert_eq!(structure.leap_month_number, Some(2));
        let leap = leap_month(&structure);
        assert_eq!(leap.month_number, 2);
        assert!(!leap.contains_major_solar_term);
        assert!(leap.term_classification_safe);
        assert_eq!(leap.start_bjt.local_date_time.year, 2023);
        assert_eq!(leap.start_bjt.local_date_time.month, 3);
        assert_eq!(leap.start_bjt.local_date_time.day, 22);
    }

    #[test]
    fn matches_hko_2025_leap_sixth_month_start() {
        let structure = lunisolar_month_structure(&SpaSolarEphemeris, 2024, 300.0).unwrap();
        assert_eq!(structure.month_count, 13);
        assert_eq!(structure.leap_month_number, Some(6));
        assert!(structure.all_boundaries_safe);
        let leap = leap_month(&structure);
        assert_eq!(leap.start_bjt.local_date_time.year, 2025);
        assert_eq!(leap.start_bjt.local_date_time.month, 7);
        assert_eq!(leap.start_bjt.local_date_time.day, 25);
        assert_eq!(structure.months.first().unwrap().month_number, 11);
        assert_eq!(structure.months.last().unwrap().month_number, 10);
    }

    #[test]
    fn twelve_month_cycle_has_no_leap_month() {
        let structure = lunisolar_month_structure(&SpaSolarEphemeris, 2023, 300.0).unwrap();
        assert_eq!(structure.month_count, 12);
        assert_eq!(structure.leap_month_number, None);
        assert!(!structure.months.iter().any(|month| month.is_leap));
        assert_eq!(structure.months.first().unwrap().month_number, 11);
        assert_eq!(structure.months.last().unwrap().month_number, 10);
    }

    #[test]
    fn invalid_margin_fails_closed() {
        assert!(matches!(
            lunisolar_month_structure(&SpaSolarEphemeris, 2024, -1.0),
            Err(LunisolarError::InvalidBoundaryMargin)
        ));
    }

    #[test]
    fn selects_cycle_by_beijing_civil_day_of_winter_solstice() {
        let before = lunisolar_month_structure_for_civil_date(
            &SpaSolarEphemeris,
            HistoricalDateTime {
                year: 2025,
                month: 7,
                day: 25,
                hour: 12,
                minute: 0,
                second: 0,
            },
            300.0,
        )
        .unwrap();
        assert_eq!(before.winter_solstice_year, 2024);
        assert_eq!(before.leap_month_number, Some(6));

        let after = lunisolar_month_structure_for_civil_date(
            &SpaSolarEphemeris,
            HistoricalDateTime {
                year: 2025,
                month: 12,
                day: 25,
                hour: 12,
                minute: 0,
                second: 0,
            },
            300.0,
        )
        .unwrap();
        assert_eq!(after.winter_solstice_year, 2025);
    }

    #[test]
    fn pre_solstice_days_in_month_eleven_select_the_new_cycle() {
        let structure = lunisolar_month_structure(&SpaSolarEphemeris, 2024, 300.0).unwrap();
        let month_eleven = structure.months.first().unwrap();
        assert!(
            month_eleven.start_bjt.day_number < structure.previous_winter_solstice_bjt.day_number
        );
        let mut target = month_eleven.start_bjt.local_date_time;
        target.hour = 12;
        target.minute = 0;
        target.second = 0;
        let selected =
            lunisolar_month_structure_for_civil_date(&SpaSolarEphemeris, target, 300.0).unwrap();
        assert_eq!(selected.winter_solstice_year, 2024);
    }

    #[test]
    fn matches_hko_2024_first_month_and_mid_autumn_dates() {
        let first_day = lunisolar_date_candidate(
            &SpaSolarEphemeris,
            HistoricalDateTime {
                year: 2024,
                month: 2,
                day: 10,
                hour: 12,
                minute: 0,
                second: 0,
            },
            300.0,
        )
        .unwrap();
        assert_eq!(first_day.nominal_lunar_year, 2024);
        assert_eq!(first_day.month_number, 1);
        assert!(!first_day.is_leap_month);
        assert_eq!(first_day.day_number, 1);
        assert!(matches!(first_day.month_length_days, 29 | 30));

        let mid_autumn = lunisolar_date_candidate(
            &SpaSolarEphemeris,
            HistoricalDateTime {
                year: 2024,
                month: 9,
                day: 17,
                hour: 12,
                minute: 0,
                second: 0,
            },
            300.0,
        )
        .unwrap();
        assert_eq!(mid_autumn.nominal_lunar_year, 2024);
        assert_eq!(mid_autumn.month_number, 8);
        assert!(!mid_autumn.is_leap_month);
        assert_eq!(mid_autumn.day_number, 15);
    }

    #[test]
    fn matches_hko_leap_month_first_days() {
        for (year, month, day, expected_month) in [(2023, 3, 22, 2), (2025, 7, 25, 6)] {
            let candidate = lunisolar_date_candidate(
                &SpaSolarEphemeris,
                HistoricalDateTime {
                    year,
                    month,
                    day,
                    hour: 12,
                    minute: 0,
                    second: 0,
                },
                300.0,
            )
            .unwrap();
            assert_eq!(candidate.nominal_lunar_year, year);
            assert_eq!(candidate.month_number, expected_month);
            assert!(candidate.is_leap_month);
            assert_eq!(candidate.day_number, 1);
            if year == 2023 {
                assert!(!candidate.structure_boundaries_safe);
                assert!(!candidate.date_assignment_safe);
            } else {
                assert!(candidate.structure_boundaries_safe);
                assert!(candidate.date_assignment_safe);
            }
        }
    }

    #[test]
    fn time_of_day_does_not_change_civil_lunar_date() {
        let mut early = HistoricalDateTime {
            year: 2024,
            month: 2,
            day: 24,
            hour: 0,
            minute: 0,
            second: 0,
        };
        let early_candidate = lunisolar_date_candidate(&SpaSolarEphemeris, early, 300.0).unwrap();
        early.hour = 23;
        early.minute = 59;
        early.second = 59;
        let late_candidate = lunisolar_date_candidate(&SpaSolarEphemeris, early, 300.0).unwrap();
        assert_eq!(early_candidate.month_number, late_candidate.month_number);
        assert_eq!(early_candidate.day_number, late_candidate.day_number);
        assert_eq!(early_candidate.day_number, 15);
    }
}
