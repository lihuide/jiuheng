                                                                           
                                                                                
   
                                                                            
                                                                         
                                                                             
                                                               

use crate::{
    EarthlyBranch, HeavenlyStem, SexagenaryDesignation, SolarTerm, SolarTermInterval, Trigram,
};
use chrono::{DateTime, FixedOffset, Timelike};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const QIMEN_JU_CANDIDATE_ALGORITHM_VERSION: &str =
    "jiuheng.qimen.received_solar_term_three_yuan_ju_candidate.v1";
pub const QIMEN_JU_CANDIDATE_STATUS: &str =
    "mechanical_ju_and_earth_plate_candidate_no_heaven_plate_stars_gates_spirits_or_interpretation.v2";
pub const QIMEN_EARTH_PLATE_ALGORITHM_VERSION: &str =
    "jiuheng.qimen.received_three_wonders_six_instruments_earth_plate.v1";
pub const QIMEN_EARTH_PLATE_STATUS: &str =
    "mechanical_earth_plate_only_no_heaven_plate_stars_gates_spirits_or_interpretation.v1";
pub const QIMEN_HOUR_XUN_CANDIDATE_ALGORITHM_VERSION: &str =
    "jiuheng.qimen.hour_xun_head_hidden_instrument_earth_palace.v1";
pub const QIMEN_HOUR_XUN_CANDIDATE_STATUS: &str =
    "mechanical_hour_xun_head_hidden_instrument_and_earth_palace_only_no_duty_chief_or_envoy.v1";
pub const QIMEN_JU_HOUR_CANDIDATE_STATUS: &str =
    "mechanical_ju_earth_plate_and_hour_xun_head_candidate_no_duty_chief_envoy_heaven_plate_stars_gates_spirits_or_interpretation.v3";
pub const QIMEN_DUTY_IDENTITY_ALGORITHM_VERSION: &str =
    "jiuheng.qimen.early_song_fixed_star_gate_duty_identity.v1";
pub const QIMEN_DUTY_IDENTITY_STATUS: &str =
    "mechanical_duty_chief_star_and_duty_envoy_gate_identity_only_no_hourly_movement_or_interpretation.v1";
pub const QIMEN_HOUR_DUTY_IDENTITY_STATUS: &str =
    "mechanical_hour_xun_head_with_fixed_duty_chief_and_envoy_identity_no_hourly_movement.v2";
pub const QIMEN_JU_DUTY_IDENTITY_STATUS: &str =
    "mechanical_ju_earth_plate_hour_xun_and_fixed_duty_identity_no_hourly_movement_heaven_plate_spirits_or_interpretation.v4";
pub const QIMEN_CENTER_HOSTING_CONVENTION: &str = "early_song_center_five_hosts_kun_two.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QimenDun {
    Yang,
    Yin,
}

