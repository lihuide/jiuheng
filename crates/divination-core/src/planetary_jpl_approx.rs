                                                                       
   
                                                                       
                                                                        
                                                          

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const JPL_LONG_RANGE_PLANETARY_ALGORITHM_VERSION: &str =
    "jiuheng.astronomy.jpl_approximate_planets_table_2_geometric_emb_j2000.v1";
pub const JPL_SHORT_RANGE_PLANETARY_ALGORITHM_VERSION: &str =
    "jiuheng.astronomy.jpl_approximate_planets_table_1_geometric_emb_j2000.v1";
pub const JPL_LONG_RANGE_PLANETARY_SOURCE: &str =
    "https://ssd.jpl.nasa.gov/planets/approx_pos.html";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisiblePlanet {
    Mercury,
    Venus,
    Mars,
    Jupiter,
    Saturn,
}

impl VisiblePlanet {
    pub const ALL: [Self; 5] = [
        Self::Mercury,
        Self::Venus,
        Self::Mars,
        Self::Jupiter,
        Self::Saturn,
    ];

    pub const fn received_zh_name(self) -> &'static str {
        match self {
            Self::Mercury => "辰星",
            Self::Venus => "太白",
            Self::Mars => "荧惑",
            Self::Jupiter => "岁星",
            Self::Saturn => "填星",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanetaryGeometricObservation {
    pub planet: VisiblePlanet,
    pub jd_tt_used_as_tdb_approx: f64,
    pub heliocentric_j2000_ecliptic_au: [f64; 3],
    pub earth_moon_barycenter_heliocentric_j2000_ecliptic_au: [f64; 3],
    pub planet_minus_earth_moon_barycenter_j2000_ecliptic_au: [f64; 3],
    pub emb_relative_longitude_deg: f64,
    pub emb_relative_latitude_deg: f64,
    pub emb_relative_distance_au: f64,
    pub algorithm_version: String,
    pub source_url: String,
    pub apparent_geocentric_position: bool,
    pub historical_mansion_eligible: bool,
    pub divination_rule_eligible: bool,
}

                                                                          
                                                                              
                                     
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanetaryFitOverlapDiagnostic {
    pub planet: VisiblePlanet,
    pub jd_tt_used_as_tdb_approx: f64,
    pub long_range_algorithm_version: String,
    pub short_range_algorithm_version: String,
    pub source_url: String,
    pub long_range_longitude_deg: f64,
    pub short_range_longitude_deg: f64,
    pub angular_disagreement_deg: f64,
    pub distance_disagreement_au: f64,
    pub independent_ephemeris_validation: bool,
    pub historical_mansion_eligible: bool,
    pub divination_rule_eligible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum JplApproxError {
    #[error("行星近似计算时间必须为有限儒略日")]
    NonFiniteJulianDay,
    #[error("行星近似计算超出 JPL 长期参数的约略适用区间")]
    OutsideLongRangeTable,
    #[error("行星双表对拍超出 JPL 短期参数的保守适用区间")]
    OutsideShortRangeTable,
    #[error("行星近似计算的轨道参数无效")]
    InvalidOrbitalElements,
    #[error("行星近似计算的开普勒方程未收敛")]
    KeplerDidNotConverge,
}

#[derive(Clone, Copy)]
struct Elements {
    base: [f64; 6],
    rate: [f64; 6],
    anomaly_extra: [f64; 4],                                  
}

                                                                         
                                                                           
                                                           
const MERCURY: Elements = Elements {
    base: [
        0.38709843,
        0.20563661,
        7.00559432,
        252.25166724,
        77.45771895,
        48.33961819,
    ],
    rate: [
        0.0,
        0.00002123,
        -0.00590158,
        149472.67486623,
        0.15940013,
        -0.12214182,
    ],
    anomaly_extra: [0.0; 4],
};
const VENUS: Elements = Elements {
    base: [
        0.72332102,
        0.00676399,
        3.39777545,
        181.97970850,
        131.76755713,
        76.67261496,
    ],
    rate: [
        -0.00000026,
        -0.00005107,
        0.00043494,
        58517.81560260,
        0.05679648,
        -0.27274174,
    ],
    anomaly_extra: [0.0; 4],
};
const EMB: Elements = Elements {
    base: [
        1.00000018,
        0.01673163,
        -0.00054346,
        100.46691572,
        102.93005885,
        -5.11260389,
    ],
    rate: [
        -0.00000003,
        -0.00003661,
        -0.01337178,
        35999.37306329,
        0.31795260,
        -0.24123856,
    ],
    anomaly_extra: [0.0; 4],
};
const MARS: Elements = Elements {
    base: [
        1.52371243,
        0.09336511,
        1.85181869,
        -4.56813164,
        -23.91744784,
        49.71320984,
    ],
    rate: [
        0.00000097,
        0.00009149,
        -0.00724757,
        19140.29934243,
        0.45223625,
        -0.26852431,
    ],
    anomaly_extra: [0.0; 4],
};
const JUPITER: Elements = Elements {
    base: [
        5.20248019,
        0.04853590,
        1.29861416,
        34.33479152,
        14.27495244,
        100.29282654,
    ],
    rate: [
        -0.00002864,
        0.00018026,
        -0.00322699,
        3034.90371757,
        0.18199196,
        0.13024619,
    ],
    anomaly_extra: [-0.00012452, 0.06064060, -0.35635438, 38.35125000],
};
const SATURN: Elements = Elements {
    base: [
        9.54149883,
        0.05550825,
        2.49424102,
        50.07571329,
        92.86136063,
        113.63998702,
    ],
    rate: [
        -0.00003065,
        -0.00032044,
        0.00451969,
        1222.11494724,
        0.54179478,
        -0.25015002,
    ],
    anomaly_extra: [0.00025899, -0.13434469, 0.87320147, 38.35125000],
};

                                                                          
                                                                           
const SHORT_MERCURY: Elements = Elements {
    base: [
        0.38709927,
        0.20563593,
        7.00497902,
        252.25032350,
        77.45779628,
        48.33076593,
    ],
    rate: [
        0.00000037,
        0.00001906,
        -0.00594749,
        149472.67411175,
        0.16047689,
        -0.12534081,
    ],
    anomaly_extra: [0.0; 4],
};
const SHORT_VENUS: Elements = Elements {
    base: [
        0.72333566,
        0.00677672,
        3.39467605,
        181.97909950,
        131.60246718,
        76.67984255,
    ],
    rate: [
        0.00000390,
        -0.00004107,
        -0.00078890,
        58517.81538729,
        0.00268329,
        -0.27769418,
    ],
    anomaly_extra: [0.0; 4],
};
const SHORT_EMB: Elements = Elements {
    base: [
        1.00000261,
        0.01671123,
        -0.00001531,
        100.46457166,
        102.93768193,
        0.0,
    ],
    rate: [
        0.00000562,
        -0.00004392,
        -0.01294668,
        35999.37244981,
        0.32327364,
        0.0,
    ],
    anomaly_extra: [0.0; 4],
};
const SHORT_MARS: Elements = Elements {
    base: [
        1.52371034,
        0.09339410,
        1.84969142,
        -4.55343205,
        -23.94362959,
        49.55953891,
    ],
    rate: [
        0.00001847,
        0.00007882,
        -0.00813131,
        19140.30268499,
        0.44441088,
        -0.29257343,
    ],
    anomaly_extra: [0.0; 4],
};
const SHORT_JUPITER: Elements = Elements {
    base: [
        5.20288700,
        0.04838624,
        1.30439695,
        34.39644051,
        14.72847983,
        100.47390909,
    ],
    rate: [
        -0.00011607,
        -0.00013253,
        -0.00183714,
        3034.74612775,
        0.21252668,
        0.20469106,
    ],
    anomaly_extra: [0.0; 4],
};
const SHORT_SATURN: Elements = Elements {
    base: [
        9.53667594,
        0.05386179,
        2.48599187,
        49.95424423,
        92.59887831,
        113.66242448,
    ],
    rate: [
        -0.00125060,
        -0.00050991,
        0.00193609,
        1222.49362201,
        -0.41897216,
        -0.28867794,
    ],
    anomaly_extra: [0.0; 4],
};

fn elements_for(planet: VisiblePlanet) -> Elements {
    match planet {
        VisiblePlanet::Mercury => MERCURY,
        VisiblePlanet::Venus => VENUS,
        VisiblePlanet::Mars => MARS,
        VisiblePlanet::Jupiter => JUPITER,
        VisiblePlanet::Saturn => SATURN,
    }
}

fn short_elements_for(planet: VisiblePlanet) -> Elements {
    match planet {
        VisiblePlanet::Mercury => SHORT_MERCURY,
        VisiblePlanet::Venus => SHORT_VENUS,
        VisiblePlanet::Mars => SHORT_MARS,
        VisiblePlanet::Jupiter => SHORT_JUPITER,
        VisiblePlanet::Saturn => SHORT_SATURN,
    }
}

fn heliocentric_vector(elements: Elements, t: f64) -> Result<[f64; 3], JplApproxError> {
    let [a, e, inclination, mean_longitude, perihelion, node] =
        std::array::from_fn(|i| elements.base[i] + elements.rate[i] * t);
    if !a.is_finite() || a <= 0.0 || !(0.0..1.0).contains(&e) {
        return Err(JplApproxError::InvalidOrbitalElements);
    }
    let [b, c, s, f] = elements.anomaly_extra;
    let anomaly_deg = (mean_longitude - perihelion
        + b * t * t
        + c * (f * t).to_radians().cos()
        + s * (f * t).to_radians().sin())
    .rem_euclid(360.0);
    let mean_anomaly = if anomaly_deg > 180.0 {
        anomaly_deg - 360.0
    } else {
        anomaly_deg
    }
    .to_radians();
    let mut eccentric_anomaly = mean_anomaly + e * mean_anomaly.sin();
    let mut converged = false;
    for _ in 0..20 {
        let delta = (mean_anomaly - eccentric_anomaly + e * eccentric_anomaly.sin())
            / (1.0 - e * eccentric_anomaly.cos());
        eccentric_anomaly += delta;
        if delta.abs() <= 1.0e-6_f64.to_radians() {
            converged = true;
            break;
        }
    }
    if !converged {
        return Err(JplApproxError::KeplerDidNotConverge);
    }
    let x_orbit = a * (eccentric_anomaly.cos() - e);
    let y_orbit = a * (1.0 - e * e).sqrt() * eccentric_anomaly.sin();
    let omega = (perihelion - node).to_radians();
    let node = node.to_radians();
    let inclination = inclination.to_radians();
    let (sin_omega, cos_omega) = omega.sin_cos();
    let (sin_node, cos_node) = node.sin_cos();
    let (sin_i, cos_i) = inclination.sin_cos();
    Ok([
        (cos_omega * cos_node - sin_omega * sin_node * cos_i) * x_orbit
            + (-sin_omega * cos_node - cos_omega * sin_node * cos_i) * y_orbit,
        (cos_omega * sin_node + sin_omega * cos_node * cos_i) * x_orbit
            + (-sin_omega * sin_node + cos_omega * cos_node * cos_i) * y_orbit,
        sin_omega * sin_i * x_orbit + cos_omega * sin_i * y_orbit,
    ])
}

                                                                             
                                                                              
                                                                             
                                                                         
pub fn five_visible_planet_geometric_observations(
    jd_tt: f64,
) -> Result<Vec<PlanetaryGeometricObservation>, JplApproxError> {
    if !jd_tt.is_finite() {
        return Err(JplApproxError::NonFiniteJulianDay);
    }
    let t = (jd_tt - 2_451_545.0) / 36_525.0;
                                                                            
                                                                          
    if !(-50.0..=10.01).contains(&t) {
        return Err(JplApproxError::OutsideLongRangeTable);
    }
    geometric_observations_for_fit(
        jd_tt,
        t,
        EMB,
        elements_for,
        JPL_LONG_RANGE_PLANETARY_ALGORITHM_VERSION,
    )
}

fn geometric_observations_for_fit(
    jd_tt: f64,
    t: f64,
    emb_elements: Elements,
    planet_elements: fn(VisiblePlanet) -> Elements,
    algorithm_version: &'static str,
) -> Result<Vec<PlanetaryGeometricObservation>, JplApproxError> {
    let emb = heliocentric_vector(emb_elements, t)?;
    VisiblePlanet::ALL
        .into_iter()
        .map(|planet| {
            let heliocentric = heliocentric_vector(planet_elements(planet), t)?;
            let relative = std::array::from_fn(|i| heliocentric[i] - emb[i]);
            let planar = relative[0].hypot(relative[1]);
            let distance = planar.hypot(relative[2]);
            Ok(PlanetaryGeometricObservation {
                planet,
                jd_tt_used_as_tdb_approx: jd_tt,
                heliocentric_j2000_ecliptic_au: heliocentric,
                earth_moon_barycenter_heliocentric_j2000_ecliptic_au: emb,
                planet_minus_earth_moon_barycenter_j2000_ecliptic_au: relative,
                emb_relative_longitude_deg: relative[1]
                    .atan2(relative[0])
                    .to_degrees()
                    .rem_euclid(360.0),
                emb_relative_latitude_deg: relative[2].atan2(planar).to_degrees(),
                emb_relative_distance_au: distance,
                algorithm_version: algorithm_version.to_owned(),
                source_url: JPL_LONG_RANGE_PLANETARY_SOURCE.to_owned(),
                apparent_geocentric_position: false,
                historical_mansion_eligible: false,
                divination_rule_eligible: false,
            })
        })
        .collect()
}

                                                                       
                                                                              
                                                                
pub fn five_visible_planet_fit_overlap_diagnostics(
    jd_tt: f64,
) -> Result<Vec<PlanetaryFitOverlapDiagnostic>, JplApproxError> {
    if !jd_tt.is_finite() {
        return Err(JplApproxError::NonFiniteJulianDay);
    }
                                                                       
                                                                             
                                            
    if !(2_378_496.5..2_469_807.5).contains(&jd_tt) {
        return Err(JplApproxError::OutsideShortRangeTable);
    }
    let t = (jd_tt - 2_451_545.0) / 36_525.0;
    let long_range = five_visible_planet_geometric_observations(jd_tt)?;
    let short_range = geometric_observations_for_fit(
        jd_tt,
        t,
        SHORT_EMB,
        short_elements_for,
        JPL_SHORT_RANGE_PLANETARY_ALGORITHM_VERSION,
    )?;
    Ok(long_range
        .into_iter()
        .zip(short_range)
        .map(|(long, short)| {
            let a = long.planet_minus_earth_moon_barycenter_j2000_ecliptic_au;
            let b = short.planet_minus_earth_moon_barycenter_j2000_ecliptic_au;
            let dot = a.iter().zip(b).map(|(x, y)| x * y).sum::<f64>();
            let cosine = (dot / (long.emb_relative_distance_au * short.emb_relative_distance_au))
                .clamp(-1.0, 1.0);
            PlanetaryFitOverlapDiagnostic {
                planet: long.planet,
                jd_tt_used_as_tdb_approx: jd_tt,
                long_range_algorithm_version: long.algorithm_version,
                short_range_algorithm_version: short.algorithm_version,
                source_url: JPL_LONG_RANGE_PLANETARY_SOURCE.to_owned(),
                long_range_longitude_deg: long.emb_relative_longitude_deg,
                short_range_longitude_deg: short.emb_relative_longitude_deg,
                angular_disagreement_deg: cosine.acos().to_degrees(),
                distance_disagreement_au: (long.emb_relative_distance_au
                    - short.emb_relative_distance_au)
                    .abs(),
                independent_ephemeris_validation: false,
                historical_mansion_eligible: false,
                divination_rule_eligible: false,
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_five_planets_have_finite_distinct_research_vectors() {
        for jd in [625_500.0, 2_451_545.0, 2_800_000.0, 2_816_975.0] {
            let observations = five_visible_planet_geometric_observations(jd).unwrap();
            assert_eq!(observations.len(), 5);
            for observation in &observations {
                assert!((0.0..360.0).contains(&observation.emb_relative_longitude_deg));
                assert!((-90.0..=90.0).contains(&observation.emb_relative_latitude_deg));
                assert!(observation.emb_relative_distance_au > 0.0);
                for i in 0..3 {
                    assert!(
                        (observation.heliocentric_j2000_ecliptic_au[i]
                            - observation.earth_moon_barycenter_heliocentric_j2000_ecliptic_au[i]
                            - observation.planet_minus_earth_moon_barycenter_j2000_ecliptic_au[i])
                            .abs()
                            < 1.0e-12
                    );
                }
                assert!(!observation.apparent_geocentric_position);
                assert!(!observation.historical_mansion_eligible);
                assert!(!observation.divination_rule_eligible);
            }
        }
    }

    #[test]
    fn j2000_orbital_radii_remain_within_physical_ranges() {
        let observations = five_visible_planet_geometric_observations(2_451_545.0).unwrap();
        let expected_au = [
            (0.30, 0.47),
            (0.71, 0.74),
            (1.38, 1.67),
            (4.9, 5.5),
            (9.0, 10.1),
        ];
        for (observation, (minimum, maximum)) in observations.iter().zip(expected_au) {
            let radius = observation
                .heliocentric_j2000_ecliptic_au
                .iter()
                .map(|value| value * value)
                .sum::<f64>()
                .sqrt();
            assert!((minimum..maximum).contains(&radius));
        }
    }

    #[test]
    fn invalid_and_out_of_range_dates_fail_closed() {
        assert_eq!(
            five_visible_planet_geometric_observations(f64::NAN),
            Err(JplApproxError::NonFiniteJulianDay)
        );
        assert_eq!(
            five_visible_planet_geometric_observations(1.0),
            Err(JplApproxError::OutsideLongRangeTable)
        );
        assert_eq!(
            five_visible_planet_geometric_observations(3_000_000.0),
            Err(JplApproxError::OutsideLongRangeTable)
        );
    }

    #[test]
    fn jupiter_and_saturn_include_required_long_range_anomaly_terms() {
        let t = 8.0;
        for elements in [JUPITER, SATURN] {
            let with_terms = heliocentric_vector(elements, t).unwrap();
            let without_terms = heliocentric_vector(
                Elements {
                    anomaly_extra: [0.0; 4],
                    ..elements
                },
                t,
            )
            .unwrap();
            assert!(with_terms
                .iter()
                .zip(without_terms)
                .any(|(left, right)| (left - right).abs() > 1.0e-6));
        }
    }

    #[test]
    fn overlap_diagnostics_are_finite_and_never_unlock_divination() {
        for jd in [2_378_496.5, 2_451_545.0, 2_460_000.5, 2_469_806.5] {
            let diagnostics = five_visible_planet_fit_overlap_diagnostics(jd).unwrap();
            assert_eq!(diagnostics.len(), 5);
            assert!(diagnostics
                .iter()
                .any(|d| d.angular_disagreement_deg > 0.001));
            for diagnostic in diagnostics {
                assert!(diagnostic.angular_disagreement_deg.is_finite());
                assert!((0.0..=180.0).contains(&diagnostic.angular_disagreement_deg));
                assert!(diagnostic.distance_disagreement_au.is_finite());
                assert!(diagnostic.distance_disagreement_au >= 0.0);
                assert_eq!(
                    diagnostic.long_range_algorithm_version,
                    JPL_LONG_RANGE_PLANETARY_ALGORITHM_VERSION
                );
                assert_eq!(
                    diagnostic.short_range_algorithm_version,
                    JPL_SHORT_RANGE_PLANETARY_ALGORITHM_VERSION
                );
                assert_eq!(diagnostic.source_url, JPL_LONG_RANGE_PLANETARY_SOURCE);
                assert!(!diagnostic.independent_ephemeris_validation);
                assert!(!diagnostic.historical_mansion_eligible);
                assert!(!diagnostic.divination_rule_eligible);
            }
        }
    }

    #[test]
    fn overlap_diagnostics_reject_dates_outside_short_fit() {
        for jd in [2_378_496.0, 2_469_807.5, f64::INFINITY] {
            assert!(five_visible_planet_fit_overlap_diagnostics(jd).is_err());
        }
    }

    #[test]
    fn j2000_saturn_two_fit_gap_is_pinned_as_regression_not_truth() {
        let diagnostics = five_visible_planet_fit_overlap_diagnostics(2_451_545.0).unwrap();
        let saturn = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.planet == VisiblePlanet::Saturn)
            .unwrap();
        assert!((saturn.angular_disagreement_deg - 0.189275691918542).abs() < 1.0e-7);
        assert!(!saturn.independent_ephemeris_validation);
    }
}
