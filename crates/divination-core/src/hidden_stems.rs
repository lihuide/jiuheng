use crate::{
    late_zi_dual_candidates, ten_god_for_visible_stems, EarthlyBranch, FourPillarsChartCandidate,
    FourPillarsPosition, HeavenlyStem, SexagenaryDesignation, TenGodIdentity,
};
use serde::{Deserialize, Serialize};

pub const HIDDEN_STEMS_ALGORITHM_VERSION: &str =
    "jiuheng.four_pillars.hidden_stems.song_order_day_master_relation.v1";
pub const HIDDEN_STEMS_CONVENTION: &str =
    "mingli_tanyuan_zhicang_renyuan_song_enumeration_no_weights.v1";

pub fn branch_hidden_stems(branch: EarthlyBranch) -> &'static [HeavenlyStem] {
    use HeavenlyStem::*;
    match branch {
        EarthlyBranch::Zi => &[Gui],
        EarthlyBranch::Chou => &[Gui, Xin, Ji],
        EarthlyBranch::Yin => &[Jia, Bing, Wu],
        EarthlyBranch::Mao => &[Yi],
        EarthlyBranch::Chen => &[Yi, Wu, Gui],
        EarthlyBranch::Si => &[Geng, Bing, Wu],
        EarthlyBranch::Wu => &[Ding, Ji],
        EarthlyBranch::Wei => &[Yi, Ji, Ding],
        EarthlyBranch::Shen => &[Geng, Ren, Wu],
        EarthlyBranch::You => &[Xin],
        EarthlyBranch::Xu => &[Xin, Ding, Wu],
        EarthlyBranch::Hai => &[Ren, Jia],
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HiddenStemObservation {
    pub stem: HeavenlyStem,
    pub ten_god: TenGodIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PillarHiddenStems {
    pub position: FourPillarsPosition,
    pub branch: EarthlyBranch,
    pub stems: Vec<HiddenStemObservation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FourPillarsHiddenStemAudit {
    pub algorithm_version: String,
    pub table_convention: String,
    pub day_convention: String,
    pub day_master: HeavenlyStem,
    pub pillars: Vec<PillarHiddenStems>,
    pub strength_assessed: bool,
    pub weights_assigned: bool,
    pub interpretation_eligible: bool,
    pub rule_eligible: bool,
}

pub fn four_pillars_hidden_stem_audits(
    chart: &FourPillarsChartCandidate,
) -> Vec<FourPillarsHiddenStemAudit> {
    let mut candidates = vec![(
        chart.day_convention.clone(),
        chart.day_pillar_true_solar_midnight,
        chart.hour_pillar_true_solar_midnight,
    )];
    if let Some(dual) = late_zi_dual_candidates(chart) {
        candidates.push((
            dual.zi_start_boundary.convention.clone(),
            dual.zi_start_boundary.day_pillar,
            dual.zi_start_boundary.hour_pillar,
        ));
    }
    candidates
        .into_iter()
        .map(|(convention, day, hour)| {
            hidden_for_pillars(
                [
                    chart.year_candidate.designation,
                    chart.month_candidate.designation,
                    day,
                    hour,
                ],
                convention,
            )
        })
        .collect()
}

fn hidden_for_pillars(
    pillars: [SexagenaryDesignation; 4],
    day_convention: String,
) -> FourPillarsHiddenStemAudit {
    let day_master = pillars[2].stem;
    let positions = [
        FourPillarsPosition::Year,
        FourPillarsPosition::Month,
        FourPillarsPosition::Day,
        FourPillarsPosition::Hour,
    ];
    FourPillarsHiddenStemAudit {
        algorithm_version: HIDDEN_STEMS_ALGORITHM_VERSION.into(),
        table_convention: HIDDEN_STEMS_CONVENTION.into(),
        day_convention,
        day_master,
        pillars: positions
            .into_iter()
            .zip(pillars)
            .map(|(position, pillar)| PillarHiddenStems {
                position,
                branch: pillar.branch,
                stems: branch_hidden_stems(pillar.branch)
                    .iter()
                    .map(|stem| HiddenStemObservation {
                        stem: *stem,
                        ten_god: ten_god_for_visible_stems(day_master, *stem),
                    })
                    .collect(),
            })
            .collect(),
        strength_assessed: false,
        weights_assigned: false,
        interpretation_eligible: false,
        rule_eligible: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twelve_branches_match_received_song_without_strength_order() {
        let expected = [
            "癸",
            "癸辛己",
            "甲丙戊",
            "乙",
            "乙戊癸",
            "庚丙戊",
            "丁己",
            "乙己丁",
            "庚壬戊",
            "辛",
            "辛丁戊",
            "壬甲",
        ];
        for (branch, expected) in EarthlyBranch::ALL.into_iter().zip(expected) {
            assert_eq!(
                branch_hidden_stems(branch)
                    .iter()
                    .map(|stem| stem.zh_name())
                    .collect::<String>(),
                expected
            );
        }
        assert_eq!(
            EarthlyBranch::ALL
                .iter()
                .map(|branch| branch_hidden_stems(*branch).len())
                .sum::<usize>(),
            28
        );
    }

    #[test]
    fn hidden_stem_relations_cover_all_day_masters_without_opening_interpretation() {
        for day_stem in HeavenlyStem::ALL {
            for branch in EarthlyBranch::ALL {
                let day = (0..60)
                    .map(SexagenaryDesignation::from_cycle_index)
                    .find(|day| day.stem == day_stem)
                    .unwrap();
                let pillar = (0..60)
                    .map(SexagenaryDesignation::from_cycle_index)
                    .find(|pillar| pillar.branch == branch)
                    .unwrap();
                let audit = hidden_for_pillars([pillar, pillar, day, pillar], "test".into());
                assert_eq!(audit.day_master, day_stem);
                assert_eq!(audit.pillars.len(), 4);
                assert_eq!(
                    audit.pillars[0].stems.len(),
                    branch_hidden_stems(branch).len()
                );
                assert!(
                    !(audit.weights_assigned
                        || audit.strength_assessed
                        || audit.interpretation_eligible
                        || audit.rule_eligible)
                );
                for stem in &audit.pillars[0].stems {
                    assert_eq!(stem.ten_god, ten_god_for_visible_stems(day_stem, stem.stem));
                }
            }
        }
    }

    #[test]
    fn fixed_ren_day_chart_labels_match_expected_hidden_relations() {
        let audit = hidden_for_pillars(
            [6, 17, 18, 40].map(SexagenaryDesignation::from_cycle_index),
            "test".into(),
        );
        let labels = audit
            .pillars
            .iter()
            .map(|pillar| {
                pillar
                    .stems
                    .iter()
                    .map(|item| format!("{}{}", item.stem.zh_name(), item.ten_god.zh_name()))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>();
        assert_eq!(
            labels,
            [
                "丁正财 己正官",
                "庚偏印 丙偏财 戊七杀",
                "丁正财 己正官",
                "乙伤官 戊七杀 癸劫财"
            ]
        );
    }
}
