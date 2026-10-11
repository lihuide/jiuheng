use crate::{
    Applicability, ApplicabilityStatus, DivinationMethod, ImplementationState, MethodResult,
    QuestionCategory, QuestionInput,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("术式未通过古籍证据与实现复核，禁止执行：{0}")]
    EvidenceGate(String),
    #[error("缺少必要参数：{0}")]
    MissingParameters(String),
    #[error("术式内部错误：{0}")]
    Internal(String),
}

pub trait DivinationEngine: Send + Sync {
    fn method(&self) -> DivinationMethod;
    fn implementation_state(&self) -> ImplementationState;
    fn assess(&self, question: &QuestionInput) -> Applicability;
    fn execute(&self, question: &QuestionInput) -> Result<MethodResult, EngineError>;
}

pub struct ChartOnlyEngine {
    method: DivinationMethod,
}

impl ChartOnlyEngine {
    pub const fn new(method: DivinationMethod) -> Self {
        Self { method }
    }
}

impl DivinationEngine for ChartOnlyEngine {
    fn method(&self) -> DivinationMethod {
        self.method
    }

    fn implementation_state(&self) -> ImplementationState {
        ImplementationState::ChartOnly
    }

    fn assess(&self, question: &QuestionInput) -> Applicability {
        if requires_natal_birth_parameters(self.method) {
            if question.category != QuestionCategory::LongTermFate {
                return Applicability {
                    method: self.method,
                    status: ApplicabilityStatus::UnsupportedQuestionType,
                    reasons: vec![
                        "当前是即时占事，禁止把问事时间冒充出生时间；本术式不参与".to_owned()
                    ],
                };
            }
            if question.birth.is_none() {
                return Applicability {
                    method: self.method,
                    status: ApplicabilityStatus::MissingParameters,
                    reasons: vec!["缺少性别、出生年月日时或出生地点；本术式自动跳过".to_owned()],
                };
            }
        }
        Applicability {
            method: self.method,
            status: ApplicabilityStatus::InsufficientEvidence,
            reasons: vec![match self.method {
                DivinationMethod::FourPillars => {
                    "机械四柱、干支阴阳五行与以日干为基准的年/月/时明干十神身份候选已实现；藏干、旺衰、格局、用神、大运流年、换界口径与古籍断法尚未完成独立证据门禁".to_owned()
                }
                DivinationMethod::MeihuaYishu => {
                    "年月日时机械起卦候选已实现；闰月、时辰口径、底本与断法尚未完成独立证据门禁"
                        .to_owned()
                }
                DivinationMethod::Liuyao => {
                        "三币六次、六十四卦结构、八宫卦序、世应、传世纳甲干支、六亲身份、六神身份、缺失六亲伏神候选、锁定月建日辰、动爻变纳甲、六旬空支、月日同支对支、月日五行相对各明爻/实际变爻的生克方向、六合六冲支对、七进七退明确传本支对、反吟伏吟前提，以及四组三合生旺墓成员在明爻/实际变爻/月建/日辰中的分层清点机械层已实现；三合成局与来源激活条件、反伏组合口径与效力、进退土支异文、整卦合冲分类、旬空月破日冲效力、生克效力、月日爻作用、飞伏作用、用神、旺衰、应期、底本与断法尚未完成独立证据门禁"
                        .to_owned()
                }
                DivinationMethod::ZiweiDoushu => {
                    "命宫、身宫、十二宫、宫干、五行局与十四主星落支机械候选已实现；年干/时辰口径、历史历法、影印底本、辅煞、四化、运限与断法尚未完成独立证据门禁"
                        .to_owned()
                }
                DivinationMethod::QizhengSiyu => {
                    "七政与紫气、月孛、罗睺、计都身份目录已实现；四余天文角色、行星位置、宿度边界、十二宫制、完整命盘与断法尚未完成独立证据门禁"
                        .to_owned()
                }
                DivinationMethod::ChineseAstrology => {
                    "三垣、二十八宿与日月五星身份目录已实现；垣墙/星官成员、宿度边界、天体动态位置、底本与占辞尚未完成独立证据门禁"
                        .to_owned()
                }
                DivinationMethod::QimenDunjia => {
                    "节气阴阳遁、甲己符头三元、局数、三奇六仪地盘、时旬首落宫与本旬值符值使身份机械候选已实现；处暑异文、超神接气、日界/时制口径、符使逐时飞转、天盘/神盘与断法尚未完成独立证据门禁"
                        .to_owned()
                }
                DivinationMethod::Taiyi => {
                    "已实现‘外部给定上元积年→太岁、太乙、天目、计神、合神、始击、主算、客算、主客大小将宫位、岁计直门、掩/囚/格/对/击/迫位置标签’年计机械框架；击迫方向由卷三立成表独立锚定，始击分前后击与内外宫击，文昌及主客四将按适用位置分内外辰宫迫。关法另穷举两目与四将八组、四将彼此六组同宫配对，并保存主客算是否相等；这些只是原始观察量，因主体配对和‘数齐’释义未决，不判关。另有‘正整数局序+调用方显式阴/阳遁→同类机械字段’七十二局机械框架，以及岁计太乙/主客大将三套旋转门盘机械候选（分别以开门锚定，中五子盘失败关闭）。两个传本积年、现代日期换算与求局、自动选遁、時計直门旋转盘、定计大将门盘、关判定及复合格局、完整局式与断法仍未完成独立证据门禁"
                        .to_owned()
                }
                _ => "起法数据流已实现并可完整重放；古籍断法规则尚未完成版本核对与独立复核"
                    .to_owned(),
            }],
        }
    }

