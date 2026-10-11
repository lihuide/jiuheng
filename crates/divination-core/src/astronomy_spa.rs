                                                                            
                                 
   
                                                                              
                                                                           
                                                                             
                                                

                                                                          
                                                                           
#![allow(clippy::approx_constant)]

use crate::astronomy::{
    normalize_deg, AstronomyError, AstronomyPrecision, EclipticPosition, EphemerisProvider,
};
use serde::{Deserialize, Serialize};

pub const SPA_REIMPLEMENTATION_VERSION: &str = "jiuheng.sun.spa_report.v1";
pub const SPA_EQUATION_OF_TIME_VERSION: &str = "jiuheng.equation_of_time.spa_report_appendix_a.v1";
pub const CROSS_CHECK_SOURCE_BLOB_SHA: &str = "7f2c3e6ffafce096a8038f0610119927cbbb8d35";

                                                                           
                                                                          
const MIN_JD_TT: f64 = 990_575.0;
const MAX_JD_TT: f64 = 3_912_515.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpaSolarCoordinates {
    pub apparent_longitude_deg: f64,
    pub geocentric_latitude_deg: f64,
    pub distance_au: f64,
    pub nutation_longitude_deg: f64,
    pub nutation_obliquity_deg: f64,
    pub true_obliquity_deg: f64,
    pub right_ascension_deg: f64,
    pub declination_deg: f64,
                                                                    
    pub equation_of_time_minutes: f64,
}

#[derive(Debug, Default)]
pub struct SpaSolarEphemeris;

impl SpaSolarEphemeris {
    pub fn calculate(&self, jd_tt: f64) -> Result<SpaSolarCoordinates, AstronomyError> {
        if !jd_tt.is_finite() {
            return Err(AstronomyError::NonFiniteInput);
        }
        if !(MIN_JD_TT..=MAX_JD_TT).contains(&jd_tt) {
            return Err(AstronomyError::DateOutsideAlgorithmRange { jd_tt });
        }

        let jce = (jd_tt - 2_451_545.0) / 36_525.0;
        let jme = jce / 10.0;
        let heliocentric_longitude_deg =
            normalize_deg(calculate_lbr_polynomial(jme, TERMS_L).to_degrees());
        let heliocentric_latitude_deg = calculate_lbr_polynomial(jme, TERMS_B).to_degrees();
        let distance_au = calculate_lbr_polynomial(jme, TERMS_R);

        let geocentric_longitude_deg = normalize_deg(heliocentric_longitude_deg + 180.0);
        let geocentric_latitude_deg = -heliocentric_latitude_deg;

        let x = calculate_nutation_terms(jce);
        let (nutation_longitude_deg, nutation_obliquity_deg) = calculate_nutation(jce, &x);
        let mean_obliquity_arcsec = polynomial(jme / 10.0, &OBLIQUITY_COEFFS);
        let true_obliquity_deg = mean_obliquity_arcsec / 3_600.0 + nutation_obliquity_deg;
        let aberration_deg = -20.4898 / (3_600.0 * distance_au);
        let apparent_longitude_deg =
            normalize_deg(geocentric_longitude_deg + nutation_longitude_deg + aberration_deg);

        let beta = geocentric_latitude_deg.to_radians();
        let epsilon = true_obliquity_deg.to_radians();
        let lambda = apparent_longitude_deg.to_radians();
        let right_ascension_deg = normalize_deg(
            (lambda.sin() * epsilon.cos() - beta.tan() * epsilon.sin())
                .atan2(lambda.cos())
                .to_degrees(),
        );
        let declination_deg = (beta.sin() * epsilon.cos()
            + beta.cos() * epsilon.sin() * lambda.sin())
        .asin()
        .to_degrees();
        let equation_of_time_minutes = calculate_equation_of_time_minutes(
            jme,
            right_ascension_deg,
            nutation_longitude_deg,
            true_obliquity_deg,
        );

        Ok(SpaSolarCoordinates {
            apparent_longitude_deg,
            geocentric_latitude_deg,
            distance_au,
            nutation_longitude_deg,
            nutation_obliquity_deg,
            true_obliquity_deg,
            right_ascension_deg,
            declination_deg,
            equation_of_time_minutes,
        })
    }
}

                                                    
fn calculate_equation_of_time_minutes(
    jme: f64,
    right_ascension_deg: f64,
    nutation_longitude_deg: f64,
    true_obliquity_deg: f64,
) -> f64 {
    let sun_mean_longitude_deg = normalize_deg(
        280.466_456_7
            + 360_007.698_277_9 * jme
            + 0.030_320_28 * jme.powi(2)
            + jme.powi(3) / 49_931.0
            - jme.powi(4) / 15_300.0
            - jme.powi(5) / 2_000_000.0,
    );
    let equation_deg = normalize_deg(
        sun_mean_longitude_deg - 0.005_718_3 - right_ascension_deg
            + nutation_longitude_deg * true_obliquity_deg.to_radians().cos(),
    );
    let mut minutes = equation_deg * 4.0;
    if minutes > 20.0 {
        minutes -= 1_440.0;
    } else if minutes < -20.0 {
        minutes += 1_440.0;
    }
    minutes
}

