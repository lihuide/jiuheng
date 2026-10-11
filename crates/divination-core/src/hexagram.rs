                                                                             
                                                                            

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const HEXAGRAM_CATALOG_ALGORITHM_VERSION: &str =
    "jiuheng.zhouyi.received_hexagram_structure.v1";
pub const HEXAGRAM_CATALOG_STATUS: &str =
    "mechanical_identity_only_no_judgment_text_or_interpretation.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trigram {
    Qian,
    Dui,
    Li,
    Zhen,
    Xun,
    Kan,
    Gen,
    Kun,
}

impl Trigram {
    pub const ALL_IN_RECEIVED_NUMBER_ORDER: [Self; 8] = [
        Self::Qian,
        Self::Dui,
        Self::Li,
        Self::Zhen,
        Self::Xun,
        Self::Kan,
        Self::Gen,
        Self::Kun,
    ];

    pub const fn received_number(self) -> u8 {
        self.catalog_index() as u8 + 1
    }

    pub const fn catalog_index(self) -> usize {
        match self {
            Self::Qian => 0,
            Self::Dui => 1,
            Self::Li => 2,
            Self::Zhen => 3,
            Self::Xun => 4,
            Self::Kan => 5,
            Self::Gen => 6,
            Self::Kun => 7,
        }
    }