    fn execute(&self, _question: &QuestionInput) -> Result<MethodResult, EngineError> {
        Err(EngineError::EvidenceGate(format!(
            "{}仅开放起法结构，不开放断法",
            self.method.zh_name()
        )))
    }
}

const fn requires_natal_birth_parameters(method: DivinationMethod) -> bool {
    matches!(
        method,
        DivinationMethod::FourPillars
            | DivinationMethod::ZiweiDoushu
            | DivinationMethod::QizhengSiyu
    )
}

pub struct EngineRegistry {
    engines: Vec<Box<dyn DivinationEngine>>,
}

impl Default for EngineRegistry {
    fn default() -> Self {
        let engines = DivinationMethod::ALL
            .into_iter()
            .map(|method| Box::new(ChartOnlyEngine::new(method)) as Box<dyn DivinationEngine>)
            .collect();
        Self { engines }
    }
}

impl EngineRegistry {
    pub fn engines(&self) -> &[Box<dyn DivinationEngine>] {
        &self.engines
    }

    pub fn assess_all(&self, question: &QuestionInput) -> Vec<Applicability> {
        self.engines
            .iter()
            .map(|engine| engine.assess(question))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BiologicalSex, BirthParameters, CoordinatePrecision, GeoLocation, QuestionInput};
    use chrono::{FixedOffset, TimeZone};
    use uuid::Uuid;

    fn question(category: QuestionCategory, with_birth: bool) -> QuestionInput {
        let zone = FixedOffset::east_opt(8 * 3600).unwrap();
        let now = zone.with_ymd_and_hms(2026, 8, 30, 12, 45, 0).unwrap();
        let location = GeoLocation {
            country_code: "CN".to_owned(),
            province: "广东省".to_owned(),
            city: "深圳市".to_owned(),
            district: Some("南山区".to_owned()),
            street: None,
            longitude_deg: 113.9304,
            latitude_deg: 22.5333,
            coordinate_precision: CoordinatePrecision::DistrictCenter,
        };
        QuestionInput {
            id: Uuid::new_v4(),
            category,
            subcategory: "测试".to_owned(),
            text: "未来趋势如何？".to_owned(),
            subject: "本人".to_owned(),
            object: String::new(),
            target_event: "长期趋势".to_owned(),
            result_terms: Vec::new(),
            options: Vec::new(),
            location: location.clone(),
            divination_time_bjt: now,
            target_time_text: "未来一年".to_owned(),
            birth: with_birth.then_some(BirthParameters {
                sex: BiologicalSex::Female,
                date_time_bjt: zone.with_ymd_and_hms(1990, 5, 17, 8, 30, 0).unwrap(),
                location,
            }),
            locked_at_bjt: now,
        }
    }

    #[test]
    fn registers_exactly_nine_distinct_methods() {
        let registry = EngineRegistry::default();
        assert_eq!(registry.engines().len(), 9);
        let mut methods = registry
            .engines()
            .iter()
            .map(|engine| engine.method())
            .collect::<Vec<_>>();
        methods.sort_by_key(|method| method.key());
        methods.dedup();
        assert_eq!(methods.len(), 9);
    }

