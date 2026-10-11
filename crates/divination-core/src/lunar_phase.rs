                                                                          
                                                                            
                                                                 

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const SYNODIC_MONTH_DAYS: f64 = 29.530_588_853;
pub const LUNAR_PHASE_ALGORITHM_VERSION: &str = "jiuheng.lunar_phase.meeus_true_phase.v1";
pub const LUNATION_INTERVAL_ALGORITHM_VERSION: &str = "jiuheng.lunation_interval.true_new_moon.v1";

const PHASE_EPOCH_JD_TT: f64 = 2_451_550.097_65;
const MIN_SUPPORTED_JD_TT: f64 = 990_557.0;
                                                                             
                                                                      
const MAX_SUPPORTED_JD_TT: f64 = 2_817_400.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LunarPhase {
    NewMoon,
    FullMoon,
}

impl LunarPhase {
    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::NewMoon => "朔",
            Self::FullMoon => "望",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LunarPhaseEvent {
    pub phase: LunarPhase,
                                                                             
                                                                  
    pub lunation_index: i64,
    pub jd_tt: f64,
    pub estimated_formula_uncertainty_seconds: f64,
    pub algorithm_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LunationInterval {
    pub previous_new_moon: LunarPhaseEvent,
    pub full_moon: LunarPhaseEvent,
    pub next_new_moon: LunarPhaseEvent,
    pub seconds_since_previous_new_moon: f64,
    pub seconds_until_next_new_moon: f64,
    pub boundary_margin_seconds: f64,
    pub new_moon_boundary_safe: bool,
    pub algorithm_version: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum LunarPhaseError {
    #[error("月相计算输入必须是有限数值")]
    NonFiniteInput,
    #[error("儒略日 {jd_tt} 超出月相算法支持范围")]
    DateOutsideAlgorithmRange { jd_tt: f64 },
    #[error("朔界安全边距必须是非负有限秒数")]
    InvalidBoundaryMargin,
    #[error("未能建立包围目标时刻的朔望月区间")]
    IntervalNotFound,
}

                                                                             
   
                                                                      
                                                                               
                                                           
pub fn true_lunar_phase(
    lunation_index: i64,
    phase: LunarPhase,
) -> Result<LunarPhaseEvent, LunarPhaseError> {
    let phase_fraction = match phase {
        LunarPhase::NewMoon => 0.0,
        LunarPhase::FullMoon => 0.5,
    };
    let k = lunation_index as f64 + phase_fraction;
    let t = k / 1_236.85;
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let jde_mean = PHASE_EPOCH_JD_TT + SYNODIC_MONTH_DAYS * k + 0.000_133_7 * t2
        - 0.000_000_150 * t3
        + 0.000_000_000_73 * t4;
    if !(MIN_SUPPORTED_JD_TT..=MAX_SUPPORTED_JD_TT).contains(&jde_mean) {
        return Err(LunarPhaseError::DateOutsideAlgorithmRange { jd_tt: jde_mean });
    }

    let eccentricity = 1.0 - 0.002_516 * t - 0.000_007_4 * t2;
    let solar_anomaly = radians(2.5534 + 29.105_356_70 * k - 0.000_001_4 * t2 - 0.000_000_11 * t3);
    let lunar_anomaly = radians(
        201.5643 + 385.816_935_28 * k + 0.010_758_2 * t2 + 0.000_012_38 * t3 - 0.000_000_058 * t4,
    );
    let argument_latitude = radians(
        160.7108 + 390.670_502_84 * k - 0.001_611_8 * t2 - 0.000_002_27 * t3 + 0.000_000_011 * t4,
    );
    let ascending_node =
        radians(124.7746 - 1.563_755_88 * k + 0.002_067_2 * t2 + 0.000_002_15 * t3);

    let phase_correction = principal_phase_correction(
        phase,
        eccentricity,
        solar_anomaly,
        lunar_anomaly,
        argument_latitude,
        ascending_node,
    );
    let planetary_correction = planetary_phase_correction(k, t2);
    let jd_tt = jde_mean + phase_correction + planetary_correction;

    Ok(LunarPhaseEvent {
        phase,
        lunation_index,
        jd_tt,
        estimated_formula_uncertainty_seconds: formula_uncertainty_seconds(jd_tt),
        algorithm_version: LUNAR_PHASE_ALGORITHM_VERSION.to_owned(),
    })
}

                                                                             
                                                                              
                              
pub fn lunation_interval(
    jd_tt: f64,
    boundary_margin_seconds: f64,
) -> Result<LunationInterval, LunarPhaseError> {
    if !jd_tt.is_finite() {
        return Err(LunarPhaseError::NonFiniteInput);
    }
    if !(MIN_SUPPORTED_JD_TT..=MAX_SUPPORTED_JD_TT).contains(&jd_tt) {
        return Err(LunarPhaseError::DateOutsideAlgorithmRange { jd_tt });
    }
    if !boundary_margin_seconds.is_finite() || boundary_margin_seconds < 0.0 {
        return Err(LunarPhaseError::InvalidBoundaryMargin);
    }

    let estimated_index = ((jd_tt - PHASE_EPOCH_JD_TT) / SYNODIC_MONTH_DAYS).floor() as i64;
    let mut candidates = Vec::with_capacity(7);
    for index in (estimated_index - 3)..=(estimated_index + 3) {
        candidates.push(true_lunar_phase(index, LunarPhase::NewMoon)?);
    }
    candidates.sort_by(|left, right| left.jd_tt.total_cmp(&right.jd_tt));

    let previous = candidates
        .iter()
        .rfind(|event| event.jd_tt <= jd_tt)
        .cloned()
        .ok_or(LunarPhaseError::IntervalNotFound)?;
    let next = candidates
        .iter()
        .find(|event| event.jd_tt > jd_tt)
        .cloned()
        .ok_or(LunarPhaseError::IntervalNotFound)?;
    if next.lunation_index != previous.lunation_index + 1 {
        return Err(LunarPhaseError::IntervalNotFound);
    }
    let full_moon = true_lunar_phase(previous.lunation_index, LunarPhase::FullMoon)?;
    if !(previous.jd_tt < full_moon.jd_tt && full_moon.jd_tt < next.jd_tt) {
        return Err(LunarPhaseError::IntervalNotFound);
    }

    let seconds_since_previous = (jd_tt - previous.jd_tt) * 86_400.0;
    let seconds_until_next = (next.jd_tt - jd_tt) * 86_400.0;
    let previous_guard = previous.estimated_formula_uncertainty_seconds + boundary_margin_seconds;
    let next_guard = next.estimated_formula_uncertainty_seconds + boundary_margin_seconds;

    Ok(LunationInterval {
        previous_new_moon: previous,
        full_moon,
        next_new_moon: next,
        seconds_since_previous_new_moon: seconds_since_previous,
        seconds_until_next_new_moon: seconds_until_next,
        boundary_margin_seconds,
        new_moon_boundary_safe: seconds_since_previous > previous_guard
            && seconds_until_next > next_guard,
        algorithm_version: LUNATION_INTERVAL_ALGORITHM_VERSION.to_owned(),
        status: "astronomical_phase_audit_not_chinese_calendar.v1".to_owned(),
    })
}

fn principal_phase_correction(
    phase: LunarPhase,
    e: f64,
    m: f64,
    mp: f64,
    f: f64,
    omega: f64,
) -> f64 {
    let e2 = e * e;
    let leading_lunar = match phase {
        LunarPhase::NewMoon => -0.407_20,
        LunarPhase::FullMoon => -0.406_14,
    };
    let leading_solar = match phase {
        LunarPhase::NewMoon => 0.172_41,
        LunarPhase::FullMoon => 0.173_02,
    };
    let second_lunar = match phase {
        LunarPhase::NewMoon => 0.016_08,
        LunarPhase::FullMoon => 0.016_14,
    };
    let second_latitude = match phase {
        LunarPhase::NewMoon => 0.010_39,
        LunarPhase::FullMoon => 0.010_43,
    };
    let mixed_difference = match phase {
        LunarPhase::NewMoon => 0.007_39,
        LunarPhase::FullMoon => 0.007_34,
    };
    let mixed_sum = match phase {
        LunarPhase::NewMoon => -0.005_14,
        LunarPhase::FullMoon => -0.005_15,
    };
    let second_solar = match phase {
        LunarPhase::NewMoon => 0.002_08,
        LunarPhase::FullMoon => 0.002_09,
    };

    leading_lunar * mp.sin()
        + leading_solar * e * m.sin()
        + second_lunar * (2.0 * mp).sin()
        + second_latitude * (2.0 * f).sin()
        + mixed_difference * e * (mp - m).sin()
        + mixed_sum * e * (mp + m).sin()
        + second_solar * e2 * (2.0 * m).sin()
        - 0.001_11 * (mp - 2.0 * f).sin()
        - 0.000_57 * (mp + 2.0 * f).sin()
        + 0.000_56 * e * (2.0 * mp + m).sin()
        - 0.000_42 * (3.0 * mp).sin()
        + 0.000_42 * e * (m + 2.0 * f).sin()
        + 0.000_38 * e * (m - 2.0 * f).sin()
        - 0.000_24 * e * (2.0 * mp - m).sin()
        - 0.000_17 * omega.sin()
        - 0.000_07 * (mp + 2.0 * m).sin()
        + 0.000_04 * (2.0 * mp - 2.0 * f).sin()
        + 0.000_04 * (3.0 * m).sin()
        + 0.000_03 * (mp + m - 2.0 * f).sin()
        + 0.000_03 * (2.0 * mp + 2.0 * f).sin()
        - 0.000_03 * (mp + m + 2.0 * f).sin()
        + 0.000_03 * (mp - m + 2.0 * f).sin()
        - 0.000_02 * (mp - m - 2.0 * f).sin()
        - 0.000_02 * (3.0 * mp + m).sin()
        + 0.000_02 * (4.0 * mp).sin()
}

fn planetary_phase_correction(k: f64, t2: f64) -> f64 {
    let arguments = [
        (0.000_325, 299.77 + 0.107_408 * k - 0.009_173 * t2),
        (0.000_165, 251.88 + 0.016_321 * k),
        (0.000_164, 251.83 + 26.651_886 * k),
        (0.000_126, 349.42 + 36.412_478 * k),
        (0.000_110, 84.66 + 18.206_239 * k),
        (0.000_062, 141.74 + 53.303_771 * k),
        (0.000_060, 207.14 + 2.453_732 * k),
        (0.000_056, 154.84 + 7.306_860 * k),
        (0.000_047, 34.52 + 27.261_239 * k),
        (0.000_042, 207.19 + 0.121_824 * k),
        (0.000_040, 291.34 + 1.844_379 * k),
        (0.000_037, 161.72 + 24.198_154 * k),
        (0.000_035, 239.56 + 25.513_099 * k),
        (0.000_023, 331.55 + 3.592_518 * k),
    ];
    arguments
        .iter()
        .map(|(coefficient, degrees)| coefficient * radians(*degrees).sin())
        .sum()
}

fn formula_uncertainty_seconds(jd_tt: f64) -> f64 {
    let centuries_from_j2000 = ((jd_tt - 2_451_545.0) / 36_525.0).abs();
    if centuries_from_j2000 <= 1.0 {
        120.0
    } else if centuries_from_j2000 <= 5.0 {
        600.0
    } else if centuries_from_j2000 <= 15.0 {
        1_800.0
    } else {
        3_600.0
    }
}

fn radians(degrees: f64) -> f64 {
    degrees.rem_euclid(360.0).to_radians()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{estimate_delta_t, julian_day_tt_from_ut1};
    use chrono::{FixedOffset, TimeZone};

    fn bjt_to_jd_tt(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> f64 {
        let bjt = FixedOffset::east_opt(8 * 3_600)
            .unwrap()
            .with_ymd_and_hms(year, month, day, hour, minute, 0)
            .unwrap();
        let jd_ut1 = bjt.timestamp_millis() as f64 / 86_400_000.0 + 2_440_587.5;
        let delta_t = estimate_delta_t(year, month as u8).unwrap();
        julian_day_tt_from_ut1(jd_ut1, delta_t.seconds).unwrap()
    }

    fn nearest_phase(target_jd_tt: f64, phase: LunarPhase) -> LunarPhaseEvent {
        let estimate = ((target_jd_tt - PHASE_EPOCH_JD_TT) / SYNODIC_MONTH_DAYS).round() as i64;
        ((estimate - 2)..=(estimate + 2))
            .map(|index| true_lunar_phase(index, phase).unwrap())
            .min_by(|left, right| {
                (left.jd_tt - target_jd_tt)
                    .abs()
                    .total_cmp(&(right.jd_tt - target_jd_tt).abs())
            })
            .unwrap()
    }

    #[test]
    fn matches_hko_2024_new_moon_minute_anchors() {
        for target in [
            bjt_to_jd_tt(2024, 1, 11, 19, 57),
            bjt_to_jd_tt(2024, 8, 4, 19, 13),
            bjt_to_jd_tt(2026, 1, 19, 3, 52),
        ] {
            let event = nearest_phase(target, LunarPhase::NewMoon);
            assert!((event.jd_tt - target).abs() * 86_400.0 <= 120.0);
            assert_eq!(event.algorithm_version, LUNAR_PHASE_ALGORITHM_VERSION);
        }
    }

    #[test]
    fn matches_hko_2024_full_moon_minute_anchors() {
        for target in [
            bjt_to_jd_tt(2024, 1, 26, 1, 54),
            bjt_to_jd_tt(2024, 8, 20, 2, 26),
            bjt_to_jd_tt(2024, 9, 18, 10, 34),
        ] {
            let event = nearest_phase(target, LunarPhase::FullMoon);
            assert!((event.jd_tt - target).abs() * 86_400.0 <= 120.0);
        }
    }

    #[test]
    fn interval_is_ordered_and_rejects_an_unsafe_exact_boundary() {
        let target = bjt_to_jd_tt(2024, 1, 20, 12, 0);
        let interval = lunation_interval(target, 300.0).unwrap();
        assert!(interval.previous_new_moon.jd_tt < interval.full_moon.jd_tt);
        assert!(interval.full_moon.jd_tt < interval.next_new_moon.jd_tt);
        assert!(interval.new_moon_boundary_safe);

        let boundary = lunation_interval(interval.previous_new_moon.jd_tt, 300.0).unwrap();
        assert!(!boundary.new_moon_boundary_safe);
    }

    #[test]
    fn interval_input_and_range_are_fail_closed() {
        assert_eq!(
            lunation_interval(f64::NAN, 300.0),
            Err(LunarPhaseError::NonFiniteInput)
        );
        assert_eq!(
            lunation_interval(2_451_545.0, -1.0),
            Err(LunarPhaseError::InvalidBoundaryMargin)
        );
        assert!(matches!(
            lunation_interval(900_000.0, 300.0),
            Err(LunarPhaseError::DateOutsideAlgorithmRange { .. })
        ));
    }
}
