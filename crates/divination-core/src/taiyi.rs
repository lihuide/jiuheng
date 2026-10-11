                                                                   
   
                                                                            
                                                                            
                                                                        
                                                                           
                                                                             
                                                                             
                                
                                                                     
                                                                      
                                                                              
                                                                              
                                                                            
                                                                              
                                                                          
                                            

use crate::{EarthlyBranch, SexagenaryDesignation};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const TAIYI_ACCUMULATED_YEAR_POSITION_ALGORITHM_VERSION: &str =
    "jiuheng.taiyi.tang_accumulated_year_palace_candidate.v1";
pub const TAIYI_ACCUMULATED_YEAR_POSITION_STATUS: &str =
    "mechanical_palace_from_supplied_accumulated_year_only_no_epoch_calendar_board_or_interpretation.v1";
pub const TAIYI_ANNUAL_FRAME_V1_ALGORITHM_VERSION: &str =
    "jiuheng.taiyi.tang_supplied_accumulated_year_frame_candidate.v1";
pub const TAIYI_ANNUAL_FRAME_V2_ALGORITHM_VERSION: &str =
    "jiuheng.taiyi.tang_supplied_accumulated_year_frame_candidate.v2";
pub const TAIYI_ANNUAL_FRAME_V3_ALGORITHM_VERSION: &str =
    "jiuheng.taiyi.tang_supplied_accumulated_year_frame_candidate.v3";
pub const TAIYI_ANNUAL_FRAME_V4_ALGORITHM_VERSION: &str =
    "jiuheng.taiyi.tang_supplied_accumulated_year_frame_candidate.v4";
pub const TAIYI_ANNUAL_FRAME_V5_ALGORITHM_VERSION: &str =
    "jiuheng.taiyi.tang_supplied_accumulated_year_frame_candidate.v5";
pub const TAIYI_ANNUAL_FRAME_V6_ALGORITHM_VERSION: &str =
    "jiuheng.taiyi.tang_supplied_accumulated_year_frame_candidate.v6";
pub const TAIYI_ANNUAL_FRAME_V7_ALGORITHM_VERSION: &str =
    "jiuheng.taiyi.tang_supplied_accumulated_year_frame_candidate.v7";
pub const TAIYI_ANNUAL_FRAME_V8_ALGORITHM_VERSION: &str =
    "jiuheng.taiyi.tang_supplied_accumulated_year_frame_candidate.v8";
pub const TAIYI_ANNUAL_FRAME_ALGORITHM_VERSION: &str =
    "jiuheng.taiyi.tang_supplied_accumulated_year_frame_candidate.v9";
pub const TAIYI_ANNUAL_FRAME_STATUS: &str =
    "mechanical_taisui_palace_tianmu_jishen_heshin_shiji_counts_generals_annual_duty_gate_position_marks_strike_pressure_opposition_barrier_raw_observations_and_open_anchored_annual_gate_boards_from_supplied_accumulated_year_no_epoch_calendar_time_gate_boards_or_interpretation.v9";
pub const TAIYI_MECHANICAL_RELATIONS_V1_ALGORITHM_VERSION: &str =
    "jiuheng.taiyi.tang_cover_imprisonment_ge_position_marks_candidate.v1";
pub const TAIYI_MECHANICAL_RELATIONS_V2_ALGORITHM_VERSION: &str =
    "jiuheng.taiyi.tang_position_marks_near_ring_geometry_opposition_candidate.v2";
pub const TAIYI_MECHANICAL_RELATIONS_V3_ALGORITHM_VERSION: &str =
    "jiuheng.taiyi.tang_strike_pressure_and_position_marks_candidate.v3";
pub const TAIYI_MECHANICAL_RELATIONS_ALGORITHM_VERSION: &str =
    "jiuheng.taiyi.tang_position_marks_and_barrier_raw_observations_candidate.v4";
pub const TAIYI_MECHANICAL_RELATIONS_STATUS: &str =
    "mechanical_cover_imprisonment_ge_opposition_strike_pressure_position_marks_and_unclassified_barrier_raw_observations_no_barrier_compound_patterns_interpretation_or_rules.v4";
pub const TAIYI_BARRIER_RAW_OBSERVATIONS_ALGORITHM_VERSION: &str =
    "jiuheng.taiyi.tang_barrier_raw_observations_candidate.v1";
pub const TAIYI_BARRIER_RAW_OBSERVATIONS_STATUS: &str =
    "exhaustive_eye_general_and_general_general_same_palace_pairs_plus_main_guest_count_equality_no_barrier_classification_interpretation_or_rules.v1";
pub const TAIYI_SEVENTY_TWO_BOARD_V1_ALGORITHM_VERSION: &str =
    "jiuheng.taiyi.tang_explicit_dun_seventy_two_board_candidate.v1";
pub const TAIYI_SEVENTY_TWO_BOARD_V2_ALGORITHM_VERSION: &str =
    "jiuheng.taiyi.tang_explicit_dun_seventy_two_board_candidate.v2";
pub const TAIYI_SEVENTY_TWO_BOARD_V3_ALGORITHM_VERSION: &str =
    "jiuheng.taiyi.tang_explicit_dun_seventy_two_board_candidate.v3";
pub const TAIYI_SEVENTY_TWO_BOARD_ALGORITHM_VERSION: &str =
    "jiuheng.taiyi.tang_explicit_dun_seventy_two_board_candidate.v4";
pub const TAIYI_SEVENTY_TWO_BOARD_STATUS: &str =
    "mechanical_taiyi_tianmu_jishen_shiji_counts_generals_strike_pressure_position_marks_and_unclassified_barrier_raw_observations_from_positive_board_count_and_explicit_dun_no_civil_time_dun_selection_complete_board_or_interpretation.v4";
pub const TAIYI_ANNUAL_GATE_BOARDS_ALGORITHM_VERSION: &str =
    "jiuheng.taiyi.tang_annual_open_anchored_gate_boards_candidate.v1";
pub const TAIYI_ANNUAL_GATE_BOARDS_STATUS: &str =
    "mechanical_annual_open_gate_rotations_for_taiyi_and_noncentral_main_guest_great_generals_center_anchors_blocked_no_time_method_fixed_count_general_interpretation_or_rules.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaiyiDun {
    #[default]
    Yang,
    Yin,
}