impl EphemerisProvider for SpaSolarEphemeris {
    fn sun_geocentric_ecliptic(&self, jd_tt: f64) -> Result<EclipticPosition, AstronomyError> {
        let coordinates = self.calculate(jd_tt)?;
        Ok(EclipticPosition {
            longitude_deg: coordinates.apparent_longitude_deg,
            latitude_deg: coordinates.geocentric_latitude_deg,
            distance_au: coordinates.distance_au,
            precision: AstronomyPrecision::SpaReferenceVectorMatched,
                                                                               
                                                                              
            estimated_error_arcmin: 0.1,
            algorithm_version: SPA_REIMPLEMENTATION_VERSION.to_owned(),
        })
    }
}

fn calculate_lbr_polynomial(jme: f64, series: &[&[[f64; 3]]]) -> f64 {
    let sums: Vec<f64> = series
        .iter()
        .map(|terms| {
            terms.iter().fold(0.0, |sum, term| {
                term[0].mul_add((term[1] + term[2] * jme).cos(), sum)
            })
        })
        .collect();
    polynomial(jme, &sums) / 100_000_000.0
}

fn calculate_nutation_terms(jce: f64) -> [f64; 5] {
    NUTATION_COEFFICIENTS.map(|coefficients| polynomial(jce, &coefficients))
}

fn calculate_nutation(jce: f64, x: &[f64; 5]) -> (f64, f64) {
    let mut delta_psi = 0.0;
    let mut delta_epsilon = 0.0;
    for (y, pe) in TERMS_Y.iter().zip(TERMS_PE.iter()) {
        let argument = x
            .iter()
            .zip(y.iter())
            .fold(0.0, |sum, (x_value, multiplier)| {
                x_value.mul_add(*multiplier, sum)
            })
            .to_radians();
        delta_psi = pe[1].mul_add(jce, pe[0]).mul_add(argument.sin(), delta_psi);
        delta_epsilon = pe[3]
            .mul_add(jce, pe[2])
            .mul_add(argument.cos(), delta_epsilon);
    }
    (delta_psi / 36_000_000.0, delta_epsilon / 36_000_000.0)
}

fn polynomial(x: f64, coefficients: &[f64]) -> f64 {
    coefficients
        .iter()
        .rev()
        .fold(0.0, |sum, coefficient| x.mul_add(sum, *coefficient))
}

