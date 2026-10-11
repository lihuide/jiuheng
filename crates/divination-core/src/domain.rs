use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DivinationMethod {
    YarrowStalk,
    ChineseAstrology,
    Taiyi,
    QimenDunjia,
    Liuyao,
    MeihuaYishu,
    FourPillars,
    ZiweiDoushu,
    QizhengSiyu,
}

impl DivinationMethod {
    pub const ALL: [Self; 9] = [
        Self::YarrowStalk,
        Self::ChineseAstrology,
        Self::Taiyi,
        Self::QimenDunjia,
        Self::Liuyao,
        Self::MeihuaYishu,
        Self::FourPillars,
        Self::ZiweiDoushu,
        Self::QizhengSiyu,
    ];

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::YarrowStalk => "蓍草筮占",
            Self::ChineseAstrology => "中国古代星占",
            Self::Taiyi => "太乙神数",
            Self::QimenDunjia => "奇门遁甲",
            Self::Liuyao => "六爻",
            Self::MeihuaYishu => "梅花易数",
            Self::FourPillars => "四柱八字",
            Self::ZiweiDoushu => "紫微斗数",
            Self::QizhengSiyu => "七政四余",
        }
    }

    pub const fn key(self) -> &'static str {
        match self {
            Self::YarrowStalk => "yarrow_stalk",
            Self::ChineseAstrology => "chinese_astrology",
            Self::Taiyi => "taiyi",
            Self::QimenDunjia => "qimen_dunjia",
            Self::Liuyao => "liuyao",
            Self::MeihuaYishu => "meihua_yishu",
            Self::FourPillars => "four_pillars",
            Self::ZiweiDoushu => "ziwei_doushu",
            Self::QizhengSiyu => "qizheng_siyu",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionCategory {
    Relationship,
    Career,
    Wealth,
    Weather,
    Travel,
    Search,
    Competition,
    Study,
    Cooperation,
    Dispute,
    Home,
    LongTermFate,
    HealthReference,
    Other,
}

impl QuestionCategory {
    pub const ALL: [Self; 14] = [
        Self::Relationship,
        Self::Career,
        Self::Wealth,
        Self::Weather,
        Self::Travel,
        Self::Search,
        Self::Competition,
        Self::Study,
        Self::Cooperation,
        Self::Dispute,
        Self::Home,
        Self::LongTermFate,
        Self::HealthReference,
        Self::Other,
    ];

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Relationship => "感情",
            Self::Career => "事业",
            Self::Wealth => "财运",
            Self::Weather => "天气",
            Self::Travel => "出行",
            Self::Search => "寻人寻物",
            Self::Competition => "比赛胜负",
            Self::Study => "学业考试",
            Self::Cooperation => "合作交易",
            Self::Dispute => "官非纠纷",
            Self::Home => "家宅",
            Self::LongTermFate => "长期命运",
            Self::HealthReference => "健康参考",
            Self::Other => "其他",
        }
    }

    pub const fn key(self) -> &'static str {
        match self {
            Self::Relationship => "relationship",
            Self::Career => "career",
            Self::Wealth => "wealth",
            Self::Weather => "weather",
            Self::Travel => "travel",
            Self::Search => "search",
            Self::Competition => "competition",
            Self::Study => "study",
            Self::Cooperation => "cooperation",
            Self::Dispute => "dispute",
            Self::Home => "home",
            Self::LongTermFate => "long_term_fate",
            Self::HealthReference => "health_reference",
            Self::Other => "other",
        }
    }

                                                                        
                                                                               
                                             
    pub const fn practical_action_notice(self) -> &'static str {
        match self {
            Self::Relationship => {
                "先核实事实并与当事人直接沟通，尊重对方意愿和边界；不要仅依据文化推演作出结婚、分手等重大决定。"
            }
            Self::Career => {
                "先确认目标、期限、资源和可逆的下一步，并以真实工作信息和专业判断作决定。"
            }
            Self::Wealth => {
                "先核实现金流、合同和最坏损失；不要仅依据文化推演投资、借贷、担保或处置重要资产。"
            }
            Self::Weather => {
                "出门前以官方气象预警和现场情况为准，必要时准备雨具并调整行程。"
            }
            Self::Travel => {
                "出行以官方天气、交通和安全信息为准，预留备选路线并告知可信联系人。"
            }
            Self::Search => {
                "涉及人员失联或现实危险时，应立即联系警方、救援机构或相关管理方，不要等待推演结果。"
            }
            Self::Competition => {
                "以正式赛制、实时信息和实际准备为准，不要用推演代替训练、风控或赛场判断。"
            }
            Self::Study => {
                "按官方考试安排制定复习和赴考计划，优先完成最可控的准备事项。"
            }
            Self::Cooperation => {
                "先核验对方资质、关键条款、付款和退出机制，重要约定应形成可追溯记录。"
            }
            Self::Dispute => {
                "注意证据和法定期限；重要法律决定应咨询有资质的专业人士，不要仅依据文化推演行动。"
            }
            Self::Home => {
                "先排查消防、燃气、电气、结构和人身安全等现实风险，紧急隐患应立即联系专业机构。"
            }
            Self::LongTermFate => {
                "把结果视为文化参考，长期选择仍应依据个人目标、现实条件和可验证的信息。"
            }
            Self::HealthReference => {
                "如有不适、危险症状或病情变化，请及时就医；本软件不能用于诊断、治疗选择、停药或延误就医。"
            }
            Self::Other => {
                "先核实与问题相关的现实信息和风险；涉及医疗、法律、财务或人身安全时应咨询相应专业机构。"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AncientQuestionType {
    RelationshipOutcome,
    ProjectOutcome,
    ProfitAndLoss,
    WeatherRain,
    TravelSafety,
    MissingPersonOrObject,
    CompetitionOutcome,
    ExaminationOutcome,
    TransactionOutcome,
    LitigationOutcome,
    DwellingFortune,
    NatalLongTerm,
    HealthTrendReference,
}

impl AncientQuestionType {
    pub const fn zh_name(&self) -> &'static str {
        match self {
            Self::RelationshipOutcome => "感情关系",
            Self::ProjectOutcome => "项目成败",
            Self::ProfitAndLoss => "财利得失",
            Self::WeatherRain => "天气雨雪",
            Self::TravelSafety => "出行",
            Self::MissingPersonOrObject => "寻人寻物",
            Self::CompetitionOutcome => "比赛胜负",
            Self::ExaminationOutcome => "学业考试",
            Self::TransactionOutcome => "合作交易",
            Self::LitigationOutcome => "官非纠纷",
            Self::DwellingFortune => "家宅",
            Self::NatalLongTerm => "长期命理",
            Self::HealthTrendReference => "健康参考",
        }
    }
}

                                                                            
                                                                            
pub fn practical_action_notice_for_classified_question(
    category: QuestionCategory,
    ancient_type: Option<&AncientQuestionType>,
) -> &'static str {
    if category != QuestionCategory::Other {
        return category.practical_action_notice();
    }
    match ancient_type {
        Some(AncientQuestionType::RelationshipOutcome) => {
            QuestionCategory::Relationship.practical_action_notice()
        }
        Some(AncientQuestionType::ProjectOutcome) => {
            QuestionCategory::Career.practical_action_notice()
        }
        Some(AncientQuestionType::ProfitAndLoss) => {
            QuestionCategory::Wealth.practical_action_notice()
        }
        Some(AncientQuestionType::WeatherRain) => {
            QuestionCategory::Weather.practical_action_notice()
        }
        Some(AncientQuestionType::TravelSafety) => {
            QuestionCategory::Travel.practical_action_notice()
        }
        Some(AncientQuestionType::MissingPersonOrObject) => {
            QuestionCategory::Search.practical_action_notice()
        }
        Some(AncientQuestionType::CompetitionOutcome) => {
            QuestionCategory::Competition.practical_action_notice()
        }
        Some(AncientQuestionType::ExaminationOutcome) => {
            QuestionCategory::Study.practical_action_notice()
        }
        Some(AncientQuestionType::TransactionOutcome) => {
            QuestionCategory::Cooperation.practical_action_notice()
        }
        Some(AncientQuestionType::LitigationOutcome) => {
            QuestionCategory::Dispute.practical_action_notice()
        }
        Some(AncientQuestionType::DwellingFortune) => {
            QuestionCategory::Home.practical_action_notice()
        }
        Some(AncientQuestionType::NatalLongTerm) => {
            QuestionCategory::LongTermFate.practical_action_notice()
        }
        Some(AncientQuestionType::HealthTrendReference) => {
            QuestionCategory::HealthReference.practical_action_notice()
        }
        None => QuestionCategory::Other.practical_action_notice(),
    }
}