impl TaiyiDun {
    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Yang => "阳遁",
            Self::Yin => "阴遁",
        }
    }

    const fn taiyi_order(self) -> [TaiyiPalace; 8] {
        match self {
            Self::Yang => TaiyiPalace::FORWARD_ORDER,
            Self::Yin => [
                TaiyiPalace::Nine,
                TaiyiPalace::Eight,
                TaiyiPalace::Seven,
                TaiyiPalace::Six,
                TaiyiPalace::Four,
                TaiyiPalace::Three,
                TaiyiPalace::Two,
                TaiyiPalace::One,
            ],
        }
    }

    const fn jishen_start(self) -> EarthlyBranch {
        match self {
            Self::Yang => EarthlyBranch::Yin,
            Self::Yin => EarthlyBranch::Shen,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaiyiPalace {
    One,
    Two,
    Three,
    Four,
    Six,
    Seven,
    Eight,
    Nine,
}

impl TaiyiPalace {
                                                                         
                                                                           
                                            
    pub const FORWARD_ORDER: [Self; 8] = [
        Self::One,
        Self::Two,
        Self::Three,
        Self::Four,
        Self::Six,
        Self::Seven,
        Self::Eight,
        Self::Nine,
    ];

    pub const fn number(self) -> u8 {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
            Self::Six => 6,
            Self::Seven => 7,
            Self::Eight => 8,
            Self::Nine => 9,
        }
    }

    pub fn compact_zh(self) -> String {
        format!("{}宫", self.number())
    }

                                                                            
                                                                            
                                                                          
                             
    const fn ring_index(self) -> usize {
        match self {
            Self::One => 14,
            Self::Two => 8,
            Self::Three => 2,
            Self::Four => 4,
            Self::Six => 12,
            Self::Seven => 10,
            Self::Eight => 0,
            Self::Nine => 6,
        }
    }

                                                                              
                                                                           
                                                     
    const fn gate_ring_index(self) -> usize {
        match self {
            Self::One => 0,
            Self::Eight => 1,
            Self::Three => 2,
            Self::Four => 3,
            Self::Nine => 4,
            Self::Two => 5,
            Self::Seven => 6,
            Self::Six => 7,
        }
    }

                                                                          
                                                                            
    pub const fn opposite(self) -> Self {
        match self {
            Self::One => Self::Nine,
            Self::Two => Self::Eight,
            Self::Three => Self::Seven,
            Self::Four => Self::Six,
            Self::Six => Self::Four,
            Self::Seven => Self::Three,
            Self::Eight => Self::Two,
            Self::Nine => Self::One,
        }
    }
}

                                                                 
   
                                                                              
                                                                              
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaiyiGeneralPalace {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

impl TaiyiGeneralPalace {
    pub const fn number(self) -> u8 {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
            Self::Five => 5,
            Self::Six => 6,
            Self::Seven => 7,
            Self::Eight => 8,
            Self::Nine => 9,
        }
    }

    pub fn compact_zh(self) -> String {
        format!("{}宫", self.number())
    }

    const fn noncentral_taiyi_palace(self) -> Option<TaiyiPalace> {
        match self {
            Self::One => Some(TaiyiPalace::One),
            Self::Two => Some(TaiyiPalace::Two),
            Self::Three => Some(TaiyiPalace::Three),
            Self::Four => Some(TaiyiPalace::Four),
            Self::Five => None,
            Self::Six => Some(TaiyiPalace::Six),
            Self::Seven => Some(TaiyiPalace::Seven),
            Self::Eight => Some(TaiyiPalace::Eight),
            Self::Nine => Some(TaiyiPalace::Nine),
        }
    }
}

                                                                       
   
                                                                            
                                                                            
                     
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaiyiGate {
    Open,
    Rest,
    Life,
    Harm,
    Block,
    View,
    Death,
    Alarm,
}

impl TaiyiGate {
    pub const RECEIVED_LEFTWARD_ORDER: [Self; 8] = [
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
            Self::Open => "开门",
            Self::Rest => "休门",
            Self::Life => "生门",
            Self::Harm => "伤门",
            Self::Block => "杜门",
            Self::View => "景门",
            Self::Death => "死门",
            Self::Alarm => "惊门",
        }
    }

                                                                            
                                                                        
    pub const fn fixed_taiyi_palace(self) -> TaiyiPalace {
        match self {
            Self::Open => TaiyiPalace::One,                       
            Self::Rest => TaiyiPalace::Eight,                
            Self::Life => TaiyiPalace::Three,                    
            Self::Harm => TaiyiPalace::Four,                 
            Self::Block => TaiyiPalace::Nine,                    
            Self::View => TaiyiPalace::Two,                 
            Self::Death => TaiyiPalace::Seven,                   
            Self::Alarm => TaiyiPalace::Six,                
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaiyiAnnualGateBoardAnchor {
    Taiyi,
    MainGreatGeneral,
    GuestGreatGeneral,
}

impl TaiyiAnnualGateBoardAnchor {
    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Taiyi => "太乙",
            Self::MainGreatGeneral => "主大将",
            Self::GuestGreatGeneral => "客大将",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaiyiGatePlacement {
    pub gate: TaiyiGate,
    pub palace: TaiyiPalace,
}

                                                                           
                                                                             
                                                     
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaiyiAnnualOpenGateBoard {
    pub anchor: TaiyiAnnualGateBoardAnchor,
    pub anchor_palace: TaiyiPalace,
    pub anchor_gate: TaiyiGate,
    pub placements: [TaiyiGatePlacement; 8],
    pub open_gate_is_at_anchor: bool,
    pub interpretation_eligible: bool,
    pub signal_eligible: bool,
    pub rule_eligible: bool,
}

                                                                           
                                                                           
                                                                            
                                                                           
                                                                             
                 
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaiyiAnnualGateBoardsCandidate {
    pub algorithm_version: String,
    pub status: String,
    pub anchor_gate: TaiyiGate,
    pub taiyi_board: TaiyiAnnualOpenGateBoard,
    pub main_great_general_board: Option<TaiyiAnnualOpenGateBoard>,
    pub main_great_general_unavailable_reason: Option<String>,
    pub guest_great_general_board: Option<TaiyiAnnualOpenGateBoard>,
    pub guest_great_general_unavailable_reason: Option<String>,
    pub all_three_boards_generated: bool,
    pub fixed_count_general_board_generated: bool,
    pub time_method_duty_gate_rotation_generated: bool,
    pub three_auspicious_gates_classified: bool,
    pub interpretation_eligible: bool,
    pub signal_eligible: bool,
    pub rule_eligible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaiyiSixteenSpirit {
    DiZhu,
    YangDe,
    HeDe,
    LuShen,
    GaoCong,
    TaiYang,
    TaiJiong,
    TaiShen,
    DaWei,
    TianDao,
    DaWu,
    WuDe,
    TaiCu,
    YinZhu,
    YinDe,
    DaYi,
}

impl TaiyiSixteenSpirit {
                                                                   
                                                                        
                      
    pub const RECEIVED_RING_ORDER: [Self; 16] = [
        Self::DiZhu,
        Self::YangDe,
        Self::HeDe,
        Self::LuShen,
        Self::GaoCong,
        Self::TaiYang,
        Self::TaiJiong,
        Self::TaiShen,
        Self::DaWei,
        Self::TianDao,
        Self::DaWu,
        Self::WuDe,
        Self::TaiCu,
        Self::YinZhu,
        Self::YinDe,
        Self::DaYi,
    ];

                                                                              
                                                                            
                                                                            
    pub const TIANMU_CYCLE_WITH_STOPS: [Self; 18] = [
        Self::WuDe,
        Self::TaiCu,
        Self::YinZhu,
        Self::YinDe,
        Self::YinDe,
        Self::DaYi,
        Self::DiZhu,
        Self::YangDe,
        Self::HeDe,
        Self::LuShen,
        Self::GaoCong,
        Self::TaiYang,
        Self::TaiJiong,
        Self::TaiShen,
        Self::DaWei,
        Self::TianDao,
        Self::DaWu,
        Self::DaWu,
    ];

                                                                       
                                                                           
    pub const YIN_TIANMU_CYCLE_WITH_STOPS: [Self; 18] = [
        Self::LuShen,
        Self::GaoCong,
        Self::TaiYang,
        Self::TaiJiong,
        Self::TaiJiong,
        Self::TaiShen,
        Self::DaWei,
        Self::TianDao,
        Self::DaWu,
        Self::WuDe,
        Self::TaiCu,
        Self::YinZhu,
        Self::YinDe,
        Self::DaYi,
        Self::DiZhu,
        Self::YangDe,
        Self::HeDe,
        Self::HeDe,
    ];

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::DiZhu => "地主",
            Self::YangDe => "阳德",
            Self::HeDe => "和德",
            Self::LuShen => "吕申",
            Self::GaoCong => "高丛",
            Self::TaiYang => "太阳",
            Self::TaiJiong => "太炅",
                                                                           
                                                                          
                                                                              
            Self::TaiShen => "太神",
            Self::DaWei => "大威",
            Self::TianDao => "天道",
            Self::DaWu => "大武",
            Self::WuDe => "武德",
            Self::TaiCu => "太蔟",
            Self::YinZhu => "阴主",
            Self::YinDe => "阴德",
            Self::DaYi => "大义",
        }
    }

    pub const fn received_table_variant(self) -> Option<&'static str> {
        match self {
            Self::TaiShen => Some("卷三立成表作‘大神’"),
            _ => None,
        }
    }

    const fn ring_index(self) -> usize {
        match self {
            Self::DiZhu => 0,
            Self::YangDe => 1,
            Self::HeDe => 2,
            Self::LuShen => 3,
            Self::GaoCong => 4,
            Self::TaiYang => 5,
            Self::TaiJiong => 6,
            Self::TaiShen => 7,
            Self::DaWei => 8,
            Self::TianDao => 9,
            Self::DaWu => 10,
            Self::WuDe => 11,
            Self::TaiCu => 12,
            Self::YinZhu => 13,
            Self::YinDe => 14,
            Self::DaYi => 15,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaiyiAccumulatedYearPosition {
    pub algorithm_version: String,
    pub status: String,
                                                                           
    pub accumulated_year_count: u64,
                                                                         
                                                                      
    pub position_in_360: u16,
    pub position_in_72: u8,
    pub position_in_24: u8,
    pub palace_step: u8,
    pub palace: TaiyiPalace,
    pub year_within_palace: u8,
    pub calendar_epoch_selected: bool,
    pub derived_from_civil_date: bool,
    pub complete_board_generated: bool,
    pub interpretation_eligible: bool,
    pub signal_eligible: bool,
    pub rule_eligible: bool,
}

                                                                            
                                                                            
                                     
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaiyiNearRingGeometry {
    PlusOneInterstitial,
    MinusOneInterstitial,
    PlusTwoPrincipalPalace,
    MinusTwoPrincipalPalace,
}

impl TaiyiNearRingGeometry {
    pub const fn compact_zh(self) -> &'static str {
        match self {
            Self::PlusOneInterstitial => "固定环序 +1（相邻辰位）",
            Self::MinusOneInterstitial => "固定环序 -1（相邻辰位）",
            Self::PlusTwoPrincipalPalace => "固定环序 +2（相邻正宫）",
            Self::MinusTwoPrincipalPalace => "固定环序 -2（相邻正宫）",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaiyiStrike {
    FrontChen,
    RearChen,
    OuterPalace,
    InnerPalace,
}

impl TaiyiStrike {
    pub const fn compact_zh(self) -> &'static str {
        match self {
            Self::FrontChen => "前击",
            Self::RearChen => "后击",
            Self::OuterPalace => "外宫击",
            Self::InnerPalace => "内宫击",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaiyiPressure {
    OuterChen,
    InnerChen,
    OuterPalace,
    InnerPalace,
}

impl TaiyiPressure {
    pub const fn compact_zh(self) -> &'static str {
        match self {
            Self::OuterChen => "外辰迫",
            Self::InnerChen => "内辰迫",
            Self::OuterPalace => "外宫迫",
            Self::InnerPalace => "内宫迫",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaiyiEyeRole {
    Tianmu,
    Shiji,
}

impl TaiyiEyeRole {
    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Tianmu => "天目",
            Self::Shiji => "始击",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaiyiGeneralRole {
    MainGreat,
    MainAssistant,
    GuestGreat,
    GuestAssistant,
}

impl TaiyiGeneralRole {
    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::MainGreat => "主大将",
            Self::MainAssistant => "主参将",
            Self::GuestGreat => "客大将",
            Self::GuestAssistant => "客参将",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaiyiEyeGeneralCoincidence {
    pub eye: TaiyiEyeRole,
    pub general: TaiyiGeneralRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaiyiGeneralCoincidence {
    pub first: TaiyiGeneralRole,
    pub second: TaiyiGeneralRole,
}

                                                                          
   
                                                                            
                                                                           
                                                                         
                                                                           
                                                                    
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaiyiBarrierRawObservations {
    pub algorithm_version: String,
    pub status: String,
    pub eye_general_same_palace: Vec<TaiyiEyeGeneralCoincidence>,
    pub general_same_palace: Vec<TaiyiGeneralCoincidence>,
    pub main_guest_counts_equal: bool,
    pub candidate_pairs_exhaustively_enumerated: bool,
    pub received_pairing_resolved: bool,
    pub received_number_equality_resolved: bool,
    pub barrier_classified: bool,
    pub interpretation_eligible: bool,
    pub signal_eligible: bool,
    pub rule_eligible: bool,
}

impl TaiyiBarrierRawObservations {
                                                                        
                                                                   
    pub fn compact_zh(&self) -> String {
        let mut pairs = self
            .eye_general_same_palace
            .iter()
            .map(|pair| format!("{}—{}", pair.eye.zh_name(), pair.general.zh_name()))
            .collect::<Vec<_>>();
        pairs.extend(
            self.general_same_palace
                .iter()
                .map(|pair| format!("{}—{}", pair.first.zh_name(), pair.second.zh_name())),
        );
        let pair_text = if pairs.is_empty() {
            "无候选主体同宫".to_owned()
        } else {
            format!("候选主体同宫：{}", pairs.join("、"))
        };
        let count_text = if self.main_guest_counts_equal {
            "主客算值相等"
        } else {
            "主客算值不等"
        };
        format!("关法原始观察：{pair_text}；{count_text}（配对与‘数齐’释义未决，不判关）")
    }
}

                                                                            
   
                                                                              
                                                                             
                                                                          
                                                                          
                                                                             
                                                                       
                                                                          
                                                                            
                             
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TaiyiMechanicalRelations {
    pub algorithm_version: String,
    pub status: String,
                        
    pub shiji_covers_taiyi: bool,
                         
    pub tianmu_imprisoned_with_taiyi: bool,
    pub main_great_general_imprisoned_with_taiyi: bool,
    pub main_assistant_general_imprisoned_with_taiyi: bool,
    pub guest_great_general_imprisoned_with_taiyi: bool,
    pub guest_assistant_general_imprisoned_with_taiyi: bool,
                        
    pub shiji_forms_ge_against_taiyi: bool,
    pub guest_great_general_forms_ge_against_taiyi: bool,
    pub guest_assistant_general_forms_ge_against_taiyi: bool,
                                                                              
                                                                             
                                                          
    #[serde(default)]
    pub shiji_near_taiyi_geometry: Option<TaiyiNearRingGeometry>,
    #[serde(default)]
    pub tianmu_near_taiyi_geometry: Option<TaiyiNearRingGeometry>,
    #[serde(default)]
    pub near_ring_geometry_computed: bool,
                            
    #[serde(default)]
    pub tianmu_opposes_taiyi: bool,
                                                                            
                                                                          
    #[serde(default)]
    pub shiji_strike: Option<TaiyiStrike>,
    #[serde(default)]
    pub tianmu_pressure: Option<TaiyiPressure>,
    #[serde(default)]
    pub main_great_general_pressure: Option<TaiyiPressure>,
    #[serde(default)]
    pub main_assistant_general_pressure: Option<TaiyiPressure>,
    #[serde(default)]
    pub guest_great_general_pressure: Option<TaiyiPressure>,
    #[serde(default)]
    pub guest_assistant_general_pressure: Option<TaiyiPressure>,
                                                                            
                                                                           
    #[serde(default)]
    pub barrier_raw_observations: Option<TaiyiBarrierRawObservations>,
    pub strike_classified: bool,
    pub pressure_classified: bool,
    pub barrier_classified: bool,
    pub opposition_classified: bool,
    pub interpretation_eligible: bool,
    pub signal_eligible: bool,
    pub rule_eligible: bool,
}

                                                                             
                                                                           
                                                                             
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaiyiSeventyTwoBoardFrame {
    pub algorithm_version: String,
    pub status: String,
    pub supplied_board_count: u64,
    pub position_in_72: u8,
    pub dun: TaiyiDun,
    pub position_in_24: u8,
    pub taiyi_palace_step: u8,
    pub taiyi_palace: TaiyiPalace,
    pub position_within_taiyi_palace: u8,
    pub tianmu_position_in_18: u8,
    pub tianmu: TaiyiSixteenSpirit,
    pub tianmu_is_repeated_stop: bool,
    pub jishen_position_in_12: u8,
    pub jishen_branch: EarthlyBranch,
    pub shiji_ring_position: u8,
    pub shiji: TaiyiSixteenSpirit,
    pub main_count: u8,
    pub guest_count: u8,
    pub main_great_general_palace: TaiyiGeneralPalace,
    pub main_assistant_general_palace: TaiyiGeneralPalace,
    pub guest_great_general_palace: TaiyiGeneralPalace,
    pub guest_assistant_general_palace: TaiyiGeneralPalace,
    pub mechanical_relations: TaiyiMechanicalRelations,
    pub dun_derived_from_civil_time: bool,
    pub board_count_derived_from_civil_time: bool,
    pub complete_board_generated: bool,
    pub interpretation_eligible: bool,
    pub signal_eligible: bool,
    pub rule_eligible: bool,
}

impl TaiyiSeventyTwoBoardFrame {
    pub fn compact_zh(&self) -> String {
        format!(
            "{}第{}局：太乙{}，天目{}，计神{}，始击{}，主算{}，主大将{}，主参将{}，客算{}，客大将{}，客参将{}；{}（仅机械候选）",
            self.dun.zh_name(),
            self.position_in_72,
            self.taiyi_palace.compact_zh(),
            self.tianmu.zh_name(),
            self.jishen_branch.zh_name(),
            self.shiji.zh_name(),
            self.main_count,
            self.main_great_general_palace.compact_zh(),
            self.main_assistant_general_palace.compact_zh(),
            self.guest_count,
            self.guest_great_general_palace.compact_zh(),
            self.guest_assistant_general_palace.compact_zh(),
            self.mechanical_relations.compact_marks_zh(),
        )
    }
}

impl TaiyiMechanicalRelations {
                                                                           
                                                                        
    pub fn compact_marks_zh(&self) -> String {
        let mut marks: Vec<String> = Vec::new();
        if self.shiji_covers_taiyi {
            marks.push("始击掩太乙".to_owned());
        }
        if self.tianmu_imprisoned_with_taiyi {
            marks.push("天目囚".to_owned());
        }
        if self.main_great_general_imprisoned_with_taiyi {
            marks.push("主大将囚".to_owned());
        }
        if self.main_assistant_general_imprisoned_with_taiyi {
            marks.push("主参将囚".to_owned());
        }
        if self.guest_great_general_imprisoned_with_taiyi {
            marks.push("客大将囚".to_owned());
        }
        if self.guest_assistant_general_imprisoned_with_taiyi {
            marks.push("客参将囚".to_owned());
        }
        if self.shiji_forms_ge_against_taiyi {
            marks.push("始击格太乙".to_owned());
        }
        if self.guest_great_general_forms_ge_against_taiyi {
            marks.push("客大将格太乙".to_owned());
        }
        if self.guest_assistant_general_forms_ge_against_taiyi {
            marks.push("客参将格太乙".to_owned());
        }
        if self.tianmu_opposes_taiyi {
            marks.push("天目对太乙".to_owned());
        }
        if let Some(strike) = self.shiji_strike {
            marks.push(format!("始击{}", strike.compact_zh()));
        }
        for (actor, pressure) in [
            ("天目", self.tianmu_pressure),
            ("主大将", self.main_great_general_pressure),
            ("主参将", self.main_assistant_general_pressure),
            ("客大将", self.guest_great_general_pressure),
            ("客参将", self.guest_assistant_general_pressure),
        ] {
            if let Some(pressure) = pressure {
                marks.push(format!("{actor}{}", pressure.compact_zh()));
            }
        }
        if self.near_ring_geometry_computed && !self.strike_classified {
            if let Some(geometry) = self.shiji_near_taiyi_geometry {
                marks.push(
                    match geometry {
                        TaiyiNearRingGeometry::PlusOneInterstitial => "始击近太乙：环序+1辰",
                        TaiyiNearRingGeometry::MinusOneInterstitial => "始击近太乙：环序-1辰",
                        TaiyiNearRingGeometry::PlusTwoPrincipalPalace => "始击近太乙：环序+2宫",
                        TaiyiNearRingGeometry::MinusTwoPrincipalPalace => "始击近太乙：环序-2宫",
                    }
                    .to_owned(),
                );
            }
        }
        if self.near_ring_geometry_computed && !self.pressure_classified {
            if let Some(geometry) = self.tianmu_near_taiyi_geometry {
                marks.push(
                    match geometry {
                        TaiyiNearRingGeometry::PlusOneInterstitial => "天目近太乙：环序+1辰",
                        TaiyiNearRingGeometry::MinusOneInterstitial => "天目近太乙：环序-1辰",
                        TaiyiNearRingGeometry::PlusTwoPrincipalPalace => "天目近太乙：环序+2宫",
                        TaiyiNearRingGeometry::MinusTwoPrincipalPalace => "天目近太乙：环序-2宫",
                    }
                    .to_owned(),
                );
            }
        }
        if marks.is_empty() {
            "无掩、囚、格、对、击、迫位置标记".to_owned()
        } else {
            marks.join("、")
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaiyiAnnualFrame {
    pub algorithm_version: String,
    pub status: String,
    pub accumulated_year_count: u64,
    pub position_in_60: u8,
    pub tai_sui_designation: SexagenaryDesignation,
    pub tai_sui_branch: EarthlyBranch,
    pub taiyi_position: TaiyiAccumulatedYearPosition,
    pub tianmu_position_in_18: u8,
    pub tianmu: TaiyiSixteenSpirit,
    pub tianmu_is_repeated_stop: bool,
    pub jishen_position_in_12: u8,
    pub jishen_branch: EarthlyBranch,
    pub heshin_position_in_12: u8,
    pub heshin_branch: EarthlyBranch,
                                                                    
                                                                        
    pub shiji_ring_position: u8,
    pub shiji: TaiyiSixteenSpirit,
    pub main_count: u8,
    pub guest_count: u8,
    pub main_great_general_palace: TaiyiGeneralPalace,
    pub main_assistant_general_palace: TaiyiGeneralPalace,
    pub guest_great_general_palace: TaiyiGeneralPalace,
    pub guest_assistant_general_palace: TaiyiGeneralPalace,
                                                                           
    pub eight_gate_position_in_720: u16,
                                                            
    pub eight_gate_position_in_240: u8,
                                                                
    pub duty_gate_step: u8,
    pub annual_duty_gate: TaiyiGate,
    pub year_within_duty_gate: u8,
                                                                           
                                                                     
    #[serde(default)]
    pub mechanical_relations: TaiyiMechanicalRelations,
                                                                        
                                                                     
    #[serde(default)]
    pub annual_open_gate_boards: Option<TaiyiAnnualGateBoardsCandidate>,
                                                                         
                                                                             
                                                                   
    pub rotated_gate_boards_generated: bool,
    pub calendar_epoch_selected: bool,
    pub derived_from_civil_date: bool,
    pub yin_yang_dun_selected: bool,
    pub complete_board_generated: bool,
    pub interpretation_eligible: bool,
    pub signal_eligible: bool,
    pub rule_eligible: bool,
}

impl TaiyiAnnualFrame {
    pub fn compact_zh(&self) -> String {
        format!(
            "积年{}算：太岁{}，太乙{}第{}年，天目{}，计神{}，合神{}，始击{}，主算{}，主大将{}，主参将{}，客算{}，客大将{}，客参将{}，岁计直门{}第{}年；{}（仅机械候选）",
            self.accumulated_year_count,
            self.tai_sui_branch.zh_name(),
            self.taiyi_position.palace.compact_zh(),
            self.taiyi_position.year_within_palace,
            self.tianmu.zh_name(),
            self.jishen_branch.zh_name(),
            self.heshin_branch.zh_name(),
            self.shiji.zh_name(),
            self.main_count,
            self.main_great_general_palace.compact_zh(),
            self.main_assistant_general_palace.compact_zh(),
            self.guest_count,
            self.guest_great_general_palace.compact_zh(),
            self.guest_assistant_general_palace.compact_zh(),
            self.annual_duty_gate.zh_name(),
            self.year_within_duty_gate,
            self.mechanical_relations.compact_marks_zh()
        )
    }
}

impl TaiyiAccumulatedYearPosition {
    pub fn compact_zh(&self) -> String {
        format!(
            "积年{}算：太乙在{}，入宫第{}年（仅机械候选）",
            self.accumulated_year_count,
            self.palace.compact_zh(),
            self.year_within_palace
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TaiyiPositionError {
    #[error("太乙积年必须是从一开始的正整数，零没有可审核的‘算外’位置")]
    ZeroAccumulatedYear,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TaiyiBoardError {
    #[error("太乙七十二局序必须是从一开始的正整数，零没有可审核的位置")]
    ZeroBoardCount,
}

const fn one_based_cycle_position(count: u64, modulus: u64) -> u64 {
    (count - 1) % modulus + 1
}

                                                                             
                                                    
   
                                                                         
                                                                             
                                                      
pub fn taiyi_year_position_from_accumulated_count(
    accumulated_year_count: u64,
) -> Result<TaiyiAccumulatedYearPosition, TaiyiPositionError> {
    if accumulated_year_count == 0 {
        return Err(TaiyiPositionError::ZeroAccumulatedYear);
    }

    let position_in_360 = one_based_cycle_position(accumulated_year_count, 360) as u16;
    let position_in_72 = one_based_cycle_position(position_in_360.into(), 72) as u8;
    let position_in_24 = one_based_cycle_position(position_in_72.into(), 24) as u8;
    let palace_step = (position_in_24 - 1) / 3;
    let palace = TaiyiPalace::FORWARD_ORDER[usize::from(palace_step)];
    let year_within_palace = (position_in_24 - 1) % 3 + 1;

    Ok(TaiyiAccumulatedYearPosition {
        algorithm_version: TAIYI_ACCUMULATED_YEAR_POSITION_ALGORITHM_VERSION.to_owned(),
        status: TAIYI_ACCUMULATED_YEAR_POSITION_STATUS.to_owned(),
        accumulated_year_count,
        position_in_360,
        position_in_72,
        position_in_24,
        palace_step,
        palace,
        year_within_palace,
        calendar_epoch_selected: false,
        derived_from_civil_date: false,
        complete_board_generated: false,
        interpretation_eligible: false,
        signal_eligible: false,
        rule_eligible: false,
    })
}

fn reverse_branch_from(start: EarthlyBranch, one_based_position: u8) -> EarthlyBranch {
    let offset = (one_based_position - 1) % 12;
    EarthlyBranch::ALL[(start.index() + 12 - offset) as usize % 12]
}

const fn branch_ring_index(branch: EarthlyBranch) -> usize {
    match branch {
        EarthlyBranch::Zi => 0,
        EarthlyBranch::Chou => 1,
        EarthlyBranch::Yin => 3,
        EarthlyBranch::Mao => 4,
        EarthlyBranch::Chen => 5,
        EarthlyBranch::Si => 7,
        EarthlyBranch::Wu => 8,
        EarthlyBranch::Wei => 9,
        EarthlyBranch::Shen => 11,
        EarthlyBranch::You => 12,
        EarthlyBranch::Xu => 13,
        EarthlyBranch::Hai => 15,
    }
}

const fn principal_palace_value_at_ring_index(index: usize) -> Option<u8> {
    match index {
        0 => Some(8),
        2 => Some(3),
        4 => Some(4),
        6 => Some(9),
        8 => Some(2),
        10 => Some(7),
        12 => Some(6),
        14 => Some(1),
        _ => None,
    }
}

                                                                           
                                                                          
                                                                             
                                                                      
fn eye_count_to_taiyi(eye: TaiyiSixteenSpirit, taiyi: TaiyiPalace) -> u8 {
    let start = eye.ring_index();
    let stop = taiyi.ring_index();
    if start == stop {
        return principal_palace_value_at_ring_index(stop)
            .expect("every Taiyi palace occupies a principal ring position");
    }

    let mut total = principal_palace_value_at_ring_index(start).unwrap_or(1);
    let mut index = (start + 1) % TaiyiSixteenSpirit::RECEIVED_RING_ORDER.len();
    while index != stop {
        if let Some(value) = principal_palace_value_at_ring_index(index) {
            total += value;
        }
        index = (index + 1) % TaiyiSixteenSpirit::RECEIVED_RING_ORDER.len();
    }
    total
}

                                                                           
                                                                           
                                                                               
                                                                      
fn general_palace_from_count(count: u8) -> TaiyiGeneralPalace {
    debug_assert!((1..=40).contains(&count));
    let units = count % 10;
    let reduced = if units == 0 { count / 10 } else { units };
    match reduced {
        1 => TaiyiGeneralPalace::One,
        2 => TaiyiGeneralPalace::Two,
        3 => TaiyiGeneralPalace::Three,
        4 => TaiyiGeneralPalace::Four,
        5 => TaiyiGeneralPalace::Five,
        6 => TaiyiGeneralPalace::Six,
        7 => TaiyiGeneralPalace::Seven,
        8 => TaiyiGeneralPalace::Eight,
        9 => TaiyiGeneralPalace::Nine,
        _ => unreachable!("Taiyi general count is bounded to 1..=40"),
    }
}

fn assistant_general_palace(great_general: TaiyiGeneralPalace) -> TaiyiGeneralPalace {
    general_palace_from_count(great_general.number() * 3)
}

const fn spirit_coincides_with_taiyi(spirit: TaiyiSixteenSpirit, taiyi: TaiyiPalace) -> bool {
    spirit.ring_index() == taiyi.ring_index()
}

const fn spirit_opposes_taiyi(spirit: TaiyiSixteenSpirit, taiyi: TaiyiPalace) -> bool {
    spirit.ring_index() == taiyi.opposite().ring_index()
}

const fn spirit_near_taiyi_geometry(
    spirit: TaiyiSixteenSpirit,
    taiyi: TaiyiPalace,
) -> Option<TaiyiNearRingGeometry> {
    let delta = (spirit.ring_index() + 16 - taiyi.ring_index()) % 16;
    match delta {
        1 => Some(TaiyiNearRingGeometry::PlusOneInterstitial),
        15 => Some(TaiyiNearRingGeometry::MinusOneInterstitial),
        2 => Some(TaiyiNearRingGeometry::PlusTwoPrincipalPalace),
        14 => Some(TaiyiNearRingGeometry::MinusTwoPrincipalPalace),
        _ => None,
    }
}

                                                                            
                                                                       
                                                             
const fn strike_from_geometry(geometry: TaiyiNearRingGeometry) -> TaiyiStrike {
    match geometry {
        TaiyiNearRingGeometry::PlusOneInterstitial => TaiyiStrike::FrontChen,
        TaiyiNearRingGeometry::MinusOneInterstitial => TaiyiStrike::RearChen,
        TaiyiNearRingGeometry::PlusTwoPrincipalPalace => TaiyiStrike::OuterPalace,
        TaiyiNearRingGeometry::MinusTwoPrincipalPalace => TaiyiStrike::InnerPalace,
    }
}

const fn pressure_from_geometry(geometry: TaiyiNearRingGeometry) -> TaiyiPressure {
    match geometry {
        TaiyiNearRingGeometry::PlusOneInterstitial => TaiyiPressure::OuterChen,
        TaiyiNearRingGeometry::MinusOneInterstitial => TaiyiPressure::InnerChen,
        TaiyiNearRingGeometry::PlusTwoPrincipalPalace => TaiyiPressure::OuterPalace,
        TaiyiNearRingGeometry::MinusTwoPrincipalPalace => TaiyiPressure::InnerPalace,
    }
}

const fn general_near_taiyi_geometry(
    general: TaiyiGeneralPalace,
    taiyi: TaiyiPalace,
) -> Option<TaiyiNearRingGeometry> {
    let palace = match general.noncentral_taiyi_palace() {
        Some(palace) => palace,
        None => return None,
    };
    let delta = (palace.ring_index() + 16 - taiyi.ring_index()) % 16;
    match delta {
        2 => Some(TaiyiNearRingGeometry::PlusTwoPrincipalPalace),
        14 => Some(TaiyiNearRingGeometry::MinusTwoPrincipalPalace),
        _ => None,
    }
}

const fn general_pressure(
    general: TaiyiGeneralPalace,
    taiyi: TaiyiPalace,
) -> Option<TaiyiPressure> {
    match general_near_taiyi_geometry(general, taiyi) {
        Some(geometry) => Some(pressure_from_geometry(geometry)),
        None => None,
    }
}

const fn general_coincides_with_taiyi(general: TaiyiGeneralPalace, taiyi: TaiyiPalace) -> bool {
    general.number() == taiyi.number()
}

const fn general_opposes_taiyi(general: TaiyiGeneralPalace, taiyi: TaiyiPalace) -> bool {
    general.number() == taiyi.opposite().number()
}

const fn spirit_coincides_with_general(
    spirit: TaiyiSixteenSpirit,
    general: TaiyiGeneralPalace,
) -> bool {
    match principal_palace_value_at_ring_index(spirit.ring_index()) {
        Some(palace_number) => palace_number == general.number(),
        None => false,
    }
}

#[derive(Debug, Clone, Copy)]
struct TaiyiRelationInputs {
    taiyi: TaiyiPalace,
    tianmu: TaiyiSixteenSpirit,
    shiji: TaiyiSixteenSpirit,
    main_count: u8,
    guest_count: u8,
    main_great: TaiyiGeneralPalace,
    main_assistant: TaiyiGeneralPalace,
    guest_great: TaiyiGeneralPalace,
    guest_assistant: TaiyiGeneralPalace,
}

fn barrier_raw_observations(inputs: &TaiyiRelationInputs) -> TaiyiBarrierRawObservations {
    let generals = [
        (TaiyiGeneralRole::MainGreat, inputs.main_great),
        (TaiyiGeneralRole::MainAssistant, inputs.main_assistant),
        (TaiyiGeneralRole::GuestGreat, inputs.guest_great),
        (TaiyiGeneralRole::GuestAssistant, inputs.guest_assistant),
    ];
    let mut eye_general_same_palace = Vec::new();
    for (eye_role, eye) in [
        (TaiyiEyeRole::Tianmu, inputs.tianmu),
        (TaiyiEyeRole::Shiji, inputs.shiji),
    ] {
        for (general_role, general) in generals {
            if spirit_coincides_with_general(eye, general) {
                eye_general_same_palace.push(TaiyiEyeGeneralCoincidence {
                    eye: eye_role,
                    general: general_role,
                });
            }
        }
    }

    let mut general_same_palace = Vec::new();
    for first_index in 0..generals.len() {
        for second_index in (first_index + 1)..generals.len() {
            let (first_role, first) = generals[first_index];
            let (second_role, second) = generals[second_index];
            if first == second {
                general_same_palace.push(TaiyiGeneralCoincidence {
                    first: first_role,
                    second: second_role,
                });
            }
        }
    }

    TaiyiBarrierRawObservations {
        algorithm_version: TAIYI_BARRIER_RAW_OBSERVATIONS_ALGORITHM_VERSION.to_owned(),
        status: TAIYI_BARRIER_RAW_OBSERVATIONS_STATUS.to_owned(),
        eye_general_same_palace,
        general_same_palace,
        main_guest_counts_equal: inputs.main_count == inputs.guest_count,
        candidate_pairs_exhaustively_enumerated: true,
        received_pairing_resolved: false,
        received_number_equality_resolved: false,
        barrier_classified: false,
        interpretation_eligible: false,
        signal_eligible: false,
        rule_eligible: false,
    }
}

fn mechanical_relations(inputs: TaiyiRelationInputs) -> TaiyiMechanicalRelations {
    let shiji_near_taiyi_geometry = spirit_near_taiyi_geometry(inputs.shiji, inputs.taiyi);
    let tianmu_near_taiyi_geometry = spirit_near_taiyi_geometry(inputs.tianmu, inputs.taiyi);
    TaiyiMechanicalRelations {
        algorithm_version: TAIYI_MECHANICAL_RELATIONS_ALGORITHM_VERSION.to_owned(),
        status: TAIYI_MECHANICAL_RELATIONS_STATUS.to_owned(),
        shiji_covers_taiyi: spirit_coincides_with_taiyi(inputs.shiji, inputs.taiyi),
        tianmu_imprisoned_with_taiyi: spirit_coincides_with_taiyi(inputs.tianmu, inputs.taiyi),
        main_great_general_imprisoned_with_taiyi: general_coincides_with_taiyi(
            inputs.main_great,
            inputs.taiyi,
        ),
        main_assistant_general_imprisoned_with_taiyi: general_coincides_with_taiyi(
            inputs.main_assistant,
            inputs.taiyi,
        ),
        guest_great_general_imprisoned_with_taiyi: general_coincides_with_taiyi(
            inputs.guest_great,
            inputs.taiyi,
        ),
        guest_assistant_general_imprisoned_with_taiyi: general_coincides_with_taiyi(
            inputs.guest_assistant,
            inputs.taiyi,
        ),
        shiji_forms_ge_against_taiyi: spirit_opposes_taiyi(inputs.shiji, inputs.taiyi),
        guest_great_general_forms_ge_against_taiyi: general_opposes_taiyi(
            inputs.guest_great,
            inputs.taiyi,
        ),
        guest_assistant_general_forms_ge_against_taiyi: general_opposes_taiyi(
            inputs.guest_assistant,
            inputs.taiyi,
        ),
        shiji_near_taiyi_geometry,
        tianmu_near_taiyi_geometry,
        near_ring_geometry_computed: true,
        tianmu_opposes_taiyi: spirit_opposes_taiyi(inputs.tianmu, inputs.taiyi),
        shiji_strike: shiji_near_taiyi_geometry.map(strike_from_geometry),
        tianmu_pressure: tianmu_near_taiyi_geometry.map(pressure_from_geometry),
        main_great_general_pressure: general_pressure(inputs.main_great, inputs.taiyi),
        main_assistant_general_pressure: general_pressure(inputs.main_assistant, inputs.taiyi),
        guest_great_general_pressure: general_pressure(inputs.guest_great, inputs.taiyi),
        guest_assistant_general_pressure: general_pressure(inputs.guest_assistant, inputs.taiyi),
        barrier_raw_observations: Some(barrier_raw_observations(&inputs)),
        strike_classified: true,
        pressure_classified: true,
        barrier_classified: false,
        opposition_classified: true,
        interpretation_eligible: false,
        signal_eligible: false,
        rule_eligible: false,
    }
}

fn annual_open_gate_board(
    anchor: TaiyiAnnualGateBoardAnchor,
    anchor_palace: TaiyiPalace,
) -> TaiyiAnnualOpenGateBoard {
    let anchor_index = anchor_palace.gate_ring_index();
    let placements = std::array::from_fn(|gate_index| {
        let gate = TaiyiGate::RECEIVED_LEFTWARD_ORDER[gate_index];
        let palace_index = (anchor_index + gate_index) % 8;
        let palace = TaiyiGate::RECEIVED_LEFTWARD_ORDER[palace_index].fixed_taiyi_palace();
        TaiyiGatePlacement { gate, palace }
    });

    TaiyiAnnualOpenGateBoard {
        anchor,
        anchor_palace,
        anchor_gate: TaiyiGate::Open,
        placements,
        open_gate_is_at_anchor: true,
        interpretation_eligible: false,
        signal_eligible: false,
        rule_eligible: false,
    }
}

fn annual_gate_boards_candidate(
    taiyi_palace: TaiyiPalace,
    main_great_general_palace: TaiyiGeneralPalace,
    guest_great_general_palace: TaiyiGeneralPalace,
) -> TaiyiAnnualGateBoardsCandidate {
    let main_palace = main_great_general_palace.noncentral_taiyi_palace();
    let guest_palace = guest_great_general_palace.noncentral_taiyi_palace();
    let main_great_general_board = main_palace
        .map(|palace| annual_open_gate_board(TaiyiAnnualGateBoardAnchor::MainGreatGeneral, palace));
    let guest_great_general_board = guest_palace.map(|palace| {
        annual_open_gate_board(TaiyiAnnualGateBoardAnchor::GuestGreatGeneral, palace)
    });

    TaiyiAnnualGateBoardsCandidate {
        algorithm_version: TAIYI_ANNUAL_GATE_BOARDS_ALGORITHM_VERSION.to_owned(),
        status: TAIYI_ANNUAL_GATE_BOARDS_STATUS.to_owned(),
        anchor_gate: TaiyiGate::Open,
        taiyi_board: annual_open_gate_board(TaiyiAnnualGateBoardAnchor::Taiyi, taiyi_palace),
        main_great_general_board,
        main_great_general_unavailable_reason: main_palace.is_none().then(|| {
            "主大将落中五宫；八门只有八方位置，未取得中五起门依据，故本子盘停用".to_owned()
        }),
        guest_great_general_board,
        guest_great_general_unavailable_reason: guest_palace.is_none().then(|| {
            "客大将落中五宫；八门只有八方位置，未取得中五起门依据，故本子盘停用".to_owned()
        }),
        all_three_boards_generated: main_palace.is_some() && guest_palace.is_some(),
        fixed_count_general_board_generated: false,
        time_method_duty_gate_rotation_generated: false,
        three_auspicious_gates_classified: false,
        interpretation_eligible: false,
        signal_eligible: false,
        rule_eligible: false,
    }
}

                                                                            
   
                                                                              
                                                                             
                                                                              
                                                                              
                           
pub fn taiyi_seventy_two_board_frame_from_count(
    supplied_board_count: u64,
    dun: TaiyiDun,
) -> Result<TaiyiSeventyTwoBoardFrame, TaiyiBoardError> {
    if supplied_board_count == 0 {
        return Err(TaiyiBoardError::ZeroBoardCount);
    }

    let position_in_72 = one_based_cycle_position(supplied_board_count, 72) as u8;
    let position_in_24 = one_based_cycle_position(u64::from(position_in_72), 24) as u8;
    let taiyi_palace_step = (position_in_24 - 1) / 3;
    let taiyi_palace = dun.taiyi_order()[usize::from(taiyi_palace_step)];
    let position_within_taiyi_palace = (position_in_24 - 1) % 3 + 1;
    let tianmu_position_in_18 = one_based_cycle_position(u64::from(position_in_72), 18) as u8;
    let tianmu_cycle = match dun {
        TaiyiDun::Yang => TaiyiSixteenSpirit::TIANMU_CYCLE_WITH_STOPS,
        TaiyiDun::Yin => TaiyiSixteenSpirit::YIN_TIANMU_CYCLE_WITH_STOPS,
    };
    let tianmu = tianmu_cycle[usize::from(tianmu_position_in_18 - 1)];
    let jishen_position_in_12 = one_based_cycle_position(u64::from(position_in_72), 12) as u8;
    let jishen_branch = reverse_branch_from(dun.jishen_start(), jishen_position_in_12);
    let shiji_index = (TaiyiSixteenSpirit::HeDe.ring_index()
        + tianmu.ring_index()
        + TaiyiSixteenSpirit::RECEIVED_RING_ORDER.len()
        - branch_ring_index(jishen_branch))
        % TaiyiSixteenSpirit::RECEIVED_RING_ORDER.len();
    let shiji = TaiyiSixteenSpirit::RECEIVED_RING_ORDER[shiji_index];
    let main_count = eye_count_to_taiyi(tianmu, taiyi_palace);
    let guest_count = eye_count_to_taiyi(shiji, taiyi_palace);
    let main_great_general_palace = general_palace_from_count(main_count);
    let main_assistant_general_palace = assistant_general_palace(main_great_general_palace);
    let guest_great_general_palace = general_palace_from_count(guest_count);
    let guest_assistant_general_palace = assistant_general_palace(guest_great_general_palace);
    let mechanical_relations = mechanical_relations(TaiyiRelationInputs {
        taiyi: taiyi_palace,
        tianmu,
        shiji,
        main_count,
        guest_count,
        main_great: main_great_general_palace,
        main_assistant: main_assistant_general_palace,
        guest_great: guest_great_general_palace,
        guest_assistant: guest_assistant_general_palace,
    });
    Ok(TaiyiSeventyTwoBoardFrame {
        algorithm_version: TAIYI_SEVENTY_TWO_BOARD_ALGORITHM_VERSION.to_owned(),
        status: TAIYI_SEVENTY_TWO_BOARD_STATUS.to_owned(),
        supplied_board_count,
        position_in_72,
        dun,
        position_in_24,
        taiyi_palace_step,
        taiyi_palace,
        position_within_taiyi_palace,
        tianmu_position_in_18,
        tianmu,
        tianmu_is_repeated_stop: matches!(tianmu_position_in_18, 5 | 18),
        jishen_position_in_12,
        jishen_branch,
        shiji_ring_position: (shiji_index + 1) as u8,
        shiji,
        main_count,
        guest_count,
        main_great_general_palace,
        main_assistant_general_palace,
        guest_great_general_palace,
        guest_assistant_general_palace,
        mechanical_relations,
        dun_derived_from_civil_time: false,
        board_count_derived_from_civil_time: false,
        complete_board_generated: false,
        interpretation_eligible: false,
        signal_eligible: false,
        rule_eligible: false,
    })
}

                                                                             
   
                                                                            
                                                                              
                                                                       
                                                                           
                                                                      
                                                                          
                                                                            
                                           
                                                                       
pub fn taiyi_annual_frame_from_accumulated_count(
    accumulated_year_count: u64,
) -> Result<TaiyiAnnualFrame, TaiyiPositionError> {
    let taiyi_position = taiyi_year_position_from_accumulated_count(accumulated_year_count)?;
    let position_in_60 = one_based_cycle_position(accumulated_year_count, 60) as u8;
    let tai_sui_designation =
        SexagenaryDesignation::from_cycle_index(i64::from(position_in_60 - 1));
    let tianmu_position_in_18 = one_based_cycle_position(accumulated_year_count, 18) as u8;
    let tianmu =
        TaiyiSixteenSpirit::TIANMU_CYCLE_WITH_STOPS[usize::from(tianmu_position_in_18 - 1)];
    let jishen_position_in_12 = one_based_cycle_position(accumulated_year_count, 12) as u8;
    let heshin_position_in_12 = jishen_position_in_12;
    let jishen_branch = reverse_branch_from(EarthlyBranch::Yin, jishen_position_in_12);
                                                                      
                                                                       
    let shiji_index = (TaiyiSixteenSpirit::HeDe.ring_index()
        + tianmu.ring_index()
        + TaiyiSixteenSpirit::RECEIVED_RING_ORDER.len()
        - branch_ring_index(jishen_branch))
        % TaiyiSixteenSpirit::RECEIVED_RING_ORDER.len();
    let shiji = TaiyiSixteenSpirit::RECEIVED_RING_ORDER[shiji_index];
    let main_count = eye_count_to_taiyi(tianmu, taiyi_position.palace);
    let guest_count = eye_count_to_taiyi(shiji, taiyi_position.palace);
    let main_great_general_palace = general_palace_from_count(main_count);
    let guest_great_general_palace = general_palace_from_count(guest_count);
    let eight_gate_position_in_720 = one_based_cycle_position(accumulated_year_count, 720) as u16;
    let eight_gate_position_in_240 =
        one_based_cycle_position(u64::from(eight_gate_position_in_720), 240) as u8;
    let duty_gate_step = (eight_gate_position_in_240 - 1) / 30;
    let annual_duty_gate = TaiyiGate::RECEIVED_LEFTWARD_ORDER[usize::from(duty_gate_step)];
    let year_within_duty_gate = (eight_gate_position_in_240 - 1) % 30 + 1;
    let main_assistant_general_palace = assistant_general_palace(main_great_general_palace);
    let guest_assistant_general_palace = assistant_general_palace(guest_great_general_palace);
    let mechanical_relations = mechanical_relations(TaiyiRelationInputs {
        taiyi: taiyi_position.palace,
        tianmu,
        shiji,
        main_count,
        guest_count,
        main_great: main_great_general_palace,
        main_assistant: main_assistant_general_palace,
        guest_great: guest_great_general_palace,
        guest_assistant: guest_assistant_general_palace,
    });
    let annual_open_gate_boards = annual_gate_boards_candidate(
        taiyi_position.palace,
        main_great_general_palace,
        guest_great_general_palace,
    );
    let rotated_gate_boards_generated = annual_open_gate_boards.all_three_boards_generated;

    Ok(TaiyiAnnualFrame {
        algorithm_version: TAIYI_ANNUAL_FRAME_ALGORITHM_VERSION.to_owned(),
        status: TAIYI_ANNUAL_FRAME_STATUS.to_owned(),
        accumulated_year_count,
        position_in_60,
        tai_sui_designation,
        tai_sui_branch: tai_sui_designation.branch,
        taiyi_position,
        tianmu_position_in_18,
        tianmu,
        tianmu_is_repeated_stop: matches!(tianmu_position_in_18, 5 | 18),
        jishen_position_in_12,
        jishen_branch,
        heshin_position_in_12,
        heshin_branch: reverse_branch_from(EarthlyBranch::Chou, heshin_position_in_12),
        shiji_ring_position: (shiji_index + 1) as u8,
        shiji,
        main_count,
        guest_count,
        main_great_general_palace,
        main_assistant_general_palace,
        guest_great_general_palace,
        guest_assistant_general_palace,
        eight_gate_position_in_720,
        eight_gate_position_in_240,
        duty_gate_step,
        annual_duty_gate,
        year_within_duty_gate,
        mechanical_relations,
        annual_open_gate_boards: Some(annual_open_gate_boards),
        rotated_gate_boards_generated,
        calendar_epoch_selected: false,
        derived_from_civil_date: false,
        yin_yang_dun_selected: false,
        complete_board_generated: false,
        interpretation_eligible: false,
        signal_eligible: false,
        rule_eligible: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_instead_of_inventing_a_cycle_boundary() {
        assert_eq!(
            taiyi_year_position_from_accumulated_count(0),
            Err(TaiyiPositionError::ZeroAccumulatedYear)
        );
    }

    #[test]
    fn matches_the_received_yang_board_table_anchors() {
        let anchors = [
            (1, TaiyiPalace::One, 1),
            (3, TaiyiPalace::One, 3),
            (4, TaiyiPalace::Two, 1),
            (12, TaiyiPalace::Four, 3),
            (13, TaiyiPalace::Six, 1),
            (24, TaiyiPalace::Nine, 3),
            (25, TaiyiPalace::One, 1),
            (48, TaiyiPalace::Nine, 3),
            (49, TaiyiPalace::One, 1),
            (72, TaiyiPalace::Nine, 3),
        ];

        for (count, palace, year_within_palace) in anchors {
            let position = taiyi_year_position_from_accumulated_count(count).unwrap();
            assert_eq!(position.palace, palace, "count {count}");
            assert_eq!(
                position.year_within_palace, year_within_palace,
                "count {count}"
            );
        }
    }

    #[test]
    fn preserves_one_based_boundaries_for_all_received_cycles() {
        let position = taiyi_year_position_from_accumulated_count(360).unwrap();
        assert_eq!(position.position_in_360, 360);
        assert_eq!(position.position_in_72, 72);
        assert_eq!(position.position_in_24, 24);
        assert_eq!(position.palace, TaiyiPalace::Nine);
        assert_eq!(position.year_within_palace, 3);

        let reset = taiyi_year_position_from_accumulated_count(361).unwrap();
        assert_eq!(reset.position_in_360, 1);
        assert_eq!(reset.position_in_72, 1);
        assert_eq!(reset.position_in_24, 1);
        assert_eq!(reset.palace, TaiyiPalace::One);
        assert_eq!(reset.year_within_palace, 1);
    }

    #[test]
    fn exhaustively_keeps_three_years_per_noncentral_palace() {
        for count in 1..=720 {
            let position = taiyi_year_position_from_accumulated_count(count).unwrap();
            let expected_small_cycle = ((count - 1) % 24 + 1) as u8;
            let expected_step = (expected_small_cycle - 1) / 3;
            assert_eq!(position.position_in_24, expected_small_cycle);
            assert_eq!(position.palace_step, expected_step);
            assert_eq!(
                position.palace,
                TaiyiPalace::FORWARD_ORDER[usize::from(expected_step)]
            );
            assert_eq!(
                position.year_within_palace,
                (expected_small_cycle - 1) % 3 + 1
            );
            assert_ne!(position.palace.number(), 5);
            assert!(!position.calendar_epoch_selected);
            assert!(!position.derived_from_civil_date);
            assert!(!position.complete_board_generated);
            assert!(!position.interpretation_eligible);
            assert!(!position.signal_eligible);
            assert!(!position.rule_eligible);
        }
    }

    #[test]
    fn large_counts_do_not_overflow_before_reduction() {
        let position = taiyi_year_position_from_accumulated_count(u64::MAX).unwrap();
        assert_eq!(position.position_in_360, 15);
        assert_eq!(position.position_in_72, 15);
        assert_eq!(position.position_in_24, 15);
        assert_eq!(position.palace, TaiyiPalace::Six);
        assert_eq!(position.year_within_palace, 3);
    }

    #[test]
    fn annual_frame_matches_the_first_twenty_four_received_table_rows() {
        let expected_tianmu = [
            TaiyiSixteenSpirit::WuDe,
            TaiyiSixteenSpirit::TaiCu,
            TaiyiSixteenSpirit::YinZhu,
            TaiyiSixteenSpirit::YinDe,
            TaiyiSixteenSpirit::YinDe,
            TaiyiSixteenSpirit::DaYi,
            TaiyiSixteenSpirit::DiZhu,
            TaiyiSixteenSpirit::YangDe,
            TaiyiSixteenSpirit::HeDe,
            TaiyiSixteenSpirit::LuShen,
            TaiyiSixteenSpirit::GaoCong,
            TaiyiSixteenSpirit::TaiYang,
            TaiyiSixteenSpirit::TaiJiong,
            TaiyiSixteenSpirit::TaiShen,
            TaiyiSixteenSpirit::DaWei,
            TaiyiSixteenSpirit::TianDao,
            TaiyiSixteenSpirit::DaWu,
            TaiyiSixteenSpirit::DaWu,
            TaiyiSixteenSpirit::WuDe,
            TaiyiSixteenSpirit::TaiCu,
            TaiyiSixteenSpirit::YinZhu,
            TaiyiSixteenSpirit::YinDe,
            TaiyiSixteenSpirit::YinDe,
            TaiyiSixteenSpirit::DaYi,
        ];
        let expected_jishen = [
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
            EarthlyBranch::Chen,
            EarthlyBranch::Mao,
        ];

        for count in 1..=24 {
            let frame = taiyi_annual_frame_from_accumulated_count(count).unwrap();
            assert_eq!(frame.tianmu, expected_tianmu[(count - 1) as usize]);
            assert_eq!(
                frame.jishen_branch,
                expected_jishen[((count - 1) % 12) as usize]
            );
        }
    }

    #[test]
    fn annual_frame_shiji_and_counts_match_all_seventy_two_yang_table_rows() {
                                                                          
                                                                           
        let expected_shiji = [
            "大武", "阴主", "大义", "阳德", "吕申", "太阳", "太神", "大武", "太蔟", "阴德", "阳德",
            "吕申", "太阳", "大威", "大武", "太蔟", "大义", "地主", "和德", "太阳", "太神", "天道",
            "武德", "阴主", "大义", "和德", "高丛", "太炅", "天道", "武德", "阴主", "地主", "和德",
            "高丛", "太神", "大威", "大武", "阴主", "大义", "阳德", "吕申", "太阳", "太神", "大武",
            "太蔟", "阴德", "阳德", "吕申", "太阳", "大威", "大武", "太蔟", "大义", "地主", "和德",
            "太阳", "太神", "天道", "武德", "阴主", "大义", "和德", "高丛", "太炅", "天道", "武德",
            "阴主", "地主", "和德", "高丛", "太神", "大威",
        ];
        let expected_main_counts = [
            7, 6, 1, 25, 25, 25, 8, 1, 3, 1, 4, 37, 18, 10, 9, 1, 7, 7, 8, 7, 2, 16, 16, 16, 39,
            32, 31, 14, 13, 10, 33, 25, 24, 26, 25, 25, 1, 6, 35, 27, 27, 27, 8, 33, 32, 5, 4, 1,
            24, 16, 15, 39, 38, 38, 16, 15, 10, 12, 12, 12, 33, 26, 25, 16, 15, 12, 25, 17, 16, 30,
            29, 29,
        ];
        let expected_guest_counts = [
            13, 1, 40, 17, 14, 10, 25, 22, 15, 12, 4, 1, 19, 9, 7, 33, 27, 26, 32, 26, 17, 30, 23,
            17, 40, 31, 28, 9, 39, 32, 10, 8, 3, 4, 28, 27, 7, 35, 34, 19, 16, 12, 17, 14, 7, 16,
            8, 5, 25, 15, 13, 31, 25, 24, 3, 34, 25, 26, 19, 13, 34, 25, 22, 11, 1, 34, 2, 8, 32,
            4, 32, 31,
        ];

        let mut count_mismatches = Vec::new();
        for count in 1..=72 {
            let frame = taiyi_annual_frame_from_accumulated_count(count).unwrap();
            let index = (count - 1) as usize;
            assert_eq!(
                frame.shiji.zh_name(),
                expected_shiji[index],
                "count {count}"
            );
            if frame.main_count != expected_main_counts[index]
                || frame.guest_count != expected_guest_counts[index]
            {
                count_mismatches.push((
                    count,
                    frame.main_count,
                    expected_main_counts[index],
                    frame.guest_count,
                    expected_guest_counts[index],
                ));
            }
        }
        assert!(
            count_mismatches.is_empty(),
            "Yang-board count mismatches (count, calculated main, table main, calculated guest, table guest): {count_mismatches:?}"
        );
    }

    #[test]
    fn eye_count_keeps_the_palace_value_when_an_eye_coincides_with_taiyi() {
        let anchors = [(9, 3, 15), (11, 4, 4), (17, 7, 27), (33, 24, 3)];
        for (count, main_count, guest_count) in anchors {
            let frame = taiyi_annual_frame_from_accumulated_count(count).unwrap();
            assert_eq!(frame.main_count, main_count, "count {count}");
            assert_eq!(frame.guest_count, guest_count, "count {count}");
        }
    }

    #[test]
    fn general_palace_reduction_matches_received_formula_and_examples() {
        let expected = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 2, 1, 2, 3, 4, 5, 6, 7, 8, 9,
            3, 1, 2, 3, 4, 5, 6, 7, 8, 9, 4,
        ];
        for count in 1..=40_u8 {
            assert_eq!(
                general_palace_from_count(count).number(),
                expected[usize::from(count - 1)],
                "count {count}"
            );
        }

                                                         
        assert_eq!(general_palace_from_count(10), TaiyiGeneralPalace::One);
        assert_eq!(general_palace_from_count(24), TaiyiGeneralPalace::Four);
    }

    #[test]
    fn explicit_yang_board_matches_the_existing_annual_mechanics() {
        for count in 1..=720 {
            let board = taiyi_seventy_two_board_frame_from_count(count, TaiyiDun::Yang).unwrap();
            let annual = taiyi_annual_frame_from_accumulated_count(count).unwrap();
            assert_eq!(board.position_in_72, annual.taiyi_position.position_in_72);
            assert_eq!(board.taiyi_palace, annual.taiyi_position.palace);
            assert_eq!(board.tianmu, annual.tianmu);
            assert_eq!(board.jishen_branch, annual.jishen_branch);
            assert_eq!(board.shiji, annual.shiji);
            assert_eq!(board.main_count, annual.main_count);
            assert_eq!(board.guest_count, annual.guest_count);
            assert_eq!(
                board.main_great_general_palace,
                annual.main_great_general_palace
            );
            assert_eq!(
                board.main_assistant_general_palace,
                annual.main_assistant_general_palace
            );
            assert_eq!(
                board.guest_great_general_palace,
                annual.guest_great_general_palace
            );
            assert_eq!(
                board.guest_assistant_general_palace,
                annual.guest_assistant_general_palace
            );
            assert_eq!(board.mechanical_relations, annual.mechanical_relations);
        }
    }

    #[test]
    fn explicit_yin_board_matches_the_received_table_with_documented_variants() {
        let expected_main_counts = [
            5, 4, 1, 25, 25, 17, 2, 1, 7, 1, 6, 35, 12, 12, 11, 1, 3, 3, 14, 13, 10, 24, 24, 16,
            21, 31, 39, 8, 7, 2, 27, 27, 26, 26, 25, 25, 1, 4, 37, 33, 33, 25, 2, 39, 38, 7, 6, 1,
            16, 16, 15, 33, 32, 32, 16, 15, 12, 18, 18, 10, 27, 26, 25, 16, 15, 10, 25, 25, 24, 22,
            31, 31,
        ];
        let expected_guest_counts = [
            29, 17, 16, 33, 30, 26, 3, 7, 33, 34, 36, 23, 37, 27, 29, 15, 9, 8, 16, 10, 1, 14, 7,
            1, 16, 7, 4, 25, 15, 8, 28, 26, 18, 22, 10, 9, 25, 13, 12, 1, 38, 34, 38, 31, 31, 2,
            32, 29, 1, 31, 29, 7, 1, 8, 18, 12, 3, 8, 1, 35, 12, 3, 4, 33, 23, 16, 26, 24, 16, 28,
            16, 15,
        ];
        let expected_shiji = [
            "大武", "阴主", "大义", "阳德", "吕申", "太阳", "太神", "大武", "太蔟", "阴德", "阳德",
            "吕申", "太阳", "大威", "大武", "太蔟", "大义", "地主", "和德", "太阳", "太神", "天道",
            "武德", "阴主", "大义", "和德", "高丛", "太炅", "天道", "武德", "阴主", "地主", "和德",
            "高丛", "太神", "大威", "大威", "阴主", "大义", "阳德", "吕申", "太阳", "大武", "太蔟",
            "太蔟", "阴德", "阳德", "吕申", "太阳", "大威", "大武", "太蔟", "大义", "地主", "和德",
            "太阳", "太神", "天道", "武德", "阴主", "大义", "和德", "高丛", "太炅", "天道", "武德",
            "阴主", "地主", "和德", "高丛", "太神", "大威",
        ];
        let expected_jishen = [
            EarthlyBranch::Shen,
            EarthlyBranch::Wei,
            EarthlyBranch::Wu,
            EarthlyBranch::Si,
            EarthlyBranch::Chen,
            EarthlyBranch::Mao,
            EarthlyBranch::Yin,
            EarthlyBranch::Chou,
            EarthlyBranch::Zi,
            EarthlyBranch::Hai,
            EarthlyBranch::Xu,
            EarthlyBranch::You,
        ];

        let mut shiji_variants = Vec::new();
        let mut count_variants = Vec::new();
        for count in 1..=72 {
            let frame = taiyi_seventy_two_board_frame_from_count(count, TaiyiDun::Yin).unwrap();
            let index = (count - 1) as usize;
            let expected_palace = TaiyiDun::Yin.taiyi_order()[index / 3 % 8];
            let expected_tianmu = TaiyiSixteenSpirit::YIN_TIANMU_CYCLE_WITH_STOPS[index % 18];
            assert_eq!(frame.position_in_72, count as u8);
            assert_eq!(frame.taiyi_palace, expected_palace, "count {count}");
            assert_eq!(frame.tianmu, expected_tianmu, "count {count}");
            assert_eq!(
                frame.jishen_branch,
                expected_jishen[index % 12],
                "count {count}"
            );
            if frame.shiji.zh_name() != expected_shiji[index] {
                shiji_variants.push((count, frame.shiji.zh_name(), expected_shiji[index]));
            }
            if frame.main_count != expected_main_counts[index]
                || frame.guest_count != expected_guest_counts[index]
            {
                count_variants.push((
                    count,
                    frame.main_count,
                    expected_main_counts[index],
                    frame.guest_count,
                    expected_guest_counts[index],
                ));
            }
        }
        assert_eq!(
            shiji_variants,
            vec![
                (37, "大武", "大威"),
                (43, "太神", "大武"),
                (44, "大武", "太蔟"),
            ]
        );
        assert_eq!(
            count_variants,
            vec![
                (11, 6, 6, 26, 36),
                (15, 11, 11, 25, 29),
                (25, 31, 21, 16, 16),
                (26, 30, 31, 7, 7),
                (27, 29, 39, 4, 4),
                (43, 2, 2, 1, 38),
                (44, 39, 39, 38, 31),
                (46, 7, 7, 1, 2),
                (70, 32, 22, 28, 28),
            ]
        );
    }

    #[test]
    fn explicit_dun_board_preserves_boundaries_and_never_enables_rules() {
        assert_eq!(
            taiyi_seventy_two_board_frame_from_count(0, TaiyiDun::Yin),
            Err(TaiyiBoardError::ZeroBoardCount)
        );

        for dun in [TaiyiDun::Yang, TaiyiDun::Yin] {
            let boundary = taiyi_seventy_two_board_frame_from_count(72, dun).unwrap();
            assert_eq!(boundary.position_in_72, 72);
            let reset = taiyi_seventy_two_board_frame_from_count(73, dun).unwrap();
            assert_eq!(reset.position_in_72, 1);
            let large = taiyi_seventy_two_board_frame_from_count(u64::MAX, dun).unwrap();
            assert_eq!(large.position_in_72, 15);

            for frame in [boundary, reset, large] {
                assert_eq!(
                    frame.algorithm_version,
                    TAIYI_SEVENTY_TWO_BOARD_ALGORITHM_VERSION
                );
                assert_eq!(frame.status, TAIYI_SEVENTY_TWO_BOARD_STATUS);
                assert!(!frame.dun_derived_from_civil_time);
                assert!(!frame.board_count_derived_from_civil_time);
                assert!(!frame.complete_board_generated);
                assert!(!frame.interpretation_eligible);
                assert!(!frame.signal_eligible);
                assert!(!frame.rule_eligible);
            }
        }
    }

    #[test]
    fn both_seventy_two_board_cycles_recompute_all_barrier_raw_observations() {
        for count in 1..=72 {
            for dun in [TaiyiDun::Yang, TaiyiDun::Yin] {
                let frame = taiyi_seventy_two_board_frame_from_count(count, dun).unwrap();
                let raw = frame
                    .mechanical_relations
                    .barrier_raw_observations
                    .as_ref()
                    .expect("current seventy-two-board frame records raw barrier inputs");
                assert_eq!(
                    raw,
                    &barrier_raw_observations(&TaiyiRelationInputs {
                        taiyi: frame.taiyi_palace,
                        tianmu: frame.tianmu,
                        shiji: frame.shiji,
                        main_count: frame.main_count,
                        guest_count: frame.guest_count,
                        main_great: frame.main_great_general_palace,
                        main_assistant: frame.main_assistant_general_palace,
                        guest_great: frame.guest_great_general_palace,
                        guest_assistant: frame.guest_assistant_general_palace,
                    }),
                    "count {count}, {dun:?}"
                );
                assert!(!frame.mechanical_relations.barrier_classified);
                assert!(!raw.barrier_classified);
                assert!(!raw.rule_eligible);
            }
        }
    }

    #[test]
    fn annual_frame_generals_match_independent_received_anchors() {
                                                             
                           
        let frame = taiyi_annual_frame_from_accumulated_count(1).unwrap();
        assert_eq!(frame.main_great_general_palace.number(), 7);
        assert_eq!(frame.main_assistant_general_palace.number(), 1);
        assert_eq!(frame.guest_great_general_palace.number(), 3);
        assert_eq!(frame.guest_assistant_general_palace.number(), 9);

                                                                        
                                                                           
                                          
        let main_great = general_palace_from_count(29);
        let guest_great = general_palace_from_count(4);
        assert_eq!(main_great.number(), 9);
        assert_eq!(assistant_general_palace(main_great).number(), 7);
        assert_eq!(guest_great.number(), 4);
        assert_eq!(assistant_general_palace(guest_great).number(), 2);

                                                                 
                                                                          
                                                               
        let yin_main_great = general_palace_from_count(5);
        let yin_guest_great = general_palace_from_count(29);
        assert_eq!(yin_main_great.number(), 5);
        assert_eq!(assistant_general_palace(yin_main_great).number(), 5);
        assert_eq!(yin_guest_great.number(), 9);
        assert_eq!(assistant_general_palace(yin_guest_great).number(), 7);
    }

    #[test]
    fn eight_gate_catalog_matches_received_names_order_and_taiyi_palaces() {
        let expected = [
            (TaiyiGate::Open, "开门", TaiyiPalace::One),
            (TaiyiGate::Rest, "休门", TaiyiPalace::Eight),
            (TaiyiGate::Life, "生门", TaiyiPalace::Three),
            (TaiyiGate::Harm, "伤门", TaiyiPalace::Four),
            (TaiyiGate::Block, "杜门", TaiyiPalace::Nine),
            (TaiyiGate::View, "景门", TaiyiPalace::Two),
            (TaiyiGate::Death, "死门", TaiyiPalace::Seven),
            (TaiyiGate::Alarm, "惊门", TaiyiPalace::Six),
        ];

        for (index, (gate, name, palace)) in expected.into_iter().enumerate() {
            assert_eq!(TaiyiGate::RECEIVED_LEFTWARD_ORDER[index], gate);
            assert_eq!(gate.zh_name(), name);
            assert_eq!(gate.fixed_taiyi_palace(), palace);
        }
    }

    #[test]
    fn annual_duty_gate_preserves_thirty_year_and_cycle_boundaries() {
        let anchors = [
            (1, TaiyiGate::Open, 1),
            (30, TaiyiGate::Open, 30),
            (31, TaiyiGate::Rest, 1),
            (60, TaiyiGate::Rest, 30),
            (61, TaiyiGate::Life, 1),
            (90, TaiyiGate::Life, 30),
            (91, TaiyiGate::Harm, 1),
            (121, TaiyiGate::Block, 1),
            (151, TaiyiGate::View, 1),
            (181, TaiyiGate::Death, 1),
            (211, TaiyiGate::Alarm, 1),
            (240, TaiyiGate::Alarm, 30),
            (241, TaiyiGate::Open, 1),
            (720, TaiyiGate::Alarm, 30),
            (721, TaiyiGate::Open, 1),
        ];

        for (count, expected_gate, expected_year) in anchors {
            let frame = taiyi_annual_frame_from_accumulated_count(count).unwrap();
            assert_eq!(frame.annual_duty_gate, expected_gate, "count {count}");
            assert_eq!(frame.year_within_duty_gate, expected_year, "count {count}");
            assert_eq!(
                frame.rotated_gate_boards_generated,
                frame.main_great_general_palace != TaiyiGeneralPalace::Five
                    && frame.guest_great_general_palace != TaiyiGeneralPalace::Five,
                "count {count}"
            );
        }
    }

    #[test]
    fn annual_open_gate_board_rotates_every_gate_across_each_noncentral_anchor() {
        for anchor_palace in TaiyiPalace::FORWARD_ORDER {
            let board = annual_open_gate_board(TaiyiAnnualGateBoardAnchor::Taiyi, anchor_palace);
            assert_eq!(board.anchor_gate, TaiyiGate::Open);
            assert!(board.open_gate_is_at_anchor);
            assert_eq!(board.placements[0].gate, TaiyiGate::Open);
            assert_eq!(board.placements[0].palace, anchor_palace);

            let mut palace_numbers = std::collections::BTreeSet::new();
            for (gate_index, placement) in board.placements.iter().enumerate() {
                let expected_gate = TaiyiGate::RECEIVED_LEFTWARD_ORDER[gate_index];
                let expected_palace_index = (anchor_palace.gate_ring_index() + gate_index) % 8;
                let expected_palace =
                    TaiyiGate::RECEIVED_LEFTWARD_ORDER[expected_palace_index].fixed_taiyi_palace();
                assert_eq!(placement.gate, expected_gate);
                assert_eq!(placement.palace, expected_palace);
                assert!(palace_numbers.insert(placement.palace.number()));
            }
            assert_eq!(palace_numbers.len(), 8);
            assert!(!board.interpretation_eligible);
            assert!(!board.signal_eligible);
            assert!(!board.rule_eligible);
        }
    }

    #[test]
    fn annual_gate_boards_match_received_open_anchor_examples_and_block_center_five() {
        let first = taiyi_annual_frame_from_accumulated_count(1).unwrap();
        let boards = first
            .annual_open_gate_boards
            .as_ref()
            .expect("v6 annual frame carries annual gate-board candidate");
        assert_eq!(
            boards.algorithm_version,
            TAIYI_ANNUAL_GATE_BOARDS_ALGORITHM_VERSION
        );
        assert_eq!(boards.status, TAIYI_ANNUAL_GATE_BOARDS_STATUS);
        assert_eq!(boards.anchor_gate, TaiyiGate::Open);
        assert_eq!(boards.taiyi_board.anchor_palace, TaiyiPalace::One);
        assert_eq!(
            boards
                .main_great_general_board
                .as_ref()
                .expect("count one main general is noncentral")
                .anchor_palace,
            TaiyiPalace::Seven
        );
        assert_eq!(
            boards
                .guest_great_general_board
                .as_ref()
                .expect("count one guest general is noncentral")
                .anchor_palace,
            TaiyiPalace::Three
        );
        assert!(boards.all_three_boards_generated);
        assert!(first.rotated_gate_boards_generated);

        let main_center = taiyi_annual_frame_from_accumulated_count(4).unwrap();
        assert_eq!(
            main_center.main_great_general_palace,
            TaiyiGeneralPalace::Five
        );
        let main_center_boards = main_center.annual_open_gate_boards.as_ref().unwrap();
        assert!(main_center_boards.main_great_general_board.is_none());
        assert!(main_center_boards
            .main_great_general_unavailable_reason
            .as_deref()
            .unwrap()
            .contains("中五宫"));
        assert!(main_center_boards.guest_great_general_board.is_some());
        assert!(!main_center_boards.all_three_boards_generated);
        assert!(!main_center.rotated_gate_boards_generated);

        let guest_center = taiyi_annual_frame_from_accumulated_count(7).unwrap();
        assert_eq!(
            guest_center.guest_great_general_palace,
            TaiyiGeneralPalace::Five
        );
        let guest_center_boards = guest_center.annual_open_gate_boards.as_ref().unwrap();
        assert!(guest_center_boards.main_great_general_board.is_some());
        assert!(guest_center_boards.guest_great_general_board.is_none());
        assert!(guest_center_boards
            .guest_great_general_unavailable_reason
            .as_deref()
            .unwrap()
            .contains("中五宫"));
        assert!(!guest_center_boards.all_three_boards_generated);

        for candidate in [boards, main_center_boards, guest_center_boards] {
            assert!(!candidate.fixed_count_general_board_generated);
            assert!(!candidate.time_method_duty_gate_rotation_generated);
            assert!(!candidate.three_auspicious_gates_classified);
            assert!(!candidate.interpretation_eligible);
            assert!(!candidate.signal_eligible);
            assert!(!candidate.rule_eligible);
        }
    }

    #[test]
    fn old_annual_frame_without_v6_gate_boards_remains_readable_without_recomputation() {
        let frame = taiyi_annual_frame_from_accumulated_count(1).unwrap();
        let mut serialized = serde_json::to_value(frame).unwrap();
        serialized
            .as_object_mut()
            .unwrap()
            .remove("annual_open_gate_boards");
        serialized["algorithm_version"] =
            serde_json::Value::String(TAIYI_ANNUAL_FRAME_V5_ALGORITHM_VERSION.to_owned());
        serialized["rotated_gate_boards_generated"] = serde_json::Value::Bool(false);

        let restored: TaiyiAnnualFrame = serde_json::from_value(serialized).unwrap();
        assert_eq!(
            restored.algorithm_version,
            TAIYI_ANNUAL_FRAME_V5_ALGORITHM_VERSION
        );
        assert!(restored.annual_open_gate_boards.is_none());
        assert!(!restored.rotated_gate_boards_generated);
    }

    #[test]
    fn old_v6_relations_remain_uncomputed_instead_of_silently_reclassified() {
        let frame = taiyi_annual_frame_from_accumulated_count(1).unwrap();
        let mut serialized = serde_json::to_value(frame).unwrap();
        serialized["algorithm_version"] =
            serde_json::Value::String(TAIYI_ANNUAL_FRAME_V6_ALGORITHM_VERSION.to_owned());
        let relations = serialized["mechanical_relations"]
            .as_object_mut()
            .expect("mechanical relations object");
        relations.insert(
            "algorithm_version".to_owned(),
            serde_json::Value::String(TAIYI_MECHANICAL_RELATIONS_V1_ALGORITHM_VERSION.to_owned()),
        );
        for field in [
            "shiji_near_taiyi_geometry",
            "tianmu_near_taiyi_geometry",
            "near_ring_geometry_computed",
            "tianmu_opposes_taiyi",
            "shiji_strike",
            "tianmu_pressure",
            "main_great_general_pressure",
            "main_assistant_general_pressure",
            "guest_great_general_pressure",
            "guest_assistant_general_pressure",
            "barrier_raw_observations",
        ] {
            relations.remove(field);
        }
        relations.insert(
            "opposition_classified".to_owned(),
            serde_json::Value::Bool(false),
        );

        let restored: TaiyiAnnualFrame = serde_json::from_value(serialized).unwrap();
        assert_eq!(
            restored.algorithm_version,
            TAIYI_ANNUAL_FRAME_V6_ALGORITHM_VERSION
        );
        let relations = restored.mechanical_relations;
        assert_eq!(
            relations.algorithm_version,
            TAIYI_MECHANICAL_RELATIONS_V1_ALGORITHM_VERSION
        );
        assert!(!relations.near_ring_geometry_computed);
        assert_eq!(relations.shiji_near_taiyi_geometry, None);
        assert_eq!(relations.tianmu_near_taiyi_geometry, None);
        assert!(!relations.tianmu_opposes_taiyi);
        assert_eq!(relations.shiji_strike, None);
        assert_eq!(relations.tianmu_pressure, None);
        assert_eq!(relations.main_great_general_pressure, None);
        assert_eq!(relations.main_assistant_general_pressure, None);
        assert_eq!(relations.guest_great_general_pressure, None);
        assert_eq!(relations.guest_assistant_general_pressure, None);
        assert!(relations.barrier_raw_observations.is_none());
        assert!(!relations.opposition_classified);
    }

    #[test]
    fn old_v7_relation_v2_keeps_geometry_but_not_new_strike_or_pressure_labels() {
        let frame = taiyi_annual_frame_from_accumulated_count(2).unwrap();
        let mut serialized = serde_json::to_value(frame).unwrap();
        serialized["algorithm_version"] =
            serde_json::Value::String(TAIYI_ANNUAL_FRAME_V7_ALGORITHM_VERSION.to_owned());
        let relations = serialized["mechanical_relations"]
            .as_object_mut()
            .expect("mechanical relations object");
        relations.insert(
            "algorithm_version".to_owned(),
            serde_json::Value::String(TAIYI_MECHANICAL_RELATIONS_V2_ALGORITHM_VERSION.to_owned()),
        );
        for field in [
            "shiji_strike",
            "tianmu_pressure",
            "main_great_general_pressure",
            "main_assistant_general_pressure",
            "guest_great_general_pressure",
            "guest_assistant_general_pressure",
            "barrier_raw_observations",
        ] {
            relations.remove(field);
        }
        relations.insert(
            "strike_classified".to_owned(),
            serde_json::Value::Bool(false),
        );
        relations.insert(
            "pressure_classified".to_owned(),
            serde_json::Value::Bool(false),
        );

        let restored: TaiyiAnnualFrame = serde_json::from_value(serialized).unwrap();
        assert_eq!(
            restored.algorithm_version,
            TAIYI_ANNUAL_FRAME_V7_ALGORITHM_VERSION
        );
        let relations = restored.mechanical_relations;
        assert_eq!(
            relations.algorithm_version,
            TAIYI_MECHANICAL_RELATIONS_V2_ALGORITHM_VERSION
        );
        assert!(relations.near_ring_geometry_computed);
        assert_eq!(
            relations.shiji_near_taiyi_geometry,
            Some(TaiyiNearRingGeometry::MinusOneInterstitial)
        );
        assert_eq!(
            relations.tianmu_near_taiyi_geometry,
            Some(TaiyiNearRingGeometry::MinusTwoPrincipalPalace)
        );
        assert_eq!(relations.shiji_strike, None);
        assert_eq!(relations.tianmu_pressure, None);
        assert!(relations.barrier_raw_observations.is_none());
        assert!(!relations.strike_classified);
        assert!(!relations.pressure_classified);
    }

    #[test]
    fn old_v8_relation_v3_keeps_strike_pressure_but_not_barrier_observations() {
        let frame = taiyi_annual_frame_from_accumulated_count(11).unwrap();
        let mut serialized = serde_json::to_value(frame).unwrap();
        serialized["algorithm_version"] =
            serde_json::Value::String(TAIYI_ANNUAL_FRAME_V8_ALGORITHM_VERSION.to_owned());
        let relations = serialized["mechanical_relations"]
            .as_object_mut()
            .expect("mechanical relations object");
        relations.insert(
            "algorithm_version".to_owned(),
            serde_json::Value::String(TAIYI_MECHANICAL_RELATIONS_V3_ALGORITHM_VERSION.to_owned()),
        );
        relations.remove("barrier_raw_observations");

        let restored: TaiyiAnnualFrame = serde_json::from_value(serialized).unwrap();
        assert_eq!(
            restored.algorithm_version,
            TAIYI_ANNUAL_FRAME_V8_ALGORITHM_VERSION
        );
        let relations = restored.mechanical_relations;
        assert_eq!(
            relations.algorithm_version,
            TAIYI_MECHANICAL_RELATIONS_V3_ALGORITHM_VERSION
        );
        assert!(relations.strike_classified);
        assert!(relations.pressure_classified);
        assert!(relations.barrier_raw_observations.is_none());
        assert!(!relations.barrier_classified);
    }

    #[test]
    fn near_ring_geometry_preserves_unoriented_offsets_and_opposition_is_exact() {
        let taiyi = TaiyiPalace::Eight;
        assert_eq!(
            spirit_near_taiyi_geometry(TaiyiSixteenSpirit::YangDe, taiyi),
            Some(TaiyiNearRingGeometry::PlusOneInterstitial)
        );
        assert_eq!(
            spirit_near_taiyi_geometry(TaiyiSixteenSpirit::DaYi, taiyi),
            Some(TaiyiNearRingGeometry::MinusOneInterstitial)
        );
        assert_eq!(
            spirit_near_taiyi_geometry(TaiyiSixteenSpirit::HeDe, taiyi),
            Some(TaiyiNearRingGeometry::PlusTwoPrincipalPalace)
        );
        assert_eq!(
            spirit_near_taiyi_geometry(TaiyiSixteenSpirit::YinDe, taiyi),
            Some(TaiyiNearRingGeometry::MinusTwoPrincipalPalace)
        );
        assert_eq!(
            spirit_near_taiyi_geometry(TaiyiSixteenSpirit::DaWei, taiyi),
            None
        );
        assert!(spirit_opposes_taiyi(TaiyiSixteenSpirit::DaWei, taiyi));
    }

    #[test]
    fn received_table_anchors_plus_as_front_outer_and_minus_as_rear_inner() {
        assert_eq!(
            pressure_from_geometry(TaiyiNearRingGeometry::PlusOneInterstitial),
            TaiyiPressure::OuterChen
        );
        assert_eq!(
            pressure_from_geometry(TaiyiNearRingGeometry::MinusOneInterstitial),
            TaiyiPressure::InnerChen
        );
        assert_eq!(
            pressure_from_geometry(TaiyiNearRingGeometry::PlusTwoPrincipalPalace),
            TaiyiPressure::OuterPalace
        );
        assert_eq!(
            pressure_from_geometry(TaiyiNearRingGeometry::MinusTwoPrincipalPalace),
            TaiyiPressure::InnerPalace
        );
        assert_eq!(
            strike_from_geometry(TaiyiNearRingGeometry::PlusOneInterstitial),
            TaiyiStrike::FrontChen
        );
        assert_eq!(
            strike_from_geometry(TaiyiNearRingGeometry::MinusOneInterstitial),
            TaiyiStrike::RearChen
        );
        assert_eq!(
            strike_from_geometry(TaiyiNearRingGeometry::PlusTwoPrincipalPalace),
            TaiyiStrike::OuterPalace
        );
        assert_eq!(
            strike_from_geometry(TaiyiNearRingGeometry::MinusTwoPrincipalPalace),
            TaiyiStrike::InnerPalace
        );

                                                                          
                                                                              
                                                                        
        assert_eq!(
            spirit_near_taiyi_geometry(TaiyiSixteenSpirit::HeDe, TaiyiPalace::Eight)
                .map(pressure_from_geometry),
            Some(TaiyiPressure::OuterPalace)
        );
        assert_eq!(
            spirit_near_taiyi_geometry(TaiyiSixteenSpirit::HeDe, TaiyiPalace::Four)
                .map(pressure_from_geometry),
            Some(TaiyiPressure::InnerPalace)
        );

                                                              
        let count = eye_count_to_taiyi(TaiyiSixteenSpirit::YangDe, TaiyiPalace::Two);
        let great_general = general_palace_from_count(count);
        assert_eq!(count, 17);
        assert_eq!(great_general, TaiyiGeneralPalace::Seven);
        assert_eq!(
            general_pressure(great_general, TaiyiPalace::Two),
            Some(TaiyiPressure::OuterPalace)
        );
    }

    #[test]
    fn barrier_raw_observations_enumerate_geometry_without_naming_barrier() {
                                                                              
                                                                           
                                                                              
                                
        let count_two = taiyi_annual_frame_from_accumulated_count(2).unwrap();
        let raw = count_two
            .mechanical_relations
            .barrier_raw_observations
            .as_ref()
            .expect("current relations record raw barrier inputs");
        assert_eq!(
            raw.eye_general_same_palace,
            vec![TaiyiEyeGeneralCoincidence {
                eye: TaiyiEyeRole::Tianmu,
                general: TaiyiGeneralRole::MainGreat,
            }]
        );
        assert!(raw.general_same_palace.is_empty());
        assert!(!raw.main_guest_counts_equal);
        assert_eq!(
            raw.compact_zh(),
            "关法原始观察：候选主体同宫：天目—主大将；主客算值不等（配对与‘数齐’释义未决，不判关）"
        );
        assert!(!count_two.mechanical_relations.barrier_classified);

                                                                        
                                                                          
                                                   
        let count_eleven = taiyi_annual_frame_from_accumulated_count(11).unwrap();
        let raw = count_eleven
            .mechanical_relations
            .barrier_raw_observations
            .as_ref()
            .expect("current relations record raw barrier inputs");
        assert_eq!(
            raw.eye_general_same_palace,
            vec![
                TaiyiEyeGeneralCoincidence {
                    eye: TaiyiEyeRole::Tianmu,
                    general: TaiyiGeneralRole::MainGreat,
                },
                TaiyiEyeGeneralCoincidence {
                    eye: TaiyiEyeRole::Tianmu,
                    general: TaiyiGeneralRole::GuestGreat,
                },
            ]
        );
        assert_eq!(
            raw.general_same_palace,
            vec![
                TaiyiGeneralCoincidence {
                    first: TaiyiGeneralRole::MainGreat,
                    second: TaiyiGeneralRole::GuestGreat,
                },
                TaiyiGeneralCoincidence {
                    first: TaiyiGeneralRole::MainAssistant,
                    second: TaiyiGeneralRole::GuestAssistant,
                },
            ]
        );
        assert!(raw.main_guest_counts_equal);
        assert!(raw.candidate_pairs_exhaustively_enumerated);
        assert!(!raw.received_pairing_resolved);
        assert!(!raw.received_number_equality_resolved);
        assert!(!raw.barrier_classified);
        assert!(!raw.interpretation_eligible);
        assert!(!raw.signal_eligible);
        assert!(!raw.rule_eligible);
        assert!(!count_eleven.mechanical_relations.barrier_classified);
    }

    #[test]
    fn annual_frame_marks_received_cover_imprisonment_and_ge_positions() {
                                                                           
                                                            
        let first = taiyi_annual_frame_from_accumulated_count(1).unwrap();
        assert!(
            first
                .mechanical_relations
                .main_assistant_general_imprisoned_with_taiyi
        );
        assert!(
            first
                .mechanical_relations
                .guest_assistant_general_forms_ge_against_taiyi
        );
        assert_eq!(
            first.mechanical_relations.compact_marks_zh(),
            "主参将囚、客参将格太乙"
        );

        let guest_great_imprisoned = taiyi_annual_frame_from_accumulated_count(2).unwrap();
        assert!(
            guest_great_imprisoned
                .mechanical_relations
                .guest_great_general_imprisoned_with_taiyi
        );

        let main_great_imprisoned = taiyi_annual_frame_from_accumulated_count(3).unwrap();
        assert!(
            main_great_imprisoned
                .mechanical_relations
                .main_great_general_imprisoned_with_taiyi
        );

                                                              
        let tianmu_imprisoned = taiyi_annual_frame_from_accumulated_count(9).unwrap();
        assert!(
            tianmu_imprisoned
                .mechanical_relations
                .tianmu_imprisoned_with_taiyi
        );

                                                                     
        let cover = taiyi_annual_frame_from_accumulated_count(33).unwrap();
        assert!(cover.mechanical_relations.shiji_covers_taiyi);

                                                                          
                                                                          
                                                     
        let shiji_ge = taiyi_annual_frame_from_accumulated_count(8).unwrap();
        assert!(shiji_ge.mechanical_relations.shiji_forms_ge_against_taiyi);
        let guest_great_ge = taiyi_annual_frame_from_accumulated_count(16).unwrap();
        assert!(
            guest_great_ge
                .mechanical_relations
                .guest_great_general_forms_ge_against_taiyi
        );
    }

    #[test]
    fn taiyi_opposite_palaces_are_symmetric_and_never_central() {
        for palace in TaiyiPalace::FORWARD_ORDER {
            assert_eq!(palace.opposite().opposite(), palace);
            assert_ne!(palace.opposite().number(), 5);
            assert_eq!(
                (palace.ring_index() + 8) % 16,
                palace.opposite().ring_index()
            );
        }
    }

    #[test]
    fn annual_frame_places_taisui_and_heshin_in_received_directions() {
        let anchors = [
            (1, "甲子", EarthlyBranch::Zi, EarthlyBranch::Chou),
            (2, "乙丑", EarthlyBranch::Chou, EarthlyBranch::Zi),
            (12, "乙亥", EarthlyBranch::Hai, EarthlyBranch::Yin),
            (13, "丙子", EarthlyBranch::Zi, EarthlyBranch::Chou),
            (60, "癸亥", EarthlyBranch::Hai, EarthlyBranch::Yin),
            (61, "甲子", EarthlyBranch::Zi, EarthlyBranch::Chou),
        ];

        for (count, designation, tai_sui, heshin) in anchors {
            let frame = taiyi_annual_frame_from_accumulated_count(count).unwrap();
            assert_eq!(frame.tai_sui_designation.zh_name(), designation);
            assert_eq!(frame.tai_sui_branch, tai_sui);
            assert_eq!(frame.heshin_branch, heshin);
        }
    }

    #[test]
    fn annual_frame_marks_only_the_second_yinde_and_dawu_counts_as_stops() {
        for count in 1..=72 {
            let frame = taiyi_annual_frame_from_accumulated_count(count).unwrap();
            assert_eq!(
                frame.tianmu_is_repeated_stop,
                matches!(frame.tianmu_position_in_18, 5 | 18)
            );
        }
    }

    #[test]
    fn annual_frame_cycles_are_exhaustive_and_never_become_rule_outputs() {
        for count in 1..=720 {
            let frame = taiyi_annual_frame_from_accumulated_count(count).unwrap();
            assert_eq!(frame.position_in_60, ((count - 1) % 60 + 1) as u8);
            assert_eq!(frame.tianmu_position_in_18, ((count - 1) % 18 + 1) as u8);
            assert_eq!(frame.jishen_position_in_12, ((count - 1) % 12 + 1) as u8);
            assert_eq!(frame.heshin_position_in_12, frame.jishen_position_in_12);
            assert!((1..=16).contains(&frame.shiji_ring_position));
            assert!((1..=40).contains(&frame.main_count));
            assert!((1..=40).contains(&frame.guest_count));
            assert!((1..=9).contains(&frame.main_great_general_palace.number()));
            assert!((1..=9).contains(&frame.main_assistant_general_palace.number()));
            assert!((1..=9).contains(&frame.guest_great_general_palace.number()));
            assert!((1..=9).contains(&frame.guest_assistant_general_palace.number()));
            assert_eq!(
                frame.eight_gate_position_in_720,
                ((count - 1) % 720 + 1) as u16
            );
            assert_eq!(
                frame.eight_gate_position_in_240,
                ((count - 1) % 240 + 1) as u8
            );
            assert_eq!(
                frame.duty_gate_step,
                (frame.eight_gate_position_in_240 - 1) / 30
            );
            assert_eq!(
                frame.annual_duty_gate,
                TaiyiGate::RECEIVED_LEFTWARD_ORDER[usize::from(frame.duty_gate_step)]
            );
            assert_eq!(
                frame.year_within_duty_gate,
                (frame.eight_gate_position_in_240 - 1) % 30 + 1
            );
            let relations = &frame.mechanical_relations;
            assert_eq!(
                relations.algorithm_version,
                TAIYI_MECHANICAL_RELATIONS_ALGORITHM_VERSION
            );
            assert_eq!(relations.status, TAIYI_MECHANICAL_RELATIONS_STATUS);
            assert_eq!(
                relations.shiji_covers_taiyi,
                spirit_coincides_with_taiyi(frame.shiji, frame.taiyi_position.palace)
            );
            assert_eq!(
                relations.tianmu_imprisoned_with_taiyi,
                spirit_coincides_with_taiyi(frame.tianmu, frame.taiyi_position.palace)
            );
            assert_eq!(
                relations.guest_great_general_forms_ge_against_taiyi,
                general_opposes_taiyi(
                    frame.guest_great_general_palace,
                    frame.taiyi_position.palace
                )
            );
            assert_eq!(
                relations.shiji_near_taiyi_geometry,
                spirit_near_taiyi_geometry(frame.shiji, frame.taiyi_position.palace)
            );
            assert_eq!(
                relations.tianmu_near_taiyi_geometry,
                spirit_near_taiyi_geometry(frame.tianmu, frame.taiyi_position.palace)
            );
            assert!(relations.near_ring_geometry_computed);
            assert_eq!(
                relations.tianmu_opposes_taiyi,
                spirit_opposes_taiyi(frame.tianmu, frame.taiyi_position.palace)
            );
            assert_eq!(
                relations.shiji_strike,
                spirit_near_taiyi_geometry(frame.shiji, frame.taiyi_position.palace)
                    .map(strike_from_geometry)
            );
            assert_eq!(
                relations.tianmu_pressure,
                spirit_near_taiyi_geometry(frame.tianmu, frame.taiyi_position.palace)
                    .map(pressure_from_geometry)
            );
            assert_eq!(
                relations.main_great_general_pressure,
                general_pressure(frame.main_great_general_palace, frame.taiyi_position.palace)
            );
            assert_eq!(
                relations.main_assistant_general_pressure,
                general_pressure(
                    frame.main_assistant_general_palace,
                    frame.taiyi_position.palace
                )
            );
            assert_eq!(
                relations.guest_great_general_pressure,
                general_pressure(
                    frame.guest_great_general_palace,
                    frame.taiyi_position.palace
                )
            );
            assert_eq!(
                relations.guest_assistant_general_pressure,
                general_pressure(
                    frame.guest_assistant_general_palace,
                    frame.taiyi_position.palace
                )
            );
            let raw = relations
                .barrier_raw_observations
                .as_ref()
                .expect("current annual frame records barrier raw observations");
            assert_eq!(
                raw,
                &barrier_raw_observations(&TaiyiRelationInputs {
                    taiyi: frame.taiyi_position.palace,
                    tianmu: frame.tianmu,
                    shiji: frame.shiji,
                    main_count: frame.main_count,
                    guest_count: frame.guest_count,
                    main_great: frame.main_great_general_palace,
                    main_assistant: frame.main_assistant_general_palace,
                    guest_great: frame.guest_great_general_palace,
                    guest_assistant: frame.guest_assistant_general_palace,
                })
            );
            assert!(raw.candidate_pairs_exhaustively_enumerated);
            assert!(!raw.received_pairing_resolved);
            assert!(!raw.received_number_equality_resolved);
            assert!(!raw.barrier_classified);
            assert!(relations.strike_classified);
            assert!(relations.pressure_classified);
            assert!(!relations.barrier_classified);
            assert!(relations.opposition_classified);
            assert!(!relations.interpretation_eligible);
            assert!(!relations.signal_eligible);
            assert!(!relations.rule_eligible);
            let gate_boards = frame
                .annual_open_gate_boards
                .as_ref()
                .expect("current annual frame has annual gate boards");
            assert_eq!(
                frame.rotated_gate_boards_generated,
                gate_boards.all_three_boards_generated
            );
            assert_eq!(
                gate_boards.main_great_general_board.is_some(),
                frame.main_great_general_palace != TaiyiGeneralPalace::Five
            );
            assert_eq!(
                gate_boards.guest_great_general_board.is_some(),
                frame.guest_great_general_palace != TaiyiGeneralPalace::Five
            );
            assert!(!gate_boards.interpretation_eligible);
            assert!(!gate_boards.signal_eligible);
            assert!(!gate_boards.rule_eligible);
            assert!(!frame.calendar_epoch_selected);
            assert!(!frame.derived_from_civil_date);
            assert!(!frame.yin_yang_dun_selected);
            assert!(!frame.complete_board_generated);
            assert!(!frame.interpretation_eligible);
            assert!(!frame.signal_eligible);
            assert!(!frame.rule_eligible);
        }
    }

    #[test]
    fn annual_frame_large_count_reduces_before_signed_conversion() {
        let frame = taiyi_annual_frame_from_accumulated_count(u64::MAX).unwrap();
        assert_eq!(frame.position_in_60, 15);
        assert_eq!(frame.tai_sui_designation.zh_name(), "戊寅");
        assert_eq!(frame.tianmu_position_in_18, 15);
        assert_eq!(frame.tianmu, TaiyiSixteenSpirit::DaWei);
        assert_eq!(frame.jishen_position_in_12, 3);
        assert_eq!(frame.jishen_branch, EarthlyBranch::Zi);
        assert_eq!(frame.heshin_branch, EarthlyBranch::Hai);
        assert_eq!(frame.shiji, TaiyiSixteenSpirit::DaWu);
        assert_eq!(frame.shiji_ring_position, 11);
        assert_eq!(frame.main_count, 9);
        assert_eq!(frame.guest_count, 7);
        assert_eq!(frame.main_great_general_palace, TaiyiGeneralPalace::Nine);
        assert_eq!(
            frame.main_assistant_general_palace,
            TaiyiGeneralPalace::Seven
        );
        assert_eq!(frame.guest_great_general_palace, TaiyiGeneralPalace::Seven);
        assert_eq!(
            frame.guest_assistant_general_palace,
            TaiyiGeneralPalace::One
        );
        assert_eq!(frame.eight_gate_position_in_720, 15);
        assert_eq!(frame.eight_gate_position_in_240, 15);
        assert_eq!(frame.annual_duty_gate, TaiyiGate::Open);
        assert_eq!(frame.year_within_duty_gate, 15);
        assert!(frame.rotated_gate_boards_generated);
        assert!(frame.annual_open_gate_boards.is_some());
    }

    #[test]
    fn annual_relation_anchors_cover_each_near_geometry_and_opposition() {
        let anchors = [
            (
                3,
                Some(TaiyiNearRingGeometry::PlusOneInterstitial),
                Some(TaiyiNearRingGeometry::MinusOneInterstitial),
            ),
            (
                2,
                Some(TaiyiNearRingGeometry::MinusOneInterstitial),
                Some(TaiyiNearRingGeometry::MinusTwoPrincipalPalace),
            ),
            (
                16,
                Some(TaiyiNearRingGeometry::PlusTwoPrincipalPalace),
                Some(TaiyiNearRingGeometry::MinusOneInterstitial),
            ),
            (
                15,
                Some(TaiyiNearRingGeometry::MinusTwoPrincipalPalace),
                None,
            ),
            (
                12,
                Some(TaiyiNearRingGeometry::MinusOneInterstitial),
                Some(TaiyiNearRingGeometry::PlusOneInterstitial),
            ),
            (
                25,
                Some(TaiyiNearRingGeometry::PlusOneInterstitial),
                Some(TaiyiNearRingGeometry::PlusTwoPrincipalPalace),
            ),
        ];
        for (count, expected_shiji, expected_tianmu) in anchors {
            let frame = taiyi_annual_frame_from_accumulated_count(count).unwrap();
            let relations = frame.mechanical_relations;
            assert_eq!(relations.shiji_near_taiyi_geometry, expected_shiji);
            assert_eq!(relations.tianmu_near_taiyi_geometry, expected_tianmu);
        }
        let opposition = taiyi_annual_frame_from_accumulated_count(22).unwrap();
        assert!(opposition.mechanical_relations.tianmu_opposes_taiyi);
        assert!(opposition.mechanical_relations.opposition_classified);
    }

    #[test]
    fn annual_frames_cover_both_pressure_directions_for_all_four_generals() {
        let count_two = taiyi_annual_frame_from_accumulated_count(2)
            .unwrap()
            .mechanical_relations;
        assert_eq!(
            count_two.main_great_general_pressure,
            Some(TaiyiPressure::InnerPalace)
        );
        assert_eq!(
            count_two.main_assistant_general_pressure,
            Some(TaiyiPressure::OuterPalace)
        );

        assert_eq!(
            taiyi_annual_frame_from_accumulated_count(14)
                .unwrap()
                .mechanical_relations
                .main_great_general_pressure,
            Some(TaiyiPressure::OuterPalace)
        );
        assert_eq!(
            taiyi_annual_frame_from_accumulated_count(10)
                .unwrap()
                .mechanical_relations
                .main_assistant_general_pressure,
            Some(TaiyiPressure::InnerPalace)
        );
        assert_eq!(
            taiyi_annual_frame_from_accumulated_count(4)
                .unwrap()
                .mechanical_relations
                .guest_great_general_pressure,
            Some(TaiyiPressure::OuterPalace)
        );

        let count_fifteen = taiyi_annual_frame_from_accumulated_count(15)
            .unwrap()
            .mechanical_relations;
        assert_eq!(
            count_fifteen.guest_great_general_pressure,
            Some(TaiyiPressure::InnerPalace)
        );
        assert_eq!(
            count_fifteen.guest_assistant_general_pressure,
            Some(TaiyiPressure::OuterPalace)
        );
        assert_eq!(
            taiyi_annual_frame_from_accumulated_count(12)
                .unwrap()
                .mechanical_relations
                .guest_assistant_general_pressure,
            Some(TaiyiPressure::InnerPalace)
        );
    }
}
