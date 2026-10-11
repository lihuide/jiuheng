use crate::{estimate_delta_t, julian_day_tt_from_ut1};
use chrono::{FixedOffset, TimeZone};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AstronomyPrecision {
                                                                       
    FoundationApproximation,
                                                                     
    SpaReferenceVectorMatched,
                                                                           
    PracticalVerified,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EclipticPosition {
    pub longitude_deg: f64,
    pub latitude_deg: f64,
    pub distance_au: f64,
    pub precision: AstronomyPrecision,
    pub estimated_error_arcmin: f64,
    pub algorithm_version: String,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum AstronomyError {
    #[error("天文计算输入必须是有限数值")]
    NonFiniteInput,
    #[error("儒略日 {jd_tt} 超出算法适用范围")]
    DateOutsideAlgorithmRange { jd_tt: f64 },
    #[error("节气搜索区间无效")]
    InvalidSearchBracket,
    #[error(
        "搜索区间没有跨越目标黄经（起点差 {start_delta_deg:.6}°，终点差 {end_delta_deg:.6}°）"
    )]
    TargetNotBracketed {
        start_delta_deg: f64,
        end_delta_deg: f64,
    },
    #[error("节气搜索没有在迭代上限内收敛")]
    SolverDidNotConverge,
    #[error("节气边界安全边距必须是非负有限秒数")]
    InvalidBoundaryMargin,
    #[error("节气参考日期无效")]
    InvalidReferenceDate,
}

pub const SOLAR_TERM_INTERVAL_ALGORITHM_VERSION: &str =
    "jiuheng.solar_term_interval.bidirectional_bisection.v1";
pub const SOLAR_TERM_CLASS_INTERVAL_ALGORITHM_VERSION: &str =
    "jiuheng.solar_term_class_interval.jie_zhongqi.v1";

pub trait EphemerisProvider: Send + Sync {
    fn sun_geocentric_ecliptic(&self, jd_tt: f64) -> Result<EclipticPosition, AstronomyError>;
}

                                                                            
                                                                         
#[derive(Debug, Default)]
pub struct FoundationEphemeris;