pub fn classified_question_requires_health_notice(
    category: QuestionCategory,
    ancient_type: Option<&AncientQuestionType>,
) -> bool {
    category == QuestionCategory::HealthReference
        || matches!(
            ancient_type,
            Some(AncientQuestionType::HealthTrendReference)
        )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    pub country_code: String,
    pub province: String,
    pub city: String,
    pub district: Option<String>,
    pub street: Option<String>,
    pub longitude_deg: f64,
    pub latitude_deg: f64,
    pub coordinate_precision: CoordinatePrecision,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinatePrecision {
    CityCenter,
    DistrictCenter,
    StreetCenter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionInput {
    pub id: Uuid,
    pub category: QuestionCategory,
    pub subcategory: String,
    pub text: String,
    pub subject: String,
    pub object: String,
    pub target_event: String,
    pub result_terms: Vec<String>,
    pub options: Vec<String>,
    pub location: GeoLocation,
    pub divination_time_bjt: DateTime<FixedOffset>,
    pub target_time_text: String,
    pub birth: Option<BirthParameters>,
    pub locked_at_bjt: DateTime<FixedOffset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirthParameters {
    pub sex: BiologicalSex,
    pub date_time_bjt: DateTime<FixedOffset>,
    pub location: GeoLocation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BiologicalSex {
    Female,
    Male,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceTier {
    EarlyCoreClassic,
    EarlyAuthoritativeCommentary,
    LaterAuthoritativeCommentary,
    Unverified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleStability {
    Stable,
    Disputed,
    Fragmentary,
    Unreviewed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionPolarity {
    Do,
    Avoid,
}

impl ActionPolarity {
    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Do => "建议",
            Self::Avoid => "避免",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleActionAdvice {
    pub rule_id: String,
    pub action_key: String,
    pub polarity: ActionPolarity,
    pub text: String,
    pub evidence_tier: EvidenceTier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleRecord {
    pub rule_id: String,
    pub method: DivinationMethod,
    pub applicable_question_types: Vec<AncientQuestionType>,
    pub condition_expression: String,
    pub result_expression: String,
    pub timing_expression: Option<String>,
    #[serde(default)]
    pub action_key: Option<String>,
    #[serde(default)]
    pub action_polarity: Option<ActionPolarity>,
    #[serde(default)]
    pub action_expression: Option<String>,
    pub priority: i32,
    pub evidence_tier: EvidenceTier,
    pub historical_period: String,
    pub book_title: String,
    pub volume: String,
    pub chapter: String,
    pub original_text: String,
    pub edition: String,
    pub source_reference: String,
    pub textual_variants: String,
    pub stability: RuleStability,
    pub implementation_version: String,
    pub reviewed: bool,
}

impl RuleRecord {
    pub fn may_execute(&self) -> bool {
        self.reviewed
            && self.stability == RuleStability::Stable
            && self.evidence_tier != EvidenceTier::Unverified
            && !self.original_text.trim().is_empty()
            && !self.source_reference.trim().is_empty()
            && self.action_metadata_valid()
    }

    pub fn action_metadata_valid(&self) -> bool {
        match (
            self.action_key.as_deref(),
            self.action_polarity,
            self.action_expression.as_deref(),
        ) {
            (None, None, None) => true,
            (Some(key), Some(_), Some(expression)) => {
                valid_action_key(key) && !expression.trim().is_empty()
            }
            _ => false,
        }
    }

    pub fn action_advice(&self) -> Option<RuleActionAdvice> {
        if !self.action_metadata_valid() {
            return None;
        }
        Some(RuleActionAdvice {
            rule_id: self.rule_id.clone(),
            action_key: self.action_key.clone()?,
            polarity: self.action_polarity?,
            text: self.action_expression.clone()?,
            evidence_tier: self.evidence_tier,
        })
    }
}

pub fn valid_action_key(value: &str) -> bool {
    let bytes = value.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= 96
        && bytes
            .first()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && bytes
            .last()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImplementationState {
    ResearchPending,
    ChartOnly,
    VerifiedRules,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicabilityStatus {
    Applicable,
    InsufficientEvidence,
    MissingParameters,
    UnsupportedQuestionType,
    NotImplemented,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Applicability {
    pub method: DivinationMethod,
    pub status: ApplicabilityStatus,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IntermediateSignals {
    pub trend: Option<i16>,
    pub subject_strength: Option<i16>,
    pub object_strength: Option<i16>,
    pub obstacle: Option<i16>,
    pub support: Option<i16>,
    pub outcome_direction: Option<i16>,
    pub change_speed: Option<i16>,
    pub active_passive: Option<i16>,
    pub gain_loss: Option<i16>,
    pub auspiciousness: Option<i16>,
    pub stability: Option<i16>,
    pub external_interference: Option<i16>,
    pub timing: Option<TimingWindow>,
                                                                               
                                                                      
    #[serde(default)]
    pub option_scores: BTreeMap<String, i16>,
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimingWindow {
    WithinOneHour,
    WithinThreeHours,
    Today,
    WithinThreeDays,
    WithinSevenDays,
    WithinHalfMonth,
    WithinOneMonth,
    WithinThreeMonths,
    WithinHalfYear,
    WithinOneYear,
    Indeterminate,
}

impl TimingWindow {
    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::WithinOneHour => "1小时内",
            Self::WithinThreeHours => "3小时内",
            Self::Today => "今日内",
            Self::WithinThreeDays => "3天内",
            Self::WithinSevenDays => "7天内",
            Self::WithinHalfMonth => "半个月内",
            Self::WithinOneMonth => "1个月内",
            Self::WithinThreeMonths => "3个月内",
            Self::WithinHalfYear => "半年内",
            Self::WithinOneYear => "1年内",
            Self::Indeterminate => "暂无法断定",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodResult {
    pub method: DivinationMethod,
    pub signals: IntermediateSignals,
    pub chart: serde_json::Value,
    pub hit_rule_ids: Vec<String>,
    #[serde(default)]
    pub action_advice: Vec<RuleActionAdvice>,
    pub evidence_tier: EvidenceTier,
    pub trace: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRevision {
    pub id: Uuid,
    pub prior_question_id: Uuid,
    pub change_description: String,
    pub created_at_bjt: DateTime<FixedOffset>,
    #[serde(default)]
    pub match_report: Option<crate::EventMatchReport>,
}

#[cfg(test)]
mod tests {
    use super::{
        classified_question_requires_health_notice,
        practical_action_notice_for_classified_question, valid_action_key, AncientQuestionType,
        QuestionCategory,
    };

    #[test]
    fn every_question_category_has_a_practical_action_notice() {
        for category in QuestionCategory::ALL {
            assert!(!category.practical_action_notice().trim().is_empty());
        }
        assert!(QuestionCategory::HealthReference
            .practical_action_notice()
            .contains("及时就医"));
        assert!(QuestionCategory::Wealth
            .practical_action_notice()
            .contains("不要仅依据文化推演"));
        assert!(QuestionCategory::Dispute
            .practical_action_notice()
            .contains("专业人士"));
    }

    #[test]
    fn action_keys_are_stable_machine_identifiers() {
        assert!(valid_action_key("project.verify_before_commit"));
        assert!(valid_action_key("travel_route-2"));
        assert!(!valid_action_key(""));
        assert!(!valid_action_key(".project"));
        assert!(!valid_action_key("project."));
        assert!(!valid_action_key("Project.verify"));
        assert!(!valid_action_key("project verify"));
        assert!(!valid_action_key("行动.核验"));
    }

    #[test]
    fn mapped_other_health_question_keeps_the_medical_safety_notice() {
        let mapped = AncientQuestionType::HealthTrendReference;
        assert!(classified_question_requires_health_notice(
            QuestionCategory::Other,
            Some(&mapped)
        ));
        assert!(practical_action_notice_for_classified_question(
            QuestionCategory::Other,
            Some(&mapped)
        )
        .contains("及时就医"));
        assert!(!classified_question_requires_health_notice(
            QuestionCategory::Other,
            Some(&AncientQuestionType::ProjectOutcome)
        ));
        assert_eq!(
            practical_action_notice_for_classified_question(QuestionCategory::Other, None),
            QuestionCategory::Other.practical_action_notice()
        );
    }
}