const L0: &[[f64; 3]] = &[
    [175347046.0, 0.0, 0.0],
    [3341656.0, 4.6692568, 6283.07585],
    [34894.0, 4.6261, 12566.1517],
    [3497.0, 2.7441, 5753.3849],
    [3418.0, 2.8289, 3.5231],
    [3136.0, 3.6277, 77713.7715],
    [2676.0, 4.4181, 7860.4194],
    [2343.0, 6.1352, 3930.2097],
    [1324.0, 0.7425, 11506.7698],
    [1273.0, 2.0371, 529.691],
    [1199.0, 1.1096, 1577.3435],
    [990.0, 5.233, 5884.927],
    [902.0, 2.045, 26.298],
    [857.0, 3.508, 398.149],
    [780.0, 1.179, 5223.694],
    [753.0, 2.533, 5507.553],
    [505.0, 4.583, 18849.228],
    [492.0, 4.205, 775.523],
    [357.0, 2.92, 0.067],
    [317.0, 5.849, 11790.629],
    [284.0, 1.899, 796.298],
    [271.0, 0.315, 10977.079],
    [243.0, 0.345, 5486.778],
    [206.0, 4.806, 2544.314],
    [205.0, 1.869, 5573.143],
    [202.0, 2.458, 6069.777],
    [156.0, 0.833, 213.299],
    [132.0, 3.411, 2942.463],
    [126.0, 1.083, 20.775],
    [115.0, 0.645, 0.98],
    [103.0, 0.636, 4694.003],
    [102.0, 0.976, 15720.839],
    [102.0, 4.267, 7.114],
    [99.0, 6.21, 2146.17],
    [98.0, 0.68, 155.42],
    [86.0, 5.98, 161000.69],
    [85.0, 1.3, 6275.96],
    [85.0, 3.67, 71430.7],
    [80.0, 1.81, 17260.15],
    [79.0, 3.04, 12036.46],
    [75.0, 1.76, 5088.63],
    [74.0, 3.5, 3154.69],
    [74.0, 4.68, 801.82],
    [70.0, 0.83, 9437.76],
    [62.0, 3.98, 8827.39],
    [61.0, 1.82, 7084.9],
    [57.0, 2.78, 6286.6],
    [56.0, 4.39, 14143.5],
    [56.0, 3.47, 6279.55],
    [52.0, 0.19, 12139.55],
    [52.0, 1.33, 1748.02],
    [51.0, 0.28, 5856.48],
    [49.0, 0.49, 1194.45],
    [41.0, 5.37, 8429.24],
    [41.0, 2.4, 19651.05],
    [39.0, 6.17, 10447.39],
    [37.0, 6.04, 10213.29],
    [37.0, 2.57, 1059.38],
    [36.0, 1.71, 2352.87],
    [36.0, 1.78, 6812.77],
    [33.0, 0.59, 17789.85],
    [30.0, 0.44, 83996.85],
    [30.0, 2.74, 1349.87],
    [25.0, 3.16, 4690.48],
];

const L1: &[[f64; 3]] = &[
    [628331966747.0, 0.0, 0.0],
    [206059.0, 2.678235, 6283.07585],
    [4303.0, 2.6351, 12566.1517],
    [425.0, 1.59, 3.523],
    [119.0, 5.796, 26.298],
    [109.0, 2.966, 1577.344],
    [93.0, 2.59, 18849.23],
    [72.0, 1.14, 529.69],
    [68.0, 1.87, 398.15],
    [67.0, 4.41, 5507.55],
    [59.0, 2.89, 5223.69],
    [56.0, 2.17, 155.42],
    [45.0, 0.4, 796.3],
    [36.0, 0.47, 775.52],
    [29.0, 2.65, 7.11],
    [21.0, 5.34, 0.98],
    [19.0, 1.85, 5486.78],
    [19.0, 4.97, 213.3],
    [17.0, 2.99, 6275.96],
    [16.0, 0.03, 2544.31],
    [16.0, 1.43, 2146.17],
    [15.0, 1.21, 10977.08],
    [12.0, 2.83, 1748.02],
    [12.0, 3.26, 5088.63],
    [12.0, 5.27, 1194.45],
    [12.0, 2.08, 4694.0],
    [11.0, 0.77, 553.57],
    [10.0, 1.3, 6286.6],
    [10.0, 4.24, 1349.87],
    [9.0, 2.7, 242.73],
    [9.0, 5.64, 951.72],
    [8.0, 5.3, 2352.87],
    [6.0, 2.65, 9437.76],
    [6.0, 4.67, 4690.48],
];