impl EphemerisProvider for FoundationEphemeris {
    fn sun_geocentric_ecliptic(&self, jd_tt: f64) -> Result<EclipticPosition, AstronomyError> {
        if !jd_tt.is_finite() {
            return Err(AstronomyError::NonFiniteInput);
        }

        let t = (jd_tt - 2_451_545.0) / 36_525.0;
        let mean_longitude = normalize_deg(280.46646 + 36_000.769_83 * t + 0.0003032 * t * t);
        let mean_anomaly = normalize_deg(357.52911 + 35_999.050_29 * t - 0.0001537 * t * t);
        let anomaly_rad = mean_anomaly.to_radians();
        let equation_center = (1.914602 - 0.004817 * t - 0.000014 * t * t) * anomaly_rad.sin()
            + (0.019993 - 0.000101 * t) * (2.0 * anomaly_rad).sin()
            + 0.000289 * (3.0 * anomaly_rad).sin();
        let true_longitude = mean_longitude + equation_center;
        let omega = (125.04 - 1934.136 * t).to_radians();
        let apparent_longitude = normalize_deg(true_longitude - 0.00569 - 0.00478 * omega.sin());
        let eccentricity = 0.016708634 - t * (0.000042037 + 0.0000001267 * t);
        let true_anomaly = mean_anomaly + equation_center;
        let distance_au = (1.000001018 * (1.0 - eccentricity * eccentricity))
            / (1.0 + eccentricity * true_anomaly.to_radians().cos());

        Ok(EclipticPosition {
            longitude_deg: apparent_longitude,
            latitude_deg: 0.0,
            distance_au,
            precision: AstronomyPrecision::FoundationApproximation,
            estimated_error_arcmin: 1.0,
            algorithm_version: "jiuheng.sun.foundation.v1".to_owned(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolarTerm {
    Chunfen,
    Qingming,
    Guyu,
    Lixia,
    Xiaoman,
    Mangzhong,
    Xiazhi,
    Xiaoshu,
    Dashu,
    Liqiu,
    Chushu,
    Bailu,
    Qiufen,
    Hanlu,
    Shuangjiang,
    Lidong,
    Xiaoxue,
    Daxue,
    Dongzhi,
    Xiaohan,
    Dahan,
    Lichun,
    Yushui,
    Jingzhe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolarTermClass {
    Jie,
    Zhongqi,
}

impl SolarTermClass {
    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Jie => "节",
            Self::Zhongqi => "中气",
        }
    }
}

impl SolarTerm {
    pub const ALL_LONGITUDE_ORDER: [Self; 24] = [
        Self::Chunfen,
        Self::Qingming,
        Self::Guyu,
        Self::Lixia,
        Self::Xiaoman,
        Self::Mangzhong,
        Self::Xiazhi,
        Self::Xiaoshu,
        Self::Dashu,
        Self::Liqiu,
        Self::Chushu,
        Self::Bailu,
        Self::Qiufen,
        Self::Hanlu,
        Self::Shuangjiang,
        Self::Lidong,
        Self::Xiaoxue,
        Self::Daxue,
        Self::Dongzhi,
        Self::Xiaohan,
        Self::Dahan,
        Self::Lichun,
        Self::Yushui,
        Self::Jingzhe,
    ];

    pub fn longitude_deg(self) -> f64 {
        Self::ALL_LONGITUDE_ORDER
            .iter()
            .position(|term| *term == self)
            .expect("solar term is in the complete table") as f64
            * 15.0
    }

    pub fn zh_name(self) -> &'static str {
        match self {
            Self::Chunfen => "春分",
            Self::Qingming => "清明",
            Self::Guyu => "谷雨",
            Self::Lixia => "立夏",
            Self::Xiaoman => "小满",
            Self::Mangzhong => "芒种",
            Self::Xiazhi => "夏至",
            Self::Xiaoshu => "小暑",
            Self::Dashu => "大暑",
            Self::Liqiu => "立秋",
            Self::Chushu => "处暑",
            Self::Bailu => "白露",
            Self::Qiufen => "秋分",
            Self::Hanlu => "寒露",
            Self::Shuangjiang => "霜降",
            Self::Lidong => "立冬",
            Self::Xiaoxue => "小雪",
            Self::Daxue => "大雪",
            Self::Dongzhi => "冬至",
            Self::Xiaohan => "小寒",
            Self::Dahan => "大寒",
            Self::Lichun => "立春",
            Self::Yushui => "雨水",
            Self::Jingzhe => "惊蛰",
        }
    }

    pub fn class(self) -> SolarTermClass {
        let index = Self::ALL_LONGITUDE_ORDER
            .iter()
            .position(|term| *term == self)
            .expect("solar term is in the complete table");
        if index % 2 == 0 {
            SolarTermClass::Zhongqi
        } else {
            SolarTermClass::Jie
        }
    }

    fn from_longitude_index(index: usize) -> Self {
        Self::ALL_LONGITUDE_ORDER[index % Self::ALL_LONGITUDE_ORDER.len()]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolarTermEvent {
    pub term: SolarTerm,
    pub jd_tt: f64,
    pub final_bracket_seconds: f64,
    pub iterations: u32,
    pub solver_uncertainty_seconds: f64,
    pub ephemeris_time_uncertainty_seconds: f64,
    pub combined_uncertainty_seconds: f64,
    pub precision: AstronomyPrecision,
    pub algorithm_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolarTermInterval {
    pub previous: SolarTermEvent,
    pub next: SolarTermEvent,
    pub seconds_since_previous: f64,
    pub seconds_until_next: f64,
    pub boundary_margin_seconds: f64,
    pub boundary_safe: bool,
    pub algorithm_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolarTermClassInterval {
    pub class: SolarTermClass,
    pub previous: SolarTermEvent,
    pub next: SolarTermEvent,
    pub seconds_since_previous: f64,
    pub seconds_until_next: f64,
    pub boundary_margin_seconds: f64,
    pub boundary_safe: bool,
    pub algorithm_version: String,
}

                                                                 
   
                                                                              
                                                                           
                                         
pub fn solve_solar_term<P: EphemerisProvider + ?Sized>(
    provider: &P,
    term: SolarTerm,
    start_jd_tt: f64,
    end_jd_tt: f64,
    tolerance_seconds: f64,
) -> Result<SolarTermEvent, AstronomyError> {
    if !start_jd_tt.is_finite()
        || !end_jd_tt.is_finite()
        || !tolerance_seconds.is_finite()
        || start_jd_tt >= end_jd_tt
        || end_jd_tt - start_jd_tt > 60.0
        || tolerance_seconds <= 0.0
    {
        return Err(AstronomyError::InvalidSearchBracket);
    }

    let target = term.longitude_deg();
    let start_position = provider.sun_geocentric_ecliptic(start_jd_tt)?;
    let end_position = provider.sun_geocentric_ecliptic(end_jd_tt)?;
    let start_delta = signed_angular_difference(start_position.longitude_deg, target);
    let mut end_delta = signed_angular_difference(end_position.longitude_deg, target);

                                                                            
    while end_delta < start_delta {
        end_delta += 360.0;
    }
    if start_delta > 0.0 || end_delta < 0.0 || end_delta - start_delta > 90.0 {
        return Err(AstronomyError::TargetNotBracketed {
            start_delta_deg: start_delta,
            end_delta_deg: end_delta,
        });
    }

    let mut low = start_jd_tt;
    let mut high = end_jd_tt;
    let mut iterations = 0_u32;
    while (high - low) * 86_400.0 > tolerance_seconds {
        if iterations >= 80 {
            return Err(AstronomyError::SolverDidNotConverge);
        }
        let mid = (low + high) / 2.0;
        let mid_position = provider.sun_geocentric_ecliptic(mid)?;
        let mut mid_delta = signed_angular_difference(mid_position.longitude_deg, target);
        while mid_delta < start_delta {
            mid_delta += 360.0;
        }
        if mid_delta < 0.0 {
            low = mid;
        } else {
            high = mid;
        }
        iterations += 1;
    }

    let jd_tt = (low + high) / 2.0;
    let position = provider.sun_geocentric_ecliptic(jd_tt)?;
    let bracket_seconds = (high - low) * 86_400.0;
    let solver_uncertainty_seconds = bracket_seconds / 2.0;
    let days = end_jd_tt - start_jd_tt;
    let forward_motion_deg = end_delta - start_delta;
    let angular_speed_deg_per_day = forward_motion_deg / days;
    let ephemeris_time_uncertainty_seconds = if angular_speed_deg_per_day > 0.0 {
        position.estimated_error_arcmin / 60.0 / angular_speed_deg_per_day * 86_400.0
    } else {
        f64::INFINITY
    };
    let combined_uncertainty_seconds =
        solver_uncertainty_seconds + ephemeris_time_uncertainty_seconds;

    Ok(SolarTermEvent {
        term,
        jd_tt,
        final_bracket_seconds: bracket_seconds,
        iterations,
        solver_uncertainty_seconds,
        ephemeris_time_uncertainty_seconds,
        combined_uncertainty_seconds,
        precision: position.precision,
        algorithm_version: position.algorithm_version,
    })
}

                                                                            
                                                            
pub fn solve_solar_term_near_bjt_date<P: EphemerisProvider + ?Sized>(
    provider: &P,
    term: SolarTerm,
    year: i32,
    month: u32,
    day: u32,
    half_span_days: f64,
) -> Result<SolarTermEvent, AstronomyError> {
    if !half_span_days.is_finite() || half_span_days <= 0.0 || half_span_days > 30.0 {
        return Err(AstronomyError::InvalidSearchBracket);
    }
    let zone = FixedOffset::east_opt(8 * 3_600).ok_or(AstronomyError::InvalidReferenceDate)?;
    let reference = zone
        .with_ymd_and_hms(year, month, day, 12, 0, 0)
        .single()
        .ok_or(AstronomyError::InvalidReferenceDate)?;
    let jd_ut1_approx = reference.timestamp_millis() as f64 / 86_400_000.0 + 2_440_587.5;
    let delta_t =
        estimate_delta_t(year, month as u8).map_err(|_| AstronomyError::InvalidReferenceDate)?;
    let reference_jd_tt = julian_day_tt_from_ut1(jd_ut1_approx, delta_t.seconds)
        .map_err(|_| AstronomyError::InvalidReferenceDate)?;
    solve_solar_term(
        provider,
        term,
        reference_jd_tt - half_span_days,
        reference_jd_tt + half_span_days,
        0.1,
    )
}

                                                       
pub fn next_solar_term<P: EphemerisProvider + ?Sized>(
    provider: &P,
    jd_tt: f64,
) -> Result<SolarTermEvent, AstronomyError> {
    let position = provider.sun_geocentric_ecliptic(jd_tt)?;
    let next_index = ((position.longitude_deg / 15.0).floor() as usize + 1) % 24;
    let term = SolarTerm::from_longitude_index(next_index);
    let forward_angle = (term.longitude_deg() - position.longitude_deg).rem_euclid(360.0);
    let estimated_days = forward_angle / 0.985_647_36;
    let start = jd_tt + (estimated_days - 3.0).max(0.000_001);
    let end = jd_tt + estimated_days + 3.0;
    solve_solar_term(provider, term, start, end, 0.1)
}

                                                                          
                                                                                
                                                                       
pub fn previous_solar_term<P: EphemerisProvider + ?Sized>(
    provider: &P,
    jd_tt: f64,
) -> Result<SolarTermEvent, AstronomyError> {
    let position = provider.sun_geocentric_ecliptic(jd_tt)?;
    let previous_index = (position.longitude_deg / 15.0).floor() as usize % 24;
    let term = SolarTerm::from_longitude_index(previous_index);
    let backward_angle = (position.longitude_deg - term.longitude_deg()).rem_euclid(360.0);
    let estimated_days = backward_angle / 0.985_647_36;
    let start = jd_tt - estimated_days - 3.0;
    let end = jd_tt - (estimated_days - 3.0).max(0.0);
    solve_solar_term(provider, term, start, end, 0.1)
}

                                                                         
                                                                              
                                                                           
pub fn solar_term_interval<P: EphemerisProvider + ?Sized>(
    provider: &P,
    jd_tt: f64,
    boundary_margin_seconds: f64,
) -> Result<SolarTermInterval, AstronomyError> {
    if !boundary_margin_seconds.is_finite() || boundary_margin_seconds < 0.0 {
        return Err(AstronomyError::InvalidBoundaryMargin);
    }
    let previous = previous_solar_term(provider, jd_tt)?;
    let next = next_solar_term(provider, jd_tt)?;
    let seconds_since_previous = (jd_tt - previous.jd_tt) * 86_400.0;
    let seconds_until_next = (next.jd_tt - jd_tt) * 86_400.0;
    let boundary_safe = seconds_since_previous
        > previous.combined_uncertainty_seconds + boundary_margin_seconds
        && seconds_until_next > next.combined_uncertainty_seconds + boundary_margin_seconds;

    Ok(SolarTermInterval {
        previous,
        next,
        seconds_since_previous,
        seconds_until_next,
        boundary_margin_seconds,
        boundary_safe,
        algorithm_version: SOLAR_TERM_INTERVAL_ALGORITHM_VERSION.to_owned(),
    })
}

pub fn previous_solar_term_of_class<P: EphemerisProvider + ?Sized>(
    provider: &P,
    jd_tt: f64,
    class: SolarTermClass,
) -> Result<SolarTermEvent, AstronomyError> {
    let position = provider.sun_geocentric_ecliptic(jd_tt)?;
    let mut index = (position.longitude_deg / 15.0).floor() as usize % 24;
    while SolarTerm::from_longitude_index(index).class() != class {
        index = (index + 23) % 24;
    }
    let term = SolarTerm::from_longitude_index(index);
    let backward_angle = (position.longitude_deg - term.longitude_deg()).rem_euclid(360.0);
    let estimated_days = backward_angle / 0.985_647_36;
    let start = jd_tt - estimated_days - 3.0;
    let end = jd_tt - (estimated_days - 3.0).max(0.0);
    solve_solar_term(provider, term, start, end, 0.1)
}

pub fn next_solar_term_of_class<P: EphemerisProvider + ?Sized>(
    provider: &P,
    jd_tt: f64,
    class: SolarTermClass,
) -> Result<SolarTermEvent, AstronomyError> {
    let position = provider.sun_geocentric_ecliptic(jd_tt)?;
    let mut index = ((position.longitude_deg / 15.0).floor() as usize + 1) % 24;
    while SolarTerm::from_longitude_index(index).class() != class {
        index = (index + 1) % 24;
    }
    let term = SolarTerm::from_longitude_index(index);
    let forward_angle = (term.longitude_deg() - position.longitude_deg).rem_euclid(360.0);
    let estimated_days = forward_angle / 0.985_647_36;
    let start = jd_tt + (estimated_days - 3.0).max(0.000_001);
    let end = jd_tt + estimated_days + 3.0;
    solve_solar_term(provider, term, start, end, 0.1)
}

pub fn solar_term_class_interval<P: EphemerisProvider + ?Sized>(
    provider: &P,
    jd_tt: f64,
    class: SolarTermClass,
    boundary_margin_seconds: f64,
) -> Result<SolarTermClassInterval, AstronomyError> {
    if !boundary_margin_seconds.is_finite() || boundary_margin_seconds < 0.0 {
        return Err(AstronomyError::InvalidBoundaryMargin);
    }
    let previous = previous_solar_term_of_class(provider, jd_tt, class)?;
    let next = next_solar_term_of_class(provider, jd_tt, class)?;
    let seconds_since_previous = (jd_tt - previous.jd_tt) * 86_400.0;
    let seconds_until_next = (next.jd_tt - jd_tt) * 86_400.0;
    let boundary_safe = seconds_since_previous
        > previous.combined_uncertainty_seconds + boundary_margin_seconds
        && seconds_until_next > next.combined_uncertainty_seconds + boundary_margin_seconds;

    Ok(SolarTermClassInterval {
        class,
        previous,
        next,
        seconds_since_previous,
        seconds_until_next,
        boundary_margin_seconds,
        boundary_safe,
        algorithm_version: SOLAR_TERM_CLASS_INTERVAL_ALGORITHM_VERSION.to_owned(),
    })
}

pub fn is_boundary_safe(value_deg: f64, boundary_step_deg: f64, error_arcmin: f64) -> bool {
    if !value_deg.is_finite()
        || !boundary_step_deg.is_finite()
        || !error_arcmin.is_finite()
        || boundary_step_deg <= 0.0
        || error_arcmin < 0.0
    {
        return false;
    }
    let remainder = value_deg.rem_euclid(boundary_step_deg);
    let distance = remainder.min(boundary_step_deg - remainder);
    distance > error_arcmin / 60.0
}

pub(crate) fn normalize_deg(value: f64) -> f64 {
    value.rem_euclid(360.0)
}

fn signed_angular_difference(value_deg: f64, target_deg: f64) -> f64 {
    (value_deg - target_deg + 180.0).rem_euclid(360.0) - 180.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{estimate_delta_t, julian_day_tt_from_ut1, SpaSolarEphemeris};
    use chrono::{FixedOffset, TimeZone};

    fn jd_tt_from_bjt(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> f64 {
        let zone = FixedOffset::east_opt(8 * 3_600).unwrap();
        let time = zone
            .with_ymd_and_hms(year, month, day, hour, minute, 0)
            .unwrap();
        let jd_ut1_approx = time.timestamp_millis() as f64 / 86_400_000.0 + 2_440_587.5;
        let delta_t = estimate_delta_t(year, month as u8).unwrap();
        julian_day_tt_from_ut1(jd_ut1_approx, delta_t.seconds).unwrap()
    }

    #[test]
    fn foundation_sun_is_near_j2000_reference_region() {
        let pos = FoundationEphemeris
            .sun_geocentric_ecliptic(2_451_545.0)
            .unwrap();
        assert!((pos.longitude_deg - 280.37).abs() < 0.2);
        assert_eq!(pos.precision, AstronomyPrecision::FoundationApproximation);
    }

    #[test]
    fn boundary_guard_rejects_values_inside_error_band_and_invalid_inputs() {
        assert!(!is_boundary_safe(29.99, 30.0, 1.0));
        assert!(is_boundary_safe(29.0, 30.0, 1.0));
        assert!(!is_boundary_safe(f64::NAN, 30.0, 1.0));
        assert!(!is_boundary_safe(29.0, 0.0, 1.0));
        assert!(!is_boundary_safe(29.0, 30.0, -1.0));
    }

    struct LinearSun;

    impl EphemerisProvider for LinearSun {
        fn sun_geocentric_ecliptic(&self, jd_tt: f64) -> Result<EclipticPosition, AstronomyError> {
            Ok(EclipticPosition {
                longitude_deg: normalize_deg(jd_tt),
                latitude_deg: 0.0,
                distance_au: 1.0,
                precision: AstronomyPrecision::PracticalVerified,
                estimated_error_arcmin: 0.0,
                algorithm_version: "test.linear".to_owned(),
            })
        }
    }

    #[test]
    fn solar_term_solver_handles_zero_degree_wrap() {
        let event = solve_solar_term(&LinearSun, SolarTerm::Chunfen, 359.0, 361.0, 0.1).unwrap();
        assert!((event.jd_tt - 360.0).abs() * 86_400.0 < 0.1);
        assert!(event.final_bracket_seconds <= 0.1);
    }

    #[test]
    fn solar_term_solver_rejects_unbracketed_target() {
        let error = solve_solar_term(&LinearSun, SolarTerm::Chunfen, 10.0, 20.0, 0.1).unwrap_err();
        assert!(matches!(error, AstronomyError::TargetNotBracketed { .. }));
    }

    #[test]
    fn finds_previous_and_next_terms_across_zero_degree_wrap() {
        let before_wrap = solar_term_interval(&LinearSun, 359.0, 300.0).unwrap();
        assert_eq!(before_wrap.previous.term, SolarTerm::Jingzhe);
        assert!((before_wrap.previous.jd_tt - 345.0).abs() * 86_400.0 < 0.1);
        assert_eq!(before_wrap.next.term, SolarTerm::Chunfen);
        assert!((before_wrap.next.jd_tt - 360.0).abs() * 86_400.0 < 0.1);
        assert!(before_wrap.boundary_safe);

        let after_wrap = solar_term_interval(&LinearSun, 367.0, 300.0).unwrap();
        assert_eq!(after_wrap.previous.term, SolarTerm::Chunfen);
        assert_eq!(after_wrap.next.term, SolarTerm::Qingming);
        assert!(after_wrap.boundary_safe);
    }

    #[test]
    fn solar_term_interval_blocks_the_solved_boundary_and_invalid_margins() {
        let on_boundary = solar_term_interval(&LinearSun, 360.0, 300.0).unwrap();
        assert!(!on_boundary.boundary_safe);
        assert!(matches!(
            solar_term_interval(&LinearSun, 360.0, f64::NAN),
            Err(AstronomyError::InvalidBoundaryMargin)
        ));
        assert!(matches!(
            solar_term_interval(&LinearSun, 360.0, -1.0),
            Err(AstronomyError::InvalidBoundaryMargin)
        ));
    }

    #[test]
    fn matches_hko_2024_lichun_and_yushui_minute_anchors() {
        let query = jd_tt_from_bjt(2024, 2, 10, 0, 0);
        let interval = solar_term_interval(&SpaSolarEphemeris, query, 300.0).unwrap();
        let hko_lichun = jd_tt_from_bjt(2024, 2, 4, 16, 27);
        let hko_yushui = jd_tt_from_bjt(2024, 2, 19, 12, 13);

        assert_eq!(interval.previous.term, SolarTerm::Lichun);
        assert_eq!(interval.next.term, SolarTerm::Yushui);
        assert!((interval.previous.jd_tt - hko_lichun).abs() * 86_400.0 < 90.0);
        assert!((interval.next.jd_tt - hko_yushui).abs() * 86_400.0 < 90.0);
    }

    #[test]
    fn classifies_all_terms_as_alternating_zhongqi_and_jie() {
        for (index, term) in SolarTerm::ALL_LONGITUDE_ORDER.into_iter().enumerate() {
            let expected = if index % 2 == 0 {
                SolarTermClass::Zhongqi
            } else {
                SolarTermClass::Jie
            };
            assert_eq!(term.class(), expected);
        }
        assert_eq!(SolarTerm::Chunfen.class(), SolarTermClass::Zhongqi);
        assert_eq!(SolarTerm::Qingming.class(), SolarTermClass::Jie);
    }

    #[test]
    fn separates_jie_and_zhongqi_intervals_across_zero_degree_wrap() {
        let jie = solar_term_class_interval(&LinearSun, 367.0, SolarTermClass::Jie, 300.0).unwrap();
        assert_eq!(jie.previous.term, SolarTerm::Jingzhe);
        assert_eq!(jie.next.term, SolarTerm::Qingming);

        let zhongqi =
            solar_term_class_interval(&LinearSun, 367.0, SolarTermClass::Zhongqi, 300.0).unwrap();
        assert_eq!(zhongqi.previous.term, SolarTerm::Chunfen);
        assert_eq!(zhongqi.next.term, SolarTerm::Guyu);
        assert!(jie.boundary_safe);
        assert!(zhongqi.boundary_safe);
    }

    #[test]
    fn matches_hko_2024_jie_and_zhongqi_interval_anchors() {
        let query = jd_tt_from_bjt(2024, 2, 10, 0, 0);
        let jie = solar_term_class_interval(&SpaSolarEphemeris, query, SolarTermClass::Jie, 300.0)
            .unwrap();
        let zhongqi =
            solar_term_class_interval(&SpaSolarEphemeris, query, SolarTermClass::Zhongqi, 300.0)
                .unwrap();

        let hko_dahan = jd_tt_from_bjt(2024, 1, 20, 22, 7);
        let hko_lichun = jd_tt_from_bjt(2024, 2, 4, 16, 27);
        let hko_yushui = jd_tt_from_bjt(2024, 2, 19, 12, 13);
        let hko_jingzhe = jd_tt_from_bjt(2024, 3, 5, 10, 23);

        assert_eq!(jie.previous.term, SolarTerm::Lichun);
        assert_eq!(jie.next.term, SolarTerm::Jingzhe);
        assert_eq!(zhongqi.previous.term, SolarTerm::Dahan);
        assert_eq!(zhongqi.next.term, SolarTerm::Yushui);
        assert!((jie.previous.jd_tt - hko_lichun).abs() * 86_400.0 < 90.0);
        assert!((jie.next.jd_tt - hko_jingzhe).abs() * 86_400.0 < 90.0);
        assert!((zhongqi.previous.jd_tt - hko_dahan).abs() * 86_400.0 < 90.0);
        assert!((zhongqi.next.jd_tt - hko_yushui).abs() * 86_400.0 < 90.0);
    }

    #[test]
    fn solves_a_term_from_a_bjt_reference_date_and_rejects_invalid_dates() {
        let lichun =
            solve_solar_term_near_bjt_date(&SpaSolarEphemeris, SolarTerm::Lichun, 2024, 2, 4, 7.0)
                .unwrap();
        let hko_lichun = jd_tt_from_bjt(2024, 2, 4, 16, 27);
        assert!((lichun.jd_tt - hko_lichun).abs() * 86_400.0 < 90.0);
        assert!(matches!(
            solve_solar_term_near_bjt_date(&SpaSolarEphemeris, SolarTerm::Lichun, 2024, 2, 31, 7.0,),
            Err(AstronomyError::InvalidReferenceDate)
        ));
    }
}