    #[test]
    fn exposes_nine_chart_only_engines_without_enabling_divination() {
        let registry = EngineRegistry::default();
        let chart_only = registry
            .engines()
            .iter()
            .filter(|engine| engine.implementation_state() == ImplementationState::ChartOnly)
            .count();
        assert_eq!(chart_only, 9);
        assert!(registry
            .engines()
            .iter()
            .all(|engine| { engine.implementation_state() != ImplementationState::VerifiedRules }));
    }

    #[test]
    fn taiyi_exposes_only_the_supplied_accumulated_year_mechanical_candidate() {
        let registry = EngineRegistry::default();
        let engine = registry
            .engines()
            .iter()
            .find(|engine| engine.method() == DivinationMethod::Taiyi)
            .unwrap();
        assert_eq!(
            engine.implementation_state(),
            ImplementationState::ChartOnly
        );
        let applicability = engine.assess(&question(QuestionCategory::Career, false));
        assert_eq!(
            applicability.status,
            ApplicabilityStatus::InsufficientEvidence
        );
        assert!(applicability.reasons[0].contains("外部给定上元积年"));
        assert!(applicability.reasons[0].contains("天目"));
        assert!(applicability.reasons[0].contains("合神"));
        assert!(applicability.reasons[0].contains("始击"));
        assert!(applicability.reasons[0].contains("主算"));
        assert!(applicability.reasons[0].contains("客算"));
        assert!(applicability.reasons[0].contains("主客大小将宫位"));
        assert!(applicability.reasons[0].contains("岁计直门"));
        assert!(applicability.reasons[0].contains("掩/囚/格/对/击/迫位置标签"));
        assert!(applicability.reasons[0].contains("显式阴/阳遁"));
        assert!(applicability.reasons[0].contains("七十二局机械框架"));
        assert!(applicability.reasons[0].contains("三套旋转门盘"));
        assert!(applicability.reasons[0].contains("中五子盘失败关闭"));
        assert!(applicability.reasons[0].contains("時計直门旋转盘"));
        assert!(applicability.reasons[0].contains("定计大将门盘"));
        assert!(applicability.reasons[0].contains("自动选遁"));
        assert!(applicability.reasons[0].contains("击迫方向由卷三立成表独立锚定"));
        assert!(applicability.reasons[0].contains("四将彼此六组同宫配对"));
        assert!(applicability.reasons[0].contains("不判关"));
        assert!(applicability.reasons[0].contains("关判定及复合格局"));
        assert!(applicability.reasons[0].contains("现代日期换算"));
        assert!(engine
            .execute(&question(QuestionCategory::Career, false))
            .is_err());
    }

    #[test]
    fn chinese_astrology_exposes_only_the_received_structure_catalogs() {
        let registry = EngineRegistry::default();
        let engine = registry
            .engines()
            .iter()
            .find(|engine| engine.method() == DivinationMethod::ChineseAstrology)
            .unwrap();
        assert_eq!(
            engine.implementation_state(),
            ImplementationState::ChartOnly
        );
        let applicability = engine.assess(&question(QuestionCategory::Weather, false));
        assert_eq!(
            applicability.status,
            ApplicabilityStatus::InsufficientEvidence
        );
        assert!(applicability.reasons[0].contains("二十八宿"));
        assert!(applicability.reasons[0].contains("三垣"));
        assert!(applicability.reasons[0].contains("日月五星"));
        assert!(engine
            .execute(&question(QuestionCategory::Weather, false))
            .is_err());
    }

    #[test]
    fn qizheng_siyu_exposes_only_the_received_identity_catalog() {
        let registry = EngineRegistry::default();
        let engine = registry
            .engines()
            .iter()
            .find(|engine| engine.method() == DivinationMethod::QizhengSiyu)
            .unwrap();
        assert_eq!(
            engine.implementation_state(),
            ImplementationState::ChartOnly
        );
        let applicability = engine.assess(&question(QuestionCategory::LongTermFate, true));
        assert_eq!(
            applicability.status,
            ApplicabilityStatus::InsufficientEvidence
        );
        assert!(applicability.reasons[0].contains("七政"));
        assert!(applicability.reasons[0].contains("紫气"));
        assert!(applicability.reasons[0].contains("四余天文角色"));
        assert!(engine
            .execute(&question(QuestionCategory::LongTermFate, true))
            .is_err());
    }