    pub const fn bits_bottom_up(self) -> u8 {
        match self {
            Self::Qian => 0b111,
            Self::Dui => 0b011,
            Self::Li => 0b101,
            Self::Zhen => 0b001,
            Self::Xun => 0b110,
            Self::Kan => 0b010,
            Self::Gen => 0b100,
            Self::Kun => 0b000,
        }
    }

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Qian => "乾",
            Self::Dui => "兑",
            Self::Li => "离",
            Self::Zhen => "震",
            Self::Xun => "巽",
            Self::Kan => "坎",
            Self::Gen => "艮",
            Self::Kun => "坤",
        }
    }

    pub const fn from_received_number(number: u8) -> Option<Self> {
        if number == 0 || number > 8 {
            return None;
        }
        Some(Self::ALL_IN_RECEIVED_NUMBER_ORDER[(number - 1) as usize])
    }

    pub const fn from_bits_bottom_up(bits: u8) -> Option<Self> {
        match bits {
            0b111 => Some(Self::Qian),
            0b011 => Some(Self::Dui),
            0b101 => Some(Self::Li),
            0b001 => Some(Self::Zhen),
            0b110 => Some(Self::Xun),
            0b010 => Some(Self::Kan),
            0b100 => Some(Self::Gen),
            0b000 => Some(Self::Kun),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HexagramIdentity {
                                                       
    pub bits_bottom_up: u8,
    pub lower_trigram: Trigram,
    pub upper_trigram: Trigram,
    pub received_sequence: u8,
    pub name_zh_simplified: String,
    pub unicode_symbol: char,
    pub unicode_code_point: u32,
    pub algorithm_version: String,
    pub status: String,
    pub interpretation_eligible: bool,
}

impl HexagramIdentity {
    pub fn compact_zh(&self) -> String {
        format!(
            "{} 第{}卦 {}（上{}下{}）",
            self.unicode_symbol,
            self.received_sequence,
            self.name_zh_simplified,
            self.upper_trigram.zh_name(),
            self.lower_trigram.zh_name()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HexagramTransformation {
    pub primary: HexagramIdentity,
    pub changed: HexagramIdentity,
    pub algorithm_version: String,
    pub status: String,
    pub interpretation_eligible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum HexagramCatalogError {
    #[error("六爻结构位图必须在 0 至 63 之间，实际为 {0}")]
    InvalidSixLineBits(u8),
}

#[derive(Clone, Copy)]
struct CatalogEntry {
    received_sequence: u8,
    name_zh_simplified: &'static str,
}

const fn entry(received_sequence: u8, name_zh_simplified: &'static str) -> CatalogEntry {
    CatalogEntry {
        received_sequence,
        name_zh_simplified,
    }
}

                                                                         
                          
const CATALOG_BY_LOWER_UPPER: [[CatalogEntry; 8]; 8] = [
    [
        entry(1, "乾"),
        entry(43, "夬"),
        entry(14, "大有"),
        entry(34, "大壮"),
        entry(9, "小畜"),
        entry(5, "需"),
        entry(26, "大畜"),
        entry(11, "泰"),
    ],
    [
        entry(10, "履"),
        entry(58, "兑"),
        entry(38, "睽"),
        entry(54, "归妹"),
        entry(61, "中孚"),
        entry(60, "节"),
        entry(41, "损"),
        entry(19, "临"),
    ],
    [
        entry(13, "同人"),
        entry(49, "革"),
        entry(30, "离"),
        entry(55, "丰"),
        entry(37, "家人"),
        entry(63, "既济"),
        entry(22, "贲"),
        entry(36, "明夷"),
    ],
    [
        entry(25, "无妄"),
        entry(17, "随"),
        entry(21, "噬嗑"),
        entry(51, "震"),
        entry(42, "益"),
        entry(3, "屯"),
        entry(27, "颐"),
        entry(24, "复"),
    ],
    [
        entry(44, "姤"),
        entry(28, "大过"),
        entry(50, "鼎"),
        entry(32, "恒"),
        entry(57, "巽"),
        entry(48, "井"),
        entry(18, "蛊"),
        entry(46, "升"),
    ],
    [
        entry(6, "讼"),
        entry(47, "困"),
        entry(64, "未济"),
        entry(40, "解"),
        entry(59, "涣"),
        entry(29, "坎"),
        entry(4, "蒙"),
        entry(7, "师"),
    ],
    [
        entry(33, "遯"),
        entry(31, "咸"),
        entry(56, "旅"),
        entry(62, "小过"),
        entry(53, "渐"),
        entry(39, "蹇"),
        entry(52, "艮"),
        entry(15, "谦"),
    ],
    [
        entry(12, "否"),
        entry(45, "萃"),
        entry(35, "晋"),
        entry(16, "豫"),
        entry(20, "观"),
        entry(8, "比"),
        entry(23, "剥"),
        entry(2, "坤"),
    ],
];

pub fn hexagram_identity(bits_bottom_up: u8) -> Result<HexagramIdentity, HexagramCatalogError> {
    if bits_bottom_up > 0b11_1111 {
        return Err(HexagramCatalogError::InvalidSixLineBits(bits_bottom_up));
    }
    let lower_trigram = Trigram::from_bits_bottom_up(bits_bottom_up & 0b111)
        .expect("every three-bit value is one trigram");
    let upper_trigram = Trigram::from_bits_bottom_up((bits_bottom_up >> 3) & 0b111)
        .expect("every three-bit value is one trigram");
    let catalog =
        CATALOG_BY_LOWER_UPPER[lower_trigram.catalog_index()][upper_trigram.catalog_index()];
    let unicode_code_point = 0x4DBF + u32::from(catalog.received_sequence);
    let unicode_symbol = char::from_u32(unicode_code_point)
        .expect("the received sequence always maps into the Unicode Yijing block");

    Ok(HexagramIdentity {
        bits_bottom_up,
        lower_trigram,
        upper_trigram,
        received_sequence: catalog.received_sequence,
        name_zh_simplified: catalog.name_zh_simplified.to_owned(),
        unicode_symbol,
        unicode_code_point,
        algorithm_version: HEXAGRAM_CATALOG_ALGORITHM_VERSION.to_owned(),
        status: HEXAGRAM_CATALOG_STATUS.to_owned(),
        interpretation_eligible: false,
    })
}

pub fn hexagram_transformation(
    primary_bits: u8,
    changed_bits: u8,
) -> Result<HexagramTransformation, HexagramCatalogError> {
    Ok(HexagramTransformation {
        primary: hexagram_identity(primary_bits)?,
        changed: hexagram_identity(changed_bits)?,
        algorithm_version: HEXAGRAM_CATALOG_ALGORITHM_VERSION.to_owned(),
        status: HEXAGRAM_CATALOG_STATUS.to_owned(),
        interpretation_eligible: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn all_sixty_four_bit_patterns_map_bijectively_to_received_sequence_and_unicode() {
        let mut sequences = BTreeSet::new();
        let mut names = BTreeSet::new();
        let mut symbols = BTreeSet::new();
        for bits in 0..64 {
            let identity = hexagram_identity(bits).unwrap();
            assert_eq!(
                identity.bits_bottom_up,
                identity.lower_trigram.bits_bottom_up()
                    | (identity.upper_trigram.bits_bottom_up() << 3)
            );
            assert_eq!(
                identity.unicode_code_point,
                0x4DBF + u32::from(identity.received_sequence)
            );
            assert!(sequences.insert(identity.received_sequence));
            assert!(names.insert(identity.name_zh_simplified));
            assert!(symbols.insert(identity.unicode_symbol));
            assert!(!identity.interpretation_eligible);
        }
        assert_eq!(sequences, (1..=64).collect());
        assert_eq!(names.len(), 64);
        assert_eq!(symbols.len(), 64);
    }

    #[test]
    fn anchors_received_pairs_and_line_direction() {
        let qian = hexagram_identity(0b111111).unwrap();
        assert_eq!(
            (qian.received_sequence, qian.name_zh_simplified),
            (1, "乾".to_owned())
        );
        assert_eq!(qian.unicode_symbol, '䷀');

        let kun = hexagram_identity(0b000000).unwrap();
        assert_eq!(
            (kun.received_sequence, kun.name_zh_simplified),
            (2, "坤".to_owned())
        );
        assert_eq!(kun.unicode_symbol, '䷁');

        let zhun = hexagram_identity(0b010001).unwrap();
        assert_eq!(
            (zhun.received_sequence, zhun.name_zh_simplified),
            (3, "屯".to_owned())
        );
        assert_eq!(
            (zhun.upper_trigram, zhun.lower_trigram),
            (Trigram::Kan, Trigram::Zhen)
        );

        let tai = hexagram_identity(0b000111).unwrap();
        let pi = hexagram_identity(0b111000).unwrap();
        assert_eq!(
            (tai.received_sequence, tai.name_zh_simplified),
            (11, "泰".to_owned())
        );
        assert_eq!(
            (pi.received_sequence, pi.name_zh_simplified),
            (12, "否".to_owned())
        );

        let jiji = hexagram_identity(0b010101).unwrap();
        let weiji = hexagram_identity(0b101010).unwrap();
        assert_eq!(
            (jiji.received_sequence, jiji.name_zh_simplified),
            (63, "既济".to_owned())
        );
        assert_eq!(
            (weiji.received_sequence, weiji.name_zh_simplified),
            (64, "未济".to_owned())
        );
    }

    #[test]
    fn meihua_example_structure_maps_to_ge_then_xian() {
        let transformation = hexagram_transformation(0b011101, 0b011100).unwrap();
        assert_eq!(transformation.primary.name_zh_simplified, "革");
        assert_eq!(transformation.changed.name_zh_simplified, "咸");
        assert!(!transformation.interpretation_eligible);
    }

    #[test]
    fn rejects_values_that_are_not_six_bit_structures() {
        assert_eq!(
            hexagram_identity(64).unwrap_err(),
            HexagramCatalogError::InvalidSixLineBits(64)
        );
        assert_eq!(
            hexagram_transformation(0, 255).unwrap_err(),
            HexagramCatalogError::InvalidSixLineBits(255)
        );
    }
}
