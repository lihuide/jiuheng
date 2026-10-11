                                                                   
   
                                                                            
                                                                              
                                                          

use crate::{seven_luminary_catalog, FivePhase, SevenLuminaryIdentity};
use serde::{Deserialize, Serialize};

pub const QIZHENG_SIYU_IDENTITY_ALGORITHM_VERSION: &str =
    "jiuheng.qizheng_siyu.received_eleven_luminary_identity.v1";
pub const QIZHENG_SIYU_IDENTITY_STATUS: &str =
    "mechanical_received_seven_governors_four_residuals_identity_only.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FourResidual {
    PurpleQi,
    LunarBo,
    Rahu,
    Ketu,
}

impl FourResidual {
    pub const ALL_IN_RECEIVED_CATALOG_ORDER: [Self; 4] =
        [Self::PurpleQi, Self::LunarBo, Self::Rahu, Self::Ketu];

    pub const fn normalized_zh_name(self) -> &'static str {
        match self {
            Self::PurpleQi => "紫气",
            Self::LunarBo => "月孛",
            Self::Rahu => "罗睺",
            Self::Ketu => "计都",
        }
    }

    pub const fn five_phase(self) -> FivePhase {
        match self {
            Self::PurpleQi => FivePhase::Wood,
            Self::LunarBo => FivePhase::Water,
            Self::Rahu => FivePhase::Fire,
            Self::Ketu => FivePhase::Earth,
        }
    }

    pub const fn catalog_sequence(self) -> u8 {
        self as u8 + 1
    }

    pub fn received_variant_zh_names(self) -> Vec<String> {
        match self {
            Self::PurpleQi => vec!["紫气".to_owned(), "紫炁".to_owned()],
            Self::LunarBo => vec!["月孛".to_owned()],
            Self::Rahu => vec!["罗睺".to_owned(), "罗㬋".to_owned(), "罗喉".to_owned()],
            Self::Ketu => vec!["计都".to_owned()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FourResidualIdentity {
    pub residual: FourResidual,
    pub catalog_sequence: u8,
    pub normalized_zh_name: String,
    pub received_variant_zh_names: Vec<String>,
    pub five_phase: FivePhase,
    pub modern_astronomical_role: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QizhengSiyuIdentityAudit {
    pub audit_version: u16,
    pub algorithm_version: String,
    pub status: String,
    pub seven_governors: Vec<SevenLuminaryIdentity>,
    pub four_residuals: Vec<FourResidualIdentity>,
    pub residual_astronomical_roles_selected: bool,
    pub planetary_position_algorithm_selected: bool,
    pub mansion_boundary_model_selected: bool,
    pub twelve_house_system_selected: bool,
    pub positions_calculated: bool,
    pub natal_chart_generated: bool,
    pub interpretation_eligible: bool,
    pub rule_eligible: bool,
    pub blocked_reasons: Vec<String>,
}

pub fn four_residual_identity_catalog() -> Vec<FourResidualIdentity> {
    FourResidual::ALL_IN_RECEIVED_CATALOG_ORDER
        .into_iter()
        .map(|residual| FourResidualIdentity {
            residual,
            catalog_sequence: residual.catalog_sequence(),
            normalized_zh_name: residual.normalized_zh_name().to_owned(),
            received_variant_zh_names: residual.received_variant_zh_names(),
            five_phase: residual.five_phase(),
            modern_astronomical_role: None,
        })
        .collect()
}

pub fn qizheng_siyu_identity_audit() -> QizhengSiyuIdentityAudit {
    QizhengSiyuIdentityAudit {
        audit_version: 1,
        algorithm_version: QIZHENG_SIYU_IDENTITY_ALGORITHM_VERSION.to_owned(),
        status: QIZHENG_SIYU_IDENTITY_STATUS.to_owned(),
        seven_governors: seven_luminary_catalog(),
        four_residuals: four_residual_identity_catalog(),
        residual_astronomical_roles_selected: false,
        planetary_position_algorithm_selected: false,
        mansion_boundary_model_selected: false,
        twelve_house_system_selected: false,
        positions_calculated: false,
        natal_chart_generated: false,
        interpretation_eligible: false,
        rule_eligible: false,
        blocked_reasons: vec![
            "four_residual_modern_astronomical_roles_have_historical_variants".to_owned(),
            "offline_planetary_and_residual_position_algorithms_not_selected".to_owned(),
            "historical_mansion_boundaries_and_twelve_house_system_not_selected".to_owned(),
            "natal_rules_editions_conditions_counterexamples_and_tests_not_reviewed".to_owned(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn identity_audit_has_exactly_seven_governors_and_four_residuals() {
        let audit = qizheng_siyu_identity_audit();
        assert_eq!(audit.seven_governors.len(), 7);
        assert_eq!(audit.four_residuals.len(), 4);
        assert_eq!(
            audit
                .four_residuals
                .iter()
                .map(|item| item.residual)
                .collect::<BTreeSet<_>>()
                .len(),
            4
        );
        assert_eq!(
            audit
                .four_residuals
                .iter()
                .map(|item| item.normalized_zh_name.as_str())
                .collect::<Vec<_>>(),
            ["紫气", "月孛", "罗睺", "计都"]
        );
    }

    #[test]
    fn four_residual_phase_labels_and_variants_are_explicit() {
        let catalog = four_residual_identity_catalog();
        assert_eq!(catalog[0].five_phase, FivePhase::Wood);
        assert_eq!(catalog[1].five_phase, FivePhase::Water);
        assert_eq!(catalog[2].five_phase, FivePhase::Fire);
        assert_eq!(catalog[3].five_phase, FivePhase::Earth);
        assert!(catalog[0]
            .received_variant_zh_names
            .contains(&"紫炁".to_owned()));
        assert!(catalog[2]
            .received_variant_zh_names
            .contains(&"罗㬋".to_owned()));
        assert!(catalog
            .iter()
            .all(|item| item.modern_astronomical_role.is_none()));
    }

    #[test]
    fn audit_fails_closed_before_positions_houses_and_rules_exist() {
        let audit = qizheng_siyu_identity_audit();
        assert!(!audit.residual_astronomical_roles_selected);
        assert!(!audit.planetary_position_algorithm_selected);
        assert!(!audit.mansion_boundary_model_selected);
        assert!(!audit.twelve_house_system_selected);
        assert!(!audit.positions_calculated);
        assert!(!audit.natal_chart_generated);
        assert!(!audit.interpretation_eligible);
        assert!(!audit.rule_eligible);
        assert_eq!(audit.blocked_reasons.len(), 4);
    }
}
