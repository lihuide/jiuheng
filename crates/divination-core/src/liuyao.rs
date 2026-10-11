                                                   
   
                                                                          
                                                                             
                                                                             
                                                                             
                                                                        
                                                                                 

use crate::{
    hexagram_identity, EarthlyBranch, FiveElement, FivePhase, HeavenlyStem, HexagramIdentity,
    JieMonthCandidate, LineType, LiuyaoCast, SexagenaryDesignation, SolarTerm, Trigram,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const LIUYAO_EIGHT_PALACE_ALGORITHM_VERSION: &str =
    "jiuheng.liuyao.received_eight_palace_host_response.v1";
pub const LIUYAO_EIGHT_PALACE_STATUS: &str =
    "mechanical_palace_stage_host_response_only_no_najia_or_interpretation.v1";
pub const LIUYAO_NAJIA_ALGORITHM_VERSION: &str = "jiuheng.liuyao.received_najia_stem_branch.v1";
pub const LIUYAO_NAJIA_STATUS: &str =
    "mechanical_received_najia_stem_branch_only_no_relatives_spirits_or_interpretation.v1";
pub const LIUYAO_SIX_RELATIVES_ALGORITHM_VERSION: &str =
    "jiuheng.liuyao.received_palace_five_element_six_relatives.v1";
pub const LIUYAO_SIX_RELATIVES_STATUS: &str =
    "mechanical_received_six_relatives_only_no_spirits_useful_god_strength_or_interpretation.v1";
pub const LIUYAO_SIX_SPIRITS_ALGORITHM_VERSION: &str =
    "jiuheng.liuyao.received_day_stem_six_spirits.bjt_midnight.v1";
pub const LIUYAO_SIX_SPIRITS_STATUS: &str =
    "mechanical_received_six_spirits_bjt_midnight_day_stem_only_no_interpretation.v1";
pub const LIUYAO_HIDDEN_SPIRITS_ALGORITHM_VERSION: &str =
    "jiuheng.liuyao.received_palace_pure_hexagram_hidden_spirits.v1";
pub const LIUYAO_HIDDEN_SPIRITS_STATUS: &str =
    "mechanical_missing_relative_palace_pure_line_candidates_only_no_interpretation.v1";
pub const LIUYAO_CALENDAR_MOVING_ALGORITHM_VERSION: &str =
    "jiuheng.liuyao.locked_jie_month_day_moving_transformation.v1";
pub const LIUYAO_CALENDAR_MOVING_STATUS: &str =
    "mechanical_locked_month_day_and_moving_transformed_lines_only_no_strength_or_interpretation.v1";
pub const LIUYAO_CALENDAR_RELATIONS_ALGORITHM_VERSION: &str =
    "jiuheng.liuyao.xun_void_month_day_branch_five_element_raw_relations.v2";
pub const LIUYAO_CALENDAR_RELATIONS_STATUS: &str =
    "mechanical_xun_void_month_day_branch_and_directional_five_element_relations_only_no_effect_or_interpretation.v2";
pub const LIUYAO_WEATHER_RAIN_CANDIDATE_ID: &str = "WEATHER-RAIN-CANDIDATE-001";
pub const LIUYAO_BRANCH_PAIR_RELATIONS_ALGORITHM_VERSION: &str =
    "jiuheng.liuyao.received_six_combination_six_opposition_raw_pairs.v1";
pub const LIUYAO_BRANCH_PAIR_RELATIONS_STATUS: &str =
    "mechanical_primary_moving_hidden_branch_pair_comparison_only_no_effect_or_interpretation.v1";
pub const LIUYAO_PROGRESS_RETREAT_ALGORITHM_VERSION: &str =
    "jiuheng.liuyao.received_seven_explicit_progress_retreat_pairs.v1";
pub const LIUYAO_PROGRESS_RETREAT_STATUS: &str =
    "mechanical_moving_transformation_progress_retreat_identity_only_two_earth_pairs_unresolved_no_effect_or_interpretation.v1";
pub const LIUYAO_FANYIN_FUYIN_ALGORITHM_VERSION: &str =
    "jiuheng.liuyao.received_fanyin_fuyin_raw_structures.v1";
pub const LIUYAO_FANYIN_FUYIN_STATUS: &str =
    "mechanical_inner_outer_trigram_and_moving_branch_prerequisites_only_no_effect_or_interpretation.v1";
pub const LIUYAO_SANHE_ALGORITHM_VERSION: &str =
    "jiuheng.liuyao.received_four_sanhe_member_occurrences.v1";
pub const LIUYAO_SANHE_STATUS: &str =
    "mechanical_four_sanhe_members_and_source_occurrence_inventory_only_no_formation_effect_or_interpretation.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiuyaoPalaceStage {
    Pure,
    FirstGeneration,
    SecondGeneration,
    ThirdGeneration,
    FourthGeneration,
    FifthGeneration,
    WanderingSoul,
    ReturningSoul,
}

impl LiuyaoPalaceStage {
    pub const ALL_IN_PALACE_ORDER: [Self; 8] = [
        Self::Pure,
        Self::FirstGeneration,
        Self::SecondGeneration,
        Self::ThirdGeneration,
        Self::FourthGeneration,
        Self::FifthGeneration,
        Self::WanderingSoul,
        Self::ReturningSoul,
    ];

    pub const fn sequence(self) -> u8 {
        self as u8
    }

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Pure => "本宫",
            Self::FirstGeneration => "一世",
            Self::SecondGeneration => "二世",
            Self::ThirdGeneration => "三世",
            Self::FourthGeneration => "四世",
            Self::FifthGeneration => "五世",
            Self::WanderingSoul => "游魂",
            Self::ReturningSoul => "归魂",
        }
    }

                                                                             
    pub const fn changed_line_mask_from_pure(self) -> u8 {
        match self {
            Self::Pure => 0b000000,
            Self::FirstGeneration => 0b000001,
            Self::SecondGeneration => 0b000011,
            Self::ThirdGeneration => 0b000111,
            Self::FourthGeneration => 0b001111,
            Self::FifthGeneration => 0b011111,
            Self::WanderingSoul => 0b010111,
            Self::ReturningSoul => 0b010000,
        }
    }

    pub const fn host_line(self) -> u8 {
        match self {
            Self::Pure => 6,
            Self::FirstGeneration => 1,
            Self::SecondGeneration => 2,
            Self::ThirdGeneration | Self::ReturningSoul => 3,
            Self::FourthGeneration | Self::WanderingSoul => 4,
            Self::FifthGeneration => 5,
        }
    }

    pub const fn response_line(self) -> u8 {
        match self.host_line() {
            1..=3 => self.host_line() + 3,
            4..=6 => self.host_line() - 3,
            _ => unreachable!(),
        }
    }
}

pub const LIUYAO_PALACES_IN_RECEIVED_ORDER: [Trigram; 8] = [
    Trigram::Qian,
    Trigram::Zhen,
    Trigram::Kan,
    Trigram::Gen,
    Trigram::Kun,
    Trigram::Xun,
    Trigram::Li,
    Trigram::Dui,
];

pub const fn trigram_five_phase(trigram: Trigram) -> FivePhase {
    match trigram {
        Trigram::Qian | Trigram::Dui => FivePhase::Metal,
        Trigram::Li => FivePhase::Fire,
        Trigram::Zhen | Trigram::Xun => FivePhase::Wood,
        Trigram::Kan => FivePhase::Water,
        Trigram::Gen | Trigram::Kun => FivePhase::Earth,
    }
}

