use chrono::{DateTime, Datelike, Duration, FixedOffset};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalendarSystem {
    Julian,
    Gregorian,
    HistoricalCutover,
}

pub const CIVIL_CALENDAR_AUDIT_VERSION: &str = "jiuheng.civil_calendar_views.v1";

                                                               
   
                                                                             
                                                                             
                                                                        
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CivilCalendarAudit {
    pub algorithm_version: String,
    pub input_system: CalendarSystem,
    pub input: HistoricalDateTime,
    pub julian_day: f64,
    pub proleptic_gregorian: HistoricalDateTime,
    pub julian: HistoricalDateTime,
    pub historical_cutover: HistoricalDateTime,
    pub historical_cutover_resolved_system: CalendarSystem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalDateTime {
                                                             
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CalendarError {
    #[error("日期字段超出范围")]
    FieldOutOfRange,
    #[error("1582-10-05 至 1582-10-14 在历史切换历中不存在")]
    GregorianCutoverGap,
    #[error("ΔT 估算仅支持公元前 1999 年至公元 3000 年")]
    DeltaTYearOutOfRange,
    #[error("真太阳时输入必须是有限数值")]
    NonFiniteSolarTimeInput,
    #[error("经度必须在 -180° 至 180° 之间")]
    LongitudeOutOfRange,
    #[error("时界安全边距不能为负数")]
    NegativeBoundaryMargin,
    #[error("儒略日必须是有限数值")]
    NonFiniteJulianDay,
    #[error("历法反算结果超出支持范围")]
    InverseDateOutOfRange,
}

impl HistoricalDateTime {
    pub fn validate(self, system: CalendarSystem) -> Result<(), CalendarError> {
        if !(1..=12).contains(&self.month)
            || self.day == 0
            || self.hour > 23
            || self.minute > 59
            || self.second > 59
        {
            return Err(CalendarError::FieldOutOfRange);
        }
        let resolved = resolve_calendar(system, self)?;
        let max_day = days_in_month(self.year, self.month, resolved);
        if self.day > max_day {
            return Err(CalendarError::FieldOutOfRange);
        }
        Ok(())
    }
}

pub fn julian_day(
    date_time: HistoricalDateTime,
    system: CalendarSystem,
) -> Result<f64, CalendarError> {
    date_time.validate(system)?;
    let resolved = resolve_calendar(system, date_time)?;
    Ok(julian_day_unchecked(date_time, resolved))
}

                                                                         
                                                                      
                                                                              
                       
pub fn historical_datetime_from_julian_day(
    julian_day: f64,
    system: CalendarSystem,
) -> Result<HistoricalDateTime, CalendarError> {
    if !julian_day.is_finite() {
        return Err(CalendarError::NonFiniteJulianDay);
    }

    let (resolved, cutover_jd) = match system {
        CalendarSystem::HistoricalCutover => {
            let cutover = julian_day_unchecked(
                HistoricalDateTime {
                    year: 1582,
                    month: 10,
                    day: 15,
                    hour: 0,
                    minute: 0,
                    second: 0,
                },
                CalendarSystem::Gregorian,
            );
            (
                if julian_day >= cutover {
                    CalendarSystem::Gregorian
                } else {
                    CalendarSystem::Julian
                },
                Some(cutover),
            )
        }
        resolved => (resolved, None),
    };

    let mut shifted = julian_day + 0.5;
    let mut whole_day = shifted.floor();
    let mut seconds_of_day = ((shifted - whole_day) * 86_400.0).round() as i64;
    if seconds_of_day >= 86_400 {
        whole_day += 1.0;
        seconds_of_day = 0;
        shifted = whole_day;
    }
    if seconds_of_day < 0 {
        return Err(CalendarError::InverseDateOutOfRange);
    }

    let z = shifted.floor();
    let a = match resolved {
        CalendarSystem::Gregorian => {
            let alpha = ((z - 1_867_216.25) / 36_524.25).floor();
            z + 1.0 + alpha - (alpha / 4.0).floor()
        }
        CalendarSystem::Julian => z,
        CalendarSystem::HistoricalCutover => unreachable!("resolved above"),
    };
    let b = a + 1_524.0;
    let c = ((b - 122.1) / 365.25).floor();
    let d = (365.25 * c).floor();
    let e = ((b - d) / 30.6001).floor();
    let day = b - d - (30.6001 * e).floor();
    let month = if e < 14.0 { e - 1.0 } else { e - 13.0 };
    let year = if month > 2.0 {
        c - 4_716.0
    } else {
        c - 4_715.0
    };

    if year < i32::MIN as f64
        || year > i32::MAX as f64
        || !(1.0..=12.0).contains(&month)
        || !(1.0..=31.0).contains(&day)
    {
        return Err(CalendarError::InverseDateOutOfRange);
    }

    let result = HistoricalDateTime {
        year: year as i32,
        month: month as u8,
        day: day as u8,
        hour: (seconds_of_day / 3_600) as u8,
        minute: ((seconds_of_day % 3_600) / 60) as u8,
        second: (seconds_of_day % 60) as u8,
    };
    result.validate(resolved)?;

    if let Some(cutover_jd) = cutover_jd {
        debug_assert_eq!(
            resolved,
            if julian_day >= cutover_jd {
                CalendarSystem::Gregorian
            } else {
                CalendarSystem::Julian
            }
        );
    }
    Ok(result)
}

pub fn civil_calendar_audit(
    input: HistoricalDateTime,
    input_system: CalendarSystem,
) -> Result<CivilCalendarAudit, CalendarError> {
    let jd = julian_day(input, input_system)?;
    let historical_cutover =
        historical_datetime_from_julian_day(jd, CalendarSystem::HistoricalCutover)?;
    let historical_cutover_resolved_system = if jd
        >= julian_day_unchecked(
            HistoricalDateTime {
                year: 1582,
                month: 10,
                day: 15,
                hour: 0,
                minute: 0,
                second: 0,
            },
            CalendarSystem::Gregorian,
        ) {
        CalendarSystem::Gregorian
    } else {
        CalendarSystem::Julian
    };

    Ok(CivilCalendarAudit {
        algorithm_version: CIVIL_CALENDAR_AUDIT_VERSION.to_owned(),
        input_system,
        input,
        julian_day: jd,
        proleptic_gregorian: historical_datetime_from_julian_day(jd, CalendarSystem::Gregorian)?,
        julian: historical_datetime_from_julian_day(jd, CalendarSystem::Julian)?,
        historical_cutover,
        historical_cutover_resolved_system,
    })
}

pub fn astronomical_year_label_zh(year: i32) -> String {
    if year > 0 {
        format!("公元 {year} 年（天文年 {year}）")
    } else {
        format!("公元前 {} 年（天文年 {year}）", 1_i64 - i64::from(year))
    }
}

pub fn historical_datetime_label_zh(value: HistoricalDateTime) -> String {
    format!(
        "{} {:02}-{:02} {:02}:{:02}:{:02}",
        astronomical_year_label_zh(value.year),
        value.month,
        value.day,
        value.hour,
        value.minute,
        value.second
    )
}

fn julian_day_unchecked(date_time: HistoricalDateTime, system: CalendarSystem) -> f64 {
    let mut year = date_time.year as f64;
    let mut month = date_time.month as f64;
    if month <= 2.0 {
        year -= 1.0;
        month += 12.0;
    }
    let correction = match system {
        CalendarSystem::Gregorian => {
            let century = (year / 100.0).floor();
            2.0 - century + (century / 4.0).floor()
        }
        CalendarSystem::Julian => 0.0,
        CalendarSystem::HistoricalCutover => unreachable!("must be resolved first"),
    };
    let fraction =
        (date_time.hour as f64 + date_time.minute as f64 / 60.0 + date_time.second as f64 / 3600.0)
            / 24.0;
    (365.25 * (year + 4716.0)).floor()
        + (30.6001 * (month + 1.0)).floor()
        + date_time.day as f64
        + fraction
        + correction
        - 1524.5
}

fn resolve_calendar(
    requested: CalendarSystem,
    date: HistoricalDateTime,
) -> Result<CalendarSystem, CalendarError> {
    if requested != CalendarSystem::HistoricalCutover {
        return Ok(requested);
    }
    let ymd = (date.year, date.month, date.day);
    if ymd >= (1582, 10, 15) {
        Ok(CalendarSystem::Gregorian)
    } else if ymd <= (1582, 10, 4) {
        Ok(CalendarSystem::Julian)
    } else {
        Err(CalendarError::GregorianCutoverGap)
    }
}

fn days_in_month(year: i32, month: u8, system: CalendarSystem) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year, system) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i32, system: CalendarSystem) -> bool {
    match system {
        CalendarSystem::Julian => year.rem_euclid(4) == 0,
        CalendarSystem::Gregorian => {
            year.rem_euclid(4) == 0 && (year.rem_euclid(100) != 0 || year.rem_euclid(400) == 0)
        }
        CalendarSystem::HistoricalCutover => unreachable!("resolve calendar first"),
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SolarTimeCorrection {
    pub longitude_correction_minutes: f64,
    pub equation_of_time_minutes: f64,
    pub total_correction_minutes: f64,
    pub boundary_safe: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeltaTEstimate {
                          
    pub seconds: f64,
                                                                           
                                                   
    pub estimated_uncertainty_seconds: f64,
    pub model_version: String,
    pub valid_from_year: i32,
    pub valid_to_year: i32,
}

                                                                           
   
                                                                              
                                                                             
                                                                             
                                             
pub fn estimate_delta_t(year: i32, month: u8) -> Result<DeltaTEstimate, CalendarError> {
    if !(-1999..=3000).contains(&year) || !(1..=12).contains(&month) {
        return Err(if !(-1999..=3000).contains(&year) {
            CalendarError::DeltaTYearOutOfRange
        } else {
            CalendarError::FieldOutOfRange
        });
    }
    let y = year as f64 + (month as f64 - 0.5) / 12.0;
    let seconds = if y < -500.0 {
        let u = (y - 1820.0) / 100.0;
        -20.0 + 32.0 * u.powi(2)
    } else if y < 500.0 {
        let u = y / 100.0;
        10_583.6 - 1_014.41 * u + 33.783_11 * u.powi(2)
            - 5.952_053 * u.powi(3)
            - 0.179_845_2 * u.powi(4)
            + 0.022_174_192 * u.powi(5)
            + 0.009_031_652_1 * u.powi(6)
    } else if y < 1600.0 {
        let u = (y - 1000.0) / 100.0;
        1_574.2 - 556.01 * u + 71.234_72 * u.powi(2) + 0.319_781 * u.powi(3)
            - 0.850_346_3 * u.powi(4)
            - 0.005_050_998 * u.powi(5)
            + 0.008_357_207_3 * u.powi(6)
    } else if y < 1700.0 {
        let t = y - 1600.0;
        120.0 - 0.9808 * t - 0.01532 * t.powi(2) + t.powi(3) / 7_129.0
    } else if y < 1800.0 {
        let t = y - 1700.0;
        8.83 + 0.1603 * t - 0.005_928_5 * t.powi(2) + 0.000_133_36 * t.powi(3)
            - t.powi(4) / 1_174_000.0
    } else if y < 1860.0 {
        let t = y - 1800.0;
        13.72 - 0.332_447 * t + 0.006_861_2 * t.powi(2) + 0.004_111_6 * t.powi(3)
            - 0.000_374_36 * t.powi(4)
            + 0.000_012_127_2 * t.powi(5)
            - 0.000_000_169_9 * t.powi(6)
            + 0.000_000_000_875 * t.powi(7)
    } else if y < 1900.0 {
        let t = y - 1860.0;
        7.62 + 0.5737 * t - 0.251_754 * t.powi(2) + 0.016_806_68 * t.powi(3)
            - 0.000_447_362_4 * t.powi(4)
            + t.powi(5) / 233_174.0
    } else if y < 1920.0 {
        let t = y - 1900.0;
        -2.79 + 1.494_119 * t - 0.059_893_9 * t.powi(2) + 0.006_196_6 * t.powi(3)
            - 0.000_197 * t.powi(4)
    } else if y < 1941.0 {
        let t = y - 1920.0;
        21.20 + 0.844_93 * t - 0.076_100 * t.powi(2) + 0.002_093_6 * t.powi(3)
    } else if y < 1961.0 {
        let t = y - 1950.0;
        29.07 + 0.407 * t - t.powi(2) / 233.0 + t.powi(3) / 2_547.0
    } else if y < 1986.0 {
        let t = y - 1975.0;
        45.45 + 1.067 * t - t.powi(2) / 260.0 - t.powi(3) / 718.0
    } else if y < 2005.0 {
        let t = y - 2000.0;
        63.86 + 0.3345 * t - 0.060_374 * t.powi(2)
            + 0.001_727_5 * t.powi(3)
            + 0.000_651_814 * t.powi(4)
            + 0.000_023_735_99 * t.powi(5)
    } else if y < 2050.0 {
        let t = y - 2000.0;
        62.92 + 0.322_17 * t + 0.005_589 * t.powi(2)
    } else if y < 2150.0 {
        -20.0 + 32.0 * ((y - 1820.0) / 100.0).powi(2) - 0.5628 * (2150.0 - y)
    } else {
        -20.0 + 32.0 * ((y - 1820.0) / 100.0).powi(2)
    };

    Ok(DeltaTEstimate {
        seconds,
        estimated_uncertainty_seconds: delta_t_uncertainty_seconds(y),
        model_version: "nasa.eclipse.delta_t.piecewise.v1".to_owned(),
        valid_from_year: -1999,
        valid_to_year: 3000,
    })
}

pub fn julian_day_tt_from_ut1(jd_ut1: f64, delta_t_seconds: f64) -> Result<f64, CalendarError> {
    if !jd_ut1.is_finite() || !delta_t_seconds.is_finite() {
        return Err(CalendarError::FieldOutOfRange);
    }
    Ok(jd_ut1 + delta_t_seconds / 86_400.0)
}

fn delta_t_uncertainty_seconds(year: f64) -> f64 {
    if year < -1000.0 {
        huber_extrapolation_uncertainty((-500.0 - year).max(0.0))
    } else if year <= 1200.0 {
        let t = (year - 1820.0) / 100.0;
        0.8 * t.powi(2)
    } else if year < 1300.0 {
        31.0 + (year - 1200.0) / 100.0 * (20.0 - 31.0)
    } else if year < 1600.0 {
        20.0
    } else if year < 1700.0 {
        20.0 + (year - 1600.0) / 100.0 * (5.0 - 20.0)
    } else if year < 1800.0 {
        5.0 + (year - 1700.0) / 100.0 * (1.0 - 5.0)
    } else if year < 1900.0 {
        1.0 + (year - 1800.0) / 100.0 * (0.1 - 1.0)
    } else if year <= 2005.0 {
        0.1
    } else {
        huber_extrapolation_uncertainty(year - 2005.0).max(0.1)
    }
}

fn huber_extrapolation_uncertainty(years_from_calibration: f64) -> f64 {
    const M: f64 = 2_500.0;
    const Q: f64 = 0.058;
    let n = years_from_calibration.max(0.0);
    365.25 * n * ((n * Q / 3.0) * (1.0 + n / M)).sqrt() / 1_000.0
}

                                                
   
                                                                             
                                                                          
pub fn true_solar_time_foundation(
    beijing_time: DateTime<FixedOffset>,
    longitude_deg: f64,
    boundary_margin_minutes: f64,
) -> (DateTime<FixedOffset>, SolarTimeCorrection) {
    let day = beijing_time.ordinal() as f64;
    let hour = beijing_time.hour() as f64
        + beijing_time.minute() as f64 / 60.0
        + beijing_time.second() as f64 / 3600.0;
    let days_in_year = if is_leap_year(beijing_time.year(), CalendarSystem::Gregorian) {
        366.0
    } else {
        365.0
    };
    let gamma = 2.0 * std::f64::consts::PI / days_in_year * (day - 1.0 + (hour - 12.0) / 24.0);
    let equation = 229.18
        * (0.000075 + 0.001868 * gamma.cos()
            - 0.032077 * gamma.sin()
            - 0.014615 * (2.0 * gamma).cos()
            - 0.040849 * (2.0 * gamma).sin());
    let longitude = (longitude_deg - 120.0) * 4.0;
    let total = longitude + equation;
    let corrected = beijing_time + Duration::milliseconds((total * 60_000.0).round() as i64);
    let minute_in_hour = corrected.minute() as f64 + corrected.second() as f64 / 60.0;
    let distance_to_hour = minute_in_hour.min(60.0 - minute_in_hour);

    (
        corrected,
        SolarTimeCorrection {
            longitude_correction_minutes: longitude,
            equation_of_time_minutes: equation,
            total_correction_minutes: total,
            boundary_safe: distance_to_hour >= boundary_margin_minutes,
        },
    )
}

                                                                     
   
                                                                          
                                                                       
pub fn true_solar_time_from_equation_of_time(
    beijing_time: DateTime<FixedOffset>,
    longitude_deg: f64,
    equation_of_time_minutes: f64,
    boundary_margin_minutes: f64,
) -> Result<(DateTime<FixedOffset>, SolarTimeCorrection), CalendarError> {
    if !longitude_deg.is_finite()
        || !equation_of_time_minutes.is_finite()
        || !boundary_margin_minutes.is_finite()
    {
        return Err(CalendarError::NonFiniteSolarTimeInput);
    }
    if !(-180.0..=180.0).contains(&longitude_deg) {
        return Err(CalendarError::LongitudeOutOfRange);
    }
    if boundary_margin_minutes < 0.0 {
        return Err(CalendarError::NegativeBoundaryMargin);
    }

    let longitude_correction_minutes = (longitude_deg - 120.0) * 4.0;
    let total_correction_minutes = longitude_correction_minutes + equation_of_time_minutes;
    let corrected =
        beijing_time + Duration::milliseconds((total_correction_minutes * 60_000.0).round() as i64);
    let minute_in_hour = corrected.minute() as f64 + corrected.second() as f64 / 60.0;
    let distance_to_hour = minute_in_hour.min(60.0 - minute_in_hour);

    Ok((
        corrected,
        SolarTimeCorrection {
            longitude_correction_minutes,
            equation_of_time_minutes,
            total_correction_minutes,
            boundary_safe: distance_to_hour >= boundary_margin_minutes,
        },
    ))
}

                                                                               
use chrono::Timelike;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn j2000_epoch_matches_reference_definition() {
        let jd = julian_day(
            HistoricalDateTime {
                year: 2000,
                month: 1,
                day: 1,
                hour: 12,
                minute: 0,
                second: 0,
            },
            CalendarSystem::Gregorian,
        )
        .unwrap();
        assert!((jd - 2_451_545.0).abs() < 1e-9);
    }

    #[test]
    fn historical_cutover_is_continuous_in_julian_days() {
        let before = julian_day(
            HistoricalDateTime {
                year: 1582,
                month: 10,
                day: 4,
                hour: 0,
                minute: 0,
                second: 0,
            },
            CalendarSystem::HistoricalCutover,
        )
        .unwrap();
        let after = julian_day(
            HistoricalDateTime {
                year: 1582,
                month: 10,
                day: 15,
                hour: 0,
                minute: 0,
                second: 0,
            },
            CalendarSystem::HistoricalCutover,
        )
        .unwrap();
        assert!((after - before - 1.0).abs() < 1e-9);
    }

    #[test]
    fn supports_1000_bce_as_astronomical_year_minus_999() {
        assert!(julian_day(
            HistoricalDateTime {
                year: -999,
                month: 6,
                day: 1,
                hour: 0,
                minute: 0,
                second: 0,
            },
            CalendarSystem::Julian,
        )
        .is_ok());
    }

    #[test]
    fn rejects_dates_inside_cutover_gap() {
        assert_eq!(
            julian_day(
                HistoricalDateTime {
                    year: 1582,
                    month: 10,
                    day: 10,
                    hour: 0,
                    minute: 0,
                    second: 0,
                },
                CalendarSystem::HistoricalCutover,
            ),
            Err(CalendarError::GregorianCutoverGap)
        );
    }

    #[test]
    fn inverse_calendar_views_preserve_one_instant() {
        let input = HistoricalDateTime {
            year: 2000,
            month: 1,
            day: 1,
            hour: 12,
            minute: 34,
            second: 56,
        };
        let audit = civil_calendar_audit(input, CalendarSystem::Gregorian).unwrap();

        assert_eq!(audit.proleptic_gregorian, input);
        assert_eq!(
            audit.julian,
            HistoricalDateTime {
                year: 1999,
                month: 12,
                day: 19,
                hour: 12,
                minute: 34,
                second: 56,
            }
        );
        assert_eq!(audit.historical_cutover, input);
        assert_eq!(
            audit.historical_cutover_resolved_system,
            CalendarSystem::Gregorian
        );
        for (date, system) in [
            (audit.proleptic_gregorian, CalendarSystem::Gregorian),
            (audit.julian, CalendarSystem::Julian),
            (audit.historical_cutover, CalendarSystem::HistoricalCutover),
        ] {
            assert!((julian_day(date, system).unwrap() - audit.julian_day).abs() < 1.0e-9);
        }
    }

    #[test]
    fn historical_inverse_selects_the_correct_side_of_the_reform() {
        let last_julian = HistoricalDateTime {
            year: 1582,
            month: 10,
            day: 4,
            hour: 23,
            minute: 59,
            second: 59,
        };
        let first_gregorian = HistoricalDateTime {
            year: 1582,
            month: 10,
            day: 15,
            hour: 0,
            minute: 0,
            second: 0,
        };
        let before = julian_day(last_julian, CalendarSystem::HistoricalCutover).unwrap();
        let after = julian_day(first_gregorian, CalendarSystem::HistoricalCutover).unwrap();

        assert_eq!(
            historical_datetime_from_julian_day(before, CalendarSystem::HistoricalCutover).unwrap(),
            last_julian
        );
        assert_eq!(
            historical_datetime_from_julian_day(after, CalendarSystem::HistoricalCutover).unwrap(),
            first_gregorian
        );
    }

    #[test]
    fn inverse_round_trip_supports_bce_and_future_dates() {
        for date in [
            HistoricalDateTime {
                year: -999,
                month: 6,
                day: 1,
                hour: 0,
                minute: 0,
                second: 0,
            },
            HistoricalDateTime {
                year: 0,
                month: 2,
                day: 29,
                hour: 12,
                minute: 0,
                second: 1,
            },
            HistoricalDateTime {
                year: 3000,
                month: 12,
                day: 31,
                hour: 23,
                minute: 59,
                second: 59,
            },
        ] {
            let jd = julian_day(date, CalendarSystem::Gregorian).unwrap();
            assert_eq!(
                historical_datetime_from_julian_day(jd, CalendarSystem::Gregorian).unwrap(),
                date
            );
        }
        assert_eq!(astronomical_year_label_zh(0), "公元前 1 年（天文年 0）");
        assert_eq!(
            astronomical_year_label_zh(-999),
            "公元前 1000 年（天文年 -999）"
        );
    }

    #[test]
    fn inverse_rejects_non_finite_julian_days() {
        assert_eq!(
            historical_datetime_from_julian_day(f64::NAN, CalendarSystem::Gregorian),
            Err(CalendarError::NonFiniteJulianDay)
        );
    }

    #[test]
    fn delta_t_matches_published_piecewise_model_examples() {
        let y_2000 = estimate_delta_t(2000, 1).unwrap();
        assert!((y_2000.seconds - 63.87).abs() < 0.2);
        assert_eq!(y_2000.model_version, "nasa.eclipse.delta_t.piecewise.v1");

        let y_1000 = estimate_delta_t(1000, 1).unwrap();
        assert!((y_1000.seconds - 1_574.2).abs() < 2.0);
        assert!(y_1000.estimated_uncertainty_seconds > 50.0);
    }

    #[test]
    fn tt_conversion_adds_delta_t_in_days() {
        let jd_tt = julian_day_tt_from_ut1(2_451_545.0, 64.0).unwrap();
        assert!((jd_tt - (2_451_545.0 + 64.0 / 86_400.0)).abs() < 1e-12);
    }

    #[test]
    fn applies_spa_equation_of_time_with_the_utc8_standard_meridian() {
        let zone = FixedOffset::east_opt(8 * 3_600).unwrap();
        let civil = zone.with_ymd_and_hms(2003, 10, 18, 3, 30, 30).unwrap();
        let (apparent, correction) =
            true_solar_time_from_equation_of_time(civil, 120.0, 14.641_503, 5.0).unwrap();

        assert_eq!(correction.longitude_correction_minutes, 0.0);
        assert!((correction.total_correction_minutes - 14.641_503).abs() < 1.0e-12);
        assert_eq!(apparent.to_rfc3339(), "2003-10-18T03:45:08.490+08:00");
        assert!(correction.boundary_safe);
    }

    #[test]
    fn rejects_invalid_true_solar_time_inputs() {
        let zone = FixedOffset::east_opt(8 * 3_600).unwrap();
        let civil = zone.with_ymd_and_hms(2026, 8, 30, 12, 45, 0).unwrap();
        assert!(matches!(
            true_solar_time_from_equation_of_time(civil, 181.0, 0.0, 5.0),
            Err(CalendarError::LongitudeOutOfRange)
        ));
        assert!(matches!(
            true_solar_time_from_equation_of_time(civil, 120.0, f64::NAN, 5.0),
            Err(CalendarError::NonFiniteSolarTimeInput)
        ));
    }
}