const L2: &[[f64; 3]] = &[
    [52919.0, 0.0, 0.0],
    [8720.0, 1.0721, 6283.0758],
    [309.0, 0.867, 12566.152],
    [27.0, 0.05, 3.52],
    [16.0, 5.19, 26.3],
    [16.0, 3.68, 155.42],
    [10.0, 0.76, 18849.23],
    [9.0, 2.06, 77713.77],
    [7.0, 0.83, 775.52],
    [5.0, 4.66, 1577.34],
    [4.0, 1.03, 7.11],
    [4.0, 3.44, 5573.14],
    [3.0, 5.14, 796.3],
    [3.0, 6.05, 5507.55],
    [3.0, 1.19, 242.73],
    [3.0, 6.12, 529.69],
    [3.0, 0.31, 398.15],
    [3.0, 2.28, 553.57],
    [2.0, 4.38, 5223.69],
    [2.0, 3.75, 0.98],
];
const L3: &[[f64; 3]] = &[
    [289.0, 5.844, 6283.076],
    [35.0, 0.0, 0.0],
    [17.0, 5.49, 12566.15],
    [3.0, 5.2, 155.42],
    [1.0, 4.72, 3.52],
    [1.0, 5.3, 18849.23],
    [1.0, 5.97, 242.73],
];
const L4: &[[f64; 3]] = &[
    [114.0, 3.142, 0.0],
    [8.0, 4.13, 6283.08],
    [1.0, 3.84, 12566.15],
];
const L5: &[[f64; 3]] = &[[1.0, 3.14, 0.0]];
const TERMS_L: &[&[[f64; 3]]] = &[L0, L1, L2, L3, L4, L5];

const B0: &[[f64; 3]] = &[
    [280.0, 3.199, 84334.662],
    [102.0, 5.422, 5507.553],
    [80.0, 3.88, 5223.69],
    [44.0, 3.7, 2352.87],
    [32.0, 4.0, 1577.34],
];
const B1: &[[f64; 3]] = &[[9.0, 3.9, 5507.55], [6.0, 1.73, 5223.69]];
const TERMS_B: &[&[[f64; 3]]] = &[B0, B1];

const R0: &[[f64; 3]] = &[
    [100013989.0, 0.0, 0.0],
    [1670700.0, 3.0984635, 6283.07585],
    [13956.0, 3.05525, 12566.1517],
    [3084.0, 5.1985, 77713.7715],
    [1628.0, 1.1739, 5753.3849],
    [1576.0, 2.8469, 7860.4194],
    [925.0, 5.453, 11506.77],
    [542.0, 4.564, 3930.21],
    [472.0, 3.661, 5884.927],
    [346.0, 0.964, 5507.553],
    [329.0, 5.9, 5223.694],
    [307.0, 0.299, 5573.143],
    [243.0, 4.273, 11790.629],
    [212.0, 5.847, 1577.344],
    [186.0, 5.022, 10977.079],
    [175.0, 3.012, 18849.228],
    [110.0, 5.055, 5486.778],
    [98.0, 0.89, 6069.78],
    [86.0, 5.69, 15720.84],
    [86.0, 1.27, 161000.69],
    [65.0, 0.27, 17260.15],
    [63.0, 0.92, 529.69],
    [57.0, 2.01, 83996.85],
    [56.0, 5.24, 71430.7],
    [49.0, 3.25, 2544.31],
    [47.0, 2.58, 775.52],
    [45.0, 5.54, 9437.76],
    [43.0, 6.01, 6275.96],
    [39.0, 5.36, 4694.0],
    [38.0, 2.39, 8827.39],
    [37.0, 0.83, 19651.05],
    [37.0, 4.9, 12139.55],
    [36.0, 1.67, 12036.46],
    [35.0, 1.84, 2942.46],
    [33.0, 0.24, 7084.9],
    [32.0, 0.18, 5088.63],
    [32.0, 1.78, 398.15],
    [28.0, 1.21, 6286.6],
    [28.0, 1.9, 6279.55],
    [26.0, 4.59, 10447.39],
];
const R1: &[[f64; 3]] = &[
    [103019.0, 1.10749, 6283.07585],
    [1721.0, 1.0644, 12566.1517],
    [702.0, 3.142, 0.0],
    [32.0, 1.02, 18849.23],
    [31.0, 2.84, 5507.55],
    [25.0, 1.32, 5223.69],
    [18.0, 1.42, 1577.34],
    [10.0, 5.91, 10977.08],
    [9.0, 1.42, 6275.96],
    [9.0, 0.27, 5486.78],
];
const R2: &[[f64; 3]] = &[
    [4359.0, 5.7846, 6283.0758],
    [124.0, 5.579, 12566.152],
    [12.0, 3.14, 0.0],
    [9.0, 3.63, 77713.77],
    [6.0, 1.87, 5573.14],
    [3.0, 5.47, 18849.23],
];
const R3: &[[f64; 3]] = &[[145.0, 4.273, 6283.076], [7.0, 3.92, 12566.15]];
const R4: &[[f64; 3]] = &[[4.0, 2.56, 6283.08]];
const TERMS_R: &[&[[f64; 3]]] = &[R0, R1, R2, R3, R4];

