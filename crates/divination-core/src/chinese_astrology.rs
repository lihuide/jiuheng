                                                                            
                                                                             
              
   
                                                                           
                                                                             
                                      

use serde::{Deserialize, Serialize};

pub const TWENTY_EIGHT_MANSION_CATALOG_ALGORITHM_VERSION: &str =
    "jiuheng.chinese_astrology.received_twenty_eight_mansion_identity.v1";
pub const CHINESE_ASTROLOGY_STRUCTURE_CATALOG_ALGORITHM_VERSION: &str =
    "jiuheng.chinese_astrology.received_structure_identity.v2";
pub const CHINESE_ASTROLOGY_STRUCTURE_CATALOG_STATUS: &str =
    "mechanical_received_three_enclosures_mansions_luminaries_no_sky_boundary_or_interpretation.v2";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreeEnclosure {
    PurpleForbidden,
    SupremeSubtle,
    HeavenlyMarket,
}

impl ThreeEnclosure {
    pub const ALL_IN_STEP_SONG_ORDER: [Self; 3] = [
        Self::PurpleForbidden,
        Self::SupremeSubtle,
        Self::HeavenlyMarket,
    ];

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::PurpleForbidden => "紫微垣",
            Self::SupremeSubtle => "太微垣",
            Self::HeavenlyMarket => "天市垣",
        }
    }

    pub const fn received_palace_name(self) -> &'static str {
        match self {
            Self::PurpleForbidden => "紫微宫",
            Self::SupremeSubtle => "太微宫",
            Self::HeavenlyMarket => "天市宫",
        }
    }

    pub const fn yuan_level(self) -> YuanLevel {
        match self {
            Self::PurpleForbidden => YuanLevel::Middle,
            Self::SupremeSubtle => YuanLevel::Upper,
            Self::HeavenlyMarket => YuanLevel::Lower,
        }
    }

    pub const fn catalog_sequence(self) -> u8 {
        self as u8 + 1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum YuanLevel {
    Upper,
    Middle,
    Lower,
}

impl YuanLevel {
    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Upper => "上元",
            Self::Middle => "中元",
            Self::Lower => "下元",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreeEnclosureIdentity {
    pub enclosure: ThreeEnclosure,
    pub catalog_sequence: u8,
    pub zh_name: String,
    pub received_palace_name: String,
    pub yuan_level: YuanLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FivePhase {
    Wood,
    Fire,
    Earth,
    Metal,
    Water,
}

impl FivePhase {
    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Wood => "木",
            Self::Fire => "火",
            Self::Earth => "土",
            Self::Metal => "金",
            Self::Water => "水",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SevenLuminary {
    Sun,
    Moon,
    Jupiter,
    Mars,
    Saturn,
    Venus,
    Mercury,
}

impl SevenLuminary {
    pub const ALL_IN_CATALOG_ORDER: [Self; 7] = [
        Self::Sun,
        Self::Moon,
        Self::Jupiter,
        Self::Mars,
        Self::Saturn,
        Self::Venus,
        Self::Mercury,
    ];

    pub const fn received_zh_name(self) -> &'static str {
        match self {
            Self::Sun => "日",
            Self::Moon => "月",
            Self::Jupiter => "岁星",
            Self::Mars => "荧惑",
            Self::Saturn => "填星",
            Self::Venus => "太白",
            Self::Mercury => "辰星",
        }
    }

    pub const fn modern_zh_name(self) -> &'static str {
        match self {
            Self::Sun => "太阳",
            Self::Moon => "月球",
            Self::Jupiter => "木星",
            Self::Mars => "火星",
            Self::Saturn => "土星",
            Self::Venus => "金星",
            Self::Mercury => "水星",
        }
    }

    pub const fn five_phase(self) -> Option<FivePhase> {
        match self {
            Self::Sun | Self::Moon => None,
            Self::Jupiter => Some(FivePhase::Wood),
            Self::Mars => Some(FivePhase::Fire),
            Self::Saturn => Some(FivePhase::Earth),
            Self::Venus => Some(FivePhase::Metal),
            Self::Mercury => Some(FivePhase::Water),
        }
    }

    pub const fn catalog_sequence(self) -> u8 {
        self as u8 + 1
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SevenLuminaryIdentity {
    pub luminary: SevenLuminary,
    pub catalog_sequence: u8,
    pub received_zh_name: String,
    pub modern_zh_name: String,
    pub five_phase: Option<FivePhase>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MansionQuadrant {
    East,
    North,
    West,
    South,
}

impl MansionQuadrant {
    pub const ALL_IN_RECEIVED_ORDER: [Self; 4] = [Self::East, Self::North, Self::West, Self::South];

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::East => "东方",
            Self::North => "北方",
            Self::West => "西方",
            Self::South => "南方",
        }
    }

    pub const fn emblem(self) -> FourEmblem {
        match self {
            Self::East => FourEmblem::AzureDragon,
            Self::North => FourEmblem::BlackTortoise,
            Self::West => FourEmblem::WhiteTiger,
            Self::South => FourEmblem::VermilionBird,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FourEmblem {
    AzureDragon,
    BlackTortoise,
    WhiteTiger,
    VermilionBird,
}

impl FourEmblem {
    pub const fn received_zh_name(self) -> &'static str {
        match self {
            Self::AzureDragon => "苍龙",
            Self::BlackTortoise => "玄武",
            Self::WhiteTiger => "白虎",
            Self::VermilionBird => "朱鸟",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LunarMansion {
    Jiao,
    Kang,
    Di,
    Fang,
    Xin,
    WeiTail,
    Ji,
    Dou,
    Niu,
    Nu,
    Xu,
    WeiDanger,
    Shi,
    BiWall,
    Kui,
    Lou,
    WeiStomach,
    Mao,
    BiNet,
    Zi,
    Shen,
    Jing,
    Gui,
    Liu,
    Xing,
    Zhang,
    Yi,
    Zhen,
}

impl LunarMansion {
    pub const ALL_IN_RECEIVED_ORDER: [Self; 28] = [
        Self::Jiao,
        Self::Kang,
        Self::Di,
        Self::Fang,
        Self::Xin,
        Self::WeiTail,
        Self::Ji,
        Self::Dou,
        Self::Niu,
        Self::Nu,
        Self::Xu,
        Self::WeiDanger,
        Self::Shi,
        Self::BiWall,
        Self::Kui,
        Self::Lou,
        Self::WeiStomach,
        Self::Mao,
        Self::BiNet,
        Self::Zi,
        Self::Shen,
        Self::Jing,
        Self::Gui,
        Self::Liu,
        Self::Xing,
        Self::Zhang,
        Self::Yi,
        Self::Zhen,
    ];

    pub const fn short_zh_name(self) -> &'static str {
        match self {
            Self::Jiao => "角",
            Self::Kang => "亢",
            Self::Di => "氐",
            Self::Fang => "房",
            Self::Xin => "心",
            Self::WeiTail => "尾",
            Self::Ji => "箕",
            Self::Dou => "斗",
            Self::Niu => "牛",
            Self::Nu => "女",
            Self::Xu => "虚",
            Self::WeiDanger => "危",
            Self::Shi => "室",
            Self::BiWall => "壁",
            Self::Kui => "奎",
            Self::Lou => "娄",
            Self::WeiStomach => "胃",
            Self::Mao => "昴",
            Self::BiNet => "毕",
            Self::Zi => "觜",
            Self::Shen => "参",
            Self::Jing => "井",
            Self::Gui => "鬼",
            Self::Liu => "柳",
            Self::Xing => "星",
            Self::Zhang => "张",
            Self::Yi => "翼",
            Self::Zhen => "轸",
        }
    }

                                                                               
                                                                                
    pub const fn received_zh_name(self) -> &'static str {
        match self {
            Self::Dou => "南斗",
            Self::Niu => "牵牛",
            Self::Nu => "须女",
            Self::Shi => "营室",
            Self::BiWall => "东壁",
            Self::Zi => "觜觿",
            Self::Jing => "东井",
            Self::Gui => "舆鬼",
            Self::Xing => "七星",
            _ => self.short_zh_name(),
        }
    }

    pub const fn received_sequence(self) -> u8 {
        self as u8 + 1
    }

    pub const fn quadrant(self) -> MansionQuadrant {
        match self.received_sequence() {
            1..=7 => MansionQuadrant::East,
            8..=14 => MansionQuadrant::North,
            15..=21 => MansionQuadrant::West,
            22..=28 => MansionQuadrant::South,
            _ => unreachable!(),
        }
    }

    pub const fn quadrant_sequence(self) -> u8 {
        (self.received_sequence() - 1) % 7 + 1
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LunarMansionIdentity {
    pub mansion: LunarMansion,
    pub received_sequence: u8,
    pub quadrant_sequence: u8,
    pub short_zh_name: String,
    pub received_zh_name: String,
    pub quadrant: MansionQuadrant,
    pub four_emblem: FourEmblem,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChineseAstrologyStructureAudit {
    pub audit_version: u16,
    pub algorithm_version: String,
    pub status: String,
    #[serde(default)]
    pub three_enclosures: Vec<ThreeEnclosureIdentity>,
    pub mansions: Vec<LunarMansionIdentity>,
    #[serde(default)]
    pub seven_luminaries: Vec<SevenLuminaryIdentity>,
    #[serde(default)]
    pub enclosure_boundary_model_selected: bool,
    pub angular_boundary_model_selected: bool,
    pub celestial_positions_calculated: bool,
    pub interpretation_eligible: bool,
    pub rule_eligible: bool,
    pub blocked_reasons: Vec<String>,
}

pub fn three_enclosure_catalog() -> Vec<ThreeEnclosureIdentity> {
    ThreeEnclosure::ALL_IN_STEP_SONG_ORDER
        .into_iter()
        .map(|enclosure| ThreeEnclosureIdentity {
            enclosure,
            catalog_sequence: enclosure.catalog_sequence(),
            zh_name: enclosure.zh_name().to_owned(),
            received_palace_name: enclosure.received_palace_name().to_owned(),
            yuan_level: enclosure.yuan_level(),
        })
        .collect()
}

pub fn twenty_eight_mansion_catalog() -> Vec<LunarMansionIdentity> {
    LunarMansion::ALL_IN_RECEIVED_ORDER
        .into_iter()
        .map(|mansion| LunarMansionIdentity {
            mansion,
            received_sequence: mansion.received_sequence(),
            quadrant_sequence: mansion.quadrant_sequence(),
            short_zh_name: mansion.short_zh_name().to_owned(),
            received_zh_name: mansion.received_zh_name().to_owned(),
            quadrant: mansion.quadrant(),
            four_emblem: mansion.quadrant().emblem(),
        })
        .collect()
}

pub fn seven_luminary_catalog() -> Vec<SevenLuminaryIdentity> {
    SevenLuminary::ALL_IN_CATALOG_ORDER
        .into_iter()
        .map(|luminary| SevenLuminaryIdentity {
            luminary,
            catalog_sequence: luminary.catalog_sequence(),
            received_zh_name: luminary.received_zh_name().to_owned(),
            modern_zh_name: luminary.modern_zh_name().to_owned(),
            five_phase: luminary.five_phase(),
        })
        .collect()
}

pub fn chinese_astrology_structure_audit() -> ChineseAstrologyStructureAudit {
    ChineseAstrologyStructureAudit {
        audit_version: 2,
        algorithm_version: CHINESE_ASTROLOGY_STRUCTURE_CATALOG_ALGORITHM_VERSION.to_owned(),
        status: CHINESE_ASTROLOGY_STRUCTURE_CATALOG_STATUS.to_owned(),
        three_enclosures: three_enclosure_catalog(),
        mansions: twenty_eight_mansion_catalog(),
        seven_luminaries: seven_luminary_catalog(),
        enclosure_boundary_model_selected: false,
        angular_boundary_model_selected: false,
        celestial_positions_calculated: false,
        interpretation_eligible: false,
        rule_eligible: false,
        blocked_reasons: vec![
            "three_enclosure_walls_member_asterisms_and_boundaries_not_selected".to_owned(),
            "historical_determinative_star_and_angular_boundary_model_not_selected".to_owned(),
            "seven_luminary_identity_only_positions_not_calculated".to_owned(),
            "guest_star_comet_eclipse_event_models_not_calculated".to_owned(),
            "omen_text_edition_passage_conditions_and_counterexamples_not_reviewed".to_owned(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn catalog_has_twenty_eight_unique_mansions_in_received_order() {
        let catalog = twenty_eight_mansion_catalog();
        assert_eq!(catalog.len(), 28);
        assert_eq!(catalog.first().unwrap().short_zh_name, "角");
        assert_eq!(catalog.last().unwrap().short_zh_name, "轸");
        assert_eq!(
            catalog
                .iter()
                .map(|item| item.mansion)
                .collect::<BTreeSet<_>>()
                .len(),
            28
        );
        assert_eq!(
            catalog
                .iter()
                .map(|item| item.received_sequence)
                .collect::<Vec<_>>(),
            (1..=28).collect::<Vec<_>>()
        );
    }

    #[test]
    fn every_quadrant_has_exactly_seven_received_mansions() {
        let catalog = twenty_eight_mansion_catalog();
        for quadrant in MansionQuadrant::ALL_IN_RECEIVED_ORDER {
            let entries = catalog
                .iter()
                .filter(|item| item.quadrant == quadrant)
                .collect::<Vec<_>>();
            assert_eq!(entries.len(), 7);
            assert_eq!(
                entries
                    .iter()
                    .map(|item| item.quadrant_sequence)
                    .collect::<Vec<_>>(),
                (1..=7).collect::<Vec<_>>()
            );
            assert!(entries
                .iter()
                .all(|item| item.four_emblem == quadrant.emblem()));
        }
    }

    #[test]
    fn exact_received_quadrant_sequences_are_fixed() {
        let catalog = twenty_eight_mansion_catalog();
        let names = |quadrant| {
            catalog
                .iter()
                .filter(|item| item.quadrant == quadrant)
                .map(|item| item.short_zh_name.as_str())
                .collect::<Vec<_>>()
                .join("")
        };
        assert_eq!(names(MansionQuadrant::East), "角亢氐房心尾箕");
        assert_eq!(names(MansionQuadrant::North), "斗牛女虚危室壁");
        assert_eq!(names(MansionQuadrant::West), "奎娄胃昴毕觜参");
        assert_eq!(names(MansionQuadrant::South), "井鬼柳星张翼轸");
    }

    #[test]
    fn audit_fails_closed_before_sky_boundaries_and_rules_exist() {
        let audit = chinese_astrology_structure_audit();
        assert_eq!(audit.audit_version, 2);
        assert_eq!(audit.three_enclosures.len(), 3);
        assert_eq!(audit.mansions.len(), 28);
        assert_eq!(audit.seven_luminaries.len(), 7);
        assert!(!audit.enclosure_boundary_model_selected);
        assert!(!audit.angular_boundary_model_selected);
        assert!(!audit.celestial_positions_calculated);
        assert!(!audit.interpretation_eligible);
        assert!(!audit.rule_eligible);
        assert_eq!(audit.blocked_reasons.len(), 5);
    }

    #[test]
    fn three_enclosures_preserve_received_palace_and_yuan_labels() {
        let catalog = three_enclosure_catalog();
        assert_eq!(catalog.len(), 3);
        assert_eq!(
            catalog
                .iter()
                .map(|item| item.zh_name.as_str())
                .collect::<Vec<_>>(),
            ["紫微垣", "太微垣", "天市垣"]
        );
        assert_eq!(catalog[0].received_palace_name, "紫微宫");
        assert_eq!(catalog[0].yuan_level, YuanLevel::Middle);
        assert_eq!(catalog[1].yuan_level, YuanLevel::Upper);
        assert_eq!(catalog[2].yuan_level, YuanLevel::Lower);
    }

    #[test]
    fn seven_luminaries_keep_classical_planet_names_and_phase_mapping() {
        let catalog = seven_luminary_catalog();
        assert_eq!(catalog.len(), 7);
        assert_eq!(
            catalog
                .iter()
                .map(|item| item.received_zh_name.as_str())
                .collect::<Vec<_>>(),
            ["日", "月", "岁星", "荧惑", "填星", "太白", "辰星"]
        );
        assert_eq!(catalog[0].five_phase, None);
        assert_eq!(catalog[1].five_phase, None);
        assert_eq!(catalog[2].five_phase, Some(FivePhase::Wood));
        assert_eq!(catalog[3].five_phase, Some(FivePhase::Fire));
        assert_eq!(catalog[4].five_phase, Some(FivePhase::Earth));
        assert_eq!(catalog[5].five_phase, Some(FivePhase::Metal));
        assert_eq!(catalog[6].five_phase, Some(FivePhase::Water));
    }
}