pub const fn trigram_five_element(trigram: Trigram) -> FiveElement {
    match trigram {
        Trigram::Qian | Trigram::Dui => FiveElement::Metal,
        Trigram::Li => FiveElement::Fire,
        Trigram::Zhen | Trigram::Xun => FiveElement::Wood,
        Trigram::Kan => FiveElement::Water,
        Trigram::Gen | Trigram::Kun => FiveElement::Earth,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoEightPalaceAudit {
    pub audit_version: u16,
    pub algorithm_version: String,
    pub status: String,
    pub hexagram: HexagramIdentity,
    pub palace: Trigram,
    pub palace_five_phase: FivePhase,
    pub stage: LiuyaoPalaceStage,
    pub stage_sequence: u8,
    pub changed_line_mask_from_pure: u8,
    pub host_line: u8,
    pub response_line: u8,
    pub na_jia_assigned: bool,
    pub six_relatives_assigned: bool,
    pub six_spirits_assigned: bool,
    pub interpretation_eligible: bool,
    pub rule_eligible: bool,
    pub blocked_reasons: Vec<String>,
}

impl LiuyaoEightPalaceAudit {
    pub fn compact_zh(&self) -> String {
        format!(
            "{}宫{} · 世在{}爻 · 应在{}爻",
            self.palace.zh_name(),
            self.stage.zh_name(),
            self.host_line,
            self.response_line
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LiuyaoEightPalaceError {
    #[error("六爻结构位图必须在 0 至 63 之间，实际为 {0}")]
    InvalidSixLineBits(u8),
    #[error("六爻结构未能映射到八宫六十四卦：{0:06b}")]
    UnmappedSixLineBits(u8),
}

pub fn liuyao_eight_palace_audit(
    bits_bottom_up: u8,
) -> Result<LiuyaoEightPalaceAudit, LiuyaoEightPalaceError> {
    if bits_bottom_up > 0b11_1111 {
        return Err(LiuyaoEightPalaceError::InvalidSixLineBits(bits_bottom_up));
    }

    for palace in LIUYAO_PALACES_IN_RECEIVED_ORDER {
        let trigram_bits = palace.bits_bottom_up();
        let pure_bits = trigram_bits | (trigram_bits << 3);
        for stage in LiuyaoPalaceStage::ALL_IN_PALACE_ORDER {
            if pure_bits ^ stage.changed_line_mask_from_pure() == bits_bottom_up {
                let hexagram = hexagram_identity(bits_bottom_up)
                    .expect("a validated six-bit structure always has a hexagram identity");
                return Ok(LiuyaoEightPalaceAudit {
                    audit_version: 1,
                    algorithm_version: LIUYAO_EIGHT_PALACE_ALGORITHM_VERSION.to_owned(),
                    status: LIUYAO_EIGHT_PALACE_STATUS.to_owned(),
                    hexagram,
                    palace,
                    palace_five_phase: trigram_five_phase(palace),
                    stage,
                    stage_sequence: stage.sequence(),
                    changed_line_mask_from_pure: stage.changed_line_mask_from_pure(),
                    host_line: stage.host_line(),
                    response_line: stage.response_line(),
                    na_jia_assigned: false,
                    six_relatives_assigned: false,
                    six_spirits_assigned: false,
                    interpretation_eligible: false,
                    rule_eligible: false,
                    blocked_reasons: vec![
                        "na_jia_stems_branches_not_bound_to_reviewed_facsimile_table".to_owned(),
                        "six_relatives_and_six_spirits_not_implemented".to_owned(),
                        "useful_god_strength_moving_line_timing_and_judgment_rules_not_reviewed"
                            .to_owned(),
                    ],
                });
            }
        }
    }

    Err(LiuyaoEightPalaceError::UnmappedSixLineBits(bits_bottom_up))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiuyaoTrigramPosition {
    Inner,
    Outer,
}

impl LiuyaoTrigramPosition {
    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Inner => "内卦",
            Self::Outer => "外卦",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoNaJiaLine {
    pub line_number_bottom_up: u8,
    pub trigram: Trigram,
    pub trigram_line_number_bottom_up: u8,
    pub trigram_position: LiuyaoTrigramPosition,
    pub stem: HeavenlyStem,
    pub branch: EarthlyBranch,
    pub branch_five_element: FiveElement,
    pub is_host: bool,
    pub is_response: bool,
}

impl LiuyaoNaJiaLine {
    pub fn designation_zh(&self) -> String {
        format!(
            "{}{}{}",
            self.stem.zh_name(),
            self.branch.zh_name(),
            self.branch_five_element.zh_name()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoNaJiaAudit {
    pub audit_version: u16,
    pub algorithm_version: String,
    pub status: String,
    pub eight_palace: LiuyaoEightPalaceAudit,
    pub lines_bottom_up: Vec<LiuyaoNaJiaLine>,
    pub na_jia_stems_assigned: bool,
    pub na_jia_branches_assigned: bool,
    pub line_five_elements_assigned: bool,
    pub host_response_bound: bool,
    pub six_relatives_assigned: bool,
    pub six_spirits_assigned: bool,
    pub hidden_spirits_assigned: bool,
    pub interpretation_eligible: bool,
    pub rule_eligible: bool,
    pub blocked_reasons: Vec<String>,
}

impl LiuyaoNaJiaAudit {
    pub fn compact_lines_zh(&self) -> String {
        self.lines_bottom_up
            .iter()
            .map(LiuyaoNaJiaLine::designation_zh)
            .collect::<Vec<_>>()
            .join(" · ")
    }
}

fn received_najia_stem(trigram: Trigram, position: LiuyaoTrigramPosition) -> HeavenlyStem {
    match (trigram, position) {
        (Trigram::Qian, LiuyaoTrigramPosition::Inner) => HeavenlyStem::Jia,
        (Trigram::Qian, LiuyaoTrigramPosition::Outer) => HeavenlyStem::Ren,
        (Trigram::Kun, LiuyaoTrigramPosition::Inner) => HeavenlyStem::Yi,
        (Trigram::Kun, LiuyaoTrigramPosition::Outer) => HeavenlyStem::Gui,
        (Trigram::Zhen, _) => HeavenlyStem::Geng,
        (Trigram::Xun, _) => HeavenlyStem::Xin,
        (Trigram::Kan, _) => HeavenlyStem::Wu,
        (Trigram::Li, _) => HeavenlyStem::Ji,
        (Trigram::Gen, _) => HeavenlyStem::Bing,
        (Trigram::Dui, _) => HeavenlyStem::Ding,
    }
}

fn received_najia_branches(
    trigram: Trigram,
    position: LiuyaoTrigramPosition,
) -> [EarthlyBranch; 3] {
    use EarthlyBranch::{Chen, Chou, Hai, Mao, Shen, Si, Wei, Wu, Xu, Yin, You, Zi};
    use LiuyaoTrigramPosition::{Inner, Outer};
    match (trigram, position) {
        (Trigram::Qian | Trigram::Zhen, Inner) => [Zi, Yin, Chen],
        (Trigram::Qian | Trigram::Zhen, Outer) => [Wu, Shen, Xu],
        (Trigram::Kan, Inner) => [Yin, Chen, Wu],
        (Trigram::Kan, Outer) => [Shen, Xu, Zi],
        (Trigram::Gen, Inner) => [Chen, Wu, Shen],
        (Trigram::Gen, Outer) => [Xu, Zi, Yin],
        (Trigram::Kun, Inner) => [Wei, Si, Mao],
        (Trigram::Kun, Outer) => [Chou, Hai, You],
        (Trigram::Xun, Inner) => [Chou, Hai, You],
        (Trigram::Xun, Outer) => [Wei, Si, Mao],
        (Trigram::Li, Inner) => [Mao, Chou, Hai],
        (Trigram::Li, Outer) => [You, Wei, Si],
        (Trigram::Dui, Inner) => [Si, Mao, Chou],
        (Trigram::Dui, Outer) => [Hai, You, Wei],
    }
}

pub fn liuyao_najia_audit(bits_bottom_up: u8) -> Result<LiuyaoNaJiaAudit, LiuyaoEightPalaceError> {
    let eight_palace = liuyao_eight_palace_audit(bits_bottom_up)?;
    let halves = [
        (
            eight_palace.hexagram.lower_trigram,
            LiuyaoTrigramPosition::Inner,
            1_u8,
        ),
        (
            eight_palace.hexagram.upper_trigram,
            LiuyaoTrigramPosition::Outer,
            4_u8,
        ),
    ];
    let mut lines_bottom_up = Vec::with_capacity(6);
    for (trigram, position, first_line) in halves {
        let stem = received_najia_stem(trigram, position);
        for (offset, branch) in received_najia_branches(trigram, position)
            .into_iter()
            .enumerate()
        {
            let line_number = first_line + offset as u8;
            lines_bottom_up.push(LiuyaoNaJiaLine {
                line_number_bottom_up: line_number,
                trigram,
                trigram_line_number_bottom_up: offset as u8 + 1,
                trigram_position: position,
                stem,
                branch,
                branch_five_element: branch.five_element(),
                is_host: line_number == eight_palace.host_line,
                is_response: line_number == eight_palace.response_line,
            });
        }
    }

    Ok(LiuyaoNaJiaAudit {
        audit_version: 1,
        algorithm_version: LIUYAO_NAJIA_ALGORITHM_VERSION.to_owned(),
        status: LIUYAO_NAJIA_STATUS.to_owned(),
        eight_palace,
        lines_bottom_up,
        na_jia_stems_assigned: true,
        na_jia_branches_assigned: true,
        line_five_elements_assigned: true,
        host_response_bound: true,
        six_relatives_assigned: false,
        six_spirits_assigned: false,
        hidden_spirits_assigned: false,
        interpretation_eligible: false,
        rule_eligible: false,
        blocked_reasons: vec![
            "facsimile_bytes_variants_and_jingfang_huozhulin_transmission_layers_not_closed"
                .to_owned(),
            "six_relatives_six_spirits_and_hidden_spirits_not_implemented".to_owned(),
            "useful_god_strength_moving_line_timing_and_judgment_rules_not_reviewed".to_owned(),
        ],
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiuyaoSixRelative {
    Parents,
    Siblings,
    Descendants,
    WifeWealth,
    OfficerGhost,
}

impl LiuyaoSixRelative {
    pub const ALL: [Self; 5] = [
        Self::Parents,
        Self::Siblings,
        Self::Descendants,
        Self::WifeWealth,
        Self::OfficerGhost,
    ];

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Parents => "父母",
            Self::Siblings => "兄弟",
            Self::Descendants => "子孙",
            Self::WifeWealth => "妻财",
            Self::OfficerGhost => "官鬼",
        }
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

                                                                            
   
                                                                               
                                                           
pub const fn liuyao_six_relative_for_elements(
    palace_element: FiveElement,
    line_element: FiveElement,
) -> LiuyaoSixRelative {
    if palace_element as u8 == line_element as u8 {
        LiuyaoSixRelative::Siblings
    } else if five_element_generates(line_element, palace_element) {
        LiuyaoSixRelative::Parents
    } else if five_element_generates(palace_element, line_element) {
        LiuyaoSixRelative::Descendants
    } else if five_element_overcomes(line_element, palace_element) {
        LiuyaoSixRelative::OfficerGhost
    } else if five_element_overcomes(palace_element, line_element) {
        LiuyaoSixRelative::WifeWealth
    } else {
        unreachable!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoSixRelativesLine {
    pub najia: LiuyaoNaJiaLine,
    pub relationship: LiuyaoSixRelative,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoSixRelativesAudit {
    pub audit_version: u16,
    pub algorithm_version: String,
    pub status: String,
    pub najia: LiuyaoNaJiaAudit,
    pub palace_element: FiveElement,
    pub lines_bottom_up: Vec<LiuyaoSixRelativesLine>,
    pub six_relatives_assigned: bool,
    pub six_spirits_assigned: bool,
    pub hidden_spirits_assigned: bool,
    pub useful_god_selected: bool,
    pub strength_assessed: bool,
    pub interpretation_eligible: bool,
    pub rule_eligible: bool,
    pub blocked_reasons: Vec<String>,
}

pub fn liuyao_six_relatives_audit(
    bits_bottom_up: u8,
) -> Result<LiuyaoSixRelativesAudit, LiuyaoEightPalaceError> {
    let najia = liuyao_najia_audit(bits_bottom_up)?;
    let palace_element = trigram_five_element(najia.eight_palace.palace);
    let lines_bottom_up = najia
        .lines_bottom_up
        .iter()
        .cloned()
        .map(|line| LiuyaoSixRelativesLine {
            relationship: liuyao_six_relative_for_elements(
                palace_element,
                line.branch_five_element,
            ),
            najia: line,
        })
        .collect();

    Ok(LiuyaoSixRelativesAudit {
        audit_version: 1,
        algorithm_version: LIUYAO_SIX_RELATIVES_ALGORITHM_VERSION.to_owned(),
        status: LIUYAO_SIX_RELATIVES_STATUS.to_owned(),
        najia,
        palace_element,
        lines_bottom_up,
        six_relatives_assigned: true,
        six_spirits_assigned: false,
        hidden_spirits_assigned: false,
        useful_god_selected: false,
        strength_assessed: false,
        interpretation_eligible: false,
        rule_eligible: false,
        blocked_reasons: vec![
            "facsimile_bytes_variants_and_transmission_layers_not_closed".to_owned(),
            "six_spirits_hidden_spirits_and_useful_god_not_implemented".to_owned(),
            "strength_moving_line_timing_and_judgment_rules_not_reviewed".to_owned(),
        ],
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiuyaoSixSpirit {
    AzureDragon,
    VermilionBird,
    HookedArray,
    SoaringSerpent,
    WhiteTiger,
    DarkWarrior,
}

impl LiuyaoSixSpirit {
    pub const RECEIVED_ORDER: [Self; 6] = [
        Self::AzureDragon,
        Self::VermilionBird,
        Self::HookedArray,
        Self::SoaringSerpent,
        Self::WhiteTiger,
        Self::DarkWarrior,
    ];

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::AzureDragon => "青龙",
            Self::VermilionBird => "朱雀",
            Self::HookedArray => "勾陈",
            Self::SoaringSerpent => "螣蛇",
            Self::WhiteTiger => "白虎",
            Self::DarkWarrior => "玄武",
        }
    }
}

pub const fn liuyao_first_line_spirit_index(day_stem: HeavenlyStem) -> usize {
    match day_stem {
        HeavenlyStem::Jia | HeavenlyStem::Yi => 0,
        HeavenlyStem::Bing | HeavenlyStem::Ding => 1,
        HeavenlyStem::Wu => 2,
        HeavenlyStem::Ji => 3,
        HeavenlyStem::Geng | HeavenlyStem::Xin => 4,
        HeavenlyStem::Ren | HeavenlyStem::Gui => 5,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoSixSpiritsLine {
    pub relative: LiuyaoSixRelativesLine,
    pub spirit: LiuyaoSixSpirit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoSixSpiritsAudit {
    pub audit_version: u16,
    pub algorithm_version: String,
    pub status: String,
    pub day_convention: String,
    pub day_stem: HeavenlyStem,
    pub first_line_spirit: LiuyaoSixSpirit,
    pub six_relatives: LiuyaoSixRelativesAudit,
    pub lines_bottom_up: Vec<LiuyaoSixSpiritsLine>,
    pub six_spirits_assigned: bool,
    pub hidden_spirits_assigned: bool,
    pub useful_god_selected: bool,
    pub strength_assessed: bool,
    pub interpretation_eligible: bool,
    pub rule_eligible: bool,
    pub blocked_reasons: Vec<String>,
}

                                                                            
                                                                           
                                                                          
pub fn liuyao_six_spirits_audit(
    bits_bottom_up: u8,
    day_stem: HeavenlyStem,
) -> Result<LiuyaoSixSpiritsAudit, LiuyaoEightPalaceError> {
    let six_relatives = liuyao_six_relatives_audit(bits_bottom_up)?;
    let start = liuyao_first_line_spirit_index(day_stem);
    let lines_bottom_up = six_relatives
        .lines_bottom_up
        .iter()
        .cloned()
        .enumerate()
        .map(|(offset, relative)| LiuyaoSixSpiritsLine {
            relative,
            spirit: LiuyaoSixSpirit::RECEIVED_ORDER
                [(start + offset) % LiuyaoSixSpirit::RECEIVED_ORDER.len()],
        })
        .collect();

    Ok(LiuyaoSixSpiritsAudit {
        audit_version: 1,
        algorithm_version: LIUYAO_SIX_SPIRITS_ALGORITHM_VERSION.to_owned(),
        status: LIUYAO_SIX_SPIRITS_STATUS.to_owned(),
        day_convention: "beijing_civil_midnight".to_owned(),
        day_stem,
        first_line_spirit: LiuyaoSixSpirit::RECEIVED_ORDER[start],
        six_relatives,
        lines_bottom_up,
        six_spirits_assigned: true,
        hidden_spirits_assigned: false,
        useful_god_selected: false,
        strength_assessed: false,
        interpretation_eligible: false,
        rule_eligible: false,
        blocked_reasons: vec![
            "facsimile_bytes_variants_and_early_transmission_layers_not_closed".to_owned(),
            "zi_hour_and_true_solar_day_conventions_not_released_for_liuyao_rules".to_owned(),
            "hidden_spirits_useful_god_strength_timing_and_judgment_not_reviewed".to_owned(),
        ],
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoHiddenSpiritLine {
    pub line_number_bottom_up: u8,
    pub flying_line: LiuyaoSixRelativesLine,
    pub hidden_line: LiuyaoSixRelativesLine,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoHiddenSpiritsAudit {
    pub audit_version: u16,
    pub algorithm_version: String,
    pub status: String,
    pub current: LiuyaoSixRelativesAudit,
    pub palace_pure: LiuyaoSixRelativesAudit,
    pub visible_relatives: Vec<LiuyaoSixRelative>,
    pub missing_relatives: Vec<LiuyaoSixRelative>,
    pub hidden_lines_bottom_up: Vec<LiuyaoHiddenSpiritLine>,
    pub hidden_spirits_assigned: bool,
    pub flying_hidden_interactions_assessed: bool,
    pub useful_god_selected: bool,
    pub strength_assessed: bool,
    pub interpretation_eligible: bool,
    pub rule_eligible: bool,
    pub blocked_reasons: Vec<String>,
}

                                                                                
                                                                             
                                                                              
                                                      
pub fn liuyao_hidden_spirits_audit(
    bits_bottom_up: u8,
) -> Result<LiuyaoHiddenSpiritsAudit, LiuyaoEightPalaceError> {
    let current = liuyao_six_relatives_audit(bits_bottom_up)?;
    let pure_trigram_bits = current.najia.eight_palace.palace.bits_bottom_up();
    let palace_pure = liuyao_six_relatives_audit(pure_trigram_bits | (pure_trigram_bits << 3))?;

    let visible_relatives = LiuyaoSixRelative::ALL
        .into_iter()
        .filter(|relationship| {
            current
                .lines_bottom_up
                .iter()
                .any(|line| line.relationship == *relationship)
        })
        .collect::<Vec<_>>();
    let missing_relatives = LiuyaoSixRelative::ALL
        .into_iter()
        .filter(|relationship| !visible_relatives.contains(relationship))
        .collect::<Vec<_>>();
    let hidden_lines_bottom_up = current
        .lines_bottom_up
        .iter()
        .cloned()
        .zip(palace_pure.lines_bottom_up.iter().cloned())
        .filter(|(_, hidden_line)| missing_relatives.contains(&hidden_line.relationship))
        .map(|(flying_line, hidden_line)| LiuyaoHiddenSpiritLine {
            line_number_bottom_up: flying_line.najia.line_number_bottom_up,
            flying_line,
            hidden_line,
        })
        .collect::<Vec<_>>();

    Ok(LiuyaoHiddenSpiritsAudit {
        audit_version: 1,
        algorithm_version: LIUYAO_HIDDEN_SPIRITS_ALGORITHM_VERSION.to_owned(),
        status: LIUYAO_HIDDEN_SPIRITS_STATUS.to_owned(),
        current,
        palace_pure,
        visible_relatives,
        missing_relatives,
        hidden_lines_bottom_up,
        hidden_spirits_assigned: true,
        flying_hidden_interactions_assessed: false,
        useful_god_selected: false,
        strength_assessed: false,
        interpretation_eligible: false,
        rule_eligible: false,
        blocked_reasons: vec![
            "facsimile_bytes_variants_and_transmission_layers_not_closed".to_owned(),
            "flying_hidden_generation_overcoming_strength_and_emergence_not_assessed".to_owned(),
            "useful_god_moving_line_timing_and_judgment_rules_not_reviewed".to_owned(),
        ],
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoMovingLineCandidate {
    pub line_number_bottom_up: u8,
    pub received_value: u8,
    pub line_type: LineType,
    pub primary_is_yang: bool,
    pub changed_is_yang: bool,
    pub primary_line: LiuyaoSixRelativesLine,
    pub transformed_stem: HeavenlyStem,
    pub transformed_branch: EarthlyBranch,
    pub transformed_branch_five_element: FiveElement,
                                                                           
                                                                          
    pub transformed_relationship_under_primary_palace: LiuyaoSixRelative,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoCalendarMovingAudit {
    pub audit_version: u16,
    pub algorithm_version: String,
    pub status: String,
    pub month_convention: String,
    pub month_designation: SexagenaryDesignation,
    pub month_previous_jie: SolarTerm,
    pub month_next_jie: SolarTerm,
    pub month_boundary_safe: bool,
    pub day_convention: String,
    pub day_designation: SexagenaryDesignation,
    pub primary_hexagram: HexagramIdentity,
    pub changed_hexagram: HexagramIdentity,
    pub moving_line_mask_bottom_up: u8,
    pub moving_line_numbers_bottom_up: Vec<u8>,
    pub moving_lines_bottom_up: Vec<LiuyaoMovingLineCandidate>,
    pub month_build_identity_assigned: bool,
    pub day_identity_assigned: bool,
    pub moving_transformation_verified: bool,
    pub month_day_line_interactions_assessed: bool,
    pub useful_god_selected: bool,
    pub strength_assessed: bool,
    pub interpretation_eligible: bool,
    pub rule_eligible: bool,
    pub blocked_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LiuyaoCalendarMovingError {
    #[error("六爻动变机械审计必须恰有六爻，实际为 {0}")]
    InvalidLineCount(usize),
    #[error("六爻第 {position} 个记录的自下而上索引应为 {expected}，实际为 {actual}")]
    InvalidLineIndex {
        position: usize,
        expected: u8,
        actual: u8,
    },
    #[error("六爻第 {line_number} 爻的数值与爻型不一致")]
    LineValueTypeMismatch { line_number: u8 },
    #[error("六爻保存的本卦或变卦位图与逐爻六七八九不一致")]
    StructureMismatch,
    #[error(transparent)]
    EightPalace(#[from] LiuyaoEightPalaceError),
}

                                                                              
                                                                              
                                                                             
                                            
pub fn liuyao_calendar_moving_audit(
    cast: &LiuyaoCast,
    month: &JieMonthCandidate,
    day: SexagenaryDesignation,
) -> Result<LiuyaoCalendarMovingAudit, LiuyaoCalendarMovingError> {
    if cast.lines.len() != 6 {
        return Err(LiuyaoCalendarMovingError::InvalidLineCount(
            cast.lines.len(),
        ));
    }

    let mut primary_bits = 0_u8;
    let mut changed_bits = 0_u8;
    let mut moving_line_mask_bottom_up = 0_u8;
    for (position, line) in cast.lines.iter().enumerate() {
        let expected = position as u8;
        if line.line_index_bottom_up != expected {
            return Err(LiuyaoCalendarMovingError::InvalidLineIndex {
                position,
                expected,
                actual: line.line_index_bottom_up,
            });
        }
        if line.value != line.line_type.value() {
            return Err(LiuyaoCalendarMovingError::LineValueTypeMismatch {
                line_number: expected + 1,
            });
        }
        if line.line_type.primary_is_yang() {
            primary_bits |= 1 << position;
        }
        if line.line_type.changed_is_yang() {
            changed_bits |= 1 << position;
        }
        if line.line_type.is_moving() {
            moving_line_mask_bottom_up |= 1 << position;
        }
    }
    if primary_bits != cast.primary_bits || changed_bits != cast.changed_bits {
        return Err(LiuyaoCalendarMovingError::StructureMismatch);
    }

    let primary = liuyao_six_relatives_audit(primary_bits)?;
    let transformed = liuyao_najia_audit(changed_bits)?;
    let palace_element = primary.palace_element;
    let moving_lines_bottom_up = cast
        .lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.line_type.is_moving())
        .map(|(position, line)| {
            let primary_line = primary.lines_bottom_up[position].clone();
            let transformed_line = &transformed.lines_bottom_up[position];
            LiuyaoMovingLineCandidate {
                line_number_bottom_up: position as u8 + 1,
                received_value: line.value,
                line_type: line.line_type,
                primary_is_yang: line.line_type.primary_is_yang(),
                changed_is_yang: line.line_type.changed_is_yang(),
                primary_line,
                transformed_stem: transformed_line.stem,
                transformed_branch: transformed_line.branch,
                transformed_branch_five_element: transformed_line.branch_five_element,
                transformed_relationship_under_primary_palace: liuyao_six_relative_for_elements(
                    palace_element,
                    transformed_line.branch_five_element,
                ),
            }
        })
        .collect::<Vec<_>>();
    let moving_line_numbers_bottom_up = moving_lines_bottom_up
        .iter()
        .map(|line| line.line_number_bottom_up)
        .collect::<Vec<_>>();

    Ok(LiuyaoCalendarMovingAudit {
        audit_version: 1,
        algorithm_version: LIUYAO_CALENDAR_MOVING_ALGORITHM_VERSION.to_owned(),
        status: LIUYAO_CALENDAR_MOVING_STATUS.to_owned(),
        month_convention: "locked_tt_solar_jie_span_with_lichun_year_stem".to_owned(),
        month_designation: month.designation,
        month_previous_jie: month.previous_jie,
        month_next_jie: month.next_jie,
        month_boundary_safe: month.boundary_safe,
        day_convention: "beijing_civil_midnight".to_owned(),
        day_designation: day,
        primary_hexagram: primary.najia.eight_palace.hexagram,
        changed_hexagram: transformed.eight_palace.hexagram,
        moving_line_mask_bottom_up,
        moving_line_numbers_bottom_up,
        moving_lines_bottom_up,
        month_build_identity_assigned: true,
        day_identity_assigned: true,
        moving_transformation_verified: true,
        month_day_line_interactions_assessed: false,
        useful_god_selected: false,
        strength_assessed: false,
        interpretation_eligible: false,
        rule_eligible: false,
        blocked_reasons: vec![
            "facsimile_bytes_variants_and_transmission_layers_not_closed".to_owned(),
            "liuyao_month_boundary_and_day_boundary_conventions_not_formally_released".to_owned(),
            "month_day_generation_overcoming_clash_combination_void_break_not_assessed".to_owned(),
            "useful_god_strength_timing_signals_advice_and_judgment_not_reviewed".to_owned(),
        ],
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiuyaoCalendarRelationLayer {
    PrimaryVisible,
    MovingTransformed,
}

                                                                    
   
                                                                             
                                                                              
                                                                               
              
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiuyaoDirectionalFiveElementRelation {
    Same,
    SourceGeneratesTarget,
    TargetGeneratesSource,
    SourceOvercomesTarget,
    TargetOvercomesSource,
}

impl LiuyaoDirectionalFiveElementRelation {
    pub const ALL: [Self; 5] = [
        Self::Same,
        Self::SourceGeneratesTarget,
        Self::TargetGeneratesSource,
        Self::SourceOvercomesTarget,
        Self::TargetOvercomesSource,
    ];

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Same => "同五行",
            Self::SourceGeneratesTarget => "基准生对象",
            Self::TargetGeneratesSource => "对象生基准",
            Self::SourceOvercomesTarget => "基准克对象",
            Self::TargetOvercomesSource => "对象克基准",
        }
    }
}

pub const fn liuyao_directional_five_element_relation(
    source: FiveElement,
    target: FiveElement,
) -> LiuyaoDirectionalFiveElementRelation {
    if source as u8 == target as u8 {
        LiuyaoDirectionalFiveElementRelation::Same
    } else if five_element_generates(source, target) {
        LiuyaoDirectionalFiveElementRelation::SourceGeneratesTarget
    } else if five_element_generates(target, source) {
        LiuyaoDirectionalFiveElementRelation::TargetGeneratesSource
    } else if five_element_overcomes(source, target) {
        LiuyaoDirectionalFiveElementRelation::SourceOvercomesTarget
    } else if five_element_overcomes(target, source) {
        LiuyaoDirectionalFiveElementRelation::TargetOvercomesSource
    } else {
        unreachable!()
    }
}

impl LiuyaoCalendarRelationLayer {
    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::PrimaryVisible => "本卦明爻",
            Self::MovingTransformed => "动爻变后",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoCalendarBranchObservation {
    pub line_number_bottom_up: u8,
    pub layer: LiuyaoCalendarRelationLayer,
    pub branch: EarthlyBranch,
    pub branch_five_element: FiveElement,
    pub month_build_five_element_relation: LiuyaoDirectionalFiveElementRelation,
    pub day_branch_five_element_relation: LiuyaoDirectionalFiveElementRelation,
    pub equals_month_build: bool,
    pub opposed_by_month_build: bool,
    pub equals_day_branch: bool,
    pub opposed_by_day_branch: bool,
    pub is_xun_void: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoCalendarRelationsAudit {
    pub audit_version: u16,
    pub algorithm_version: String,
    pub status: String,
    pub calendar_moving: LiuyaoCalendarMovingAudit,
    pub xun_index_zero_based: u8,
    pub xun_start: SexagenaryDesignation,
    pub xun_void_branches: [EarthlyBranch; 2],
    pub month_build_branch: EarthlyBranch,
    pub month_build_five_element: FiveElement,
    pub month_opposite_branch: EarthlyBranch,
    pub day_branch: EarthlyBranch,
    pub day_branch_five_element: FiveElement,
    pub day_opposite_branch: EarthlyBranch,
    pub primary_observations_bottom_up: Vec<LiuyaoCalendarBranchObservation>,
    pub transformed_moving_observations_bottom_up: Vec<LiuyaoCalendarBranchObservation>,
    pub raw_relations_identified: bool,
    pub relation_effects_assessed: bool,
    pub useful_god_selected: bool,
    pub strength_assessed: bool,
    pub interpretation_eligible: bool,
    pub rule_eligible: bool,
    pub blocked_reasons: Vec<String>,
}

pub fn liuyao_xun_void_branches(
    day: SexagenaryDesignation,
) -> (SexagenaryDesignation, [EarthlyBranch; 2]) {
    let xun_start_index = (day.cycle_index / 10) * 10;
    let xun_start = SexagenaryDesignation::from_cycle_index(i64::from(xun_start_index));
    let branch_index = xun_start.branch.index();
    (
        xun_start,
        [
            EarthlyBranch::ALL[((branch_index + 10) % 12) as usize],
            EarthlyBranch::ALL[((branch_index + 11) % 12) as usize],
        ],
    )
}

pub fn liuyao_six_opposition_partner(branch: EarthlyBranch) -> EarthlyBranch {
    EarthlyBranch::ALL[((branch.index() + 6) % 12) as usize]
}

pub const fn liuyao_six_combination_partner(branch: EarthlyBranch) -> EarthlyBranch {
    use EarthlyBranch::{Chen, Chou, Hai, Mao, Shen, Si, Wei, Wu, Xu, Yin, You, Zi};
    match branch {
        Zi => Chou,
        Chou => Zi,
        Yin => Hai,
        Hai => Yin,
        Mao => Xu,
        Xu => Mao,
        Chen => You,
        You => Chen,
        Si => Shen,
        Shen => Si,
        Wu => Wei,
        Wei => Wu,
    }
}

fn calendar_branch_observation(
    line_number_bottom_up: u8,
    layer: LiuyaoCalendarRelationLayer,
    branch: EarthlyBranch,
    month_build_branch: EarthlyBranch,
    day_branch: EarthlyBranch,
    xun_void_branches: [EarthlyBranch; 2],
) -> LiuyaoCalendarBranchObservation {
    LiuyaoCalendarBranchObservation {
        line_number_bottom_up,
        layer,
        branch,
        branch_five_element: branch.five_element(),
        month_build_five_element_relation: liuyao_directional_five_element_relation(
            month_build_branch.five_element(),
            branch.five_element(),
        ),
        day_branch_five_element_relation: liuyao_directional_five_element_relation(
            day_branch.five_element(),
            branch.five_element(),
        ),
        equals_month_build: branch == month_build_branch,
        opposed_by_month_build: branch == liuyao_six_opposition_partner(month_build_branch),
        equals_day_branch: branch == day_branch,
        opposed_by_day_branch: branch == liuyao_six_opposition_partner(day_branch),
        is_xun_void: xun_void_branches.contains(&branch),
    }
}

                                                                              
                                                                              
                                                                               
                                                                         
pub fn liuyao_calendar_relations_audit(
    cast: &LiuyaoCast,
    month: &JieMonthCandidate,
    day: SexagenaryDesignation,
) -> Result<LiuyaoCalendarRelationsAudit, LiuyaoCalendarMovingError> {
    let calendar_moving = liuyao_calendar_moving_audit(cast, month, day)?;
    let primary = liuyao_najia_audit(cast.primary_bits)?;
    let (xun_start, xun_void_branches) = liuyao_xun_void_branches(day);
    let month_build_branch = month.designation.branch;
    let day_branch = day.branch;
    let primary_observations_bottom_up = primary
        .lines_bottom_up
        .iter()
        .map(|line| {
            calendar_branch_observation(
                line.line_number_bottom_up,
                LiuyaoCalendarRelationLayer::PrimaryVisible,
                line.branch,
                month_build_branch,
                day_branch,
                xun_void_branches,
            )
        })
        .collect();
    let transformed_moving_observations_bottom_up = calendar_moving
        .moving_lines_bottom_up
        .iter()
        .map(|line| {
            calendar_branch_observation(
                line.line_number_bottom_up,
                LiuyaoCalendarRelationLayer::MovingTransformed,
                line.transformed_branch,
                month_build_branch,
                day_branch,
                xun_void_branches,
            )
        })
        .collect();

    Ok(LiuyaoCalendarRelationsAudit {
        audit_version: 2,
        algorithm_version: LIUYAO_CALENDAR_RELATIONS_ALGORITHM_VERSION.to_owned(),
        status: LIUYAO_CALENDAR_RELATIONS_STATUS.to_owned(),
        calendar_moving,
        xun_index_zero_based: day.cycle_index / 10,
        xun_start,
        xun_void_branches,
        month_build_branch,
        month_build_five_element: month_build_branch.five_element(),
        month_opposite_branch: liuyao_six_opposition_partner(month_build_branch),
        day_branch,
        day_branch_five_element: day_branch.five_element(),
        day_opposite_branch: liuyao_six_opposition_partner(day_branch),
        primary_observations_bottom_up,
        transformed_moving_observations_bottom_up,
        raw_relations_identified: true,
        relation_effects_assessed: false,
        useful_god_selected: false,
        strength_assessed: false,
        interpretation_eligible: false,
        rule_eligible: false,
        blocked_reasons: vec![
            "facsimile_bytes_variants_and_transmission_layers_not_closed".to_owned(),
            "month_and_day_boundary_conventions_not_formally_released_for_liuyao".to_owned(),
            "xun_void_month_break_day_clash_and_five_element_relation_effects_not_assessed"
                .to_owned(),
            "strength_useful_god_timing_signals_advice_and_judgment_not_reviewed".to_owned(),
        ],
    })
}

                                                                          
                                                                                
                                                                             
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoWeatherRainCandidateAudit {
    pub candidate_id: String,
    pub status: String,
    pub visible_parent_lines_bottom_up: Vec<u8>,
    pub moving_parent_lines_bottom_up: Vec<u8>,
    pub day_opposed_visible_lines_bottom_up: Vec<u8>,
    pub day_opposed_parent_lines_bottom_up: Vec<u8>,
    pub month_opposed_visible_lines_bottom_up: Vec<u8>,
    pub xun_void_visible_lines_bottom_up: Vec<u8>,
    pub parent_combined_visible_pairs_bottom_up: Vec<[u8; 2]>,
    pub parent_combined_month_lines_bottom_up: Vec<u8>,
    pub parent_combined_day_lines_bottom_up: Vec<u8>,
    pub moving_officer_ghost_lines_bottom_up: Vec<u8>,
    pub transformed_parent_lines_bottom_up: Vec<u8>,
    pub narrow_visible_line_pattern_present: bool,
    pub month_boundary_safe: bool,
    pub day_boundary_safety_assessed: bool,
    pub formal_rule_id: Option<String>,
    pub prediction_eligible: bool,
    pub blocked_reasons: Vec<String>,
}

pub fn liuyao_weather_rain_candidate_audit(
    relations: &LiuyaoCalendarRelationsAudit,
) -> Result<LiuyaoWeatherRainCandidateAudit, LiuyaoEightPalaceError> {
    let relatives =
        liuyao_six_relatives_audit(relations.calendar_moving.primary_hexagram.bits_bottom_up)?;
    let visible_parent_lines_bottom_up = relatives
        .lines_bottom_up
        .iter()
        .filter(|line| line.relationship == LiuyaoSixRelative::Parents)
        .map(|line| line.najia.line_number_bottom_up)
        .collect::<Vec<_>>();
    let moving_parent_lines_bottom_up = relations
        .calendar_moving
        .moving_lines_bottom_up
        .iter()
        .filter(|line| line.primary_line.relationship == LiuyaoSixRelative::Parents)
        .map(|line| line.line_number_bottom_up)
        .collect::<Vec<_>>();
    let day_opposed_visible_lines_bottom_up = relatives
        .lines_bottom_up
        .iter()
        .filter(|line| line.najia.branch == relations.day_opposite_branch)
        .map(|line| line.najia.line_number_bottom_up)
        .collect::<Vec<_>>();
    let day_opposed_parent_lines_bottom_up = relatives
        .lines_bottom_up
        .iter()
        .filter(|line| {
            line.relationship == LiuyaoSixRelative::Parents
                && line.najia.branch == relations.day_opposite_branch
        })
        .map(|line| line.najia.line_number_bottom_up)
        .collect::<Vec<_>>();
    let month_opposed_visible_lines_bottom_up = relatives
        .lines_bottom_up
        .iter()
        .filter(|line| line.najia.branch == relations.month_opposite_branch)
        .map(|line| line.najia.line_number_bottom_up)
        .collect::<Vec<_>>();
    let xun_void_visible_lines_bottom_up = relatives
        .lines_bottom_up
        .iter()
        .filter(|line| relations.xun_void_branches.contains(&line.najia.branch))
        .map(|line| line.najia.line_number_bottom_up)
        .collect::<Vec<_>>();
    let mut parent_combined_visible_pairs_bottom_up = Vec::new();
    let mut parent_combined_month_lines_bottom_up = Vec::new();
    let mut parent_combined_day_lines_bottom_up = Vec::new();
    for parent in relatives
        .lines_bottom_up
        .iter()
        .filter(|line| line.relationship == LiuyaoSixRelative::Parents)
    {
        let partner_branch = liuyao_six_combination_partner(parent.najia.branch);
        let parent_line = parent.najia.line_number_bottom_up;
        if partner_branch == relations.month_build_branch {
            parent_combined_month_lines_bottom_up.push(parent_line);
        }
        if partner_branch == relations.day_branch {
            parent_combined_day_lines_bottom_up.push(parent_line);
        }
        for other in relatives.lines_bottom_up.iter().filter(|line| {
            line.najia.line_number_bottom_up != parent_line && line.najia.branch == partner_branch
        }) {
            parent_combined_visible_pairs_bottom_up
                .push([parent_line, other.najia.line_number_bottom_up]);
        }
    }
    let moving_officer_ghost_lines_bottom_up = relations
        .calendar_moving
        .moving_lines_bottom_up
        .iter()
        .filter(|line| line.primary_line.relationship == LiuyaoSixRelative::OfficerGhost)
        .map(|line| line.line_number_bottom_up)
        .collect::<Vec<_>>();
    let transformed_parent_lines_bottom_up = relations
        .calendar_moving
        .moving_lines_bottom_up
        .iter()
        .filter(|line| {
            line.transformed_relationship_under_primary_palace == LiuyaoSixRelative::Parents
        })
        .map(|line| line.line_number_bottom_up)
        .collect::<Vec<_>>();
    let narrow_visible_line_pattern_present = !visible_parent_lines_bottom_up.is_empty()
        && moving_parent_lines_bottom_up.is_empty()
        && day_opposed_visible_lines_bottom_up.is_empty()
        && month_opposed_visible_lines_bottom_up.is_empty()
        && xun_void_visible_lines_bottom_up.is_empty()
        && parent_combined_visible_pairs_bottom_up.is_empty()
        && parent_combined_month_lines_bottom_up.is_empty()
        && parent_combined_day_lines_bottom_up.is_empty()
        && relations.calendar_moving.moving_lines_bottom_up.len() == 1
        && moving_officer_ghost_lines_bottom_up.len() == 1
        && transformed_parent_lines_bottom_up.is_empty();

    let mut blocked_reasons = vec![
        "day_boundary_and_requested_time_span_not_reviewed".to_owned(),
        "hidden_lines_and_same_chapter_conflict_priority_not_reviewed".to_owned(),
        "parent_combination_and_ghost_clash_effects_not_reviewed".to_owned(),
        "source_transcription_logic_implementation_reviews_not_complete".to_owned(),
    ];
    if !relations.calendar_moving.month_boundary_safe {
        blocked_reasons.push("month_boundary_unsafe".to_owned());
    }

    Ok(LiuyaoWeatherRainCandidateAudit {
        candidate_id: LIUYAO_WEATHER_RAIN_CANDIDATE_ID.to_owned(),
        status: "research_visible_line_prerequisites_only_no_weather_judgment.v3".to_owned(),
        visible_parent_lines_bottom_up,
        moving_parent_lines_bottom_up,
        day_opposed_visible_lines_bottom_up,
        day_opposed_parent_lines_bottom_up,
        month_opposed_visible_lines_bottom_up,
        xun_void_visible_lines_bottom_up,
        parent_combined_visible_pairs_bottom_up,
        parent_combined_month_lines_bottom_up,
        parent_combined_day_lines_bottom_up,
        moving_officer_ghost_lines_bottom_up,
        transformed_parent_lines_bottom_up,
        narrow_visible_line_pattern_present,
        month_boundary_safe: relations.calendar_moving.month_boundary_safe,
        day_boundary_safety_assessed: false,
        formal_rule_id: None,
        prediction_eligible: false,
        blocked_reasons,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiuyaoBranchPairLayer {
    PrimaryVisible,
    MovingTransformed,
    HiddenCandidate,
}

impl LiuyaoBranchPairLayer {
    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::PrimaryVisible => "本卦明爻",
            Self::MovingTransformed => "动爻变后",
            Self::HiddenCandidate => "伏神候选",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiuyaoBranchPairScope {
    PrimaryVisiblePair,
    MovingTransformationSameLine,
    FlyingHiddenSameLine,
}

impl LiuyaoBranchPairScope {
    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::PrimaryVisiblePair => "本卦明爻两两比较",
            Self::MovingTransformationSameLine => "动爻本变同位比较",
            Self::FlyingHiddenSameLine => "飞伏同位比较",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoBranchPairEndpoint {
    pub layer: LiuyaoBranchPairLayer,
    pub line_number_bottom_up: u8,
    pub branch: EarthlyBranch,
    pub branch_five_element: FiveElement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoBranchPairObservation {
    pub scope: LiuyaoBranchPairScope,
    pub first: LiuyaoBranchPairEndpoint,
    pub second: LiuyaoBranchPairEndpoint,
    pub is_six_combination_pair: bool,
    pub is_six_opposition_pair: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoBranchPairRelationsAudit {
    pub audit_version: u16,
    pub algorithm_version: String,
    pub status: String,
    pub calendar_relations: LiuyaoCalendarRelationsAudit,
    pub hidden_spirits: LiuyaoHiddenSpiritsAudit,
    pub primary_visible_pair_observations: Vec<LiuyaoBranchPairObservation>,
    pub moving_transformation_observations: Vec<LiuyaoBranchPairObservation>,
    pub flying_hidden_observations: Vec<LiuyaoBranchPairObservation>,
    pub raw_branch_pairs_compared: bool,
    pub whole_hexagram_six_combination_classified: bool,
    pub whole_hexagram_six_opposition_classified: bool,
    pub relation_effects_assessed: bool,
    pub useful_god_selected: bool,
    pub strength_assessed: bool,
    pub interpretation_eligible: bool,
    pub rule_eligible: bool,
    pub blocked_reasons: Vec<String>,
}

fn branch_pair_endpoint(
    layer: LiuyaoBranchPairLayer,
    line_number_bottom_up: u8,
    branch: EarthlyBranch,
) -> LiuyaoBranchPairEndpoint {
    LiuyaoBranchPairEndpoint {
        layer,
        line_number_bottom_up,
        branch,
        branch_five_element: branch.five_element(),
    }
}

fn branch_pair_observation(
    scope: LiuyaoBranchPairScope,
    first: LiuyaoBranchPairEndpoint,
    second: LiuyaoBranchPairEndpoint,
) -> LiuyaoBranchPairObservation {
    LiuyaoBranchPairObservation {
        scope,
        is_six_combination_pair: liuyao_six_combination_partner(first.branch) == second.branch,
        is_six_opposition_pair: liuyao_six_opposition_partner(first.branch) == second.branch,
        first,
        second,
    }
}

                                                                        
                                                                              
                                                                           
                                                                          
                                                                        
pub fn liuyao_branch_pair_relations_audit(
    cast: &LiuyaoCast,
    month: &JieMonthCandidate,
    day: SexagenaryDesignation,
) -> Result<LiuyaoBranchPairRelationsAudit, LiuyaoCalendarMovingError> {
    let calendar_relations = liuyao_calendar_relations_audit(cast, month, day)?;
    let hidden_spirits = liuyao_hidden_spirits_audit(cast.primary_bits)?;

    let primary_endpoints = calendar_relations
        .primary_observations_bottom_up
        .iter()
        .map(|line| {
            branch_pair_endpoint(
                LiuyaoBranchPairLayer::PrimaryVisible,
                line.line_number_bottom_up,
                line.branch,
            )
        })
        .collect::<Vec<_>>();
    let mut primary_visible_pair_observations = Vec::with_capacity(15);
    for first_index in 0..primary_endpoints.len() {
        for second_index in (first_index + 1)..primary_endpoints.len() {
            primary_visible_pair_observations.push(branch_pair_observation(
                LiuyaoBranchPairScope::PrimaryVisiblePair,
                primary_endpoints[first_index].clone(),
                primary_endpoints[second_index].clone(),
            ));
        }
    }

    let moving_transformation_observations = calendar_relations
        .transformed_moving_observations_bottom_up
        .iter()
        .map(|transformed| {
            let primary = primary_endpoints
                .iter()
                .find(|line| line.line_number_bottom_up == transformed.line_number_bottom_up)
                .expect("a validated moving line always has a primary endpoint")
                .clone();
            branch_pair_observation(
                LiuyaoBranchPairScope::MovingTransformationSameLine,
                primary,
                branch_pair_endpoint(
                    LiuyaoBranchPairLayer::MovingTransformed,
                    transformed.line_number_bottom_up,
                    transformed.branch,
                ),
            )
        })
        .collect();

    let flying_hidden_observations = hidden_spirits
        .hidden_lines_bottom_up
        .iter()
        .map(|line| {
            branch_pair_observation(
                LiuyaoBranchPairScope::FlyingHiddenSameLine,
                branch_pair_endpoint(
                    LiuyaoBranchPairLayer::PrimaryVisible,
                    line.line_number_bottom_up,
                    line.flying_line.najia.branch,
                ),
                branch_pair_endpoint(
                    LiuyaoBranchPairLayer::HiddenCandidate,
                    line.line_number_bottom_up,
                    line.hidden_line.najia.branch,
                ),
            )
        })
        .collect();

    Ok(LiuyaoBranchPairRelationsAudit {
        audit_version: 1,
        algorithm_version: LIUYAO_BRANCH_PAIR_RELATIONS_ALGORITHM_VERSION.to_owned(),
        status: LIUYAO_BRANCH_PAIR_RELATIONS_STATUS.to_owned(),
        calendar_relations,
        hidden_spirits,
        primary_visible_pair_observations,
        moving_transformation_observations,
        flying_hidden_observations,
        raw_branch_pairs_compared: true,
        whole_hexagram_six_combination_classified: false,
        whole_hexagram_six_opposition_classified: false,
        relation_effects_assessed: false,
        useful_god_selected: false,
        strength_assessed: false,
        interpretation_eligible: false,
        rule_eligible: false,
        blocked_reasons: vec![
            "facsimile_bytes_variants_and_transmission_layers_not_closed".to_owned(),
            "whole_hexagram_combination_opposition_definitions_not_formally_released".to_owned(),
            "combination_transformation_clash_break_movement_and_exception_rules_not_assessed"
                .to_owned(),
            "strength_useful_god_timing_signals_advice_and_judgment_not_reviewed".to_owned(),
        ],
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiuyaoProgressRetreatIdentity {
    Progress,
    Retreat,
    Unclassified,
}

impl LiuyaoProgressRetreatIdentity {
    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Progress => "进神支对",
            Self::Retreat => "退神支对",
            Self::Unclassified => "未分类",
        }
    }
}

                                                                             
                                                                          
                                                                           
                                                                       
                                    
pub const fn liuyao_progress_retreat_identity(
    primary: EarthlyBranch,
    transformed: EarthlyBranch,
) -> LiuyaoProgressRetreatIdentity {
    use EarthlyBranch::{Chen, Chou, Hai, Mao, Shen, Si, Wei, Wu, Xu, Yin, You, Zi};
    match (primary, transformed) {
        (Hai, Zi)
        | (Yin, Mao)
        | (Si, Wu)
        | (Shen, You)
        | (Chou, Chen)
        | (Chen, Wei)
        | (Wei, Xu) => LiuyaoProgressRetreatIdentity::Progress,
        (Zi, Hai)
        | (Mao, Yin)
        | (Wu, Si)
        | (You, Shen)
        | (Chen, Chou)
        | (Wei, Chen)
        | (Xu, Wei) => LiuyaoProgressRetreatIdentity::Retreat,
        _ => LiuyaoProgressRetreatIdentity::Unclassified,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoProgressRetreatObservation {
    pub line_number_bottom_up: u8,
    pub primary_branch: EarthlyBranch,
    pub primary_branch_five_element: FiveElement,
    pub transformed_branch: EarthlyBranch,
    pub transformed_branch_five_element: FiveElement,
    pub identity: LiuyaoProgressRetreatIdentity,
    pub received_pair_explicitly_listed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoProgressRetreatAudit {
    pub audit_version: u16,
    pub algorithm_version: String,
    pub status: String,
    pub branch_pair_relations: LiuyaoBranchPairRelationsAudit,
    pub moving_observations_bottom_up: Vec<LiuyaoProgressRetreatObservation>,
    pub explicit_progress_pair_count: usize,
    pub explicit_retreat_pair_count: usize,
    pub raw_identity_assigned: bool,
    pub omitted_earth_cycle_pairs_left_unclassified: bool,
    pub progression_or_retreat_effect_assessed: bool,
    pub useful_god_selected: bool,
    pub strength_assessed: bool,
    pub timing_assessed: bool,
    pub interpretation_eligible: bool,
    pub rule_eligible: bool,
    pub blocked_reasons: Vec<String>,
}

                                                                            
                                                                            
                                                                               
pub fn liuyao_progress_retreat_audit(
    cast: &LiuyaoCast,
    month: &JieMonthCandidate,
    day: SexagenaryDesignation,
) -> Result<LiuyaoProgressRetreatAudit, LiuyaoCalendarMovingError> {
    let branch_pair_relations = liuyao_branch_pair_relations_audit(cast, month, day)?;
    let moving_observations_bottom_up = branch_pair_relations
        .moving_transformation_observations
        .iter()
        .map(|observation| {
            let identity = liuyao_progress_retreat_identity(
                observation.first.branch,
                observation.second.branch,
            );
            LiuyaoProgressRetreatObservation {
                line_number_bottom_up: observation.first.line_number_bottom_up,
                primary_branch: observation.first.branch,
                primary_branch_five_element: observation.first.branch_five_element,
                transformed_branch: observation.second.branch,
                transformed_branch_five_element: observation.second.branch_five_element,
                received_pair_explicitly_listed: identity
                    != LiuyaoProgressRetreatIdentity::Unclassified,
                identity,
            }
        })
        .collect();

    Ok(LiuyaoProgressRetreatAudit {
        audit_version: 1,
        algorithm_version: LIUYAO_PROGRESS_RETREAT_ALGORITHM_VERSION.to_owned(),
        status: LIUYAO_PROGRESS_RETREAT_STATUS.to_owned(),
        branch_pair_relations,
        moving_observations_bottom_up,
        explicit_progress_pair_count: 7,
        explicit_retreat_pair_count: 7,
        raw_identity_assigned: true,
        omitted_earth_cycle_pairs_left_unclassified: true,
        progression_or_retreat_effect_assessed: false,
        useful_god_selected: false,
        strength_assessed: false,
        timing_assessed: false,
        interpretation_eligible: false,
        rule_eligible: false,
        blocked_reasons: vec![
            "facsimile_bytes_variants_and_transmission_layers_not_closed".to_owned(),
            "xu_to_chou_and_chou_to_xu_missing_from_checked_chapter_transcription".to_owned(),
            "other_received_texts_list_only_six_pairs_and_require_edition_comparison".to_owned(),
            "progress_retreat_effect_strength_exceptions_timing_advice_and_judgment_not_reviewed"
                .to_owned(),
        ],
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiuyaoFanyinFuyinScope {
    Inner,
    Outer,
}

impl LiuyaoFanyinFuyinScope {
    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Inner => "内卦",
            Self::Outer => "外卦",
        }
    }
}

                                                                           
                                                                          
                         
pub const fn liuyao_fanyin_trigram_partner(trigram: Trigram) -> Trigram {
    match trigram {
        Trigram::Qian => Trigram::Xun,
        Trigram::Xun => Trigram::Qian,
        Trigram::Kan => Trigram::Li,
        Trigram::Li => Trigram::Kan,
        Trigram::Zhen => Trigram::Dui,
        Trigram::Dui => Trigram::Zhen,
        Trigram::Kun => Trigram::Gen,
        Trigram::Gen => Trigram::Kun,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoFanyinFuyinScopeObservation {
    pub scope: LiuyaoFanyinFuyinScope,
    pub line_numbers_bottom_up: [u8; 3],
    pub primary_trigram: Trigram,
    pub transformed_trigram: Trigram,
    pub moving_line_numbers_bottom_up: Vec<u8>,
    pub primary_branches_bottom_up: [EarthlyBranch; 3],
    pub transformed_branches_bottom_up: [EarthlyBranch; 3],
    pub is_received_direction_opposition_pair: bool,
    pub all_same_position_branches_equal: bool,
    pub all_same_position_branches_opposed: bool,
    pub raw_trigram_fanyin_candidate: bool,
    pub raw_line_fanyin_candidate: bool,
    pub raw_fuyin_candidate: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoFanyinFuyinMovingLineObservation {
    pub line_number_bottom_up: u8,
    pub primary_branch: EarthlyBranch,
    pub transformed_branch: EarthlyBranch,
    pub same_branch: bool,
    pub opposed_branch: bool,
    pub raw_line_fuyin_candidate: bool,
    pub raw_line_fanyin_candidate: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoFanyinFuyinAudit {
    pub audit_version: u16,
    pub algorithm_version: String,
    pub status: String,
    pub progress_retreat: LiuyaoProgressRetreatAudit,
    pub inner_scope: LiuyaoFanyinFuyinScopeObservation,
    pub outer_scope: LiuyaoFanyinFuyinScopeObservation,
    pub moving_line_observations_bottom_up: Vec<LiuyaoFanyinFuyinMovingLineObservation>,
    pub raw_prerequisites_identified: bool,
    pub whole_hexagram_fanyin_classified: bool,
    pub whole_hexagram_fuyin_classified: bool,
    pub relation_effects_assessed: bool,
    pub useful_god_selected: bool,
    pub strength_assessed: bool,
    pub timing_assessed: bool,
    pub interpretation_eligible: bool,
    pub rule_eligible: bool,
    pub blocked_reasons: Vec<String>,
}

fn fanyin_fuyin_scope_observation(
    scope: LiuyaoFanyinFuyinScope,
    primary: &LiuyaoNaJiaAudit,
    transformed: &LiuyaoNaJiaAudit,
    moving_line_mask_bottom_up: u8,
) -> LiuyaoFanyinFuyinScopeObservation {
    let first_index = match scope {
        LiuyaoFanyinFuyinScope::Inner => 0,
        LiuyaoFanyinFuyinScope::Outer => 3,
    };
    let line_numbers_bottom_up = std::array::from_fn(|offset| (first_index + offset + 1) as u8);
    let primary_branches_bottom_up =
        std::array::from_fn(|offset| primary.lines_bottom_up[first_index + offset].branch);
    let transformed_branches_bottom_up =
        std::array::from_fn(|offset| transformed.lines_bottom_up[first_index + offset].branch);
    let moving_line_numbers_bottom_up = line_numbers_bottom_up
        .into_iter()
        .filter(|line_number| moving_line_mask_bottom_up & (1 << (line_number - 1)) != 0)
        .collect::<Vec<_>>();
    let primary_trigram = match scope {
        LiuyaoFanyinFuyinScope::Inner => primary.eight_palace.hexagram.lower_trigram,
        LiuyaoFanyinFuyinScope::Outer => primary.eight_palace.hexagram.upper_trigram,
    };
    let transformed_trigram = match scope {
        LiuyaoFanyinFuyinScope::Inner => transformed.eight_palace.hexagram.lower_trigram,
        LiuyaoFanyinFuyinScope::Outer => transformed.eight_palace.hexagram.upper_trigram,
    };
    let is_received_direction_opposition_pair =
        liuyao_fanyin_trigram_partner(primary_trigram) == transformed_trigram;
    let all_same_position_branches_equal = primary_branches_bottom_up
        .iter()
        .zip(transformed_branches_bottom_up.iter())
        .all(|(primary_branch, transformed_branch)| primary_branch == transformed_branch);
    let all_same_position_branches_opposed = primary_branches_bottom_up
        .iter()
        .zip(transformed_branches_bottom_up.iter())
        .all(|(primary_branch, transformed_branch)| {
            liuyao_six_opposition_partner(*primary_branch) == *transformed_branch
        });
    let has_actual_moving_line = !moving_line_numbers_bottom_up.is_empty();

    LiuyaoFanyinFuyinScopeObservation {
        scope,
        line_numbers_bottom_up,
        primary_trigram,
        transformed_trigram,
        moving_line_numbers_bottom_up,
        primary_branches_bottom_up,
        transformed_branches_bottom_up,
        is_received_direction_opposition_pair,
        all_same_position_branches_equal,
        all_same_position_branches_opposed,
        raw_trigram_fanyin_candidate: has_actual_moving_line
            && is_received_direction_opposition_pair,
        raw_line_fanyin_candidate: has_actual_moving_line && all_same_position_branches_opposed,
        raw_fuyin_candidate: has_actual_moving_line && all_same_position_branches_equal,
    }
}

                                                                        
                                                                      
                                                                               
                                                                            
                                                        
pub fn liuyao_fanyin_fuyin_audit(
    cast: &LiuyaoCast,
    month: &JieMonthCandidate,
    day: SexagenaryDesignation,
) -> Result<LiuyaoFanyinFuyinAudit, LiuyaoCalendarMovingError> {
    let progress_retreat = liuyao_progress_retreat_audit(cast, month, day)?;
    let primary = liuyao_najia_audit(cast.primary_bits)?;
    let transformed = liuyao_najia_audit(cast.changed_bits)?;
    let moving_line_mask_bottom_up = progress_retreat
        .branch_pair_relations
        .calendar_relations
        .calendar_moving
        .moving_line_mask_bottom_up;
    let inner_scope = fanyin_fuyin_scope_observation(
        LiuyaoFanyinFuyinScope::Inner,
        &primary,
        &transformed,
        moving_line_mask_bottom_up,
    );
    let outer_scope = fanyin_fuyin_scope_observation(
        LiuyaoFanyinFuyinScope::Outer,
        &primary,
        &transformed,
        moving_line_mask_bottom_up,
    );
    let moving_line_observations_bottom_up = progress_retreat
        .moving_observations_bottom_up
        .iter()
        .map(|observation| {
            let same_branch = observation.primary_branch == observation.transformed_branch;
            let opposed_branch = liuyao_six_opposition_partner(observation.primary_branch)
                == observation.transformed_branch;
            LiuyaoFanyinFuyinMovingLineObservation {
                line_number_bottom_up: observation.line_number_bottom_up,
                primary_branch: observation.primary_branch,
                transformed_branch: observation.transformed_branch,
                same_branch,
                opposed_branch,
                raw_line_fuyin_candidate: same_branch,
                raw_line_fanyin_candidate: opposed_branch,
            }
        })
        .collect();

    Ok(LiuyaoFanyinFuyinAudit {
        audit_version: 1,
        algorithm_version: LIUYAO_FANYIN_FUYIN_ALGORITHM_VERSION.to_owned(),
        status: LIUYAO_FANYIN_FUYIN_STATUS.to_owned(),
        progress_retreat,
        inner_scope,
        outer_scope,
        moving_line_observations_bottom_up,
        raw_prerequisites_identified: true,
        whole_hexagram_fanyin_classified: false,
        whole_hexagram_fuyin_classified: false,
        relation_effects_assessed: false,
        useful_god_selected: false,
        strength_assessed: false,
        timing_assessed: false,
        interpretation_eligible: false,
        rule_eligible: false,
        blocked_reasons: vec![
            "facsimile_page_hash_transcription_and_edition_variants_not_closed".to_owned(),
            "trigram_fanyin_line_fanyin_and_fuyin_taxonomy_not_formally_released".to_owned(),
            "whole_hexagram_inner_outer_combination_rules_not_formally_released".to_owned(),
            "effects_strength_useful_god_timing_advice_and_judgment_not_reviewed".to_owned(),
        ],
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiuyaoSanheMemberRole {
    Birth,
    Prosperity,
    Tomb,
}

impl LiuyaoSanheMemberRole {
    pub const ALL: [Self; 3] = [Self::Birth, Self::Prosperity, Self::Tomb];

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Birth => "生",
            Self::Prosperity => "旺",
            Self::Tomb => "墓",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiuyaoSanheOccurrenceLayer {
    PrimaryVisible,
    MovingTransformed,
    LockedMonth,
    LockedDay,
}

impl LiuyaoSanheOccurrenceLayer {
    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::PrimaryVisible => "本卦明爻",
            Self::MovingTransformed => "动爻变后",
            Self::LockedMonth => "锁定月建",
            Self::LockedDay => "锁定日辰",
        }
    }
}

                                                                           
                                                                              
                                  
pub const fn liuyao_sanhe_groups() -> [(FiveElement, [EarthlyBranch; 3]); 4] {
    use EarthlyBranch::{Chen, Chou, Hai, Mao, Shen, Si, Wei, Wu, Xu, Yin, You, Zi};
    [
        (FiveElement::Water, [Shen, Zi, Chen]),
        (FiveElement::Wood, [Hai, Mao, Wei]),
        (FiveElement::Fire, [Yin, Wu, Xu]),
        (FiveElement::Metal, [Si, You, Chou]),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoSanheOccurrence {
    pub layer: LiuyaoSanheOccurrenceLayer,
    pub line_number_bottom_up: Option<u8>,
    pub branch: EarthlyBranch,
    pub member_role: LiuyaoSanheMemberRole,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoSanheGroupObservation {
    pub result_element: FiveElement,
    pub members_birth_prosperity_tomb: [EarthlyBranch; 3],
    pub occurrences: Vec<LiuyaoSanheOccurrence>,
    pub primary_visible_presence: [bool; 3],
    pub moving_transformed_presence: [bool; 3],
    pub locked_month_presence: [bool; 3],
    pub locked_day_presence: [bool; 3],
    pub raw_union_presence: [bool; 3],
    pub raw_union_contains_all_three_members: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoSanheAudit {
    pub audit_version: u16,
    pub algorithm_version: String,
    pub status: String,
    pub fanyin_fuyin: LiuyaoFanyinFuyinAudit,
    pub groups: Vec<LiuyaoSanheGroupObservation>,
    pub canonical_group_count: usize,
    pub raw_occurrence_inventory_built: bool,
    pub source_scope_activation_requirements_classified: bool,
    pub any_sanhe_formation_classified: bool,
    pub combination_effects_assessed: bool,
    pub useful_god_selected: bool,
    pub strength_assessed: bool,
    pub timing_assessed: bool,
    pub interpretation_eligible: bool,
    pub rule_eligible: bool,
    pub blocked_reasons: Vec<String>,
}

fn sanhe_member_index(
    members_birth_prosperity_tomb: [EarthlyBranch; 3],
    branch: EarthlyBranch,
) -> Option<usize> {
    members_birth_prosperity_tomb
        .iter()
        .position(|member| *member == branch)
}

fn push_sanhe_occurrence(
    occurrences: &mut Vec<LiuyaoSanheOccurrence>,
    presence: &mut [bool; 3],
    members: [EarthlyBranch; 3],
    layer: LiuyaoSanheOccurrenceLayer,
    line_number_bottom_up: Option<u8>,
    branch: EarthlyBranch,
) {
    if let Some(index) = sanhe_member_index(members, branch) {
        presence[index] = true;
        occurrences.push(LiuyaoSanheOccurrence {
            layer,
            line_number_bottom_up,
            branch,
            member_role: LiuyaoSanheMemberRole::ALL[index],
        });
    }
}

                                                                            
                                                                        
                                                                            
                                                                             
                                                                           
                                                                       
pub fn liuyao_sanhe_audit(
    cast: &LiuyaoCast,
    month: &JieMonthCandidate,
    day: SexagenaryDesignation,
) -> Result<LiuyaoSanheAudit, LiuyaoCalendarMovingError> {
    let fanyin_fuyin = liuyao_fanyin_fuyin_audit(cast, month, day)?;
    let calendar = &fanyin_fuyin
        .progress_retreat
        .branch_pair_relations
        .calendar_relations;
    let groups = liuyao_sanhe_groups()
        .into_iter()
        .map(|(result_element, members)| {
            let mut occurrences = Vec::new();
            let mut primary_visible_presence = [false; 3];
            let mut moving_transformed_presence = [false; 3];
            let mut locked_month_presence = [false; 3];
            let mut locked_day_presence = [false; 3];

            for line in &calendar.primary_observations_bottom_up {
                push_sanhe_occurrence(
                    &mut occurrences,
                    &mut primary_visible_presence,
                    members,
                    LiuyaoSanheOccurrenceLayer::PrimaryVisible,
                    Some(line.line_number_bottom_up),
                    line.branch,
                );
            }
            for line in &calendar.transformed_moving_observations_bottom_up {
                push_sanhe_occurrence(
                    &mut occurrences,
                    &mut moving_transformed_presence,
                    members,
                    LiuyaoSanheOccurrenceLayer::MovingTransformed,
                    Some(line.line_number_bottom_up),
                    line.branch,
                );
            }
            push_sanhe_occurrence(
                &mut occurrences,
                &mut locked_month_presence,
                members,
                LiuyaoSanheOccurrenceLayer::LockedMonth,
                None,
                calendar.month_build_branch,
            );
            push_sanhe_occurrence(
                &mut occurrences,
                &mut locked_day_presence,
                members,
                LiuyaoSanheOccurrenceLayer::LockedDay,
                None,
                calendar.day_branch,
            );

            let raw_union_presence = std::array::from_fn(|index| {
                primary_visible_presence[index]
                    || moving_transformed_presence[index]
                    || locked_month_presence[index]
                    || locked_day_presence[index]
            });
            LiuyaoSanheGroupObservation {
                result_element,
                members_birth_prosperity_tomb: members,
                occurrences,
                primary_visible_presence,
                moving_transformed_presence,
                locked_month_presence,
                locked_day_presence,
                raw_union_contains_all_three_members: raw_union_presence.iter().all(|seen| *seen),
                raw_union_presence,
            }
        })
        .collect::<Vec<_>>();

    Ok(LiuyaoSanheAudit {
        audit_version: 1,
        algorithm_version: LIUYAO_SANHE_ALGORITHM_VERSION.to_owned(),
        status: LIUYAO_SANHE_STATUS.to_owned(),
        fanyin_fuyin,
        groups,
        canonical_group_count: 4,
        raw_occurrence_inventory_built: true,
        source_scope_activation_requirements_classified: false,
        any_sanhe_formation_classified: false,
        combination_effects_assessed: false,
        useful_god_selected: false,
        strength_assessed: false,
        timing_assessed: false,
        interpretation_eligible: false,
        rule_eligible: false,
        blocked_reasons: vec![
            "facsimile_page_hash_transcription_and_edition_variants_not_closed".to_owned(),
            "moving_static_transformed_month_and_day_activation_scopes_not_formally_released"
                .to_owned(),
            "full_partial_false_true_and_transformation_definitions_not_formally_released"
                .to_owned(),
            "effects_strength_useful_god_timing_advice_and_judgment_not_reviewed".to_owned(),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn all_sixty_four_structures_map_bijectively_to_one_palace_stage() {
        let audits = (0..64)
            .map(|bits| liuyao_eight_palace_audit(bits).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(audits.len(), 64);
        assert_eq!(
            audits
                .iter()
                .map(|audit| (audit.palace, audit.stage))
                .collect::<HashSet<_>>()
                .len(),
            64
        );
    }

    #[test]
    fn received_eight_palace_sequences_match_all_hexagram_numbers() {
        let expected = [
            [1, 44, 33, 12, 20, 23, 35, 14],
            [51, 16, 40, 32, 46, 48, 28, 17],
            [29, 60, 3, 63, 49, 55, 36, 7],
            [52, 22, 26, 41, 38, 10, 61, 53],
            [2, 24, 19, 11, 34, 43, 5, 8],
            [57, 9, 37, 42, 25, 21, 27, 18],
            [30, 56, 50, 64, 4, 59, 6, 13],
            [58, 47, 45, 31, 39, 15, 62, 54],
        ];
        for (palace_index, palace) in LIUYAO_PALACES_IN_RECEIVED_ORDER.iter().enumerate() {
            let pure = palace.bits_bottom_up() | (palace.bits_bottom_up() << 3);
            for stage in LiuyaoPalaceStage::ALL_IN_PALACE_ORDER {
                let audit =
                    liuyao_eight_palace_audit(pure ^ stage.changed_line_mask_from_pure()).unwrap();
                assert_eq!(
                    audit.hexagram.received_sequence,
                    expected[palace_index][stage.sequence() as usize]
                );
            }
        }
    }

    #[test]
    fn host_and_response_lines_follow_received_oppositions() {
        let expected = [
            (6, 3),
            (1, 4),
            (2, 5),
            (3, 6),
            (4, 1),
            (5, 2),
            (4, 1),
            (3, 6),
        ];
        for (stage, expected) in LiuyaoPalaceStage::ALL_IN_PALACE_ORDER
            .into_iter()
            .zip(expected)
        {
            assert_eq!((stage.host_line(), stage.response_line()), expected);
        }
    }

    #[test]
    fn palace_five_phases_are_explicit_and_complete() {
        let phases = LIUYAO_PALACES_IN_RECEIVED_ORDER
            .into_iter()
            .map(trigram_five_phase)
            .collect::<Vec<_>>();
        assert_eq!(
            phases,
            [
                FivePhase::Metal,
                FivePhase::Wood,
                FivePhase::Water,
                FivePhase::Earth,
                FivePhase::Earth,
                FivePhase::Wood,
                FivePhase::Fire,
                FivePhase::Metal,
            ]
        );
    }

    #[test]
    fn audit_fails_closed_before_najia_relatives_spirits_and_rules() {
        let audit = liuyao_eight_palace_audit(0b11_1111).unwrap();
        assert_eq!(audit.compact_zh(), "乾宫本宫 · 世在6爻 · 应在3爻");
        assert!(!audit.na_jia_assigned);
        assert!(!audit.six_relatives_assigned);
        assert!(!audit.six_spirits_assigned);
        assert!(!audit.interpretation_eligible);
        assert!(!audit.rule_eligible);
        assert_eq!(audit.blocked_reasons.len(), 3);
        assert!(liuyao_eight_palace_audit(64).is_err());
    }

    #[test]
    fn eight_pure_hexagrams_match_received_najia_tables_bottom_to_top() {
        let expected = [
            (
                Trigram::Qian,
                ["甲子水", "甲寅木", "甲辰土", "壬午火", "壬申金", "壬戌土"],
            ),
            (
                Trigram::Dui,
                ["丁巳火", "丁卯木", "丁丑土", "丁亥水", "丁酉金", "丁未土"],
            ),
            (
                Trigram::Li,
                ["己卯木", "己丑土", "己亥水", "己酉金", "己未土", "己巳火"],
            ),
            (
                Trigram::Zhen,
                ["庚子水", "庚寅木", "庚辰土", "庚午火", "庚申金", "庚戌土"],
            ),
            (
                Trigram::Xun,
                ["辛丑土", "辛亥水", "辛酉金", "辛未土", "辛巳火", "辛卯木"],
            ),
            (
                Trigram::Kan,
                ["戊寅木", "戊辰土", "戊午火", "戊申金", "戊戌土", "戊子水"],
            ),
            (
                Trigram::Gen,
                ["丙辰土", "丙午火", "丙申金", "丙戌土", "丙子水", "丙寅木"],
            ),
            (
                Trigram::Kun,
                ["乙未土", "乙巳火", "乙卯木", "癸丑土", "癸亥水", "癸酉金"],
            ),
        ];
        for (trigram, expected_lines) in expected {
            let pure = trigram.bits_bottom_up() | (trigram.bits_bottom_up() << 3);
            let audit = liuyao_najia_audit(pure).unwrap();
            assert_eq!(
                audit
                    .lines_bottom_up
                    .iter()
                    .map(LiuyaoNaJiaLine::designation_zh)
                    .collect::<Vec<_>>(),
                expected_lines
            );
        }
    }

    #[test]
    fn all_hexagrams_bind_lower_inner_and_upper_outer_najia() {
        for bits in 0..64 {
            let audit = liuyao_najia_audit(bits).unwrap();
            assert_eq!(audit.lines_bottom_up.len(), 6);
            for line in &audit.lines_bottom_up[..3] {
                assert_eq!(line.trigram, audit.eight_palace.hexagram.lower_trigram);
                assert_eq!(line.trigram_position, LiuyaoTrigramPosition::Inner);
            }
            for line in &audit.lines_bottom_up[3..] {
                assert_eq!(line.trigram, audit.eight_palace.hexagram.upper_trigram);
                assert_eq!(line.trigram_position, LiuyaoTrigramPosition::Outer);
            }
        }
    }

    #[test]
    fn najia_lines_bind_exactly_one_host_and_one_response() {
        for bits in 0..64 {
            let audit = liuyao_najia_audit(bits).unwrap();
            let hosts = audit
                .lines_bottom_up
                .iter()
                .filter(|line| line.is_host)
                .count();
            let responses = audit
                .lines_bottom_up
                .iter()
                .filter(|line| line.is_response)
                .count();
            assert_eq!(hosts, 1);
            assert_eq!(responses, 1);
            assert_ne!(
                audit.eight_palace.host_line,
                audit.eight_palace.response_line
            );
        }
    }

    #[test]
    fn najia_audit_stays_fail_closed_before_relatives_spirits_and_rules() {
        let audit = liuyao_najia_audit(0b11_1111).unwrap();
        assert!(audit.na_jia_stems_assigned);
        assert!(audit.na_jia_branches_assigned);
        assert!(audit.line_five_elements_assigned);
        assert!(audit.host_response_bound);
        assert!(!audit.six_relatives_assigned);
        assert!(!audit.six_spirits_assigned);
        assert!(!audit.hidden_spirits_assigned);
        assert!(!audit.interpretation_eligible);
        assert!(!audit.rule_eligible);
        assert_eq!(audit.blocked_reasons.len(), 3);
        assert!(liuyao_najia_audit(64).is_err());
    }

    #[test]
    fn five_palace_element_tables_map_all_six_relative_relationships() {
        use FiveElement::{Earth, Fire, Metal, Water, Wood};
        use LiuyaoSixRelative::{Descendants, OfficerGhost, Parents, Siblings, WifeWealth};
        let expected = [
            (
                Metal,
                [WifeWealth, OfficerGhost, Parents, Siblings, Descendants],
            ),
            (
                Water,
                [Descendants, WifeWealth, OfficerGhost, Parents, Siblings],
            ),
            (
                Earth,
                [OfficerGhost, Parents, Siblings, Descendants, WifeWealth],
            ),
            (
                Fire,
                [Parents, Siblings, Descendants, WifeWealth, OfficerGhost],
            ),
            (
                Wood,
                [Siblings, Descendants, WifeWealth, OfficerGhost, Parents],
            ),
        ];
        let line_elements = [Wood, Fire, Earth, Metal, Water];
        for (palace, expected_relatives) in expected {
            for (line, expected_relative) in line_elements.into_iter().zip(expected_relatives) {
                assert_eq!(
                    liuyao_six_relative_for_elements(palace, line),
                    expected_relative
                );
            }
        }
    }

    #[test]
    fn qian_and_kun_pure_hexagrams_match_received_six_relative_tables() {
        let expected = [
            (
                Trigram::Qian,
                ["子孙", "妻财", "父母", "官鬼", "兄弟", "父母"],
            ),
            (
                Trigram::Kun,
                ["兄弟", "父母", "官鬼", "兄弟", "妻财", "子孙"],
            ),
        ];
        for (trigram, expected_relatives) in expected {
            let pure = trigram.bits_bottom_up() | (trigram.bits_bottom_up() << 3);
            let audit = liuyao_six_relatives_audit(pure).unwrap();
            assert_eq!(
                audit
                    .lines_bottom_up
                    .iter()
                    .map(|line| line.relationship.zh_name())
                    .collect::<Vec<_>>(),
                expected_relatives
            );
        }
    }

    #[test]
    fn all_hexagram_lines_receive_exactly_one_relative_and_stay_fail_closed() {
        let mut relationships = HashSet::new();
        let mut line_count = 0;
        for bits in 0..64 {
            let audit = liuyao_six_relatives_audit(bits).unwrap();
            assert_eq!(audit.lines_bottom_up.len(), 6);
            assert!(audit.six_relatives_assigned);
            assert!(!audit.six_spirits_assigned);
            assert!(!audit.hidden_spirits_assigned);
            assert!(!audit.useful_god_selected);
            assert!(!audit.strength_assessed);
            assert!(!audit.interpretation_eligible);
            assert!(!audit.rule_eligible);
            for line in audit.lines_bottom_up {
                assert_eq!(
                    line.relationship,
                    liuyao_six_relative_for_elements(
                        audit.palace_element,
                        line.najia.branch_five_element,
                    )
                );
                relationships.insert(line.relationship);
                line_count += 1;
            }
        }
        assert_eq!(line_count, 384);
        assert_eq!(relationships.len(), LiuyaoSixRelative::ALL.len());
        assert!(liuyao_six_relatives_audit(64).is_err());
    }

    #[test]
    fn all_ten_day_stems_match_the_received_six_spirit_table() {
        let expected = [
            (
                HeavenlyStem::Jia,
                ["青龙", "朱雀", "勾陈", "螣蛇", "白虎", "玄武"],
            ),
            (
                HeavenlyStem::Yi,
                ["青龙", "朱雀", "勾陈", "螣蛇", "白虎", "玄武"],
            ),
            (
                HeavenlyStem::Bing,
                ["朱雀", "勾陈", "螣蛇", "白虎", "玄武", "青龙"],
            ),
            (
                HeavenlyStem::Ding,
                ["朱雀", "勾陈", "螣蛇", "白虎", "玄武", "青龙"],
            ),
            (
                HeavenlyStem::Wu,
                ["勾陈", "螣蛇", "白虎", "玄武", "青龙", "朱雀"],
            ),
            (
                HeavenlyStem::Ji,
                ["螣蛇", "白虎", "玄武", "青龙", "朱雀", "勾陈"],
            ),
            (
                HeavenlyStem::Geng,
                ["白虎", "玄武", "青龙", "朱雀", "勾陈", "螣蛇"],
            ),
            (
                HeavenlyStem::Xin,
                ["白虎", "玄武", "青龙", "朱雀", "勾陈", "螣蛇"],
            ),
            (
                HeavenlyStem::Ren,
                ["玄武", "青龙", "朱雀", "勾陈", "螣蛇", "白虎"],
            ),
            (
                HeavenlyStem::Gui,
                ["玄武", "青龙", "朱雀", "勾陈", "螣蛇", "白虎"],
            ),
        ];
        for (stem, expected_lines) in expected {
            let audit = liuyao_six_spirits_audit(0b11_1111, stem).unwrap();
            assert_eq!(audit.day_stem, stem);
            assert_eq!(audit.first_line_spirit.zh_name(), expected_lines[0]);
            assert_eq!(
                audit
                    .lines_bottom_up
                    .iter()
                    .map(|line| line.spirit.zh_name())
                    .collect::<Vec<_>>(),
                expected_lines
            );
        }
    }

    #[test]
    fn six_spirit_assignment_covers_every_hexagram_and_stays_fail_closed() {
        let mut line_count = 0;
        for bits in 0..64 {
            for stem in HeavenlyStem::ALL {
                let audit = liuyao_six_spirits_audit(bits, stem).unwrap();
                assert_eq!(audit.lines_bottom_up.len(), 6);
                assert!(audit.six_spirits_assigned);
                assert!(!audit.hidden_spirits_assigned);
                assert!(!audit.useful_god_selected);
                assert!(!audit.strength_assessed);
                assert!(!audit.interpretation_eligible);
                assert!(!audit.rule_eligible);
                assert_eq!(
                    audit
                        .lines_bottom_up
                        .iter()
                        .map(|line| line.spirit)
                        .collect::<HashSet<_>>()
                        .len(),
                    6
                );
                line_count += audit.lines_bottom_up.len();
            }
        }
        assert_eq!(line_count, 3_840);
        assert!(liuyao_six_spirits_audit(64, HeavenlyStem::Jia).is_err());
    }

    #[test]
    fn qian_palace_hidden_lines_match_received_jingfang_examples() {
        let examples = [
            (
                LiuyaoPalaceStage::FirstGeneration,
                vec![(2, "甲寅木", LiuyaoSixRelative::WifeWealth)],
            ),
            (
                LiuyaoPalaceStage::SecondGeneration,
                vec![
                    (1, "甲子水", LiuyaoSixRelative::Descendants),
                    (2, "甲寅木", LiuyaoSixRelative::WifeWealth),
                ],
            ),
            (
                LiuyaoPalaceStage::FourthGeneration,
                vec![
                    (1, "甲子水", LiuyaoSixRelative::Descendants),
                    (5, "壬申金", LiuyaoSixRelative::Siblings),
                ],
            ),
            (
                LiuyaoPalaceStage::FifthGeneration,
                vec![(5, "壬申金", LiuyaoSixRelative::Siblings)],
            ),
        ];
        let pure = 0b11_1111;
        for (stage, expected) in examples {
            let audit =
                liuyao_hidden_spirits_audit(pure ^ stage.changed_line_mask_from_pure()).unwrap();
            assert_eq!(audit.current.najia.eight_palace.palace, Trigram::Qian);
            assert_eq!(
                audit
                    .hidden_lines_bottom_up
                    .iter()
                    .map(|line| (
                        line.line_number_bottom_up,
                        line.hidden_line.najia.designation_zh(),
                        line.hidden_line.relationship,
                    ))
                    .collect::<Vec<_>>(),
                expected
                    .into_iter()
                    .map(|(number, designation, relationship)| {
                        (number, designation.to_owned(), relationship)
                    })
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn hidden_spirit_search_is_complete_for_all_sixty_four_hexagrams() {
        let mut hidden_count = 0;
        for bits in 0..64 {
            let audit = liuyao_hidden_spirits_audit(bits).unwrap();
            assert!(audit.hidden_spirits_assigned);
            assert!(!audit.flying_hidden_interactions_assessed);
            assert!(!audit.useful_god_selected);
            assert!(!audit.strength_assessed);
            assert!(!audit.interpretation_eligible);
            assert!(!audit.rule_eligible);
            assert_eq!(
                audit.visible_relatives.len() + audit.missing_relatives.len(),
                LiuyaoSixRelative::ALL.len()
            );
            for hidden in &audit.hidden_lines_bottom_up {
                assert_eq!(
                    hidden.line_number_bottom_up,
                    hidden.flying_line.najia.line_number_bottom_up
                );
                assert_eq!(
                    hidden.line_number_bottom_up,
                    hidden.hidden_line.najia.line_number_bottom_up
                );
                assert!(audit
                    .missing_relatives
                    .contains(&hidden.hidden_line.relationship));
                assert_eq!(
                    &hidden.hidden_line,
                    &audit.palace_pure.lines_bottom_up[(hidden.line_number_bottom_up - 1) as usize]
                );
            }
            for missing in &audit.missing_relatives {
                assert!(audit
                    .hidden_lines_bottom_up
                    .iter()
                    .any(|line| line.hidden_line.relationship == *missing));
            }
            if audit.current.najia.eight_palace.stage == LiuyaoPalaceStage::Pure {
                assert!(audit.missing_relatives.is_empty());
                assert!(audit.hidden_lines_bottom_up.is_empty());
            }
            hidden_count += audit.hidden_lines_bottom_up.len();
        }
        assert!(hidden_count > 0);
        assert!(liuyao_hidden_spirits_audit(64).is_err());
    }

    fn mechanical_month(boundary_safe: bool) -> JieMonthCandidate {
        JieMonthCandidate {
            designation: SexagenaryDesignation::from_cycle_index(2),
            previous_jie: SolarTerm::Lichun,
            next_jie: SolarTerm::Jingzhe,
            lichun_year_stem: HeavenlyStem::Jia,
            boundary_safe,
            algorithm_version: "test.jie.month.v1".to_owned(),
            status: "audit_only".to_owned(),
        }
    }

    fn mechanical_cast(types: [LineType; 6]) -> LiuyaoCast {
        use crate::{CoinFace, LineCast, LIUYAO_ALGORITHM_VERSION};
        use uuid::Uuid;

        let mut primary_bits = 0_u8;
        let mut changed_bits = 0_u8;
        let lines = types
            .into_iter()
            .enumerate()
            .map(|(position, line_type)| {
                if line_type.primary_is_yang() {
                    primary_bits |= 1 << position;
                }
                if line_type.changed_is_yang() {
                    changed_bits |= 1 << position;
                }
                LineCast {
                    line_index_bottom_up: position as u8,
                    coins: [CoinFace::Yin; 3],
                    value: line_type.value(),
                    line_type,
                    line_seed_hex: format!("test-{position}"),
                }
            })
            .collect();
        LiuyaoCast {
            algorithm_version: LIUYAO_ALGORITHM_VERSION.to_owned(),
            question_id: Uuid::nil(),
            transcript_hash: "test-transcript".to_owned(),
            master_seed_hex: "test-seed".to_owned(),
            lines,
            primary_bits,
            changed_bits,
        }
    }

    #[test]
    fn weather_rain_research_gate_excludes_day_clash_parent_movement_and_changed_parent() {
        let month = mechanical_month(true);
        let mut witness = None;
        let mut transformed_parent_example = None;
        for bits in 0_u8..64 {
            let relatives = liuyao_six_relatives_audit(bits).unwrap();
            for (position, line) in relatives.lines_bottom_up.iter().enumerate() {
                if line.relationship != LiuyaoSixRelative::OfficerGhost {
                    continue;
                }
                let types = std::array::from_fn(|index| {
                    match (bits & (1 << index) != 0, index == position) {
                        (false, false) => LineType::YoungYin,
                        (true, false) => LineType::YoungYang,
                        (false, true) => LineType::OldYin,
                        (true, true) => LineType::OldYang,
                    }
                });
                let cast = mechanical_cast(types);
                for day_index in 0..12 {
                    let day = SexagenaryDesignation::from_cycle_index(day_index);
                    let relations = liuyao_calendar_relations_audit(&cast, &month, day).unwrap();
                    let candidate = liuyao_weather_rain_candidate_audit(&relations).unwrap();
                    let has_isolated_nonparent_clash =
                        relatives.lines_bottom_up.iter().any(|line| {
                            line.relationship != LiuyaoSixRelative::Parents
                                && !relatives.lines_bottom_up.iter().any(|parent| {
                                    parent.relationship == LiuyaoSixRelative::Parents
                                        && parent.najia.branch == line.najia.branch
                                })
                        });
                    if candidate.narrow_visible_line_pattern_present
                        && has_isolated_nonparent_clash
                        && witness.is_none()
                    {
                        witness = Some((cast.clone(), day, candidate.clone()));
                    }
                    if !candidate.visible_parent_lines_bottom_up.is_empty()
                        && candidate.moving_parent_lines_bottom_up.is_empty()
                        && candidate.day_opposed_visible_lines_bottom_up.is_empty()
                        && !candidate.transformed_parent_lines_bottom_up.is_empty()
                        && transformed_parent_example.is_none()
                    {
                        transformed_parent_example = Some(candidate);
                    }
                }
            }
        }

        let (cast, day, candidate) = witness.expect("a narrow visible-line example exists");
        assert!(!candidate.visible_parent_lines_bottom_up.is_empty());
        assert_eq!(candidate.moving_officer_ghost_lines_bottom_up.len(), 1);
        assert!(candidate.moving_parent_lines_bottom_up.is_empty());
        assert!(candidate.day_opposed_visible_lines_bottom_up.is_empty());
        assert!(candidate.day_opposed_parent_lines_bottom_up.is_empty());
        assert!(candidate.month_opposed_visible_lines_bottom_up.is_empty());
        assert!(candidate.xun_void_visible_lines_bottom_up.is_empty());
        assert!(candidate.parent_combined_visible_pairs_bottom_up.is_empty());
        assert!(candidate.parent_combined_month_lines_bottom_up.is_empty());
        assert!(candidate.parent_combined_day_lines_bottom_up.is_empty());
        assert!(candidate.transformed_parent_lines_bottom_up.is_empty());
        assert!(!candidate.prediction_eligible);
        assert_eq!(candidate.formal_rule_id, None);
        assert!(!candidate.day_boundary_safety_assessed);

        let parent_position = usize::from(candidate.visible_parent_lines_bottom_up[0] - 1);
        let primary_relatives = liuyao_six_relatives_audit(cast.primary_bits).unwrap();
        let parent_branch = primary_relatives.lines_bottom_up[parent_position]
            .najia
            .branch;
        let mut opposed_month = month.clone();
        opposed_month.designation = (0..60)
            .map(SexagenaryDesignation::from_cycle_index)
            .find(|month| month.branch == liuyao_six_opposition_partner(parent_branch))
            .unwrap();
        let opposed_month_relations =
            liuyao_calendar_relations_audit(&cast, &opposed_month, day).unwrap();
        let opposed_month_candidate =
            liuyao_weather_rain_candidate_audit(&opposed_month_relations).unwrap();
        assert!(!opposed_month_candidate.narrow_visible_line_pattern_present);
        assert!(opposed_month_candidate
            .month_opposed_visible_lines_bottom_up
            .contains(&candidate.visible_parent_lines_bottom_up[0]));

        let void_day = (0..60)
            .map(SexagenaryDesignation::from_cycle_index)
            .find(|day| liuyao_xun_void_branches(*day).1.contains(&parent_branch))
            .unwrap();
        let void_relations = liuyao_calendar_relations_audit(&cast, &month, void_day).unwrap();
        let void_candidate = liuyao_weather_rain_candidate_audit(&void_relations).unwrap();
        assert!(!void_candidate.narrow_visible_line_pattern_present);
        assert!(void_candidate
            .xun_void_visible_lines_bottom_up
            .contains(&candidate.visible_parent_lines_bottom_up[0]));

        let clashing_day = (0..60)
            .map(SexagenaryDesignation::from_cycle_index)
            .find(|day| day.branch == liuyao_six_opposition_partner(parent_branch))
            .unwrap();
        let clashing_relations =
            liuyao_calendar_relations_audit(&cast, &month, clashing_day).unwrap();
        let clashing_candidate = liuyao_weather_rain_candidate_audit(&clashing_relations).unwrap();
        assert!(!clashing_candidate.narrow_visible_line_pattern_present);
        assert!(clashing_candidate
            .day_opposed_parent_lines_bottom_up
            .contains(&candidate.visible_parent_lines_bottom_up[0]));

        let nonparent_line = primary_relatives
            .lines_bottom_up
            .iter()
            .find(|line| {
                line.relationship != LiuyaoSixRelative::Parents
                    && !primary_relatives.lines_bottom_up.iter().any(|parent| {
                        parent.relationship == LiuyaoSixRelative::Parents
                            && parent.najia.branch == line.najia.branch
                    })
            })
            .unwrap();
        let nonparent_clashing_day = (0..60)
            .map(SexagenaryDesignation::from_cycle_index)
            .find(|day| day.branch == liuyao_six_opposition_partner(nonparent_line.najia.branch))
            .unwrap();
        let nonparent_clashing_relations =
            liuyao_calendar_relations_audit(&cast, &month, nonparent_clashing_day).unwrap();
        let nonparent_clashing_candidate =
            liuyao_weather_rain_candidate_audit(&nonparent_clashing_relations).unwrap();
        assert!(!nonparent_clashing_candidate.narrow_visible_line_pattern_present);
        assert!(nonparent_clashing_candidate
            .day_opposed_visible_lines_bottom_up
            .contains(&nonparent_line.najia.line_number_bottom_up));
        assert!(nonparent_clashing_candidate
            .day_opposed_parent_lines_bottom_up
            .is_empty());

        let mut parent_moving_types =
            std::array::from_fn(|position| cast.lines[position].line_type);
        parent_moving_types[parent_position] =
            if cast.lines[parent_position].line_type.primary_is_yang() {
                LineType::OldYang
            } else {
                LineType::OldYin
            };
        let parent_moving_cast = mechanical_cast(parent_moving_types);
        let parent_moving_relations =
            liuyao_calendar_relations_audit(&parent_moving_cast, &month, day).unwrap();
        let parent_moving_candidate =
            liuyao_weather_rain_candidate_audit(&parent_moving_relations).unwrap();
        assert!(!parent_moving_candidate.narrow_visible_line_pattern_present);
        assert!(parent_moving_candidate
            .moving_parent_lines_bottom_up
            .contains(&candidate.visible_parent_lines_bottom_up[0]));

        let other_position = primary_relatives
            .lines_bottom_up
            .iter()
            .enumerate()
            .position(|(index, line)| {
                line.relationship != LiuyaoSixRelative::Parents
                    && line.relationship != LiuyaoSixRelative::OfficerGhost
                    && !cast.lines[index].line_type.is_moving()
            })
            .unwrap();
        let mut two_mover_types = std::array::from_fn(|position| cast.lines[position].line_type);
        two_mover_types[other_position] = if cast.lines[other_position].line_type.primary_is_yang()
        {
            LineType::OldYang
        } else {
            LineType::OldYin
        };
        let two_mover_cast = mechanical_cast(two_mover_types);
        let two_mover_relations =
            liuyao_calendar_relations_audit(&two_mover_cast, &month, day).unwrap();
        let two_mover_candidate =
            liuyao_weather_rain_candidate_audit(&two_mover_relations).unwrap();
        assert!(!two_mover_candidate.narrow_visible_line_pattern_present);
        assert!(two_mover_candidate.moving_parent_lines_bottom_up.is_empty());
        assert_eq!(
            two_mover_candidate
                .moving_officer_ghost_lines_bottom_up
                .len(),
            1
        );

        let changed_parent = transformed_parent_example.expect("an officer can change to parent");
        assert!(!changed_parent.narrow_visible_line_pattern_present);
        assert!(!changed_parent.transformed_parent_lines_bottom_up.is_empty());

        let unsafe_month = mechanical_month(false);
        let unsafe_relations = liuyao_calendar_relations_audit(&cast, &unsafe_month, day).unwrap();
        let unsafe_candidate = liuyao_weather_rain_candidate_audit(&unsafe_relations).unwrap();
        assert!(unsafe_candidate.narrow_visible_line_pattern_present);
        assert!(!unsafe_candidate.month_boundary_safe);
        assert!(unsafe_candidate
            .blocked_reasons
            .iter()
            .any(|reason| reason == "month_boundary_unsafe"));
        assert!(!unsafe_candidate.prediction_eligible);
    }

    #[test]
    fn weather_rain_research_gate_excludes_raw_parent_combination_pairs() {
        let month = mechanical_month(true);
        let mut accepted = 0;
        let mut excluded_visible_combination = 0;
        let mut excluded_calendar_combination = 0;
        for bits in 0_u8..64 {
            let relatives = liuyao_six_relatives_audit(bits).unwrap();
            for (position, ghost) in relatives.lines_bottom_up.iter().enumerate() {
                if ghost.relationship != LiuyaoSixRelative::OfficerGhost {
                    continue;
                }
                let types = std::array::from_fn(|index| {
                    match (bits & (1 << index) != 0, index == position) {
                        (false, false) => LineType::YoungYin,
                        (true, false) => LineType::YoungYang,
                        (false, true) => LineType::OldYin,
                        (true, true) => LineType::OldYang,
                    }
                });
                let cast = mechanical_cast(types);
                for day_index in 0..60 {
                    let day = SexagenaryDesignation::from_cycle_index(day_index);
                    let relations = liuyao_calendar_relations_audit(&cast, &month, day).unwrap();
                    let candidate = liuyao_weather_rain_candidate_audit(&relations).unwrap();
                    let prerequisites_except_combination =
                        !candidate.visible_parent_lines_bottom_up.is_empty()
                            && candidate.moving_parent_lines_bottom_up.is_empty()
                            && candidate.day_opposed_visible_lines_bottom_up.is_empty()
                            && candidate.month_opposed_visible_lines_bottom_up.is_empty()
                            && candidate.xun_void_visible_lines_bottom_up.is_empty()
                            && relations.calendar_moving.moving_lines_bottom_up.len() == 1
                            && candidate.moving_officer_ghost_lines_bottom_up.len() == 1
                            && candidate.transformed_parent_lines_bottom_up.is_empty();
                    if !prerequisites_except_combination {
                        continue;
                    }
                    let visible_combination =
                        !candidate.parent_combined_visible_pairs_bottom_up.is_empty();
                    let calendar_combination =
                        !candidate.parent_combined_month_lines_bottom_up.is_empty()
                            || !candidate.parent_combined_day_lines_bottom_up.is_empty();
                    if visible_combination {
                        excluded_visible_combination += 1;
                    }
                    if calendar_combination {
                        excluded_calendar_combination += 1;
                    }
                    if visible_combination || calendar_combination {
                        assert!(!candidate.narrow_visible_line_pattern_present);
                    } else {
                        accepted += 1;
                        assert!(candidate.narrow_visible_line_pattern_present);
                    }
                }
            }
        }
        assert!(accepted > 0);
        assert!(excluded_visible_combination > 0);
        assert!(excluded_calendar_combination > 0);
    }

    #[test]
    fn locked_month_day_and_moving_lines_are_structural_and_fail_closed() {
        let cast = mechanical_cast([
            LineType::OldYin,
            LineType::YoungYang,
            LineType::YoungYin,
            LineType::OldYang,
            LineType::YoungYang,
            LineType::YoungYin,
        ]);
        let day = SexagenaryDesignation::from_cycle_index(40);
        let audit = liuyao_calendar_moving_audit(&cast, &mechanical_month(true), day).unwrap();

        assert_eq!(audit.month_designation.zh_name(), "丙寅");
        assert_eq!(audit.day_designation.zh_name(), "甲辰");
        assert_eq!(audit.moving_line_mask_bottom_up, 0b00_1001);
        assert_eq!(audit.moving_line_numbers_bottom_up, vec![1, 4]);
        assert_eq!(audit.primary_hexagram.bits_bottom_up, 0b01_1010);
        assert_eq!(audit.changed_hexagram.bits_bottom_up, 0b01_0011);
        assert_eq!(audit.moving_lines_bottom_up.len(), 2);
        assert_eq!(audit.moving_lines_bottom_up[0].line_type, LineType::OldYin);
        assert!(!audit.moving_lines_bottom_up[0].primary_is_yang);
        assert!(audit.moving_lines_bottom_up[0].changed_is_yang);
        assert_eq!(audit.moving_lines_bottom_up[1].line_type, LineType::OldYang);
        assert!(audit.moving_lines_bottom_up[1].primary_is_yang);
        assert!(!audit.moving_lines_bottom_up[1].changed_is_yang);
        assert!(audit.month_build_identity_assigned);
        assert!(audit.day_identity_assigned);
        assert!(audit.moving_transformation_verified);
        assert!(!audit.month_day_line_interactions_assessed);
        assert!(!audit.useful_god_selected);
        assert!(!audit.strength_assessed);
        assert!(!audit.interpretation_eligible);
        assert!(!audit.rule_eligible);

        let unsafe_boundary =
            liuyao_calendar_moving_audit(&cast, &mechanical_month(false), day).unwrap();
        assert!(!unsafe_boundary.month_boundary_safe);
        assert!(!unsafe_boundary.rule_eligible);
    }

    #[test]
    fn all_four_to_the_six_line_type_combinations_reconstruct_exactly() {
        let types = [
            LineType::OldYin,
            LineType::YoungYang,
            LineType::YoungYin,
            LineType::OldYang,
        ];
        let month = mechanical_month(true);
        let day = SexagenaryDesignation::from_cycle_index(0);
        for encoded in 0_u16..4_u16.pow(6) {
            let selected = std::array::from_fn(|position| {
                types[((encoded >> (position * 2)) & 0b11) as usize]
            });
            let cast = mechanical_cast(selected);
            let audit = liuyao_calendar_moving_audit(&cast, &month, day).unwrap();
            assert_eq!(
                audit.changed_hexagram.bits_bottom_up,
                audit.primary_hexagram.bits_bottom_up ^ audit.moving_line_mask_bottom_up
            );
            assert_eq!(
                audit.moving_lines_bottom_up.len(),
                audit.moving_line_mask_bottom_up.count_ones() as usize
            );
            assert!(audit.moving_lines_bottom_up.iter().all(|line| {
                line.primary_is_yang != line.changed_is_yang
                    && line.received_value == line.line_type.value()
            }));
            assert!(!audit.interpretation_eligible);
            assert!(!audit.rule_eligible);
        }
    }

    #[test]
    fn moving_audit_rejects_corrupted_cast_material() {
        let mut cast = mechanical_cast([LineType::YoungYang; 6]);
        cast.changed_bits ^= 1;
        assert!(matches!(
            liuyao_calendar_moving_audit(
                &cast,
                &mechanical_month(true),
                SexagenaryDesignation::from_cycle_index(0)
            ),
            Err(LiuyaoCalendarMovingError::StructureMismatch)
        ));

        let mut cast = mechanical_cast([LineType::YoungYin; 6]);
        cast.lines[2].value = 9;
        assert!(matches!(
            liuyao_calendar_moving_audit(
                &cast,
                &mechanical_month(true),
                SexagenaryDesignation::from_cycle_index(0)
            ),
            Err(LiuyaoCalendarMovingError::LineValueTypeMismatch { line_number: 3 })
        ));
    }

    #[test]
    fn all_six_xun_void_pairs_follow_the_two_unpaired_branches() {
        let expected = [
            (0, [EarthlyBranch::Xu, EarthlyBranch::Hai]),
            (10, [EarthlyBranch::Shen, EarthlyBranch::You]),
            (20, [EarthlyBranch::Wu, EarthlyBranch::Wei]),
            (30, [EarthlyBranch::Chen, EarthlyBranch::Si]),
            (40, [EarthlyBranch::Yin, EarthlyBranch::Mao]),
            (50, [EarthlyBranch::Zi, EarthlyBranch::Chou]),
        ];
        for (cycle_index, void_branches) in expected {
            for offset in 0..10 {
                let day = SexagenaryDesignation::from_cycle_index(cycle_index + offset);
                let (xun_start, actual) = liuyao_xun_void_branches(day);
                assert_eq!(xun_start.cycle_index, cycle_index as u8);
                assert_eq!(xun_start.stem, HeavenlyStem::Jia);
                assert_eq!(actual, void_branches);
            }
        }
    }

    #[test]
    fn directional_five_element_relations_cover_the_complete_25_cell_table() {
        use LiuyaoDirectionalFiveElementRelation::{
            Same, SourceGeneratesTarget, SourceOvercomesTarget, TargetGeneratesSource,
            TargetOvercomesSource,
        };

        let elements = [
            FiveElement::Wood,
            FiveElement::Fire,
            FiveElement::Earth,
            FiveElement::Metal,
            FiveElement::Water,
        ];
        let mut counts = [0_usize; 5];
        for source in elements {
            for target in elements {
                let relation = liuyao_directional_five_element_relation(source, target);
                let index = match relation {
                    Same => 0,
                    SourceGeneratesTarget => 1,
                    TargetGeneratesSource => 2,
                    SourceOvercomesTarget => 3,
                    TargetOvercomesSource => 4,
                };
                counts[index] += 1;
                let reverse = liuyao_directional_five_element_relation(target, source);
                assert_eq!(
                    reverse,
                    match relation {
                        Same => Same,
                        SourceGeneratesTarget => TargetGeneratesSource,
                        TargetGeneratesSource => SourceGeneratesTarget,
                        SourceOvercomesTarget => TargetOvercomesSource,
                        TargetOvercomesSource => SourceOvercomesTarget,
                    }
                );
            }
        }
        assert_eq!(counts, [5, 5, 5, 5, 5]);
    }

    #[test]
    fn calendar_branch_observations_cover_all_days_and_hexagrams_without_judgment() {
        let month = mechanical_month(true);
        let mut observed_primary_lines = 0;
        for day_index in 0..60 {
            let day = SexagenaryDesignation::from_cycle_index(day_index);
            for bits in 0_u8..64 {
                let types = std::array::from_fn(|position| {
                    if bits & (1 << position) == 0 {
                        LineType::YoungYin
                    } else {
                        LineType::YoungYang
                    }
                });
                let cast = mechanical_cast(types);
                let audit = liuyao_calendar_relations_audit(&cast, &month, day).unwrap();
                assert_eq!(audit.primary_observations_bottom_up.len(), 6);
                assert!(audit.transformed_moving_observations_bottom_up.is_empty());
                assert_eq!(audit.xun_index_zero_based, day.cycle_index / 10);
                assert_eq!(audit.xun_start.stem, HeavenlyStem::Jia);
                assert_eq!(
                    audit.month_build_five_element,
                    audit.month_build_branch.five_element()
                );
                assert_eq!(
                    audit.day_branch_five_element,
                    audit.day_branch.five_element()
                );
                for observation in &audit.primary_observations_bottom_up {
                    assert_eq!(
                        observation.month_build_five_element_relation,
                        liuyao_directional_five_element_relation(
                            audit.month_build_five_element,
                            observation.branch_five_element,
                        )
                    );
                    assert_eq!(
                        observation.day_branch_five_element_relation,
                        liuyao_directional_five_element_relation(
                            audit.day_branch_five_element,
                            observation.branch_five_element,
                        )
                    );
                    assert_eq!(
                        observation.equals_month_build,
                        observation.branch == audit.month_build_branch
                    );
                    assert_eq!(
                        observation.opposed_by_month_build,
                        observation.branch == audit.month_opposite_branch
                    );
                    assert_eq!(
                        observation.equals_day_branch,
                        observation.branch == audit.day_branch
                    );
                    assert_eq!(
                        observation.opposed_by_day_branch,
                        observation.branch == audit.day_opposite_branch
                    );
                    assert_eq!(
                        observation.is_xun_void,
                        audit.xun_void_branches.contains(&observation.branch)
                    );
                }
                assert!(audit.raw_relations_identified);
                assert!(!audit.relation_effects_assessed);
                assert!(!audit.useful_god_selected);
                assert!(!audit.strength_assessed);
                assert!(!audit.interpretation_eligible);
                assert!(!audit.rule_eligible);
                observed_primary_lines += audit.primary_observations_bottom_up.len();
            }
        }
        assert_eq!(observed_primary_lines, 60 * 64 * 6);

        let moving = mechanical_cast([
            LineType::OldYin,
            LineType::YoungYang,
            LineType::YoungYin,
            LineType::OldYang,
            LineType::YoungYang,
            LineType::YoungYin,
        ]);
        let audit = liuyao_calendar_relations_audit(
            &moving,
            &month,
            SexagenaryDesignation::from_cycle_index(40),
        )
        .unwrap();
        assert_eq!(
            audit
                .transformed_moving_observations_bottom_up
                .iter()
                .map(|line| line.line_number_bottom_up)
                .collect::<Vec<_>>(),
            vec![1, 4]
        );
        assert!(audit
            .transformed_moving_observations_bottom_up
            .iter()
            .all(|line| line.layer == LiuyaoCalendarRelationLayer::MovingTransformed));
    }

    #[test]
    fn received_six_combination_and_six_opposition_tables_are_symmetric() {
        use EarthlyBranch::{Chen, Chou, Hai, Mao, Shen, Si, Wei, Wu, Xu, Yin, You, Zi};

        let combinations = [
            (Zi, Chou),
            (Yin, Hai),
            (Mao, Xu),
            (Chen, You),
            (Si, Shen),
            (Wu, Wei),
        ];
        let oppositions = [
            (Zi, Wu),
            (Chou, Wei),
            (Yin, Shen),
            (Mao, You),
            (Chen, Xu),
            (Si, Hai),
        ];
        for (first, second) in combinations {
            assert_eq!(liuyao_six_combination_partner(first), second);
            assert_eq!(liuyao_six_combination_partner(second), first);
        }
        for (first, second) in oppositions {
            assert_eq!(liuyao_six_opposition_partner(first), second);
            assert_eq!(liuyao_six_opposition_partner(second), first);
        }
        for branch in EarthlyBranch::ALL {
            assert_ne!(liuyao_six_combination_partner(branch), branch);
            assert_ne!(liuyao_six_opposition_partner(branch), branch);
            assert_ne!(
                liuyao_six_combination_partner(branch),
                liuyao_six_opposition_partner(branch)
            );
        }
    }

    #[test]
    fn branch_pair_observations_cover_all_line_types_without_judgment() {
        let types = [
            LineType::OldYin,
            LineType::YoungYang,
            LineType::YoungYin,
            LineType::OldYang,
        ];
        let month = mechanical_month(true);
        let day = SexagenaryDesignation::from_cycle_index(0);
        let mut primary_pair_count = 0;
        let mut moving_pair_count = 0;
        let mut flying_hidden_pair_count = 0;

        for encoded in 0_u16..4_u16.pow(6) {
            let selected = std::array::from_fn(|position| {
                types[((encoded >> (position * 2)) & 0b11) as usize]
            });
            let cast = mechanical_cast(selected);
            let audit = liuyao_branch_pair_relations_audit(&cast, &month, day).unwrap();
            assert_eq!(audit.primary_visible_pair_observations.len(), 15);
            assert_eq!(
                audit.moving_transformation_observations.len(),
                cast.lines
                    .iter()
                    .filter(|line| line.line_type.is_moving())
                    .count()
            );
            assert_eq!(
                audit.flying_hidden_observations.len(),
                audit.hidden_spirits.hidden_lines_bottom_up.len()
            );
            for observation in audit
                .primary_visible_pair_observations
                .iter()
                .chain(audit.moving_transformation_observations.iter())
                .chain(audit.flying_hidden_observations.iter())
            {
                assert_eq!(
                    observation.is_six_combination_pair,
                    liuyao_six_combination_partner(observation.first.branch)
                        == observation.second.branch
                );
                assert_eq!(
                    observation.is_six_opposition_pair,
                    liuyao_six_opposition_partner(observation.first.branch)
                        == observation.second.branch
                );
                assert!(
                    !(observation.is_six_combination_pair && observation.is_six_opposition_pair)
                );
            }
            assert!(audit.raw_branch_pairs_compared);
            assert!(!audit.whole_hexagram_six_combination_classified);
            assert!(!audit.whole_hexagram_six_opposition_classified);
            assert!(!audit.relation_effects_assessed);
            assert!(!audit.useful_god_selected);
            assert!(!audit.strength_assessed);
            assert!(!audit.interpretation_eligible);
            assert!(!audit.rule_eligible);

            primary_pair_count += audit.primary_visible_pair_observations.len();
            moving_pair_count += audit.moving_transformation_observations.len();
            flying_hidden_pair_count += audit.flying_hidden_observations.len();
        }

        assert_eq!(primary_pair_count, 4_usize.pow(6) * 15);
        assert_eq!(moving_pair_count, 4_usize.pow(6) * 3);
        assert!(flying_hidden_pair_count > 0);
    }

    #[test]
    fn progress_retreat_catalog_keeps_only_fourteen_explicitly_listed_directions() {
        use EarthlyBranch::{Chen, Chou, Hai, Mao, Shen, Si, Wei, Wu, Xu, Yin, You, Zi};

        let progress = [
            (Hai, Zi),
            (Yin, Mao),
            (Si, Wu),
            (Shen, You),
            (Chou, Chen),
            (Chen, Wei),
            (Wei, Xu),
        ];
        let retreat = [
            (Zi, Hai),
            (Mao, Yin),
            (Wu, Si),
            (You, Shen),
            (Chen, Chou),
            (Wei, Chen),
            (Xu, Wei),
        ];
        for (primary, transformed) in progress {
            assert_eq!(
                liuyao_progress_retreat_identity(primary, transformed),
                LiuyaoProgressRetreatIdentity::Progress
            );
            assert_eq!(
                liuyao_progress_retreat_identity(transformed, primary),
                LiuyaoProgressRetreatIdentity::Retreat
            );
        }
        for (primary, transformed) in retreat {
            assert_eq!(
                liuyao_progress_retreat_identity(primary, transformed),
                LiuyaoProgressRetreatIdentity::Retreat
            );
            assert_eq!(
                liuyao_progress_retreat_identity(transformed, primary),
                LiuyaoProgressRetreatIdentity::Progress
            );
        }
        assert_eq!(
            liuyao_progress_retreat_identity(Xu, Chou),
            LiuyaoProgressRetreatIdentity::Unclassified
        );
        assert_eq!(
            liuyao_progress_retreat_identity(Chou, Xu),
            LiuyaoProgressRetreatIdentity::Unclassified
        );
        assert_eq!(
            EarthlyBranch::ALL
                .into_iter()
                .flat_map(|primary| EarthlyBranch::ALL
                    .into_iter()
                    .map(move |transformed| (primary, transformed)))
                .filter(|(primary, transformed)| liuyao_progress_retreat_identity(
                    *primary,
                    *transformed
                ) != LiuyaoProgressRetreatIdentity::Unclassified)
                .count(),
            14
        );
    }

    #[test]
    fn progress_retreat_audit_covers_only_actual_moving_lines_without_judgment() {
        let types = [
            LineType::OldYin,
            LineType::YoungYang,
            LineType::YoungYin,
            LineType::OldYang,
        ];
        let month = mechanical_month(true);
        let day = SexagenaryDesignation::from_cycle_index(0);
        let mut moving_observation_count = 0;
        let mut explicitly_listed_count = 0;

        for encoded in 0_u16..4_u16.pow(6) {
            let selected = std::array::from_fn(|position| {
                types[((encoded >> (position * 2)) & 0b11) as usize]
            });
            let cast = mechanical_cast(selected);
            let audit = liuyao_progress_retreat_audit(&cast, &month, day).unwrap();
            assert_eq!(
                audit.moving_observations_bottom_up.len(),
                cast.lines
                    .iter()
                    .filter(|line| line.line_type.is_moving())
                    .count()
            );
            for observation in &audit.moving_observations_bottom_up {
                assert_eq!(
                    observation.identity,
                    liuyao_progress_retreat_identity(
                        observation.primary_branch,
                        observation.transformed_branch
                    )
                );
                assert_eq!(
                    observation.received_pair_explicitly_listed,
                    observation.identity != LiuyaoProgressRetreatIdentity::Unclassified
                );
                if observation.received_pair_explicitly_listed {
                    explicitly_listed_count += 1;
                }
            }
            assert_eq!(audit.explicit_progress_pair_count, 7);
            assert_eq!(audit.explicit_retreat_pair_count, 7);
            assert!(audit.raw_identity_assigned);
            assert!(audit.omitted_earth_cycle_pairs_left_unclassified);
            assert!(!audit.progression_or_retreat_effect_assessed);
            assert!(!audit.useful_god_selected);
            assert!(!audit.strength_assessed);
            assert!(!audit.timing_assessed);
            assert!(!audit.interpretation_eligible);
            assert!(!audit.rule_eligible);
            moving_observation_count += audit.moving_observations_bottom_up.len();
        }

        assert_eq!(moving_observation_count, 4_usize.pow(6) * 3);
        assert!(explicitly_listed_count > 0);
    }

    #[test]
    fn fanyin_received_trigram_pairs_are_symmetric_and_complete() {
        let expected = [
            (Trigram::Qian, Trigram::Xun),
            (Trigram::Kan, Trigram::Li),
            (Trigram::Zhen, Trigram::Dui),
            (Trigram::Kun, Trigram::Gen),
        ];
        for (first, second) in expected {
            assert_eq!(liuyao_fanyin_trigram_partner(first), second);
            assert_eq!(liuyao_fanyin_trigram_partner(second), first);
        }
        for trigram in Trigram::ALL_IN_RECEIVED_NUMBER_ORDER {
            assert_ne!(liuyao_fanyin_trigram_partner(trigram), trigram);
            assert_eq!(
                liuyao_fanyin_trigram_partner(liuyao_fanyin_trigram_partner(trigram)),
                trigram
            );
        }
    }

    #[test]
    fn fanyin_fuyin_prerequisites_cover_all_line_types_without_judgment() {
        let types = [
            LineType::OldYin,
            LineType::YoungYang,
            LineType::YoungYin,
            LineType::OldYang,
        ];
        let month = mechanical_month(true);
        let day = SexagenaryDesignation::from_cycle_index(0);
        let mut moving_observation_count = 0;
        let mut trigram_fanyin_candidate_count = 0;
        let mut line_fanyin_scope_candidate_count = 0;
        let mut fuyin_scope_candidate_count = 0;
        let mut moving_line_fanyin_candidate_count = 0;
        let mut moving_line_fuyin_candidate_count = 0;

        for encoded in 0_u16..4_u16.pow(6) {
            let selected = std::array::from_fn(|position| {
                types[((encoded >> (position * 2)) & 0b11) as usize]
            });
            let cast = mechanical_cast(selected);
            let audit = liuyao_fanyin_fuyin_audit(&cast, &month, day).unwrap();
            assert_eq!(
                audit.moving_line_observations_bottom_up.len(),
                cast.lines
                    .iter()
                    .filter(|line| line.line_type.is_moving())
                    .count()
            );
            for scope in [&audit.inner_scope, &audit.outer_scope] {
                assert_eq!(scope.line_numbers_bottom_up.len(), 3);
                assert_eq!(scope.primary_branches_bottom_up.len(), 3);
                assert_eq!(scope.transformed_branches_bottom_up.len(), 3);
                assert_eq!(
                    scope.is_received_direction_opposition_pair,
                    liuyao_fanyin_trigram_partner(scope.primary_trigram)
                        == scope.transformed_trigram
                );
                assert_eq!(
                    scope.all_same_position_branches_equal,
                    scope
                        .primary_branches_bottom_up
                        .iter()
                        .zip(scope.transformed_branches_bottom_up.iter())
                        .all(|(primary, transformed)| primary == transformed)
                );
                assert_eq!(
                    scope.all_same_position_branches_opposed,
                    scope
                        .primary_branches_bottom_up
                        .iter()
                        .zip(scope.transformed_branches_bottom_up.iter())
                        .all(|(primary, transformed)| {
                            liuyao_six_opposition_partner(*primary) == *transformed
                        })
                );
                if scope.moving_line_numbers_bottom_up.is_empty() {
                    assert!(!scope.raw_trigram_fanyin_candidate);
                    assert!(!scope.raw_line_fanyin_candidate);
                    assert!(!scope.raw_fuyin_candidate);
                }
                if scope.raw_trigram_fanyin_candidate {
                    trigram_fanyin_candidate_count += 1;
                }
                if scope.raw_line_fanyin_candidate {
                    line_fanyin_scope_candidate_count += 1;
                }
                if scope.raw_fuyin_candidate {
                    fuyin_scope_candidate_count += 1;
                }
            }
            for observation in &audit.moving_line_observations_bottom_up {
                assert_eq!(
                    observation.same_branch,
                    observation.primary_branch == observation.transformed_branch
                );
                assert_eq!(
                    observation.opposed_branch,
                    liuyao_six_opposition_partner(observation.primary_branch)
                        == observation.transformed_branch
                );
                assert_eq!(
                    observation.raw_line_fuyin_candidate,
                    observation.same_branch
                );
                assert_eq!(
                    observation.raw_line_fanyin_candidate,
                    observation.opposed_branch
                );
                if observation.raw_line_fanyin_candidate {
                    moving_line_fanyin_candidate_count += 1;
                }
                if observation.raw_line_fuyin_candidate {
                    moving_line_fuyin_candidate_count += 1;
                }
            }
            assert!(audit.raw_prerequisites_identified);
            assert!(!audit.whole_hexagram_fanyin_classified);
            assert!(!audit.whole_hexagram_fuyin_classified);
            assert!(!audit.relation_effects_assessed);
            assert!(!audit.useful_god_selected);
            assert!(!audit.strength_assessed);
            assert!(!audit.timing_assessed);
            assert!(!audit.interpretation_eligible);
            assert!(!audit.rule_eligible);
            moving_observation_count += audit.moving_line_observations_bottom_up.len();
        }

        assert_eq!(moving_observation_count, 4_usize.pow(6) * 3);
        assert!(trigram_fanyin_candidate_count > 0);
        assert!(line_fanyin_scope_candidate_count > 0);
        assert!(fuyin_scope_candidate_count > 0);
        assert!(moving_line_fanyin_candidate_count > 0);
        assert!(moving_line_fuyin_candidate_count > 0);
    }

    #[test]
    fn sanhe_catalog_covers_each_branch_once_in_birth_prosperity_tomb_order() {
        use EarthlyBranch::{Chen, Chou, Hai, Mao, Shen, Si, Wei, Wu, Xu, Yin, You, Zi};
        let expected = [
            (FiveElement::Water, [Shen, Zi, Chen]),
            (FiveElement::Wood, [Hai, Mao, Wei]),
            (FiveElement::Fire, [Yin, Wu, Xu]),
            (FiveElement::Metal, [Si, You, Chou]),
        ];
        assert_eq!(liuyao_sanhe_groups(), expected);
        let members = liuyao_sanhe_groups()
            .into_iter()
            .flat_map(|(_, group)| group)
            .collect::<Vec<_>>();
        assert_eq!(members.len(), EarthlyBranch::ALL.len());
        let mut seen = [false; 12];
        for member in &members {
            assert!(!seen[member.index() as usize]);
            seen[member.index() as usize] = true;
        }
        assert!(seen.into_iter().all(|value| value));
        for branch in EarthlyBranch::ALL {
            assert_eq!(
                members.iter().filter(|member| **member == branch).count(),
                1
            );
        }
    }

    #[test]
    fn sanhe_raw_occurrences_cover_all_line_types_without_classifying_formation() {
        let types = [
            LineType::OldYin,
            LineType::YoungYang,
            LineType::YoungYin,
            LineType::OldYang,
        ];
        let month = mechanical_month(true);
        let day = SexagenaryDesignation::from_cycle_index(0);
        let mut total_occurrences = 0;
        let mut total_moving_lines = 0;
        let mut raw_union_complete_count = 0;

        for encoded in 0_u16..4_u16.pow(6) {
            let selected = std::array::from_fn(|position| {
                types[((encoded >> (position * 2)) & 0b11) as usize]
            });
            let cast = mechanical_cast(selected);
            let moving_lines = cast
                .lines
                .iter()
                .filter(|line| line.line_type.is_moving())
                .count();
            let audit = liuyao_sanhe_audit(&cast, &month, day).unwrap();
            assert_eq!(audit.groups.len(), 4);
            assert_eq!(audit.canonical_group_count, 4);

            let layer_count = |layer| {
                audit
                    .groups
                    .iter()
                    .flat_map(|group| &group.occurrences)
                    .filter(|occurrence| occurrence.layer == layer)
                    .count()
            };
            assert_eq!(layer_count(LiuyaoSanheOccurrenceLayer::PrimaryVisible), 6);
            assert_eq!(
                layer_count(LiuyaoSanheOccurrenceLayer::MovingTransformed),
                moving_lines
            );
            assert_eq!(layer_count(LiuyaoSanheOccurrenceLayer::LockedMonth), 1);
            assert_eq!(layer_count(LiuyaoSanheOccurrenceLayer::LockedDay), 1);

            for group in &audit.groups {
                let mut primary = [false; 3];
                let mut transformed = [false; 3];
                let mut locked_month = [false; 3];
                let mut locked_day = [false; 3];
                for occurrence in &group.occurrences {
                    let member_index = match occurrence.member_role {
                        LiuyaoSanheMemberRole::Birth => 0,
                        LiuyaoSanheMemberRole::Prosperity => 1,
                        LiuyaoSanheMemberRole::Tomb => 2,
                    };
                    assert_eq!(
                        group.members_birth_prosperity_tomb[member_index],
                        occurrence.branch
                    );
                    let presence = match occurrence.layer {
                        LiuyaoSanheOccurrenceLayer::PrimaryVisible => {
                            assert!(occurrence.line_number_bottom_up.is_some());
                            &mut primary
                        }
                        LiuyaoSanheOccurrenceLayer::MovingTransformed => {
                            assert!(occurrence.line_number_bottom_up.is_some());
                            &mut transformed
                        }
                        LiuyaoSanheOccurrenceLayer::LockedMonth => {
                            assert!(occurrence.line_number_bottom_up.is_none());
                            &mut locked_month
                        }
                        LiuyaoSanheOccurrenceLayer::LockedDay => {
                            assert!(occurrence.line_number_bottom_up.is_none());
                            &mut locked_day
                        }
                    };
                    presence[member_index] = true;
                }
                assert_eq!(group.primary_visible_presence, primary);
                assert_eq!(group.moving_transformed_presence, transformed);
                assert_eq!(group.locked_month_presence, locked_month);
                assert_eq!(group.locked_day_presence, locked_day);
                let union = std::array::from_fn(|index| {
                    primary[index] || transformed[index] || locked_month[index] || locked_day[index]
                });
                assert_eq!(group.raw_union_presence, union);
                assert_eq!(
                    group.raw_union_contains_all_three_members,
                    union.iter().all(|seen| *seen)
                );
                if group.raw_union_contains_all_three_members {
                    raw_union_complete_count += 1;
                }
            }
            assert!(audit.raw_occurrence_inventory_built);
            assert!(!audit.source_scope_activation_requirements_classified);
            assert!(!audit.any_sanhe_formation_classified);
            assert!(!audit.combination_effects_assessed);
            assert!(!audit.useful_god_selected);
            assert!(!audit.strength_assessed);
            assert!(!audit.timing_assessed);
            assert!(!audit.interpretation_eligible);
            assert!(!audit.rule_eligible);
            total_occurrences += audit
                .groups
                .iter()
                .map(|group| group.occurrences.len())
                .sum::<usize>();
            total_moving_lines += moving_lines;
        }

        assert_eq!(total_moving_lines, 4_usize.pow(6) * 3);
        assert_eq!(total_occurrences, 4_usize.pow(6) * 8 + total_moving_lines);
        assert!(raw_union_complete_count > 0);
    }
}
