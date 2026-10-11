                                                                     
                                                                             
                                                                      

use crate::{
    nayin_for_designation, sexagenary_year_for_astronomical_year_label, EarthlyBranch, FiveElement,
    HeavenlyStem, LichunYearCandidate, LunisolarDateCandidate, NayinDesignation,
    SexagenaryDesignation, ShichenBoundary,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const ZIWEI_LIFE_BODY_ALGORITHM_VERSION: &str =
    "jiuheng.ziwei.received_month_hour_life_body_twelve_palaces.v1";
pub const ZIWEI_LIFE_BODY_STATUS: &str =
    "mechanical_palaces_only_no_stars_interpretation_or_runtime_signals.v1";
pub const ZIWEI_FIVE_ELEMENT_BUREAU_ALGORITHM_VERSION: &str =
    "jiuheng.ziwei.received_yin_stem_nayin_five_element_bureau.v1";
pub const ZIWEI_MECHANICAL_AUDIT_ALGORITHM_VERSION: &str =
    "jiuheng.ziwei.received_palaces_bureau_fourteen_stars_four_auxiliaries.v5";
pub const ZIWEI_MECHANICAL_AUDIT_STATUS: &str =
    "mechanical_palaces_bureau_stars_auxiliaries_no_interpretation_or_runtime_signals.v5";
pub const ZIWEI_STAR_ANCHOR_ALGORITHM_VERSION: &str =
    "jiuheng.ziwei.received_bureau_lunar_day_ziwei_star_anchor.v1";
pub const ZIWEI_MAJOR_STARS_ALGORITHM_VERSION: &str =
    "jiuheng.ziwei.received_north_south_dipper_fourteen_star_placements.v1";
pub const ZIWEI_AUXILIARY_STARS_ALGORITHM_VERSION: &str =
    "jiuheng.ziwei.received_month_hour_four_auxiliary_star_placements.v1";
pub const ZIWEI_LIFE_BODY_AUDIT_VERSION: u16 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZiweiPalaceKind {
    Life,
    Siblings,
    Spouse,
    Children,
    Wealth,
    Health,
    Travel,
    Servants,
    Career,
    Property,
    Fortune,
    Parents,
}

impl ZiweiPalaceKind {
    pub const ALL_IN_RECEIVED_REVERSE_ORDER: [Self; 12] = [
        Self::Life,
        Self::Siblings,
        Self::Spouse,
        Self::Children,
        Self::Wealth,
        Self::Health,
        Self::Travel,
        Self::Servants,
        Self::Career,
        Self::Property,
        Self::Fortune,
        Self::Parents,
    ];

    pub const fn ui_zh_name(self) -> &'static str {
        match self {
            Self::Life => "命宫",
            Self::Siblings => "兄弟宫",
            Self::Spouse => "夫妻宫",
            Self::Children => "子女宫",
            Self::Wealth => "财帛宫",
            Self::Health => "疾厄宫",
            Self::Travel => "迁移宫",
            Self::Servants => "奴仆宫",
            Self::Career => "官禄宫",
            Self::Property => "田宅宫",
            Self::Fortune => "福德宫",
            Self::Parents => "父母宫",
        }
    }

    pub const fn received_zh_label(self) -> &'static str {
        match self {
            Self::Life => "命宫",
            Self::Siblings => "兄弟",
            Self::Spouse => "妻妾",
            Self::Children => "子女",
            Self::Wealth => "财帛",
            Self::Health => "疾厄",
            Self::Travel => "迁移",
            Self::Servants => "奴仆",
            Self::Career => "官禄",
            Self::Property => "田宅",
            Self::Fortune => "福德",
            Self::Parents => "父母",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZiweiLifeBodyInput {
    pub lunar_month_number: u8,
    pub is_leap_month: bool,
    pub birth_hour_branch: EarthlyBranch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZiweiPalacePlacement {
    pub received_order_from_life: u8,
    pub kind: ZiweiPalaceKind,
    pub branch: EarthlyBranch,
    pub received_zh_label: String,
    pub ui_zh_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZiweiLifeBodyChart {
    pub input: ZiweiLifeBodyInput,
    pub effective_month_number: u8,
    pub leap_month_treated_as_next_month: bool,
    pub month_anchor_branch: EarthlyBranch,
    pub life_palace_branch: EarthlyBranch,
    pub body_palace_branch: EarthlyBranch,
    pub palaces: Vec<ZiweiPalacePlacement>,
    pub algorithm_version: String,
    pub status: String,
    pub rule_eligible: bool,
}

impl ZiweiLifeBodyChart {
    pub fn compact_zh(&self) -> String {
        format!(
            "命宫{} · 身宫{} · 月基{}{}",
            self.life_palace_branch.zh_name(),
            self.body_palace_branch.zh_name(),
            self.month_anchor_branch.zh_name(),
            if self.leap_month_treated_as_next_month {
                "（闰月作下月）"
            } else {
                ""
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZiweiFiveElementBureau {
    WaterTwo,
    WoodThree,
    MetalFour,
    EarthFive,
    FireSix,
}

impl ZiweiFiveElementBureau {
    pub const fn from_element(element: FiveElement) -> Self {
        match element {
            FiveElement::Water => Self::WaterTwo,
            FiveElement::Wood => Self::WoodThree,
            FiveElement::Metal => Self::MetalFour,
            FiveElement::Earth => Self::EarthFive,
            FiveElement::Fire => Self::FireSix,
        }
    }

    pub const fn number(self) -> u8 {
        match self {
            Self::WaterTwo => 2,
            Self::WoodThree => 3,
            Self::MetalFour => 4,
            Self::EarthFive => 5,
            Self::FireSix => 6,
        }
    }

    pub const fn five_element(self) -> FiveElement {
        match self {
            Self::WaterTwo => FiveElement::Water,
            Self::WoodThree => FiveElement::Wood,
            Self::MetalFour => FiveElement::Metal,
            Self::EarthFive => FiveElement::Earth,
            Self::FireSix => FiveElement::Fire,
        }
    }

    pub fn zh_name(self) -> String {
        format!("{}{}局", self.five_element().zh_name(), self.number())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZiweiBranchStemPlacement {
    pub branch: EarthlyBranch,
    pub stem: HeavenlyStem,
    pub designation: SexagenaryDesignation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZiweiBureauCandidate {
    pub life_palace_branch: EarthlyBranch,
    pub yin_palace_stem: HeavenlyStem,
    pub branch_stems: Vec<ZiweiBranchStemPlacement>,
    pub life_palace_designation: SexagenaryDesignation,
    pub life_palace_nayin: NayinDesignation,
    pub five_element_bureau: ZiweiFiveElementBureau,
    pub mechanical_boundaries_safe: bool,
    pub algorithm_version: String,
    pub status: String,
    pub interpretation_eligible: bool,
    pub rule_eligible: bool,
}

impl ZiweiBureauCandidate {
    pub fn compact_zh(&self) -> String {
        format!(
            "命宫{} · {} · {}",
            self.life_palace_designation.zh_name(),
            self.life_palace_nayin.received_zh_name,
            self.five_element_bureau.zh_name()
        )
    }
}

                                                                         
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZiweiStarAnchorCandidate {
    pub lunar_day: u8,
    pub five_element_bureau: ZiweiFiveElementBureau,
    pub complement_to_divisible: u8,
    pub quotient_from_yin: u8,
    pub ziwei_branch: EarthlyBranch,
    pub mechanical_boundaries_safe: bool,
    pub algorithm_version: String,
    pub interpretation_eligible: bool,
    pub rule_eligible: bool,
}

impl ZiweiStarAnchorCandidate {
    pub fn compact_zh(&self) -> String {
        format!(
            "{}日 · {} · 补{}成整局 · 自寅数{} · 紫微在{}",
            self.lunar_day,
            self.five_element_bureau.zh_name(),
            self.complement_to_divisible,
            self.quotient_from_yin,
            self.ziwei_branch.zh_name()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZiweiStarAnchorConventionCandidate {
    pub year_convention: String,
    pub hour_convention: String,
    pub anchor: ZiweiStarAnchorCandidate,
    #[serde(default)]
    pub major_stars: Option<ZiweiMajorStarChart>,
    #[serde(default)]
    pub auxiliary_stars: Option<ZiweiAuxiliaryStarChart>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZiweiAuxiliaryStarKind {
    Wenchang,
    Wenqu,
    Zuofu,
    Youbi,
}

impl ZiweiAuxiliaryStarKind {
    pub const ALL_IN_RECEIVED_ORDER: [Self; 4] =
        [Self::Wenchang, Self::Wenqu, Self::Zuofu, Self::Youbi];

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Wenchang => "文昌",
            Self::Wenqu => "文曲",
            Self::Zuofu => "左辅",
            Self::Youbi => "右弼",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZiweiAuxiliaryStarPlacement {
    pub kind: ZiweiAuxiliaryStarKind,
    pub branch: EarthlyBranch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZiweiAuxiliaryStarChart {
    pub effective_month_number: u8,
    pub birth_hour_branch: EarthlyBranch,
    pub placements: Vec<ZiweiAuxiliaryStarPlacement>,
    pub mechanical_boundaries_safe: bool,
    pub algorithm_version: String,
    pub interpretation_eligible: bool,
    pub rule_eligible: bool,
}

impl ZiweiAuxiliaryStarChart {
    pub fn compact_zh(&self) -> String {
        self.placements
            .iter()
            .map(|placement| format!("{}{}", placement.kind.zh_name(), placement.branch.zh_name()))
            .collect::<Vec<_>>()
            .join(" · ")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZiweiMajorStarKind {
    Ziwei,
    Tianji,
    Taiyang,
    Wuqu,
    Tiantong,
    Lianzhen,
    Tianfu,
    Taiyin,
    Tanlang,
    Jumen,
    Tianxiang,
    Tianliang,
    Qisha,
    Pojun,
}

impl ZiweiMajorStarKind {
    pub const ALL_IN_RECEIVED_ORDER: [Self; 14] = [
        Self::Ziwei,
        Self::Tianji,
        Self::Taiyang,
        Self::Wuqu,
        Self::Tiantong,
        Self::Lianzhen,
        Self::Tianfu,
        Self::Taiyin,
        Self::Tanlang,
        Self::Jumen,
        Self::Tianxiang,
        Self::Tianliang,
        Self::Qisha,
        Self::Pojun,
    ];

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Ziwei => "紫微",
            Self::Tianji => "天机",
            Self::Taiyang => "太阳",
            Self::Wuqu => "武曲",
            Self::Tiantong => "天同",
            Self::Lianzhen => "廉贞",
            Self::Tianfu => "天府",
            Self::Taiyin => "太阴",
            Self::Tanlang => "贪狼",
            Self::Jumen => "巨门",
            Self::Tianxiang => "天相",
            Self::Tianliang => "天梁",
            Self::Qisha => "七杀",
            Self::Pojun => "破军",
        }
    }

    pub const fn group(self) -> &'static str {
        match self {
            Self::Ziwei
            | Self::Tianji
            | Self::Taiyang
            | Self::Wuqu
            | Self::Tiantong
            | Self::Lianzhen => "ziwei_reverse",
            _ => "tianfu_forward",
        }
    }

    pub const fn offset_from_group_anchor(self) -> i8 {
        match self {
            Self::Ziwei | Self::Tianfu => 0,
            Self::Tianji => -1,
            Self::Taiyang => -3,
            Self::Wuqu => -4,
            Self::Tiantong => -5,
            Self::Lianzhen => -8,
            Self::Taiyin => 1,
            Self::Tanlang => 2,
            Self::Jumen => 3,
            Self::Tianxiang => 4,
            Self::Tianliang => 5,
            Self::Qisha => 6,
            Self::Pojun => 10,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZiweiMajorStarPlacement {
    pub kind: ZiweiMajorStarKind,
    pub group: String,
    pub offset_from_group_anchor: i8,
    pub branch: EarthlyBranch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZiweiMajorStarChart {
    pub ziwei_branch: EarthlyBranch,
    pub tianfu_branch: EarthlyBranch,
    pub placements: Vec<ZiweiMajorStarPlacement>,
    pub mechanical_boundaries_safe: bool,
    pub algorithm_version: String,
    pub interpretation_eligible: bool,
    pub rule_eligible: bool,
}

impl ZiweiMajorStarChart {
    pub fn compact_zh(&self) -> String {
        self.placements
            .iter()
            .map(|placement| format!("{}{}", placement.kind.zh_name(), placement.branch.zh_name()))
            .collect::<Vec<_>>()
            .join(" · ")
    }

    pub fn group_compact_zh(&self, group: &str) -> String {
        self.placements
            .iter()
            .filter(|placement| placement.group == group)
            .map(|placement| format!("{}{}", placement.kind.zh_name(), placement.branch.zh_name()))
            .collect::<Vec<_>>()
            .join(" · ")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZiweiYearStemConventionCandidate {
    pub year_convention: String,
    pub year_designation: SexagenaryDesignation,
    pub year_boundary_safe: bool,
    pub beijing_time_candidate: ZiweiBureauCandidate,
    pub true_solar_time_candidate: ZiweiBureauCandidate,
    pub hour_candidates_identical: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZiweiHourConventionCandidate {
    pub hour_convention: String,
    pub shichen: ShichenBoundary,
    pub mechanical_boundaries_safe: bool,
    pub chart: ZiweiLifeBodyChart,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZiweiLifeBodyAudit {
    pub audit_version: u16,
    pub lunisolar_date_candidate: LunisolarDateCandidate,
    pub calendar_convention: String,
    pub beijing_time_candidate: ZiweiHourConventionCandidate,
    pub true_solar_time_candidate: ZiweiHourConventionCandidate,
    pub candidates_identical: bool,
    pub selected_hour_convention: Option<String>,
    #[serde(default)]
    pub year_stem_convention_candidates: Vec<ZiweiYearStemConventionCandidate>,
    #[serde(default)]
    pub year_stem_conventions_identical: bool,
    #[serde(default)]
    pub selected_year_stem_convention: Option<String>,
    #[serde(default)]
    pub star_anchor_candidates: Vec<ZiweiStarAnchorConventionCandidate>,
    pub blocked_reasons: Vec<String>,
    pub algorithm_version: String,
    pub status: String,
    pub rule_eligible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ZiweiLifeBodyError {
    #[error("紫微命身宫的出生月必须为 1 至 12")]
    InvalidLunarMonth,
    #[error("所核传本文字只说闰月作下月，未说明闰十二月如何跨年，失败关闭")]
    LeapTwelfthMonthUnresolved,
    #[error("安紫微星的农历日必须为 1 至 30")]
    InvalidLunarDay,
}

pub fn ziwei_star_anchor_candidate(
    lunar_day: u8,
    bureau: ZiweiFiveElementBureau,
    mechanical_boundaries_safe: bool,
) -> Result<ZiweiStarAnchorCandidate, ZiweiLifeBodyError> {
    if !(1..=30).contains(&lunar_day) {
        return Err(ZiweiLifeBodyError::InvalidLunarDay);
    }
    let number = bureau.number();
    let complement = (number - lunar_day % number) % number;
    let quotient = (lunar_day + complement) / number;
    let correction = if complement.is_multiple_of(2) {
        i16::from(complement)
    } else {
        -i16::from(complement)
    };
    let index = (i16::from(EarthlyBranch::Yin.index()) + i16::from(quotient) - 1 + correction)
        .rem_euclid(12) as usize;
    Ok(ZiweiStarAnchorCandidate {
        lunar_day,
        five_element_bureau: bureau,
        complement_to_divisible: complement,
        quotient_from_yin: quotient,
        ziwei_branch: EarthlyBranch::ALL[index],
        mechanical_boundaries_safe,
        algorithm_version: ZIWEI_STAR_ANCHOR_ALGORITHM_VERSION.to_owned(),
        interpretation_eligible: false,
        rule_eligible: false,
    })
}

pub fn ziwei_major_star_chart(anchor: &ZiweiStarAnchorCandidate) -> ZiweiMajorStarChart {
    let ziwei_index = i16::from(anchor.ziwei_branch.index());
                                                                                   
                                                                                
    let tianfu_index = (4 - ziwei_index).rem_euclid(12);
    let tianfu_branch = EarthlyBranch::ALL[tianfu_index as usize];
    let placements = ZiweiMajorStarKind::ALL_IN_RECEIVED_ORDER
        .into_iter()
        .map(|kind| {
            let base = if kind.group() == "ziwei_reverse" {
                ziwei_index
            } else {
                tianfu_index
            };
            let index = (base + i16::from(kind.offset_from_group_anchor())).rem_euclid(12);
            ZiweiMajorStarPlacement {
                kind,
                group: kind.group().to_owned(),
                offset_from_group_anchor: kind.offset_from_group_anchor(),
                branch: EarthlyBranch::ALL[index as usize],
            }
        })
        .collect();
    ZiweiMajorStarChart {
        ziwei_branch: anchor.ziwei_branch,
        tianfu_branch,
        placements,
        mechanical_boundaries_safe: anchor.mechanical_boundaries_safe,
        algorithm_version: ZIWEI_MAJOR_STARS_ALGORITHM_VERSION.to_owned(),
        interpretation_eligible: false,
        rule_eligible: false,
    }
}

                                                                               
                                                                
pub fn ziwei_auxiliary_star_chart(
    effective_month_number: u8,
    birth_hour_branch: EarthlyBranch,
    mechanical_boundaries_safe: bool,
) -> Result<ZiweiAuxiliaryStarChart, ZiweiLifeBodyError> {
    if !(1..=12).contains(&effective_month_number) {
        return Err(ZiweiLifeBodyError::InvalidLunarMonth);
    }
    let hour_offset = i16::from(birth_hour_branch.index());
    let month_offset = i16::from(effective_month_number - 1);
    let branch_at = |anchor: EarthlyBranch, offset: i16| {
        EarthlyBranch::ALL[(i16::from(anchor.index()) + offset).rem_euclid(12) as usize]
    };
    let placements = [
        (
            ZiweiAuxiliaryStarKind::Wenchang,
            branch_at(EarthlyBranch::Xu, -hour_offset),
        ),
        (
            ZiweiAuxiliaryStarKind::Wenqu,
            branch_at(EarthlyBranch::Chen, hour_offset),
        ),
        (
            ZiweiAuxiliaryStarKind::Zuofu,
            branch_at(EarthlyBranch::Chen, month_offset),
        ),
        (
            ZiweiAuxiliaryStarKind::Youbi,
            branch_at(EarthlyBranch::Xu, -month_offset),
        ),
    ]
    .into_iter()
    .map(|(kind, branch)| ZiweiAuxiliaryStarPlacement { kind, branch })
    .collect();
    Ok(ZiweiAuxiliaryStarChart {
        effective_month_number,
        birth_hour_branch,
        placements,
        mechanical_boundaries_safe,
        algorithm_version: ZIWEI_AUXILIARY_STARS_ALGORITHM_VERSION.to_owned(),
        interpretation_eligible: false,
        rule_eligible: false,
    })
}

pub fn ziwei_life_body_chart(
    input: ZiweiLifeBodyInput,
) -> Result<ZiweiLifeBodyChart, ZiweiLifeBodyError> {
    if !(1..=12).contains(&input.lunar_month_number) {
        return Err(ZiweiLifeBodyError::InvalidLunarMonth);
    }
    if input.is_leap_month && input.lunar_month_number == 12 {
        return Err(ZiweiLifeBodyError::LeapTwelfthMonthUnresolved);
    }

    let effective_month_number = input.lunar_month_number + u8::from(input.is_leap_month);
    let month_anchor_index = (EarthlyBranch::Yin.index() + effective_month_number - 1) % 12;
    let hour_index = input.birth_hour_branch.index();
    let life_index = (month_anchor_index + 12 - hour_index) % 12;
    let body_index = (month_anchor_index + hour_index) % 12;
    let month_anchor_branch = EarthlyBranch::ALL[month_anchor_index as usize];
    let life_palace_branch = EarthlyBranch::ALL[life_index as usize];
    let body_palace_branch = EarthlyBranch::ALL[body_index as usize];

    let palaces = ZiweiPalaceKind::ALL_IN_RECEIVED_REVERSE_ORDER
        .into_iter()
        .enumerate()
        .map(|(index, kind)| ZiweiPalacePlacement {
            received_order_from_life: index as u8 + 1,
            kind,
            branch: EarthlyBranch::ALL[((life_index + 12 - index as u8) % 12) as usize],
            received_zh_label: kind.received_zh_label().to_owned(),
            ui_zh_name: kind.ui_zh_name().to_owned(),
        })
        .collect();

    Ok(ZiweiLifeBodyChart {
        input,
        effective_month_number,
        leap_month_treated_as_next_month: input.is_leap_month,
        month_anchor_branch,
        life_palace_branch,
        body_palace_branch,
        palaces,
        algorithm_version: ZIWEI_LIFE_BODY_ALGORITHM_VERSION.to_owned(),
        status: ZIWEI_LIFE_BODY_STATUS.to_owned(),
        rule_eligible: false,
    })
}

fn designation_for(stem: HeavenlyStem, branch: EarthlyBranch) -> SexagenaryDesignation {
    (0..60)
        .map(SexagenaryDesignation::from_cycle_index)
        .find(|candidate| candidate.stem == stem && candidate.branch == branch)
        .expect("the received Yin-stem sequence always forms a valid designation")
}

pub fn ziwei_bureau_candidate(
    year_stem: HeavenlyStem,
    chart: &ZiweiLifeBodyChart,
    mechanical_boundaries_safe: bool,
) -> ZiweiBureauCandidate {
    let yin_palace_stem = HeavenlyStem::ALL[(((year_stem.index() % 5) * 2 + 2) % 10) as usize];
    let branch_stems = EarthlyBranch::ALL
        .into_iter()
        .map(|branch| {
            let offset = (branch.index() + 12 - EarthlyBranch::Yin.index()) % 12;
            let stem = HeavenlyStem::ALL[((yin_palace_stem.index() + offset) % 10) as usize];
            ZiweiBranchStemPlacement {
                branch,
                stem,
                designation: designation_for(stem, branch),
            }
        })
        .collect::<Vec<_>>();
    let life_palace_designation = branch_stems
        .iter()
        .find(|placement| placement.branch == chart.life_palace_branch)
        .expect("all twelve branches are assigned")
        .designation;
    let life_palace_nayin = nayin_for_designation(life_palace_designation);
    let five_element_bureau = ZiweiFiveElementBureau::from_element(life_palace_nayin.five_element);

    ZiweiBureauCandidate {
        life_palace_branch: chart.life_palace_branch,
        yin_palace_stem,
        branch_stems,
        life_palace_designation,
        life_palace_nayin,
        five_element_bureau,
        mechanical_boundaries_safe,
        algorithm_version: ZIWEI_FIVE_ELEMENT_BUREAU_ALGORITHM_VERSION.to_owned(),
        status: "mechanical_palaces_and_bureau_no_stars_interpretation_or_runtime_signals.v2"
            .to_owned(),
        interpretation_eligible: false,
        rule_eligible: false,
    }
}

fn year_convention_candidate(
    convention: &str,
    year_designation: SexagenaryDesignation,
    year_boundary_safe: bool,
    beijing_chart: &ZiweiLifeBodyChart,
    true_solar_chart: &ZiweiLifeBodyChart,
    beijing_hour_safe: bool,
    true_solar_hour_safe: bool,
) -> ZiweiYearStemConventionCandidate {
    let beijing_time_candidate = ziwei_bureau_candidate(
        year_designation.stem,
        beijing_chart,
        year_boundary_safe && beijing_hour_safe,
    );
    let true_solar_time_candidate = ziwei_bureau_candidate(
        year_designation.stem,
        true_solar_chart,
        year_boundary_safe && true_solar_hour_safe,
    );
    let hour_candidates_identical =
        bureau_structures_match(&beijing_time_candidate, &true_solar_time_candidate);
    ZiweiYearStemConventionCandidate {
        year_convention: convention.to_owned(),
        year_designation,
        year_boundary_safe,
        beijing_time_candidate,
        true_solar_time_candidate,
        hour_candidates_identical,
    }
}

fn bureau_structures_match(left: &ZiweiBureauCandidate, right: &ZiweiBureauCandidate) -> bool {
    left.life_palace_branch == right.life_palace_branch
        && left.yin_palace_stem == right.yin_palace_stem
        && left.branch_stems == right.branch_stems
        && left.life_palace_designation == right.life_palace_designation
        && left.life_palace_nayin == right.life_palace_nayin
        && left.five_element_bureau == right.five_element_bureau
}

pub fn ziwei_life_body_audit(
    lunisolar_date: &LunisolarDateCandidate,
    lichun_year: &LichunYearCandidate,
    beijing_shichen: ShichenBoundary,
    true_solar_shichen: ShichenBoundary,
) -> Result<ZiweiLifeBodyAudit, ZiweiLifeBodyError> {
    let common = ZiweiLifeBodyInput {
        lunar_month_number: lunisolar_date.month_number,
        is_leap_month: lunisolar_date.is_leap_month,
        birth_hour_branch: beijing_shichen.branch,
    };
    let beijing_chart = ziwei_life_body_chart(common)?;
    let true_solar_chart = ziwei_life_body_chart(ZiweiLifeBodyInput {
        birth_hour_branch: true_solar_shichen.branch,
        ..common
    })?;
    let candidates_identical = beijing_chart == true_solar_chart;
    let nominal_lunar_year =
        sexagenary_year_for_astronomical_year_label(lunisolar_date.nominal_lunar_year);
    let nominal_candidate = year_convention_candidate(
        "modern_nominal_lunar_year_label.v1",
        nominal_lunar_year,
        lunisolar_date.date_assignment_safe,
        &beijing_chart,
        &true_solar_chart,
        beijing_shichen.boundary_safe,
        true_solar_shichen.boundary_safe,
    );
    let lichun_candidate = year_convention_candidate(
        "astronomical_lichun_year_candidate.v1",
        lichun_year.designation,
        lichun_year.boundary_safe,
        &beijing_chart,
        &true_solar_chart,
        beijing_shichen.boundary_safe,
        true_solar_shichen.boundary_safe,
    );
    let year_stem_conventions_identical =
        nominal_candidate.year_designation.stem == lichun_candidate.year_designation.stem;
    let mut blocked_reasons = vec![
        "received_book_attribution_and_edition_history_not_fully_resolved".to_owned(),
        "modern_official_lunisolar_candidate_not_verified_as_historical_birth_calendar".to_owned(),
        "beijing_vs_true_solar_birth_hour_basis_not_selected".to_owned(),
        "nominal_lunar_year_vs_lichun_birth_year_stem_basis_not_selected".to_owned(),
        "received_nayin_text_not_bound_to_reviewed_primary_facsimile_pages".to_owned(),
        "ziwei_star_anchor_transcription_not_bound_to_reviewed_facsimile_pages".to_owned(),
        "fourteen_main_star_placements_not_bound_to_reviewed_facsimile_pages".to_owned(),
        "four_auxiliary_star_placements_not_bound_to_reviewed_facsimile_pages".to_owned(),
        "remaining_auxiliary_stars_four_transformations_limits_and_interpretation_not_implemented"
            .to_owned(),
    ];
    if !lunisolar_date.date_assignment_safe {
        blocked_reasons.push("birth_lunisolar_date_assignment_boundary_not_safe".to_owned());
    }

    let mut star_anchor_candidates = Vec::with_capacity(4);
    for year in [&nominal_candidate, &lichun_candidate] {
        for (hour_convention, bureau, chart) in [
            (
                "beijing_standard_two_hour_branch.v1",
                &year.beijing_time_candidate,
                &beijing_chart,
            ),
            (
                "local_true_solar_two_hour_branch.v1",
                &year.true_solar_time_candidate,
                &true_solar_chart,
            ),
        ] {
            let anchor = ziwei_star_anchor_candidate(
                lunisolar_date.day_number,
                bureau.five_element_bureau,
                lunisolar_date.date_assignment_safe && bureau.mechanical_boundaries_safe,
            )?;
            star_anchor_candidates.push(ZiweiStarAnchorConventionCandidate {
                year_convention: year.year_convention.clone(),
                hour_convention: hour_convention.to_owned(),
                major_stars: Some(ziwei_major_star_chart(&anchor)),
                auxiliary_stars: Some(ziwei_auxiliary_star_chart(
                    chart.effective_month_number,
                    chart.input.birth_hour_branch,
                    lunisolar_date.date_assignment_safe && bureau.mechanical_boundaries_safe,
                )?),
                anchor,
            });
        }
    }

    Ok(ZiweiLifeBodyAudit {
        audit_version: ZIWEI_LIFE_BODY_AUDIT_VERSION,
        lunisolar_date_candidate: lunisolar_date.clone(),
        calendar_convention: "modern_official_lunisolar_candidate_not_historical_calendar.v1"
            .to_owned(),
        beijing_time_candidate: ZiweiHourConventionCandidate {
            hour_convention: "beijing_standard_two_hour_branch.v1".to_owned(),
            shichen: beijing_shichen,
            mechanical_boundaries_safe: lunisolar_date.date_assignment_safe
                && beijing_shichen.boundary_safe,
            chart: beijing_chart,
        },
        true_solar_time_candidate: ZiweiHourConventionCandidate {
            hour_convention: "local_true_solar_two_hour_branch.v1".to_owned(),
            shichen: true_solar_shichen,
            mechanical_boundaries_safe: lunisolar_date.date_assignment_safe
                && true_solar_shichen.boundary_safe,
            chart: true_solar_chart,
        },
        candidates_identical,
        selected_hour_convention: None,
        year_stem_convention_candidates: vec![nominal_candidate, lichun_candidate],
        year_stem_conventions_identical,
        selected_year_stem_convention: None,
        star_anchor_candidates,
        blocked_reasons,
        algorithm_version: ZIWEI_MECHANICAL_AUDIT_ALGORITHM_VERSION.to_owned(),
        status: ZIWEI_MECHANICAL_AUDIT_STATUS.to_owned(),
        rule_eligible: false,
    })
}

pub fn ziwei_life_body_audits_match(
    current: &ZiweiLifeBodyAudit,
    stored: &ZiweiLifeBodyAudit,
) -> bool {
    if current.audit_version == stored.audit_version {
        return current == stored;
    }
    let stored_version = stored.audit_version;
    current.audit_version == 5
        && matches!(stored.audit_version, 1..=4)
        && current.lunisolar_date_candidate == stored.lunisolar_date_candidate
        && current.calendar_convention == stored.calendar_convention
        && current.beijing_time_candidate == stored.beijing_time_candidate
        && current.true_solar_time_candidate == stored.true_solar_time_candidate
        && current.candidates_identical == stored.candidates_identical
        && current.selected_hour_convention == stored.selected_hour_convention
        && (stored.audit_version < 2
            || (current.year_stem_convention_candidates == stored.year_stem_convention_candidates
                && current.year_stem_conventions_identical
                    == stored.year_stem_conventions_identical
                && current.selected_year_stem_convention == stored.selected_year_stem_convention))
        && (stored.audit_version < 3
            || (current.star_anchor_candidates.len() == stored.star_anchor_candidates.len()
                && current
                    .star_anchor_candidates
                    .iter()
                    .zip(&stored.star_anchor_candidates)
                    .all(|(current, stored)| {
                        current.year_convention == stored.year_convention
                            && current.hour_convention == stored.hour_convention
                            && current.anchor == stored.anchor
                            && (stored_version < 4
                                || current.major_stars == stored.major_stars)
                    })))
        && stored.blocked_reasons.iter().all(|reason| {
            current.blocked_reasons.contains(reason)
                || matches!(
                    reason.as_str(),
                    "main_star_and_interpretation_layers_not_implemented"
                        | "fourteen_main_stars_and_interpretation_layers_not_implemented"
                        | "auxiliary_stars_four_transformations_limits_and_interpretation_not_implemented"
                )
        })
        && !current.rule_eligible
        && !stored.rule_eligible
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HistoricalDateTime;

    fn shichen(branch: EarthlyBranch) -> ShichenBoundary {
        ShichenBoundary {
            branch,
            minutes_to_boundary: 30.0,
            boundary_margin_minutes: 5.0,
            boundary_safe: true,
        }
    }

    fn lunar(month: u8, leap: bool) -> LunisolarDateCandidate {
        LunisolarDateCandidate {
            target_bjt: HistoricalDateTime {
                year: 2026,
                month: 8,
                day: 30,
                hour: 8,
                minute: 30,
                second: 0,
            },
            target_day_number: 0,
            structure_winter_solstice_year: 2025,
            nominal_lunar_year: 2026,
            month_sequence_index: month,
            month_number: month,
            is_leap_month: leap,
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
            algorithm_version: "test".to_owned(),
            status: "test".to_owned(),
        }
    }

    fn lichun_year(year: i32) -> LichunYearCandidate {
        LichunYearCandidate {
            civil_year: year,
            effective_year_label: year,
            designation: sexagenary_year_for_astronomical_year_label(year),
            boundary_jd_tt: 2_460_000.0,
            seconds_from_boundary: 86_400.0,
            boundary_margin_seconds: 300.0,
            boundary_safe: true,
            algorithm_version: "test".to_owned(),
            status: "test".to_owned(),
        }
    }

    #[test]
    fn reproduces_yuyue_second_month_chen_hour_example() {
        let chart = ziwei_life_body_chart(ZiweiLifeBodyInput {
            lunar_month_number: 2,
            is_leap_month: false,
            birth_hour_branch: EarthlyBranch::Chen,
        })
        .unwrap();
        assert_eq!(chart.month_anchor_branch, EarthlyBranch::Mao);
        assert_eq!(chart.life_palace_branch, EarthlyBranch::Hai);
        assert_eq!(chart.body_palace_branch, EarthlyBranch::Wei);
        assert!(!chart.rule_eligible);
    }

    #[test]
    fn reproduces_received_first_month_hour_examples() {
        let zi = ziwei_life_body_chart(ZiweiLifeBodyInput {
            lunar_month_number: 1,
            is_leap_month: false,
            birth_hour_branch: EarthlyBranch::Zi,
        })
        .unwrap();
        assert_eq!(
            (zi.life_palace_branch, zi.body_palace_branch),
            (EarthlyBranch::Yin, EarthlyBranch::Yin)
        );

        let chou = ziwei_life_body_chart(ZiweiLifeBodyInput {
            birth_hour_branch: EarthlyBranch::Chou,
            ..zi.input
        })
        .unwrap();
        assert_eq!(
            (chou.life_palace_branch, chou.body_palace_branch),
            (EarthlyBranch::Chou, EarthlyBranch::Mao)
        );

        let yin = ziwei_life_body_chart(ZiweiLifeBodyInput {
            birth_hour_branch: EarthlyBranch::Yin,
            ..zi.input
        })
        .unwrap();
        assert_eq!(
            (yin.life_palace_branch, yin.body_palace_branch),
            (EarthlyBranch::Zi, EarthlyBranch::Chen)
        );
    }

    #[test]
    fn distributes_all_twelve_palaces_in_received_reverse_order() {
        let chart = ziwei_life_body_chart(ZiweiLifeBodyInput {
            lunar_month_number: 11,
            is_leap_month: false,
            birth_hour_branch: EarthlyBranch::Zi,
        })
        .unwrap();
        assert_eq!(chart.life_palace_branch, EarthlyBranch::Zi);
        assert_eq!(chart.palaces.len(), 12);
        assert_eq!(chart.palaces[0].kind, ZiweiPalaceKind::Life);
        assert_eq!(chart.palaces[0].branch, EarthlyBranch::Zi);
        assert_eq!(chart.palaces[1].kind, ZiweiPalaceKind::Siblings);
        assert_eq!(chart.palaces[1].branch, EarthlyBranch::Hai);
        assert_eq!(chart.palaces[2].kind, ZiweiPalaceKind::Spouse);
        assert_eq!(chart.palaces[2].branch, EarthlyBranch::Xu);
        assert_eq!(chart.palaces[11].kind, ZiweiPalaceKind::Parents);
        assert_eq!(chart.palaces[11].branch, EarthlyBranch::Chou);
    }

    #[test]
    fn received_leap_month_rule_uses_the_next_month() {
        let leap_first = ziwei_life_body_chart(ZiweiLifeBodyInput {
            lunar_month_number: 1,
            is_leap_month: true,
            birth_hour_branch: EarthlyBranch::Zi,
        })
        .unwrap();
        let ordinary_second = ziwei_life_body_chart(ZiweiLifeBodyInput {
            lunar_month_number: 2,
            is_leap_month: false,
            birth_hour_branch: EarthlyBranch::Zi,
        })
        .unwrap();
        assert_eq!(
            leap_first.life_palace_branch,
            ordinary_second.life_palace_branch
        );
        assert_eq!(
            leap_first.body_palace_branch,
            ordinary_second.body_palace_branch
        );
        assert!(leap_first.leap_month_treated_as_next_month);
    }

    #[test]
    fn life_and_body_coincide_only_at_zi_and_wu_hours_for_every_month() {
        for month in 1..=12 {
            for branch in EarthlyBranch::ALL {
                let chart = ziwei_life_body_chart(ZiweiLifeBodyInput {
                    lunar_month_number: month,
                    is_leap_month: false,
                    birth_hour_branch: branch,
                })
                .unwrap();
                assert_eq!(
                    chart.life_palace_branch == chart.body_palace_branch,
                    matches!(branch, EarthlyBranch::Zi | EarthlyBranch::Wu)
                );
            }
        }
    }

    #[test]
    fn rejects_invalid_month_and_unresolved_leap_twelfth_month() {
        assert_eq!(
            ziwei_life_body_chart(ZiweiLifeBodyInput {
                lunar_month_number: 0,
                is_leap_month: false,
                birth_hour_branch: EarthlyBranch::Zi,
            })
            .unwrap_err(),
            ZiweiLifeBodyError::InvalidLunarMonth
        );
        assert_eq!(
            ziwei_life_body_chart(ZiweiLifeBodyInput {
                lunar_month_number: 12,
                is_leap_month: true,
                birth_hour_branch: EarthlyBranch::Zi,
            })
            .unwrap_err(),
            ZiweiLifeBodyError::LeapTwelfthMonthUnresolved
        );
    }

    #[test]
    fn audit_keeps_beijing_and_true_solar_hour_candidates_parallel() {
        let audit = ziwei_life_body_audit(
            &lunar(2, false),
            &lichun_year(2026),
            shichen(EarthlyBranch::Chen),
            shichen(EarthlyBranch::Si),
        )
        .unwrap();
        assert!(!audit.candidates_identical);
        assert_eq!(audit.selected_hour_convention, None);
        assert_eq!(
            audit.beijing_time_candidate.chart.life_palace_branch,
            EarthlyBranch::Hai
        );
        assert_eq!(
            audit.true_solar_time_candidate.chart.life_palace_branch,
            EarthlyBranch::Xu
        );
        assert_eq!(audit.year_stem_convention_candidates.len(), 2);
        assert!(audit.year_stem_conventions_identical);
        assert_eq!(audit.selected_year_stem_convention, None);
        assert!(!audit.rule_eligible);
    }

    #[test]
    fn reproduces_received_jia_ji_year_bureau_examples() {
        let yin_chart = ziwei_life_body_chart(ZiweiLifeBodyInput {
            lunar_month_number: 1,
            is_leap_month: false,
            birth_hour_branch: EarthlyBranch::Zi,
        })
        .unwrap();
        let yin = ziwei_bureau_candidate(HeavenlyStem::Jia, &yin_chart, true);
        assert_eq!(yin.yin_palace_stem, HeavenlyStem::Bing);
        assert_eq!(yin.life_palace_designation.zh_name(), "丙寅");
        assert_eq!(yin.life_palace_nayin.received_zh_name, "炉中火");
        assert_eq!(yin.five_element_bureau, ZiweiFiveElementBureau::FireSix);

        let chen_chart = ziwei_life_body_chart(ZiweiLifeBodyInput {
            lunar_month_number: 3,
            is_leap_month: false,
            birth_hour_branch: EarthlyBranch::Zi,
        })
        .unwrap();
        let chen = ziwei_bureau_candidate(HeavenlyStem::Ji, &chen_chart, true);
        assert_eq!(chen.life_palace_designation.zh_name(), "戊辰");
        assert_eq!(chen.life_palace_nayin.received_zh_name, "大林木");
        assert_eq!(chen.five_element_bureau, ZiweiFiveElementBureau::WoodThree);
    }

    #[test]
    fn reproduces_received_yi_geng_year_zi_chou_fire_bureau_example() {
        for life_branch in [EarthlyBranch::Zi, EarthlyBranch::Chou] {
            let chart = ZiweiLifeBodyChart {
                input: ZiweiLifeBodyInput {
                    lunar_month_number: 1,
                    is_leap_month: false,
                    birth_hour_branch: EarthlyBranch::Zi,
                },
                effective_month_number: 1,
                leap_month_treated_as_next_month: false,
                month_anchor_branch: EarthlyBranch::Yin,
                life_palace_branch: life_branch,
                body_palace_branch: EarthlyBranch::Yin,
                palaces: Vec::new(),
                algorithm_version: ZIWEI_LIFE_BODY_ALGORITHM_VERSION.to_owned(),
                status: ZIWEI_LIFE_BODY_STATUS.to_owned(),
                rule_eligible: false,
            };
            let bureau = ziwei_bureau_candidate(HeavenlyStem::Geng, &chart, true);
            assert_eq!(bureau.yin_palace_stem, HeavenlyStem::Wu);
            assert!(matches!(
                bureau.life_palace_designation.zh_name().as_str(),
                "戊子" | "己丑"
            ));
            assert_eq!(bureau.life_palace_nayin.received_zh_name, "霹雳火");
            assert_eq!(bureau.five_element_bureau, ZiweiFiveElementBureau::FireSix);
        }
    }

    #[test]
    fn assigns_valid_stem_designations_to_all_twelve_branches() {
        let chart = ziwei_life_body_chart(ZiweiLifeBodyInput {
            lunar_month_number: 2,
            is_leap_month: false,
            birth_hour_branch: EarthlyBranch::Chen,
        })
        .unwrap();
        for year_stem in HeavenlyStem::ALL {
            let bureau = ziwei_bureau_candidate(year_stem, &chart, false);
            assert_eq!(bureau.branch_stems.len(), 12);
            for (index, placement) in bureau.branch_stems.iter().enumerate() {
                assert_eq!(placement.branch, EarthlyBranch::ALL[index]);
                assert_eq!(placement.designation.stem, placement.stem);
                assert_eq!(placement.designation.branch, placement.branch);
            }
            assert!(!bureau.mechanical_boundaries_safe);
            assert!(!bureau.interpretation_eligible);
            assert!(!bureau.rule_eligible);
        }
    }

    #[test]
    fn keeps_lunar_and_lichun_year_stems_parallel_when_they_disagree() {
        let mut different_lichun = lichun_year(2025);
        different_lichun.boundary_safe = false;
        let audit = ziwei_life_body_audit(
            &lunar(2, false),
            &different_lichun,
            shichen(EarthlyBranch::Chen),
            shichen(EarthlyBranch::Chen),
        )
        .unwrap();
        assert!(!audit.year_stem_conventions_identical);
        assert_eq!(audit.selected_year_stem_convention, None);
        assert_eq!(
            audit.year_stem_convention_candidates[0]
                .year_designation
                .zh_name(),
            "丙午"
        );
        assert_eq!(
            audit.year_stem_convention_candidates[1]
                .year_designation
                .zh_name(),
            "乙巳"
        );
        assert!(!audit.year_stem_convention_candidates[1].year_boundary_safe);
    }

    #[test]
    fn received_five_bureau_day_anchors_match_transcribed_examples() {
        let anchors = [
            (ZiweiFiveElementBureau::WaterTwo, 1, EarthlyBranch::Chou),
            (ZiweiFiveElementBureau::WaterTwo, 2, EarthlyBranch::Yin),
            (ZiweiFiveElementBureau::WoodThree, 1, EarthlyBranch::Chen),
            (ZiweiFiveElementBureau::WoodThree, 2, EarthlyBranch::Chou),
            (ZiweiFiveElementBureau::MetalFour, 1, EarthlyBranch::Hai),
            (ZiweiFiveElementBureau::MetalFour, 2, EarthlyBranch::Chen),
            (ZiweiFiveElementBureau::EarthFive, 1, EarthlyBranch::Wu),
            (ZiweiFiveElementBureau::EarthFive, 2, EarthlyBranch::Hai),
            (ZiweiFiveElementBureau::FireSix, 1, EarthlyBranch::You),
            (ZiweiFiveElementBureau::FireSix, 2, EarthlyBranch::Wu),
            (ZiweiFiveElementBureau::FireSix, 4, EarthlyBranch::Chen),
        ];
        for (bureau, day, expected) in anchors {
            let actual = ziwei_star_anchor_candidate(day, bureau, true).unwrap();
            assert_eq!(
                (actual.five_element_bureau, actual.ziwei_branch),
                (bureau, expected)
            );
            assert!(!actual.interpretation_eligible);
            assert!(!actual.rule_eligible);
        }
    }

    #[test]
    fn star_anchor_covers_all_thirty_days_and_rejects_invalid_days() {
        for bureau in [
            ZiweiFiveElementBureau::WaterTwo,
            ZiweiFiveElementBureau::WoodThree,
            ZiweiFiveElementBureau::MetalFour,
            ZiweiFiveElementBureau::EarthFive,
            ZiweiFiveElementBureau::FireSix,
        ] {
            for day in 1..=30 {
                let anchor = ziwei_star_anchor_candidate(day, bureau, false).unwrap();
                assert!(anchor.ziwei_branch.index() < 12);
                assert_eq!((day + anchor.complement_to_divisible) % bureau.number(), 0);
                assert!(!anchor.mechanical_boundaries_safe);
            }
            for day in [0, 31] {
                assert_eq!(
                    ziwei_star_anchor_candidate(day, bureau, true).unwrap_err(),
                    ZiweiLifeBodyError::InvalidLunarDay
                );
            }
        }
    }

    #[test]
    fn audit_retains_four_star_anchor_conventions_and_v2_compatibility() {
        let audit = ziwei_life_body_audit(
            &lunar(2, false),
            &lichun_year(2025),
            shichen(EarthlyBranch::Chen),
            shichen(EarthlyBranch::Si),
        )
        .unwrap();
        assert_eq!(audit.star_anchor_candidates.len(), 4);
        assert!(audit
            .star_anchor_candidates
            .iter()
            .all(|c| !c.anchor.rule_eligible));
        assert!(audit.star_anchor_candidates.iter().all(|c| c
            .major_stars
            .as_ref()
            .is_some_and(|stars| stars.placements.len() == 14)));
        assert!(audit.star_anchor_candidates.iter().all(|c| c
            .auxiliary_stars
            .as_ref()
            .is_some_and(|stars| stars.placements.len() == 4)));
        let mut old = audit.clone();
        old.audit_version = 2;
        old.star_anchor_candidates.clear();
        old.algorithm_version = "jiuheng.ziwei.received_life_body_palaces_and_bureau.v2".to_owned();
        old.status = "mechanical_palaces_and_bureau_no_stars_interpretation_or_runtime_signals.v2"
            .to_owned();
        old.blocked_reasons.retain(|reason| {
            reason != "fourteen_main_stars_and_interpretation_layers_not_implemented"
                && reason != "ziwei_star_anchor_transcription_not_bound_to_reviewed_facsimile_pages"
        });
        assert!(ziwei_life_body_audits_match(&audit, &old));
        let mut tampered = audit.clone();
        let branch = &mut tampered.star_anchor_candidates[0].anchor.ziwei_branch;
        *branch = if *branch == EarthlyBranch::Zi {
            EarthlyBranch::Chou
        } else {
            EarthlyBranch::Zi
        };
        assert!(!ziwei_life_body_audits_match(&audit, &tampered));
    }

    #[test]
    fn fourteen_stars_follow_the_received_reverse_and_forward_offsets() {
        let anchor =
            ziwei_star_anchor_candidate(2, ZiweiFiveElementBureau::WaterTwo, true).unwrap();
        assert_eq!(anchor.ziwei_branch, EarthlyBranch::Yin);
        let chart = ziwei_major_star_chart(&anchor);
        assert_eq!(chart.tianfu_branch, EarthlyBranch::Yin);
        let expected = [
            EarthlyBranch::Yin,
            EarthlyBranch::Chou,
            EarthlyBranch::Hai,
            EarthlyBranch::Xu,
            EarthlyBranch::You,
            EarthlyBranch::Wu,
            EarthlyBranch::Yin,
            EarthlyBranch::Mao,
            EarthlyBranch::Chen,
            EarthlyBranch::Si,
            EarthlyBranch::Wu,
            EarthlyBranch::Wei,
            EarthlyBranch::Shen,
            EarthlyBranch::Zi,
        ];
        assert_eq!(chart.placements.len(), 14);
        for ((placement, kind), expected_branch) in chart
            .placements
            .iter()
            .zip(ZiweiMajorStarKind::ALL_IN_RECEIVED_ORDER)
            .zip(expected)
        {
            assert_eq!(placement.kind, kind);
            assert_eq!(placement.branch, expected_branch);
        }
        assert!(!chart.interpretation_eligible);
        assert!(!chart.rule_eligible);
    }

    #[test]
    fn tianfu_diagram_pairs_all_twelve_ziwei_branches() {
        let expected = [
            EarthlyBranch::Chen,
            EarthlyBranch::Mao,
            EarthlyBranch::Yin,
            EarthlyBranch::Chou,
            EarthlyBranch::Zi,
            EarthlyBranch::Hai,
            EarthlyBranch::Xu,
            EarthlyBranch::You,
            EarthlyBranch::Shen,
            EarthlyBranch::Wei,
            EarthlyBranch::Wu,
            EarthlyBranch::Si,
        ];
        for (ziwei_branch, tianfu_branch) in EarthlyBranch::ALL.into_iter().zip(expected) {
            let mut anchor =
                ziwei_star_anchor_candidate(1, ZiweiFiveElementBureau::WaterTwo, false).unwrap();
            anchor.ziwei_branch = ziwei_branch;
            let chart = ziwei_major_star_chart(&anchor);
            assert_eq!(chart.tianfu_branch, tianfu_branch);
            assert_eq!(chart.placements.len(), 14);
            assert!(!chart.mechanical_boundaries_safe);
        }
    }

    #[test]
    fn four_auxiliary_stars_follow_month_and_hour_in_opposite_directions() {
        let first = ziwei_auxiliary_star_chart(1, EarthlyBranch::Zi, true).unwrap();
        assert_eq!(
            first
                .placements
                .iter()
                .map(|p| p.branch)
                .collect::<Vec<_>>(),
            [
                EarthlyBranch::Xu,
                EarthlyBranch::Chen,
                EarthlyBranch::Chen,
                EarthlyBranch::Xu
            ]
        );
        let second = ziwei_auxiliary_star_chart(2, EarthlyBranch::Chou, true).unwrap();
        assert_eq!(
            second
                .placements
                .iter()
                .map(|p| p.branch)
                .collect::<Vec<_>>(),
            [
                EarthlyBranch::You,
                EarthlyBranch::Si,
                EarthlyBranch::Si,
                EarthlyBranch::You
            ]
        );
        for month in 1..=12 {
            for hour in EarthlyBranch::ALL {
                let chart = ziwei_auxiliary_star_chart(month, hour, false).unwrap();
                assert_eq!(chart.placements.len(), 4);
                assert_eq!(chart.birth_hour_branch, hour);
                assert_eq!(chart.effective_month_number, month);
                assert!(!chart.mechanical_boundaries_safe);
                assert!(!chart.interpretation_eligible);
                assert!(!chart.rule_eligible);
            }
        }
        for month in [0, 13] {
            assert_eq!(
                ziwei_auxiliary_star_chart(month, EarthlyBranch::Zi, true).unwrap_err(),
                ZiweiLifeBodyError::InvalidLunarMonth
            );
        }
    }

    #[test]
    fn v4_fourteen_star_snapshot_ignores_new_auxiliaries_but_v5_detects_tampering() {
        let current = ziwei_life_body_audit(
            &lunar(2, false),
            &lichun_year(2025),
            shichen(EarthlyBranch::Chen),
            shichen(EarthlyBranch::Si),
        )
        .unwrap();
        let mut old = current.clone();
        old.audit_version = 4;
        old.algorithm_version =
            "jiuheng.ziwei.received_life_body_palaces_bureau_fourteen_stars.v4".to_owned();
        old.status =
            "mechanical_palaces_bureau_fourteen_stars_no_interpretation_or_runtime_signals.v4"
                .to_owned();
        old.star_anchor_candidates
            .iter_mut()
            .for_each(|candidate| candidate.auxiliary_stars = None);
        old.blocked_reasons.retain(|reason| {
            reason != "four_auxiliary_star_placements_not_bound_to_reviewed_facsimile_pages"
                && reason != "remaining_auxiliary_stars_four_transformations_limits_and_interpretation_not_implemented"
        });
        old.blocked_reasons.push(
            "auxiliary_stars_four_transformations_limits_and_interpretation_not_implemented"
                .to_owned(),
        );
        assert!(ziwei_life_body_audits_match(&current, &old));
        let mut tampered_old = old.clone();
        let old_branch = &mut tampered_old.star_anchor_candidates[0]
            .major_stars
            .as_mut()
            .unwrap()
            .placements[0]
            .branch;
        *old_branch = if *old_branch == EarthlyBranch::Zi {
            EarthlyBranch::Chou
        } else {
            EarthlyBranch::Zi
        };
        assert!(!ziwei_life_body_audits_match(&current, &tampered_old));
        let mut tampered = current.clone();
        let branch = &mut tampered.star_anchor_candidates[0]
            .auxiliary_stars
            .as_mut()
            .unwrap()
            .placements[0]
            .branch;
        *branch = if *branch == EarthlyBranch::Zi {
            EarthlyBranch::Chou
        } else {
            EarthlyBranch::Zi
        };
        assert!(!ziwei_life_body_audits_match(&current, &tampered));
    }

    #[test]
    fn v3_anchor_snapshot_ignores_absent_major_stars_but_v4_detects_tampering() {
        let current = ziwei_life_body_audit(
            &lunar(2, false),
            &lichun_year(2025),
            shichen(EarthlyBranch::Chen),
            shichen(EarthlyBranch::Si),
        )
        .unwrap();
        let mut old = current.clone();
        old.audit_version = 3;
        old.algorithm_version =
            "jiuheng.ziwei.received_life_body_palaces_bureau_star_anchor.v3".to_owned();
        old.status =
            "mechanical_palaces_bureau_ziwei_anchor_no_interpretation_or_runtime_signals.v3"
                .to_owned();
        old.star_anchor_candidates
            .iter_mut()
            .for_each(|candidate| candidate.major_stars = None);
        old.blocked_reasons.retain(|reason| {
            reason != "fourteen_main_star_placements_not_bound_to_reviewed_facsimile_pages"
                && reason != "auxiliary_stars_four_transformations_limits_and_interpretation_not_implemented"
        });
        old.blocked_reasons
            .push("fourteen_main_stars_and_interpretation_layers_not_implemented".to_owned());
        assert!(ziwei_life_body_audits_match(&current, &old));
        let mut tampered = current.clone();
        let branch = &mut tampered.star_anchor_candidates[0]
            .major_stars
            .as_mut()
            .unwrap()
            .placements[0]
            .branch;
        *branch = if *branch == EarthlyBranch::Zi {
            EarthlyBranch::Chou
        } else {
            EarthlyBranch::Zi
        };
        assert!(!ziwei_life_body_audits_match(&current, &tampered));
    }
}