const NUTATION_COEFFICIENTS: [[f64; 4]; 5] = [
    [297.85036, 445267.111480, -0.0019142, 1.0 / 189474.0],
    [357.52772, 35999.050340, -0.0001603, -1.0 / 300000.0],
    [134.96298, 477198.867398, 0.0086972, 1.0 / 56250.0],
    [93.27191, 483202.017538, -0.0036825, 1.0 / 327270.0],
    [125.04452, -1934.136261, 0.0020708, 1.0 / 450000.0],
];

const TERMS_Y: [[f64; 5]; 63] = [
    [0., 0., 0., 0., 1.],
    [-2., 0., 0., 2., 2.],
    [0., 0., 0., 2., 2.],
    [0., 0., 0., 0., 2.],
    [0., 1., 0., 0., 0.],
    [0., 0., 1., 0., 0.],
    [-2., 1., 0., 2., 2.],
    [0., 0., 0., 2., 1.],
    [0., 0., 1., 2., 2.],
    [-2., -1., 0., 2., 2.],
    [-2., 0., 1., 0., 0.],
    [-2., 0., 0., 2., 1.],
    [0., 0., -1., 2., 2.],
    [2., 0., 0., 0., 0.],
    [0., 0., 1., 0., 1.],
    [2., 0., -1., 2., 2.],
    [0., 0., -1., 0., 1.],
    [0., 0., 1., 2., 1.],
    [-2., 0., 2., 0., 0.],
    [0., 0., -2., 2., 1.],
    [2., 0., 0., 2., 2.],
    [0., 0., 2., 2., 2.],
    [0., 0., 2., 0., 0.],
    [-2., 0., 1., 2., 2.],
    [0., 0., 0., 2., 0.],
    [-2., 0., 0., 2., 0.],
    [0., 0., -1., 2., 1.],
    [0., 2., 0., 0., 0.],
    [2., 0., -1., 0., 1.],
    [-2., 2., 0., 2., 2.],
    [0., 1., 0., 0., 1.],
    [-2., 0., 1., 0., 1.],
    [0., -1., 0., 0., 1.],
    [0., 0., 2., -2., 0.],
    [2., 0., -1., 2., 1.],
    [2., 0., 1., 2., 2.],
    [0., 1., 0., 2., 2.],
    [-2., 1., 1., 0., 0.],
    [0., -1., 0., 2., 2.],
    [2., 0., 0., 2., 1.],
    [2., 0., 1., 0., 0.],
    [-2., 0., 2., 2., 2.],
    [-2., 0., 1., 2., 1.],
    [2., 0., -2., 0., 1.],
    [2., 0., 0., 0., 1.],
    [0., -1., 1., 0., 0.],
    [-2., -1., 0., 2., 1.],
    [-2., 0., 0., 0., 1.],
    [0., 0., 2., 2., 1.],
    [-2., 0., 2., 0., 1.],
    [-2., 1., 0., 2., 1.],
    [0., 0., 1., -2., 0.],
    [-1., 0., 1., 0., 0.],
    [-2., 1., 0., 0., 0.],
    [1., 0., 0., 0., 0.],
    [0., 0., 1., 2., 0.],
    [0., 0., -2., 2., 2.],
    [-1., -1., 1., 0., 0.],
    [0., 1., 1., 0., 0.],
    [0., -1., 1., 2., 2.],
    [2., -1., -1., 2., 2.],
    [0., 0., 3., 2., 2.],
    [2., -1., 0., 2., 2.],
];