impl QimenDun {
    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Yang => "阳遁",
            Self::Yin => "阴遁",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QimenThreeYuan {
    Upper,
    Middle,
    Lower,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QimenPalace {
    KanOne,
    KunTwo,
    ZhenThree,
    XunFour,
    CenterFive,
    QianSix,
    DuiSeven,
    GenEight,
    LiNine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QimenStar {
    Tianpeng,
    Tianrui,
    Tianchong,
    Tianfu,
    Tianqin,
    Tianxin,
    Tianzhu,
    Tianren,
    Tianying,
}

impl QimenStar {
    pub const ALL_IN_PALACE_ORDER: [Self; 9] = [
        Self::Tianpeng,
        Self::Tianrui,
        Self::Tianchong,
        Self::Tianfu,
        Self::Tianqin,
        Self::Tianxin,
        Self::Tianzhu,
        Self::Tianren,
        Self::Tianying,
    ];

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Tianpeng => "天蓬",
            Self::Tianrui => "天芮",
            Self::Tianchong => "天冲",
            Self::Tianfu => "天辅",
            Self::Tianqin => "天禽",
            Self::Tianxin => "天心",
            Self::Tianzhu => "天柱",
            Self::Tianren => "天任",
            Self::Tianying => "天英",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QimenGate {
    Rest,
    Death,
    Harm,
    Block,
    Open,
    Alarm,
    Life,
    View,
}

impl QimenGate {
    pub const ALL_IN_RECEIVED_ORDER: [Self; 8] = [
        Self::Open,
        Self::Rest,
        Self::Life,
        Self::Harm,
        Self::Block,
        Self::View,
        Self::Death,
        Self::Alarm,
    ];

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Rest => "休门",
            Self::Death => "死门",
            Self::Harm => "伤门",
            Self::Block => "杜门",
            Self::Open => "开门",
            Self::Alarm => "惊门",
            Self::Life => "生门",
            Self::View => "景门",
        }
    }
}

impl QimenPalace {
    pub const ALL_IN_NUMBER_ORDER: [Self; 9] = [
        Self::KanOne,
        Self::KunTwo,
        Self::ZhenThree,
        Self::XunFour,
        Self::CenterFive,
        Self::QianSix,
        Self::DuiSeven,
        Self::GenEight,
        Self::LiNine,
    ];

    pub const fn number(self) -> u8 {
        match self {
            Self::KanOne => 1,
            Self::KunTwo => 2,
            Self::ZhenThree => 3,
            Self::XunFour => 4,
            Self::CenterFive => 5,
            Self::QianSix => 6,
            Self::DuiSeven => 7,
            Self::GenEight => 8,
            Self::LiNine => 9,
        }
    }

    pub const fn trigram(self) -> Option<Trigram> {
        match self {
            Self::KanOne => Some(Trigram::Kan),
            Self::KunTwo => Some(Trigram::Kun),
            Self::ZhenThree => Some(Trigram::Zhen),
            Self::XunFour => Some(Trigram::Xun),
            Self::CenterFive => None,
            Self::QianSix => Some(Trigram::Qian),
            Self::DuiSeven => Some(Trigram::Dui),
            Self::GenEight => Some(Trigram::Gen),
            Self::LiNine => Some(Trigram::Li),
        }
    }

    pub const fn fixed_star(self) -> QimenStar {
        match self {
            Self::KanOne => QimenStar::Tianpeng,
            Self::KunTwo => QimenStar::Tianrui,
            Self::ZhenThree => QimenStar::Tianchong,
            Self::XunFour => QimenStar::Tianfu,
            Self::CenterFive => QimenStar::Tianqin,
            Self::QianSix => QimenStar::Tianxin,
            Self::DuiSeven => QimenStar::Tianzhu,
            Self::GenEight => QimenStar::Tianren,
            Self::LiNine => QimenStar::Tianying,
        }
    }

    pub const fn fixed_gate(self) -> Option<QimenGate> {
        match self {
            Self::KanOne => Some(QimenGate::Rest),
            Self::KunTwo => Some(QimenGate::Death),
            Self::ZhenThree => Some(QimenGate::Harm),
            Self::XunFour => Some(QimenGate::Block),
            Self::CenterFive => None,
            Self::QianSix => Some(QimenGate::Open),
            Self::DuiSeven => Some(QimenGate::Alarm),
            Self::GenEight => Some(QimenGate::Life),
            Self::LiNine => Some(QimenGate::View),
        }
    }

    pub const fn early_song_duty_envoy_host(self) -> Self {
        match self {
            Self::CenterFive => Self::KunTwo,
            other => other,
        }
    }

    pub fn compact_zh(self) -> String {
        self.trigram().map_or_else(
            || "中五宫".to_owned(),
            |trigram| format!("{}{}宫", trigram.zh_name(), self.number()),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QimenEarthPlateCell {
    pub palace: QimenPalace,
    pub instrument_or_wonder: HeavenlyStem,
    pub hidden_jia: Option<SexagenaryDesignation>,
}

impl QimenEarthPlateCell {
    pub fn compact_zh(&self) -> String {
        let hidden = self.hidden_jia.map_or_else(String::new, |designation| {
            format!("隐{}", designation.zh_name())
        });
        format!(
            "{}={}{}",
            self.palace.compact_zh(),
            self.instrument_or_wonder.zh_name(),
            hidden
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QimenEarthPlate {
    pub audit_version: u16,
    pub algorithm_version: String,
    pub status: String,
    pub dun: QimenDun,
    pub bureau_number: u8,
                                                                          
                        
    pub cells_by_palace_number: Vec<QimenEarthPlateCell>,
    pub earth_plate_complete: bool,
    pub interpretation_eligible: bool,
    pub rule_eligible: bool,
}

impl QimenEarthPlate {
    pub fn compact_zh(&self) -> String {
        self.cells_by_palace_number
            .iter()
            .map(QimenEarthPlateCell::compact_zh)
            .collect::<Vec<_>>()
            .join(" · ")
    }
}

impl QimenThreeYuan {
    pub const ALL: [Self; 3] = [Self::Upper, Self::Middle, Self::Lower];

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Upper => "上元",
            Self::Middle => "中元",
            Self::Lower => "下元",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Upper => 0,
            Self::Middle => 1,
            Self::Lower => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QimenDayConventionCandidate {
    pub day_convention: String,
    pub day_designation: SexagenaryDesignation,
    pub symbol_head: SexagenaryDesignation,
    pub yuan: QimenThreeYuan,
    pub bureau_number: Option<u8>,
    #[serde(default)]
    pub earth_plate: Option<QimenEarthPlate>,
    #[serde(default)]
    pub hour_xun_candidate: Option<QimenHourXunCandidate>,
    pub day_boundary_safe: bool,
    pub mechanical_boundaries_safe: bool,
}

impl QimenDayConventionCandidate {
    pub fn compact_zh(&self, dun: QimenDun) -> String {
        let bureau = self.bureau_number.map_or_else(
            || "局数停用".to_owned(),
            |number| format!("{}{}局", dun.zh_name(), number),
        );
        format!(
            "{}（符头{}）· {} · {} · {}",
            self.day_designation.zh_name(),
            self.symbol_head.zh_name(),
            self.yuan.zh_name(),
            bureau,
            if self.day_boundary_safe {
                "日界安全"
            } else {
                "日界不安全"
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QimenHourXunCandidate {
    pub audit_version: u16,
    pub algorithm_version: String,
    pub status: String,
    pub time_convention: String,
    pub hour_designation: SexagenaryDesignation,
    pub xun_head: SexagenaryDesignation,
    pub hidden_instrument: HeavenlyStem,
    pub xun_head_palace: Option<QimenPalace>,
    #[serde(default)]
    pub duty_identity_candidate: Option<QimenDutyIdentityCandidate>,
    pub hour_boundary_safe: bool,
    pub mechanical_boundaries_safe: bool,
    pub interpretation_eligible: bool,
    pub rule_eligible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QimenDutyIdentityCandidate {
    pub audit_version: u16,
    pub algorithm_version: String,
    pub status: String,
    pub xun_head_palace: QimenPalace,
    pub duty_chief_star: QimenStar,
    pub duty_envoy_gate: QimenGate,
    pub duty_envoy_source_palace: QimenPalace,
    pub center_hosting_applied: bool,
    pub center_hosting_convention: Option<String>,
    pub mechanical_boundaries_safe: bool,
    pub duty_chief_hourly_palace: Option<QimenPalace>,
    pub duty_envoy_hourly_palace: Option<QimenPalace>,
    pub heaven_plate_calculated: bool,
    pub interpretation_eligible: bool,
    pub rule_eligible: bool,
}

impl QimenDutyIdentityCandidate {
    pub fn compact_zh(&self) -> String {
        let hosting = if self.center_hosting_applied {
            format!(" · 中五寄{}", self.duty_envoy_source_palace.compact_zh())
        } else {
            String::new()
        };
        format!(
            "值符{} · 值使{}{} · 仅本旬身份，未逐时飞转",
            self.duty_chief_star.zh_name(),
            self.duty_envoy_gate.zh_name(),
            hosting
        )
    }
}

impl QimenHourXunCandidate {
    pub fn compact_zh(&self) -> String {
        let palace = self.xun_head_palace.map_or_else(
            || "旬首落宫未生成".to_owned(),
            |palace| format!("旬首落{}", palace.compact_zh()),
        );
        format!(
            "{} · 旬首{}隐{} · {} · {}",
            self.hour_designation.zh_name(),
            self.xun_head.zh_name(),
            self.hidden_instrument.zh_name(),
            palace,
            if self.hour_boundary_safe {
                "时界安全"
            } else {
                "时界不安全"
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QimenJuAudit {
    pub audit_version: u16,
    pub algorithm_version: String,
    pub status: String,
    pub active_solar_term: SolarTerm,
    pub next_solar_term: SolarTerm,
    pub solar_term_boundary_safe: bool,
    pub dun: QimenDun,
    pub received_bureau_numbers: Option<[u8; 3]>,
    pub table_entry_evidence_conflict: bool,
    pub beijing_midnight_candidate: QimenDayConventionCandidate,
    pub true_solar_midnight_candidate: QimenDayConventionCandidate,
    pub candidates_identical: bool,
    pub selected_day_convention: Option<String>,
    #[serde(default)]
    pub selected_hour_convention: Option<String>,
    pub transmission_calendar_adjustment_selected: Option<String>,
    pub rule_eligible: bool,
    pub blocked_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum QimenCandidateError {
    #[error("奇门日界安全边距必须是非负有限分钟数")]
    InvalidDayBoundaryMargin,
    #[error("奇门局数必须在一至九之间，实际为 {0}")]
    InvalidBureauNumber(u8),
}

const QIMEN_EARTH_PLATE_SEQUENCE: [HeavenlyStem; 9] = [
    HeavenlyStem::Wu,
    HeavenlyStem::Ji,
    HeavenlyStem::Geng,
    HeavenlyStem::Xin,
    HeavenlyStem::Ren,
    HeavenlyStem::Gui,
    HeavenlyStem::Ding,
    HeavenlyStem::Bing,
    HeavenlyStem::Yi,
];

pub fn qimen_hidden_jia(instrument: HeavenlyStem) -> Option<SexagenaryDesignation> {
    let cycle_index = match instrument {
        HeavenlyStem::Wu => 0,
        HeavenlyStem::Ji => 10,
        HeavenlyStem::Geng => 20,
        HeavenlyStem::Xin => 30,
        HeavenlyStem::Ren => 40,
        HeavenlyStem::Gui => 50,
        HeavenlyStem::Jia | HeavenlyStem::Yi | HeavenlyStem::Bing | HeavenlyStem::Ding => {
            return None
        }
    };
    Some(SexagenaryDesignation::from_cycle_index(cycle_index))
}

pub fn qimen_xun_head(designation: SexagenaryDesignation) -> SexagenaryDesignation {
    SexagenaryDesignation::from_cycle_index(i64::from(
        designation.cycle_index - designation.cycle_index % 10,
    ))
}

pub fn qimen_hidden_instrument_for_xun_head(
    xun_head: SexagenaryDesignation,
) -> Option<HeavenlyStem> {
    if xun_head != SexagenaryDesignation::from_cycle_index(i64::from(xun_head.cycle_index)) {
        return None;
    }
    match xun_head.cycle_index {
        0 => Some(HeavenlyStem::Wu),
        10 => Some(HeavenlyStem::Ji),
        20 => Some(HeavenlyStem::Geng),
        30 => Some(HeavenlyStem::Xin),
        40 => Some(HeavenlyStem::Ren),
        50 => Some(HeavenlyStem::Gui),
        _ => None,
    }
}

pub fn qimen_instrument_palace(
    earth_plate: &QimenEarthPlate,
    instrument: HeavenlyStem,
) -> Option<QimenPalace> {
    earth_plate
        .cells_by_palace_number
        .iter()
        .find(|cell| cell.instrument_or_wonder == instrument)
        .map(|cell| cell.palace)
}

pub fn qimen_earth_plate(
    dun: QimenDun,
    bureau_number: u8,
) -> Result<QimenEarthPlate, QimenCandidateError> {
    if !(1..=9).contains(&bureau_number) {
        return Err(QimenCandidateError::InvalidBureauNumber(bureau_number));
    }
    let mut stems_by_palace = [HeavenlyStem::Jia; 9];
    for (offset, stem) in QIMEN_EARTH_PLATE_SEQUENCE.into_iter().enumerate() {
        let signed_offset = match dun {
            QimenDun::Yang => offset as i16,
            QimenDun::Yin => -(offset as i16),
        };
        let palace_index = (i16::from(bureau_number) - 1 + signed_offset).rem_euclid(9) as usize;
        stems_by_palace[palace_index] = stem;
    }
    let cells_by_palace_number = QimenPalace::ALL_IN_NUMBER_ORDER
        .into_iter()
        .zip(stems_by_palace)
        .map(|(palace, instrument_or_wonder)| QimenEarthPlateCell {
            palace,
            instrument_or_wonder,
            hidden_jia: qimen_hidden_jia(instrument_or_wonder),
        })
        .collect();
    Ok(QimenEarthPlate {
        audit_version: 1,
        algorithm_version: QIMEN_EARTH_PLATE_ALGORITHM_VERSION.to_owned(),
        status: QIMEN_EARTH_PLATE_STATUS.to_owned(),
        dun,
        bureau_number,
        cells_by_palace_number,
        earth_plate_complete: true,
        interpretation_eligible: false,
        rule_eligible: false,
    })
}

pub const fn qimen_dun_for_term(term: SolarTerm) -> QimenDun {
    match term {
        SolarTerm::Dongzhi
        | SolarTerm::Xiaohan
        | SolarTerm::Dahan
        | SolarTerm::Lichun
        | SolarTerm::Yushui
        | SolarTerm::Jingzhe
        | SolarTerm::Chunfen
        | SolarTerm::Qingming
        | SolarTerm::Guyu
        | SolarTerm::Lixia
        | SolarTerm::Xiaoman
        | SolarTerm::Mangzhong => QimenDun::Yang,
        SolarTerm::Xiazhi
        | SolarTerm::Xiaoshu
        | SolarTerm::Dashu
        | SolarTerm::Liqiu
        | SolarTerm::Chushu
        | SolarTerm::Bailu
        | SolarTerm::Qiufen
        | SolarTerm::Hanlu
        | SolarTerm::Shuangjiang
        | SolarTerm::Lidong
        | SolarTerm::Xiaoxue
        | SolarTerm::Daxue => QimenDun::Yin,
    }
}

                                                                          
                                                                           
                                                                    
pub const fn qimen_bureau_numbers_for_term(term: SolarTerm) -> Option<[u8; 3]> {
    let numbers = match term {
        SolarTerm::Dongzhi | SolarTerm::Jingzhe => [1, 7, 4],
        SolarTerm::Xiaohan => [2, 8, 5],
        SolarTerm::Dahan | SolarTerm::Chunfen => [3, 9, 6],
        SolarTerm::Lichun => [8, 5, 2],
        SolarTerm::Yushui => [9, 6, 3],
        SolarTerm::Qingming | SolarTerm::Lixia => [4, 1, 7],
        SolarTerm::Guyu | SolarTerm::Xiaoman => [5, 2, 8],
        SolarTerm::Mangzhong => [6, 3, 9],
        SolarTerm::Xiazhi | SolarTerm::Bailu => [9, 3, 6],
        SolarTerm::Xiaoshu => [8, 2, 5],
        SolarTerm::Dashu | SolarTerm::Qiufen => [7, 1, 4],
        SolarTerm::Liqiu => [2, 5, 8],
        SolarTerm::Hanlu | SolarTerm::Lidong => [6, 9, 3],
        SolarTerm::Shuangjiang | SolarTerm::Xiaoxue => [5, 8, 2],
        SolarTerm::Daxue => [4, 7, 1],
        SolarTerm::Chushu => return None,
    };
    Some(numbers)
}

pub fn qimen_symbol_head(day: SexagenaryDesignation) -> SexagenaryDesignation {
    SexagenaryDesignation::from_cycle_index(i64::from(day.cycle_index - day.cycle_index % 5))
}

pub const fn qimen_yuan_from_symbol_head_branch(branch: EarthlyBranch) -> QimenThreeYuan {
    match branch {
        EarthlyBranch::Zi | EarthlyBranch::Wu | EarthlyBranch::Mao | EarthlyBranch::You => {
            QimenThreeYuan::Upper
        }
        EarthlyBranch::Yin | EarthlyBranch::Shen | EarthlyBranch::Si | EarthlyBranch::Hai => {
            QimenThreeYuan::Middle
        }
        EarthlyBranch::Chen | EarthlyBranch::Xu | EarthlyBranch::Chou | EarthlyBranch::Wei => {
            QimenThreeYuan::Lower
        }
    }
}

pub fn qimen_midnight_boundary_safe(
    time: DateTime<FixedOffset>,
    boundary_margin_minutes: f64,
) -> Result<bool, QimenCandidateError> {
    if !boundary_margin_minutes.is_finite() || boundary_margin_minutes < 0.0 {
        return Err(QimenCandidateError::InvalidDayBoundaryMargin);
    }
    let seconds_since_midnight = f64::from(time.num_seconds_from_midnight());
    let seconds_to_next_midnight = 86_400.0 - seconds_since_midnight;
    Ok(seconds_since_midnight.min(seconds_to_next_midnight) > boundary_margin_minutes * 60.0)
}

fn day_candidate(
    convention: &str,
    dun: QimenDun,
    day: SexagenaryDesignation,
    day_boundary_safe: bool,
    solar_term_boundary_safe: bool,
    received_bureau_numbers: Option<[u8; 3]>,
) -> QimenDayConventionCandidate {
    let symbol_head = qimen_symbol_head(day);
    let yuan = qimen_yuan_from_symbol_head_branch(symbol_head.branch);
    let bureau_number = received_bureau_numbers.map(|numbers| numbers[yuan.index()]);
    let earth_plate = bureau_number.and_then(|number| qimen_earth_plate(dun, number).ok());
    QimenDayConventionCandidate {
        day_convention: convention.to_owned(),
        day_designation: day,
        symbol_head,
        yuan,
        bureau_number,
        earth_plate,
        hour_xun_candidate: None,
        day_boundary_safe,
        mechanical_boundaries_safe: day_boundary_safe
            && solar_term_boundary_safe
            && bureau_number.is_some(),
    }
}

fn hour_xun_candidate(
    time_convention: &str,
    hour_designation: SexagenaryDesignation,
    hour_boundary_safe: bool,
    day_candidate: &QimenDayConventionCandidate,
) -> QimenHourXunCandidate {
    let xun_head = qimen_xun_head(hour_designation);
    let hidden_instrument = qimen_hidden_instrument_for_xun_head(xun_head)
        .expect("every sexagenary designation belongs to one of the six Jia xun");
    let xun_head_palace = day_candidate
        .earth_plate
        .as_ref()
        .and_then(|plate| qimen_instrument_palace(plate, hidden_instrument));
    QimenHourXunCandidate {
        audit_version: 1,
        algorithm_version: QIMEN_HOUR_XUN_CANDIDATE_ALGORITHM_VERSION.to_owned(),
        status: QIMEN_HOUR_XUN_CANDIDATE_STATUS.to_owned(),
        time_convention: time_convention.to_owned(),
        hour_designation,
        xun_head,
        hidden_instrument,
        xun_head_palace,
        duty_identity_candidate: None,
        hour_boundary_safe,
        mechanical_boundaries_safe: day_candidate.mechanical_boundaries_safe
            && hour_boundary_safe
            && xun_head_palace.is_some(),
        interpretation_eligible: false,
        rule_eligible: false,
    }
}

fn duty_identity_candidate(
    xun_head_palace: QimenPalace,
    mechanical_boundaries_safe: bool,
) -> QimenDutyIdentityCandidate {
    let duty_envoy_source_palace = xun_head_palace.early_song_duty_envoy_host();
    let center_hosting_applied = xun_head_palace == QimenPalace::CenterFive;
    QimenDutyIdentityCandidate {
        audit_version: 1,
        algorithm_version: QIMEN_DUTY_IDENTITY_ALGORITHM_VERSION.to_owned(),
        status: QIMEN_DUTY_IDENTITY_STATUS.to_owned(),
        xun_head_palace,
        duty_chief_star: xun_head_palace.fixed_star(),
        duty_envoy_gate: duty_envoy_source_palace
            .fixed_gate()
            .expect("the early-Song center hosting convention always resolves an eight gate"),
        duty_envoy_source_palace,
        center_hosting_applied,
        center_hosting_convention: center_hosting_applied
            .then(|| QIMEN_CENTER_HOSTING_CONVENTION.to_owned()),
        mechanical_boundaries_safe,
        duty_chief_hourly_palace: None,
        duty_envoy_hourly_palace: None,
        heaven_plate_calculated: false,
        interpretation_eligible: false,
        rule_eligible: false,
    }
}

fn hour_xun_candidates_identical(
    left: &QimenHourXunCandidate,
    right: &QimenHourXunCandidate,
) -> bool {
    left.hour_designation == right.hour_designation
        && left.xun_head == right.xun_head
        && left.hidden_instrument == right.hidden_instrument
        && left.xun_head_palace == right.xun_head_palace
        && left.hour_boundary_safe == right.hour_boundary_safe
        && left.mechanical_boundaries_safe == right.mechanical_boundaries_safe
}

pub fn qimen_ju_audit(
    interval: &SolarTermInterval,
    beijing_midnight_day: SexagenaryDesignation,
    beijing_day_boundary_safe: bool,
    true_solar_midnight_day: SexagenaryDesignation,
    true_solar_day_boundary_safe: bool,
) -> QimenJuAudit {
    let active_solar_term = interval.previous.term;
    let dun = qimen_dun_for_term(active_solar_term);
    let received_bureau_numbers = qimen_bureau_numbers_for_term(active_solar_term);
    let table_entry_evidence_conflict = active_solar_term == SolarTerm::Chushu;
    let beijing_midnight_candidate = day_candidate(
        "beijing_civil_midnight.v1",
        dun,
        beijing_midnight_day,
        beijing_day_boundary_safe,
        interval.boundary_safe,
        received_bureau_numbers,
    );
    let true_solar_midnight_candidate = day_candidate(
        "true_solar_civil_midnight.v1",
        dun,
        true_solar_midnight_day,
        true_solar_day_boundary_safe,
        interval.boundary_safe,
        received_bureau_numbers,
    );
    let candidates_identical = beijing_midnight_candidate.day_designation
        == true_solar_midnight_candidate.day_designation
        && beijing_midnight_candidate.symbol_head == true_solar_midnight_candidate.symbol_head
        && beijing_midnight_candidate.yuan == true_solar_midnight_candidate.yuan
        && beijing_midnight_candidate.bureau_number == true_solar_midnight_candidate.bureau_number
        && beijing_midnight_candidate.day_boundary_safe
            == true_solar_midnight_candidate.day_boundary_safe;
    let mut blocked_reasons = vec![
        "beijing_vs_true_solar_day_boundary_convention_not_selected".to_owned(),
        "chao_shen_jie_qi_transmission_calendar_adjustment_not_selected".to_owned(),
        "heaven_plate_nine_stars_eight_gates_and_spirits_not_implemented".to_owned(),
        "qimen_interpretation_rules_not_reviewed".to_owned(),
    ];
    if table_entry_evidence_conflict {
        blocked_reasons
            .push("chushu_bureau_numbers_conflict_between_public_transcriptions".to_owned());
    }
    QimenJuAudit {
        audit_version: 2,
        algorithm_version: QIMEN_JU_CANDIDATE_ALGORITHM_VERSION.to_owned(),
        status: QIMEN_JU_CANDIDATE_STATUS.to_owned(),
        active_solar_term,
        next_solar_term: interval.next.term,
        solar_term_boundary_safe: interval.boundary_safe,
        dun,
        received_bureau_numbers,
        table_entry_evidence_conflict,
        beijing_midnight_candidate,
        true_solar_midnight_candidate,
        candidates_identical,
        selected_day_convention: None,
        selected_hour_convention: None,
        transmission_calendar_adjustment_selected: None,
        rule_eligible: false,
        blocked_reasons,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn qimen_ju_hour_audit(
    interval: &SolarTermInterval,
    beijing_midnight_day: SexagenaryDesignation,
    beijing_day_boundary_safe: bool,
    beijing_hour: SexagenaryDesignation,
    beijing_hour_boundary_safe: bool,
    true_solar_midnight_day: SexagenaryDesignation,
    true_solar_day_boundary_safe: bool,
    true_solar_hour: SexagenaryDesignation,
    true_solar_hour_boundary_safe: bool,
) -> QimenJuAudit {
    let mut audit = qimen_ju_audit(
        interval,
        beijing_midnight_day,
        beijing_day_boundary_safe,
        true_solar_midnight_day,
        true_solar_day_boundary_safe,
    );
    audit.beijing_midnight_candidate.hour_xun_candidate = Some(hour_xun_candidate(
        "beijing_civil_clock_and_midnight.v1",
        beijing_hour,
        beijing_hour_boundary_safe,
        &audit.beijing_midnight_candidate,
    ));
    audit.true_solar_midnight_candidate.hour_xun_candidate = Some(hour_xun_candidate(
        "true_solar_clock_and_civil_midnight.v1",
        true_solar_hour,
        true_solar_hour_boundary_safe,
        &audit.true_solar_midnight_candidate,
    ));
    audit.candidates_identical = audit.beijing_midnight_candidate.day_designation
        == audit.true_solar_midnight_candidate.day_designation
        && audit.beijing_midnight_candidate.symbol_head
            == audit.true_solar_midnight_candidate.symbol_head
        && audit.beijing_midnight_candidate.yuan == audit.true_solar_midnight_candidate.yuan
        && audit.beijing_midnight_candidate.bureau_number
            == audit.true_solar_midnight_candidate.bureau_number
        && audit.beijing_midnight_candidate.day_boundary_safe
            == audit.true_solar_midnight_candidate.day_boundary_safe
        && match (
            &audit.beijing_midnight_candidate.hour_xun_candidate,
            &audit.true_solar_midnight_candidate.hour_xun_candidate,
        ) {
            (Some(left), Some(right)) => hour_xun_candidates_identical(left, right),
            (None, None) => true,
            _ => false,
        };
    audit.audit_version = 3;
    audit.status = QIMEN_JU_HOUR_CANDIDATE_STATUS.to_owned();
    audit.blocked_reasons.insert(
        2,
        "beijing_vs_true_solar_hour_clock_and_boundary_convention_not_selected".to_owned(),
    );
    audit.blocked_reasons.insert(
        3,
        "duty_chief_and_envoy_selection_not_implemented".to_owned(),
    );
    audit
}

#[allow(clippy::too_many_arguments)]
pub fn qimen_ju_duty_identity_audit(
    interval: &SolarTermInterval,
    beijing_midnight_day: SexagenaryDesignation,
    beijing_day_boundary_safe: bool,
    beijing_hour: SexagenaryDesignation,
    beijing_hour_boundary_safe: bool,
    true_solar_midnight_day: SexagenaryDesignation,
    true_solar_day_boundary_safe: bool,
    true_solar_hour: SexagenaryDesignation,
    true_solar_hour_boundary_safe: bool,
) -> QimenJuAudit {
    let mut audit = qimen_ju_hour_audit(
        interval,
        beijing_midnight_day,
        beijing_day_boundary_safe,
        beijing_hour,
        beijing_hour_boundary_safe,
        true_solar_midnight_day,
        true_solar_day_boundary_safe,
        true_solar_hour,
        true_solar_hour_boundary_safe,
    );
    for candidate in [
        &mut audit.beijing_midnight_candidate,
        &mut audit.true_solar_midnight_candidate,
    ] {
        if let Some(hour_xun) = candidate.hour_xun_candidate.as_mut() {
            hour_xun.duty_identity_candidate = hour_xun
                .xun_head_palace
                .map(|palace| duty_identity_candidate(palace, hour_xun.mechanical_boundaries_safe));
            hour_xun.audit_version = 2;
            hour_xun.status = QIMEN_HOUR_DUTY_IDENTITY_STATUS.to_owned();
        }
    }
    audit.audit_version = 4;
    audit.status = QIMEN_JU_DUTY_IDENTITY_STATUS.to_owned();
    audit.blocked_reasons.retain(|reason| {
        reason != "duty_chief_and_envoy_selection_not_implemented"
            && reason != "heaven_plate_nine_stars_eight_gates_and_spirits_not_implemented"
    });
    audit.blocked_reasons.insert(
        3,
        "duty_chief_and_envoy_hourly_movement_not_implemented".to_owned(),
    );
    audit.blocked_reasons.insert(
        4,
        "heaven_plate_and_star_gate_movement_not_implemented".to_owned(),
    );
    audit
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AstronomyPrecision, SolarTermEvent};
    use chrono::TimeZone;

    fn interval(previous: SolarTerm, next: SolarTerm, safe: bool) -> SolarTermInterval {
        let event = |term, jd_tt| SolarTermEvent {
            term,
            jd_tt,
            final_bracket_seconds: 0.1,
            iterations: 20,
            solver_uncertainty_seconds: 0.1,
            ephemeris_time_uncertainty_seconds: 1.0,
            combined_uncertainty_seconds: 1.1,
            precision: AstronomyPrecision::SpaReferenceVectorMatched,
            algorithm_version: "test".to_owned(),
        };
        SolarTermInterval {
            previous: event(previous, 2_460_000.0),
            next: event(next, 2_460_015.0),
            seconds_since_previous: 100_000.0,
            seconds_until_next: 1_196_000.0,
            boundary_margin_seconds: 300.0,
            boundary_safe: safe,
            algorithm_version: "test".to_owned(),
        }
    }

    #[test]
    fn all_terms_have_dun_and_only_chushu_is_evidence_blocked() {
        let mut yang = 0;
        let mut yin = 0;
        let mut missing = Vec::new();
        for term in SolarTerm::ALL_LONGITUDE_ORDER {
            match qimen_dun_for_term(term) {
                QimenDun::Yang => yang += 1,
                QimenDun::Yin => yin += 1,
            }
            if qimen_bureau_numbers_for_term(term).is_none() {
                missing.push(term);
            }
        }
        assert_eq!(yang, 12);
        assert_eq!(yin, 12);
        assert_eq!(missing, vec![SolarTerm::Chushu]);
    }

    #[test]
    fn received_table_anchors_match_both_dun_halves() {
        assert_eq!(
            qimen_bureau_numbers_for_term(SolarTerm::Dongzhi),
            Some([1, 7, 4])
        );
        assert_eq!(
            qimen_bureau_numbers_for_term(SolarTerm::Yushui),
            Some([9, 6, 3])
        );
        assert_eq!(
            qimen_bureau_numbers_for_term(SolarTerm::Mangzhong),
            Some([6, 3, 9])
        );
        assert_eq!(
            qimen_bureau_numbers_for_term(SolarTerm::Xiazhi),
            Some([9, 3, 6])
        );
        assert_eq!(
            qimen_bureau_numbers_for_term(SolarTerm::Lidong),
            Some([6, 9, 3])
        );
        assert_eq!(
            qimen_bureau_numbers_for_term(SolarTerm::Daxue),
            Some([4, 7, 1])
        );
    }

    #[test]
    fn received_nine_palace_identities_are_complete_and_number_ordered() {
        let expected = [
            Some(Trigram::Kan),
            Some(Trigram::Kun),
            Some(Trigram::Zhen),
            Some(Trigram::Xun),
            None,
            Some(Trigram::Qian),
            Some(Trigram::Dui),
            Some(Trigram::Gen),
            Some(Trigram::Li),
        ];
        for (index, palace) in QimenPalace::ALL_IN_NUMBER_ORDER.into_iter().enumerate() {
            assert_eq!(palace.number(), index as u8 + 1);
            assert_eq!(palace.trigram(), expected[index]);
        }
    }

    #[test]
    fn early_song_fixed_star_and_gate_catalogs_match_all_nine_palaces() {
        let expected_gates = [
            Some(QimenGate::Rest),
            Some(QimenGate::Death),
            Some(QimenGate::Harm),
            Some(QimenGate::Block),
            None,
            Some(QimenGate::Open),
            Some(QimenGate::Alarm),
            Some(QimenGate::Life),
            Some(QimenGate::View),
        ];
        for (index, palace) in QimenPalace::ALL_IN_NUMBER_ORDER.into_iter().enumerate() {
            assert_eq!(palace.fixed_star(), QimenStar::ALL_IN_PALACE_ORDER[index]);
            assert_eq!(palace.fixed_gate(), expected_gates[index]);
        }
        assert_eq!(
            QimenPalace::CenterFive.early_song_duty_envoy_host(),
            QimenPalace::KunTwo
        );
        for palace in QimenPalace::ALL_IN_NUMBER_ORDER
            .into_iter()
            .filter(|palace| *palace != QimenPalace::CenterFive)
        {
            assert_eq!(palace.early_song_duty_envoy_host(), palace);
        }
    }

    #[test]
    fn six_instruments_hide_the_six_jia_in_received_order() {
        let expected = [
            (HeavenlyStem::Wu, "甲子"),
            (HeavenlyStem::Ji, "甲戌"),
            (HeavenlyStem::Geng, "甲申"),
            (HeavenlyStem::Xin, "甲午"),
            (HeavenlyStem::Ren, "甲辰"),
            (HeavenlyStem::Gui, "甲寅"),
        ];
        for (instrument, hidden_name) in expected {
            assert_eq!(qimen_hidden_jia(instrument).unwrap().zh_name(), hidden_name);
        }
        for wonder in [HeavenlyStem::Yi, HeavenlyStem::Bing, HeavenlyStem::Ding] {
            assert_eq!(qimen_hidden_jia(wonder), None);
        }
    }

    #[test]
    fn every_hour_designation_resolves_to_one_of_six_xun_heads_and_instruments() {
        let instruments = [
            HeavenlyStem::Wu,
            HeavenlyStem::Ji,
            HeavenlyStem::Geng,
            HeavenlyStem::Xin,
            HeavenlyStem::Ren,
            HeavenlyStem::Gui,
        ];
        for cycle_index in 0..60 {
            let hour = SexagenaryDesignation::from_cycle_index(cycle_index);
            let xun_head = qimen_xun_head(hour);
            assert_eq!(xun_head.cycle_index, (cycle_index / 10 * 10) as u8);
            assert_eq!(xun_head.stem, HeavenlyStem::Jia);
            assert_eq!(
                qimen_hidden_instrument_for_xun_head(xun_head),
                Some(instruments[cycle_index as usize / 10])
            );
        }
        assert_eq!(
            qimen_hidden_instrument_for_xun_head(SexagenaryDesignation::from_cycle_index(1)),
            None
        );
        assert_eq!(
            qimen_hidden_instrument_for_xun_head(SexagenaryDesignation {
                cycle_index: 0,
                stem: HeavenlyStem::Yi,
                branch: EarthlyBranch::Zi,
            }),
            None
        );
    }

    #[test]
    fn yang_one_and_yin_nine_match_received_earth_plate_examples() {
        let yang = qimen_earth_plate(QimenDun::Yang, 1).unwrap();
        assert_eq!(
            yang.cells_by_palace_number
                .iter()
                .map(|cell| cell.instrument_or_wonder)
                .collect::<Vec<_>>(),
            vec![
                HeavenlyStem::Wu,
                HeavenlyStem::Ji,
                HeavenlyStem::Geng,
                HeavenlyStem::Xin,
                HeavenlyStem::Ren,
                HeavenlyStem::Gui,
                HeavenlyStem::Ding,
                HeavenlyStem::Bing,
                HeavenlyStem::Yi,
            ]
        );
        let yin = qimen_earth_plate(QimenDun::Yin, 9).unwrap();
        assert_eq!(
            yin.cells_by_palace_number
                .iter()
                .map(|cell| cell.instrument_or_wonder)
                .collect::<Vec<_>>(),
            vec![
                HeavenlyStem::Yi,
                HeavenlyStem::Bing,
                HeavenlyStem::Ding,
                HeavenlyStem::Gui,
                HeavenlyStem::Ren,
                HeavenlyStem::Xin,
                HeavenlyStem::Geng,
                HeavenlyStem::Ji,
                HeavenlyStem::Wu,
            ]
        );
        assert!(yang.earth_plate_complete);
        assert!(!yang.interpretation_eligible);
        assert!(!yang.rule_eligible);
    }

    #[test]
    fn every_dun_and_bureau_places_each_instrument_or_wonder_exactly_once() {
        for dun in [QimenDun::Yang, QimenDun::Yin] {
            for bureau_number in 1..=9 {
                let plate = qimen_earth_plate(dun, bureau_number).unwrap();
                assert_eq!(plate.cells_by_palace_number.len(), 9);
                let mut seen = [false; 10];
                for cell in &plate.cells_by_palace_number {
                    let index = cell.instrument_or_wonder.index() as usize;
                    assert!(!seen[index]);
                    seen[index] = true;
                }
                assert!(!seen[HeavenlyStem::Jia.index() as usize]);
                assert_eq!(seen.into_iter().filter(|value| *value).count(), 9);
                let start = &plate.cells_by_palace_number[(bureau_number - 1) as usize];
                assert_eq!(start.instrument_or_wonder, HeavenlyStem::Wu);
                assert_eq!(start.hidden_jia.unwrap().zh_name(), "甲子");
            }
        }
        assert!(qimen_earth_plate(QimenDun::Yang, 0).is_err());
        assert!(qimen_earth_plate(QimenDun::Yin, 10).is_err());
    }

    #[test]
    fn six_xun_heads_resolve_to_received_palaces_on_yang_one_and_yin_nine() {
        let yang = qimen_earth_plate(QimenDun::Yang, 1).unwrap();
        let yin = qimen_earth_plate(QimenDun::Yin, 9).unwrap();
        for (xun_index, instrument) in [
            HeavenlyStem::Wu,
            HeavenlyStem::Ji,
            HeavenlyStem::Geng,
            HeavenlyStem::Xin,
            HeavenlyStem::Ren,
            HeavenlyStem::Gui,
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(
                qimen_instrument_palace(&yang, instrument).unwrap().number(),
                xun_index as u8 + 1
            );
            assert_eq!(
                qimen_instrument_palace(&yin, instrument).unwrap().number(),
                9 - xun_index as u8
            );
        }
    }

    #[test]
    fn every_dun_bureau_and_xun_resolves_fixed_duty_identities_without_movement() {
        for dun in [QimenDun::Yang, QimenDun::Yin] {
            for bureau_number in 1..=9 {
                let plate = qimen_earth_plate(dun, bureau_number).unwrap();
                for instrument in [
                    HeavenlyStem::Wu,
                    HeavenlyStem::Ji,
                    HeavenlyStem::Geng,
                    HeavenlyStem::Xin,
                    HeavenlyStem::Ren,
                    HeavenlyStem::Gui,
                ] {
                    let palace = qimen_instrument_palace(&plate, instrument).unwrap();
                    let identity = duty_identity_candidate(palace, true);
                    assert_eq!(identity.xun_head_palace, palace);
                    assert_eq!(identity.duty_chief_star, palace.fixed_star());
                    assert_eq!(
                        identity.duty_envoy_gate,
                        palace.early_song_duty_envoy_host().fixed_gate().unwrap()
                    );
                    assert_eq!(
                        identity.center_hosting_applied,
                        palace == QimenPalace::CenterFive
                    );
                    assert_eq!(identity.duty_chief_hourly_palace, None);
                    assert_eq!(identity.duty_envoy_hourly_palace, None);
                    assert!(!identity.heaven_plate_calculated);
                    assert!(!identity.interpretation_eligible);
                    assert!(!identity.rule_eligible);
                }
            }
        }
    }

    #[test]
    fn every_five_day_block_has_one_jia_or_ji_symbol_head_and_three_yuan_repeat() {
        let mut yuan_day_counts = [0; 3];
        for cycle_index in 0..60 {
            let day = SexagenaryDesignation::from_cycle_index(cycle_index);
            let symbol_head = qimen_symbol_head(day);
            assert!(matches!(
                symbol_head.stem,
                crate::HeavenlyStem::Jia | crate::HeavenlyStem::Ji
            ));
            assert_eq!(symbol_head.cycle_index % 5, 0);
            assert!(day.cycle_index - symbol_head.cycle_index < 5);
            let yuan = qimen_yuan_from_symbol_head_branch(symbol_head.branch);
            yuan_day_counts[yuan.index()] += 1;
        }
        assert_eq!(yuan_day_counts, [20, 20, 20]);
    }

    #[test]
    fn audit_keeps_day_conventions_parallel_and_never_enables_rules() {
        let audit = qimen_ju_audit(
            &interval(SolarTerm::Dongzhi, SolarTerm::Xiaohan, true),
            SexagenaryDesignation::from_cycle_index(0),
            true,
            SexagenaryDesignation::from_cycle_index(59),
            false,
        );
        assert_eq!(audit.dun, QimenDun::Yang);
        assert_eq!(audit.received_bureau_numbers, Some([1, 7, 4]));
        assert_eq!(audit.beijing_midnight_candidate.yuan, QimenThreeYuan::Upper);
        assert_eq!(audit.beijing_midnight_candidate.bureau_number, Some(1));
        assert_eq!(
            audit
                .beijing_midnight_candidate
                .earth_plate
                .as_ref()
                .unwrap()
                .bureau_number,
            1
        );
        assert!(audit.beijing_midnight_candidate.mechanical_boundaries_safe);
        assert!(
            !audit
                .true_solar_midnight_candidate
                .mechanical_boundaries_safe
        );
        assert!(!audit.candidates_identical);
        assert_eq!(audit.selected_day_convention, None);
        assert_eq!(audit.transmission_calendar_adjustment_selected, None);
        assert!(!audit.rule_eligible);
    }

    #[test]
    fn chushu_and_exact_day_boundaries_fail_closed() {
        let zone = FixedOffset::east_opt(8 * 3600).unwrap();
        let exact_midnight = zone.with_ymd_and_hms(2026, 8, 23, 0, 0, 0).unwrap();
        assert!(!qimen_midnight_boundary_safe(exact_midnight, 5.0).unwrap());
        assert!(qimen_midnight_boundary_safe(
            zone.with_ymd_and_hms(2026, 8, 23, 0, 6, 0).unwrap(),
            5.0
        )
        .unwrap());
        assert!(qimen_midnight_boundary_safe(exact_midnight, -1.0).is_err());

        let audit = qimen_ju_audit(
            &interval(SolarTerm::Chushu, SolarTerm::Bailu, true),
            SexagenaryDesignation::from_cycle_index(12),
            true,
            SexagenaryDesignation::from_cycle_index(12),
            true,
        );
        assert_eq!(audit.received_bureau_numbers, None);
        assert!(audit.table_entry_evidence_conflict);
        assert_eq!(audit.beijing_midnight_candidate.bureau_number, None);
        assert_eq!(audit.beijing_midnight_candidate.earth_plate, None);
        assert!(!audit.beijing_midnight_candidate.mechanical_boundaries_safe);
        assert!(!audit.rule_eligible);
    }

    #[test]
    fn hour_audit_keeps_clock_conventions_parallel_and_fails_closed_at_boundaries() {
        let audit = qimen_ju_hour_audit(
            &interval(SolarTerm::Dongzhi, SolarTerm::Xiaohan, true),
            SexagenaryDesignation::from_cycle_index(0),
            true,
            SexagenaryDesignation::from_cycle_index(0),
            true,
            SexagenaryDesignation::from_cycle_index(0),
            true,
            SexagenaryDesignation::from_cycle_index(10),
            false,
        );
        let beijing = audit
            .beijing_midnight_candidate
            .hour_xun_candidate
            .as_ref()
            .unwrap();
        assert_eq!(audit.audit_version, 3);
        assert_eq!(beijing.xun_head.zh_name(), "甲子");
        assert_eq!(beijing.hidden_instrument, HeavenlyStem::Wu);
        assert_eq!(beijing.xun_head_palace, Some(QimenPalace::KanOne));
        assert!(beijing.mechanical_boundaries_safe);
        let true_solar = audit
            .true_solar_midnight_candidate
            .hour_xun_candidate
            .as_ref()
            .unwrap();
        assert_eq!(true_solar.xun_head.zh_name(), "甲戌");
        assert_eq!(true_solar.hidden_instrument, HeavenlyStem::Ji);
        assert_eq!(true_solar.xun_head_palace, Some(QimenPalace::KunTwo));
        assert!(!true_solar.mechanical_boundaries_safe);
        assert!(!audit.candidates_identical);
        assert_eq!(audit.selected_hour_convention, None);
        assert!(!audit.rule_eligible);
    }

    #[test]
    fn chushu_hour_xun_is_preserved_but_cannot_claim_an_earth_palace() {
        let audit = qimen_ju_duty_identity_audit(
            &interval(SolarTerm::Chushu, SolarTerm::Bailu, true),
            SexagenaryDesignation::from_cycle_index(12),
            true,
            SexagenaryDesignation::from_cycle_index(25),
            true,
            SexagenaryDesignation::from_cycle_index(12),
            true,
            SexagenaryDesignation::from_cycle_index(25),
            true,
        );
        let candidate = audit.beijing_midnight_candidate.hour_xun_candidate.unwrap();
        assert_eq!(candidate.hour_designation.cycle_index, 25);
        assert_eq!(candidate.xun_head.cycle_index, 20);
        assert_eq!(candidate.hidden_instrument, HeavenlyStem::Geng);
        assert_eq!(candidate.xun_head_palace, None);
        assert_eq!(candidate.duty_identity_candidate, None);
        assert!(!candidate.mechanical_boundaries_safe);
    }

    #[test]
    fn yang_one_six_xun_duty_identities_match_received_examples() {
        let expected = [
            (QimenStar::Tianpeng, QimenGate::Rest, false),
            (QimenStar::Tianrui, QimenGate::Death, false),
            (QimenStar::Tianchong, QimenGate::Harm, false),
            (QimenStar::Tianfu, QimenGate::Block, false),
            (QimenStar::Tianqin, QimenGate::Death, true),
            (QimenStar::Tianxin, QimenGate::Open, false),
        ];
        for (xun_index, (star, gate, center_hosting)) in expected.into_iter().enumerate() {
            let hour = SexagenaryDesignation::from_cycle_index((xun_index * 10) as i64);
            let audit = qimen_ju_duty_identity_audit(
                &interval(SolarTerm::Dongzhi, SolarTerm::Xiaohan, true),
                SexagenaryDesignation::from_cycle_index(0),
                true,
                hour,
                true,
                SexagenaryDesignation::from_cycle_index(0),
                true,
                hour,
                true,
            );
            assert_eq!(audit.audit_version, 4);
            let identity = audit
                .beijing_midnight_candidate
                .hour_xun_candidate
                .unwrap()
                .duty_identity_candidate
                .unwrap();
            assert_eq!(identity.duty_chief_star, star);
            assert_eq!(identity.duty_envoy_gate, gate);
            assert_eq!(identity.center_hosting_applied, center_hosting);
            assert_eq!(
                identity.center_hosting_convention.as_deref(),
                center_hosting.then_some(QIMEN_CENTER_HOSTING_CONVENTION)
            );
            assert!(identity.mechanical_boundaries_safe);
        }
    }
}
