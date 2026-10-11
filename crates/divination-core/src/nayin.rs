                                               
   
                                                                              
                                                   

use crate::{FiveElement, SexagenaryDesignation};
use serde::{Deserialize, Serialize};

pub const NAYIN_CATALOG_ALGORITHM_VERSION: &str = "jiuheng.nayin.received_sixty_designations.v1";
pub const NAYIN_CATALOG_STATUS: &str =
    "mechanical_received_catalog_no_interpretation_or_runtime_signals.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NayinDesignation {
                                                          
    pub pair_index: u8,
    pub received_zh_name: String,
    pub five_element: FiveElement,
    pub algorithm_version: String,
    pub status: String,
    pub interpretation_eligible: bool,
}

const NAYIN_PAIRS: [(&str, FiveElement); 30] = [
    ("海中金", FiveElement::Metal),
    ("炉中火", FiveElement::Fire),
    ("大林木", FiveElement::Wood),
    ("路旁土", FiveElement::Earth),
    ("剑锋金", FiveElement::Metal),
    ("山头火", FiveElement::Fire),
    ("涧下水", FiveElement::Water),
    ("城头土", FiveElement::Earth),
    ("白蜡金", FiveElement::Metal),
    ("杨柳木", FiveElement::Wood),
    ("泉中水", FiveElement::Water),
    ("屋上土", FiveElement::Earth),
    ("霹雳火", FiveElement::Fire),
    ("松柏木", FiveElement::Wood),
    ("长流水", FiveElement::Water),
    ("沙中金", FiveElement::Metal),
    ("山下火", FiveElement::Fire),
    ("平地木", FiveElement::Wood),
    ("壁上土", FiveElement::Earth),
    ("金箔金", FiveElement::Metal),
    ("覆灯火", FiveElement::Fire),
    ("天河水", FiveElement::Water),
    ("大驿土", FiveElement::Earth),
    ("钗钏金", FiveElement::Metal),
    ("桑柘木", FiveElement::Wood),
    ("大溪水", FiveElement::Water),
    ("沙中土", FiveElement::Earth),
    ("天上火", FiveElement::Fire),
    ("石榴木", FiveElement::Wood),
    ("大海水", FiveElement::Water),
];

pub fn nayin_for_designation(designation: SexagenaryDesignation) -> NayinDesignation {
    let pair_index = designation.cycle_index / 2;
    let (received_zh_name, five_element) = NAYIN_PAIRS[pair_index as usize];
    NayinDesignation {
        pair_index,
        received_zh_name: received_zh_name.to_owned(),
        five_element,
        algorithm_version: NAYIN_CATALOG_ALGORITHM_VERSION.to_owned(),
        status: NAYIN_CATALOG_STATUS.to_owned(),
        interpretation_eligible: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_consecutive_pair_shares_one_of_thirty_nayin_entries() {
        for pair_index in 0..30 {
            let first =
                nayin_for_designation(SexagenaryDesignation::from_cycle_index(pair_index * 2));
            let second =
                nayin_for_designation(SexagenaryDesignation::from_cycle_index(pair_index * 2 + 1));
            assert_eq!(first, second);
            assert_eq!(first.pair_index, pair_index as u8);
            assert!(!first.interpretation_eligible);
        }
    }

    #[test]
    fn matches_received_ziwei_examples_and_endpoints() {
        let jia_zi = nayin_for_designation(SexagenaryDesignation::from_cycle_index(0));
        assert_eq!(jia_zi.received_zh_name, "海中金");
        assert_eq!(jia_zi.five_element, FiveElement::Metal);

        let bing_yin = nayin_for_designation(SexagenaryDesignation::from_cycle_index(2));
        assert_eq!(bing_yin.received_zh_name, "炉中火");
        assert_eq!(bing_yin.five_element, FiveElement::Fire);

        let wu_chen = nayin_for_designation(SexagenaryDesignation::from_cycle_index(4));
        assert_eq!(wu_chen.received_zh_name, "大林木");
        assert_eq!(wu_chen.five_element, FiveElement::Wood);

        let gui_hai = nayin_for_designation(SexagenaryDesignation::from_cycle_index(59));
        assert_eq!(gui_hai.received_zh_name, "大海水");
        assert_eq!(gui_hai.five_element, FiveElement::Water);
    }

    #[test]
    fn each_element_occupies_six_nayin_pairs() {
        for element in [
            FiveElement::Wood,
            FiveElement::Fire,
            FiveElement::Earth,
            FiveElement::Metal,
            FiveElement::Water,
        ] {
            assert_eq!(
                NAYIN_PAIRS
                    .iter()
                    .filter(|(_, candidate)| *candidate == element)
                    .count(),
                6
            );
        }
    }
}