const TERMS_PE: [[f64; 4]; 63] = [
    [-171996., -174.2, 92025., 8.9],
    [-13187., -1.6, 5736., -3.1],
    [-2274., -0.2, 977., -0.5],
    [2062., 0.2, -895., 0.5],
    [1426., -3.4, 54., -0.1],
    [712., 0.1, -7., 0.],
    [-517., 1.2, 224., -0.6],
    [-386., -0.4, 200., 0.],
    [-301., 0., 129., -0.1],
    [217., -0.5, -95., 0.3],
    [-158., 0., 0., 0.],
    [129., 0.1, -70., 0.],
    [123., 0., -53., 0.],
    [63., 0., 0., 0.],
    [63., 0.1, -33., 0.],
    [-59., 0., 26., 0.],
    [-58., -0.1, 32., 0.],
    [-51., 0., 27., 0.],
    [48., 0., 0., 0.],
    [46., 0., -24., 0.],
    [-38., 0., 16., 0.],
    [-31., 0., 13., 0.],
    [29., 0., 0., 0.],
    [29., 0., -12., 0.],
    [26., 0., 0., 0.],
    [-22., 0., 0., 0.],
    [21., 0., -10., 0.],
    [17., -0.1, 0., 0.],
    [16., 0., -8., 0.],
    [-16., 0.1, 7., 0.],
    [-15., 0., 9., 0.],
    [-13., 0., 7., 0.],
    [-12., 0., 6., 0.],
    [11., 0., 0., 0.],
    [-10., 0., 5., 0.],
    [-8., 0., 3., 0.],
    [7., 0., -3., 0.],
    [-7., 0., 0., 0.],
    [-7., 0., 3., 0.],
    [-7., 0., 3., 0.],
    [6., 0., 0., 0.],
    [6., 0., -3., 0.],
    [6., 0., -3., 0.],
    [-6., 0., 3., 0.],
    [-6., 0., 3., 0.],
    [5., 0., 0., 0.],
    [-5., 0., 3., 0.],
    [-5., 0., 3., 0.],
    [-5., 0., 3., 0.],
    [4., 0., 0., 0.],
    [4., 0., 0., 0.],
    [4., 0., 0., 0.],
    [-4., 0., 0., 0.],
    [-4., 0., 0., 0.],
    [-4., 0., 0., 0.],
    [3., 0., 0., 0.],
    [-3., 0., 0., 0.],
    [-3., 0., 0., 0.],
    [-3., 0., 0., 0.],
    [-3., 0., 0., 0.],
    [-3., 0., 0., 0.],
    [-3., 0., 0., 0.],
    [-3., 0., 0., 0.],
];

const OBLIQUITY_COEFFS: [f64; 11] = [
    84381.448, -4680.93, -1.55, 1999.25, 51.38, -249.67, -39.05, 7.12, 27.87, 5.79, 2.45,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_published_spa_reference_vector() {
                                                                       
        let jd_tt = 2_452_930.312_847 + 67.0 / 86_400.0;
        let coordinates = SpaSolarEphemeris.calculate(jd_tt).unwrap();
                                                                             
                                                                               
        assert!(
            (coordinates.apparent_longitude_deg - 204.008_551_928_1).abs() < 3.0e-7,
            "coordinates={coordinates:?}"
        );
        assert!((coordinates.geocentric_latitude_deg - 0.000_101_121_9).abs() < 1.0e-8);
        assert!((coordinates.distance_au - 0.996_542_297_4).abs() < 1.0e-9);
        assert!((coordinates.nutation_longitude_deg - -0.003_998_40).abs() < 1.0e-8);
        assert!((coordinates.nutation_obliquity_deg - 0.001_666_57).abs() < 1.0e-8);
        assert!((coordinates.true_obliquity_deg - 23.440_465).abs() < 1.0e-6);
        assert!(
            (coordinates.equation_of_time_minutes - 14.641_510_770_8).abs() < 1.0e-7,
            "coordinates={coordinates:?}"
        );
    }

    #[test]
    fn rejects_non_finite_and_out_of_range_dates() {
        assert!(matches!(
            SpaSolarEphemeris.calculate(f64::NAN),
            Err(AstronomyError::NonFiniteInput)
        ));
        assert!(matches!(
            SpaSolarEphemeris.calculate(500_000.0),
            Err(AstronomyError::DateOutsideAlgorithmRange { .. })
        ));
    }
}