    #[test]
    fn liuyao_exposes_eight_palace_najia_relatives_spirits_and_hidden_without_rules() {
        let registry = EngineRegistry::default();
        let engine = registry
            .engines()
            .iter()
            .find(|engine| engine.method() == DivinationMethod::Liuyao)
            .unwrap();
        assert_eq!(
            engine.implementation_state(),
            ImplementationState::ChartOnly
        );
        let applicability = engine.assess(&question(QuestionCategory::Career, false));
        assert_eq!(
            applicability.status,
            ApplicabilityStatus::InsufficientEvidence
        );
        assert!(applicability.reasons[0].contains("八宫卦序"));
        assert!(applicability.reasons[0].contains("世应"));
        assert!(applicability.reasons[0].contains("纳甲干支"));
        assert!(applicability.reasons[0].contains("六亲身份"));
        assert!(applicability.reasons[0].contains("六神身份"));
        assert!(applicability.reasons[0].contains("伏神候选"));
        assert!(applicability.reasons[0].contains("月建日辰"));
        assert!(applicability.reasons[0].contains("动爻变纳甲"));
        assert!(applicability.reasons[0].contains("六旬空支"));
        assert!(applicability.reasons[0].contains("同支对支"));
        assert!(applicability.reasons[0].contains("六合六冲支对"));
        assert!(applicability.reasons[0].contains("整卦合冲分类"));
        assert!(applicability.reasons[0].contains("旬空月破日冲效力"));
        assert!(applicability.reasons[0].contains("月日爻作用"));
        assert!(applicability.reasons[0].contains("飞伏作用"));
        assert!(engine
            .execute(&question(QuestionCategory::Career, false))
            .is_err());
    }

    #[test]
    fn qimen_exposes_mechanical_duty_identities_without_enabling_rules() {
        let registry = EngineRegistry::default();
        let engine = registry
            .engines()
            .iter()
            .find(|engine| engine.method() == DivinationMethod::QimenDunjia)
            .unwrap();
        assert_eq!(
            engine.implementation_state(),
            ImplementationState::ChartOnly
        );
        let applicability = engine.assess(&question(QuestionCategory::Career, false));
        assert_eq!(
            applicability.status,
            ApplicabilityStatus::InsufficientEvidence
        );
        assert!(applicability.reasons[0].contains("阴阳遁"));
        assert!(applicability.reasons[0].contains("三奇六仪地盘"));
        assert!(applicability.reasons[0].contains("时旬首落宫"));
        assert!(applicability.reasons[0].contains("本旬值符值使身份"));
        assert!(engine
            .execute(&question(QuestionCategory::Career, false))
            .is_err());
    }

    #[test]
    fn natal_engines_skip_long_term_questions_without_birth_parameters() {
        let reports =
            EngineRegistry::default().assess_all(&question(QuestionCategory::LongTermFate, false));
        for method in [
            DivinationMethod::FourPillars,
            DivinationMethod::ZiweiDoushu,
            DivinationMethod::QizhengSiyu,
        ] {
            let report = reports
                .iter()
                .find(|report| report.method == method)
                .unwrap();
            assert_eq!(report.status, ApplicabilityStatus::MissingParameters);
        }
    }

    #[test]
    fn natal_engines_never_use_instant_question_time_as_birth_time() {
        let reports =
            EngineRegistry::default().assess_all(&question(QuestionCategory::Weather, false));
        for method in [
            DivinationMethod::FourPillars,
            DivinationMethod::ZiweiDoushu,
            DivinationMethod::QizhengSiyu,
        ] {
            let report = reports
                .iter()
                .find(|report| report.method == method)
                .unwrap();
            assert_eq!(report.status, ApplicabilityStatus::UnsupportedQuestionType);
        }
    }

    #[test]
    fn natal_engines_with_birth_still_stop_at_the_evidence_gate() {
        let reports =
            EngineRegistry::default().assess_all(&question(QuestionCategory::LongTermFate, true));
        for method in [
            DivinationMethod::FourPillars,
            DivinationMethod::ZiweiDoushu,
            DivinationMethod::QizhengSiyu,
        ] {
            let report = reports
                .iter()
                .find(|report| report.method == method)
                .unwrap();
            assert_eq!(report.status, ApplicabilityStatus::InsufficientEvidence);
        }
    }
}
