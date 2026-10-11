                                                                          
   
                                                                           
                                                                              
                                                      

use crate::{
    parse_rule_program_output, valid_action_key, AncientQuestionType, DivinationMethod,
    EvidenceTier, IntermediateSignals, MethodResult, QuestionInput, RuleProgramEvaluation,
    TimingWindow,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const RULE_RUNTIME_INPUT_SCHEMA_VERSION: u16 = 1;
pub const RULE_METHOD_AGGREGATION_VERSION: &str =
    "jiuheng.rule_runtime.non_conflicting_signal_merge.v1";
pub const RULE_RUNTIME_MAX_CHART_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleRuntimeInputSnapshot {
    pub schema_version: u16,
    pub method: DivinationMethod,
    pub ancient_question_type: AncientQuestionType,
    pub question: QuestionInput,
    pub chart: Value,
}

impl RuleRuntimeInputSnapshot {
    pub fn new(
        method: DivinationMethod,
        ancient_question_type: AncientQuestionType,
        question: QuestionInput,
        chart: Value,
    ) -> Result<Self, RuleRuntimeError> {
        let snapshot = Self {
            schema_version: RULE_RUNTIME_INPUT_SCHEMA_VERSION,
            method,
            ancient_question_type,
            question,
            chart,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }

    pub fn validate(&self) -> Result<(), RuleRuntimeError> {
        if self.schema_version != RULE_RUNTIME_INPUT_SCHEMA_VERSION {
            return Err(RuleRuntimeError::UnsupportedInputSchema {
                actual: self.schema_version,
                expected: RULE_RUNTIME_INPUT_SCHEMA_VERSION,
            });
        }
        validate_chart(&self.chart)
    }

    pub fn to_value(&self) -> Result<Value, RuleRuntimeError> {
        serde_json::to_value(self)
            .map_err(|error| RuleRuntimeError::InvalidInput(error.to_string()))
    }
}

pub fn parse_rule_runtime_input_snapshot(
    value: &Value,
) -> Result<RuleRuntimeInputSnapshot, RuleRuntimeError> {
    let snapshot = serde_json::from_value::<RuleRuntimeInputSnapshot>(value.clone())
        .map_err(|error| RuleRuntimeError::InvalidInput(error.to_string()))?;
    snapshot.validate()?;
    Ok(snapshot)
}

#[derive(Debug, Clone)]
pub struct RuleRuntimeMatch {
    pub rule_id: String,
    pub evidence_tier: EvidenceTier,
    pub evaluation: RuleProgramEvaluation,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RuleRuntimeError {
    #[error("规则运行输入无效：{0}")]
    InvalidInput(String),
    #[error("规则运行输入版本为 {actual}，当前仅支持 {expected}")]
    UnsupportedInputSchema { actual: u16, expected: u16 },
    #[error("术式排盘快照必须是 JSON 对象")]
    ChartMustBeObject,
    #[error("术式排盘快照超过 {maximum} 字节上限")]
    ChartTooLarge { maximum: usize },
    #[error("没有命中的已放行规则，不能生成术式结果")]
    NoMatchedRules,
    #[error("重复的规则命中 ID：{0}")]
    DuplicateRuleId(String),
    #[error("规则 {rule_id} 的命中输出结构无效：{reason}")]
    InvalidMatch { rule_id: String, reason: String },
    #[error("规则 {first_rule_id} 与 {second_rule_id} 对 {field} 给出冲突值，术式结果失败关闭")]
    ConflictingOutput {
        field: String,
        first_rule_id: String,
        second_rule_id: String,
    },
}

pub fn aggregate_rule_matches(
    method: DivinationMethod,
    chart: Value,
    matches: &[RuleRuntimeMatch],
) -> Result<MethodResult, RuleRuntimeError> {
    validate_chart(&chart)?;
    if matches.is_empty() {
        return Err(RuleRuntimeError::NoMatchedRules);
    }

    let mut ordered = matches.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| left.rule_id.cmp(&right.rule_id));
    let mut seen = BTreeSet::new();
    let mut owners = BTreeMap::<&'static str, String>::new();
    let mut signals = IntermediateSignals::default();
    let mut hit_rule_ids = Vec::with_capacity(ordered.len());
    let mut action_advice = Vec::new();
    let mut trace = vec![format!(
        "术内规则合并算法：{RULE_METHOD_AGGREGATION_VERSION}；重叠字段必须完全相同"
    )];
    let mut evidence_tier = EvidenceTier::EarlyCoreClassic;

    for item in ordered {
        if item.rule_id.trim().is_empty() {
            return Err(RuleRuntimeError::InvalidMatch {
                rule_id: item.rule_id.clone(),
                reason: "Rule ID 不能为空".to_owned(),
            });
        }
        if !seen.insert(item.rule_id.clone()) {
            return Err(RuleRuntimeError::DuplicateRuleId(item.rule_id.clone()));
        }
        if item.evidence_tier == EvidenceTier::Unverified {
            return Err(RuleRuntimeError::InvalidMatch {
                rule_id: item.rule_id.clone(),
                reason: "未核验证据等级不能进入术式结果".to_owned(),
            });
        }
        if !item.evaluation.matched {
            return Err(RuleRuntimeError::InvalidMatch {
                rule_id: item.rule_id.clone(),
                reason: "未命中的规则不能进入术式结果".to_owned(),
            });
        }
        let output =
            item.evaluation
                .signals
                .as_ref()
                .ok_or_else(|| RuleRuntimeError::InvalidMatch {
                    rule_id: item.rule_id.clone(),
                    reason: "命中规则缺少标准化信号".to_owned(),
                })?;
        let auditable_output =
            serde_json::to_value(item.evaluation.auditable_output()).map_err(|error| {
                RuleRuntimeError::InvalidMatch {
                    rule_id: item.rule_id.clone(),
                    reason: error.to_string(),
                }
            })?;
        parse_rule_program_output(&auditable_output).map_err(|error| {
            RuleRuntimeError::InvalidMatch {
                rule_id: item.rule_id.clone(),
                reason: error.to_string(),
            }
        })?;
        if !output.option_scores.is_empty() || !output.extra.is_empty() {
            return Err(RuleRuntimeError::InvalidMatch {
                rule_id: item.rule_id.clone(),
                reason: "规则程序 v1 不允许 option_scores 或 extra".to_owned(),
            });
        }

        merge_scalar(
            &mut signals.trend,
            output.trend,
            "trend",
            &item.rule_id,
            &mut owners,
        )?;
        merge_scalar(
            &mut signals.subject_strength,
            output.subject_strength,
            "subject_strength",
            &item.rule_id,
            &mut owners,
        )?;
        merge_scalar(
            &mut signals.object_strength,
            output.object_strength,
            "object_strength",
            &item.rule_id,
            &mut owners,
        )?;
        merge_scalar(
            &mut signals.obstacle,
            output.obstacle,
            "obstacle",
            &item.rule_id,
            &mut owners,
        )?;
        merge_scalar(
            &mut signals.support,
            output.support,
            "support",
            &item.rule_id,
            &mut owners,
        )?;
        merge_scalar(
            &mut signals.outcome_direction,
            output.outcome_direction,
            "outcome_direction",
            &item.rule_id,
            &mut owners,
        )?;
        merge_scalar(
            &mut signals.change_speed,
            output.change_speed,
            "change_speed",
            &item.rule_id,
            &mut owners,
        )?;
        merge_scalar(
            &mut signals.active_passive,
            output.active_passive,
            "active_passive",
            &item.rule_id,
            &mut owners,
        )?;
        merge_scalar(
            &mut signals.gain_loss,
            output.gain_loss,
            "gain_loss",
            &item.rule_id,
            &mut owners,
        )?;
        merge_scalar(
            &mut signals.auspiciousness,
            output.auspiciousness,
            "auspiciousness",
            &item.rule_id,
            &mut owners,
        )?;
        merge_scalar(
            &mut signals.stability,
            output.stability,
            "stability",
            &item.rule_id,
            &mut owners,
        )?;
        merge_scalar(
            &mut signals.external_interference,
            output.external_interference,
            "external_interference",
            &item.rule_id,
            &mut owners,
        )?;
        merge_timing(
            &mut signals.timing,
            output.timing,
            &item.rule_id,
            &mut owners,
        )?;

        if let Some(advice) = &item.evaluation.action_advice {
            if advice.rule_id != item.rule_id
                || advice.evidence_tier != item.evidence_tier
                || !valid_action_key(&advice.action_key)
                || advice.text.trim().is_empty()
            {
                return Err(RuleRuntimeError::InvalidMatch {
                    rule_id: item.rule_id.clone(),
                    reason: "行动建议的 Rule ID、证据等级、行动键或正文无效".to_owned(),
                });
            }
            action_advice.push(advice.clone());
        }
        evidence_tier = weaker_tier(evidence_tier, item.evidence_tier);
        hit_rule_ids.push(item.rule_id.clone());
        trace.extend(
            item.evaluation
                .trace
                .iter()
                .map(|line| format!("{}: {line}", item.rule_id)),
        );
    }
    trace.push(format!(
        "{} 条命中规则无冲突合并为一个 {} 术式结果",
        hit_rule_ids.len(),
        method.zh_name()
    ));

    Ok(MethodResult {
        method,
        signals,
        chart,
        hit_rule_ids,
        action_advice,
        evidence_tier,
        trace,
    })
}

fn validate_chart(chart: &Value) -> Result<(), RuleRuntimeError> {
    if !chart.is_object() {
        return Err(RuleRuntimeError::ChartMustBeObject);
    }
    let size = serde_json::to_vec(chart)
        .map_err(|error| RuleRuntimeError::InvalidInput(error.to_string()))?
        .len();
    if size > RULE_RUNTIME_MAX_CHART_BYTES {
        return Err(RuleRuntimeError::ChartTooLarge {
            maximum: RULE_RUNTIME_MAX_CHART_BYTES,
        });
    }
    Ok(())
}

fn merge_scalar(
    target: &mut Option<i16>,
    incoming: Option<i16>,
    field: &'static str,
    rule_id: &str,
    owners: &mut BTreeMap<&'static str, String>,
) -> Result<(), RuleRuntimeError> {
    let Some(incoming) = incoming else {
        return Ok(());
    };
    match target {
        None => {
            *target = Some(incoming);
            owners.insert(field, rule_id.to_owned());
            Ok(())
        }
        Some(existing) if *existing == incoming => Ok(()),
        Some(_) => Err(RuleRuntimeError::ConflictingOutput {
            field: field.to_owned(),
            first_rule_id: owners
                .get(field)
                .cloned()
                .unwrap_or_else(|| "unknown".to_owned()),
            second_rule_id: rule_id.to_owned(),
        }),
    }
}

fn merge_timing(
    target: &mut Option<TimingWindow>,
    incoming: Option<TimingWindow>,
    rule_id: &str,
    owners: &mut BTreeMap<&'static str, String>,
) -> Result<(), RuleRuntimeError> {
    let Some(incoming) = incoming else {
        return Ok(());
    };
    match target {
        None => {
            *target = Some(incoming);
            owners.insert("timing", rule_id.to_owned());
            Ok(())
        }
        Some(existing) if *existing == incoming => Ok(()),
        Some(_) => Err(RuleRuntimeError::ConflictingOutput {
            field: "timing".to_owned(),
            first_rule_id: owners
                .get("timing")
                .cloned()
                .unwrap_or_else(|| "unknown".to_owned()),
            second_rule_id: rule_id.to_owned(),
        }),
    }
}

const fn weaker_tier(left: EvidenceTier, right: EvidenceTier) -> EvidenceTier {
    if tier_strength(left) <= tier_strength(right) {
        left
    } else {
        right
    }
}

const fn tier_strength(tier: EvidenceTier) -> u8 {
    match tier {
        EvidenceTier::EarlyCoreClassic => 3,
        EvidenceTier::EarlyAuthoritativeCommentary => 2,
        EvidenceTier::LaterAuthoritativeCommentary => 1,
        EvidenceTier::Unverified => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ActionPolarity, CoordinatePrecision, GeoLocation, RuleActionAdvice};
    use chrono::{FixedOffset, TimeZone};
    use uuid::Uuid;

    fn question() -> QuestionInput {
        let zone = FixedOffset::east_opt(8 * 3600).unwrap();
        let time = zone.with_ymd_and_hms(2026, 9, 7, 12, 0, 0).unwrap();
        QuestionInput {
            id: Uuid::nil(),
            category: crate::QuestionCategory::Career,
            subcategory: "项目成败".to_owned(),
            text: "项目能否完成？".to_owned(),
            subject: "本人".to_owned(),
            object: "项目".to_owned(),
            target_event: "项目完成".to_owned(),
            result_terms: vec!["完成".to_owned()],
            options: vec![],
            location: GeoLocation {
                country_code: "CN".to_owned(),
                province: "广东省".to_owned(),
                city: "深圳市".to_owned(),
                district: Some("南山区".to_owned()),
                street: None,
                longitude_deg: 113.9304,
                latitude_deg: 22.5333,
                coordinate_precision: CoordinatePrecision::DistrictCenter,
            },
            divination_time_bjt: time,
            target_time_text: "一个月内".to_owned(),
            birth: None,
            locked_at_bjt: time,
        }
    }

    fn matched(
        rule_id: &str,
        tier: EvidenceTier,
        signals: IntermediateSignals,
    ) -> RuleRuntimeMatch {
        RuleRuntimeMatch {
            rule_id: rule_id.to_owned(),
            evidence_tier: tier,
            evaluation: RuleProgramEvaluation {
                matched: true,
                signals: Some(signals),
                action_advice: Some(RuleActionAdvice {
                    rule_id: rule_id.to_owned(),
                    action_key: format!("test.{}", rule_id.to_ascii_lowercase()),
                    polarity: ActionPolarity::Do,
                    text: "测试建议".to_owned(),
                    evidence_tier: tier,
                }),
                trace: vec!["条件结果为 true".to_owned()],
            },
        }
    }

    #[test]
    fn runtime_input_is_versioned_bounded_and_strict() {
        let snapshot = RuleRuntimeInputSnapshot::new(
            DivinationMethod::Liuyao,
            AncientQuestionType::ProjectOutcome,
            question(),
            serde_json::json!({"fixture": true}),
        )
        .unwrap();
        let value = snapshot.to_value().unwrap();
        let parsed = parse_rule_runtime_input_snapshot(&value).unwrap();
        assert_eq!(parsed.schema_version, RULE_RUNTIME_INPUT_SCHEMA_VERSION);
        assert_eq!(parsed.method, DivinationMethod::Liuyao);

        let mut unknown = value;
        unknown["invented"] = serde_json::json!(true);
        assert!(matches!(
            parse_rule_runtime_input_snapshot(&unknown),
            Err(RuleRuntimeError::InvalidInput(_))
        ));
        assert!(matches!(
            RuleRuntimeInputSnapshot::new(
                DivinationMethod::Liuyao,
                AncientQuestionType::ProjectOutcome,
                question(),
                serde_json::json!([]),
            ),
            Err(RuleRuntimeError::ChartMustBeObject)
        ));

        let oversized = serde_json::json!({
            "payload": "x".repeat(RULE_RUNTIME_MAX_CHART_BYTES + 1)
        });
        assert!(matches!(
            RuleRuntimeInputSnapshot::new(
                DivinationMethod::Liuyao,
                AncientQuestionType::ProjectOutcome,
                question(),
                oversized,
            ),
            Err(RuleRuntimeError::ChartTooLarge { .. })
        ));
    }

    #[test]
    fn non_overlapping_rule_outputs_merge_without_weighting_rules() {
        let first = matched(
            "RULE-A",
            EvidenceTier::EarlyCoreClassic,
            IntermediateSignals {
                trend: Some(60),
                timing: Some(TimingWindow::WithinOneMonth),
                ..IntermediateSignals::default()
            },
        );
        let second = matched(
            "RULE-B",
            EvidenceTier::LaterAuthoritativeCommentary,
            IntermediateSignals {
                support: Some(40),
                timing: Some(TimingWindow::WithinOneMonth),
                ..IntermediateSignals::default()
            },
        );
        let result = aggregate_rule_matches(
            DivinationMethod::Liuyao,
            serde_json::json!({"fixture": true}),
            &[second, first],
        )
        .unwrap();
        assert_eq!(result.hit_rule_ids, vec!["RULE-A", "RULE-B"]);
        assert_eq!(result.signals.trend, Some(60));
        assert_eq!(result.signals.support, Some(40));
        assert_eq!(
            result.evidence_tier,
            EvidenceTier::LaterAuthoritativeCommentary
        );
        assert_eq!(result.action_advice.len(), 2);
    }

    #[test]
    fn conflicting_rule_outputs_fail_closed_without_averaging() {
        let positive = matched(
            "RULE-POSITIVE",
            EvidenceTier::EarlyCoreClassic,
            IntermediateSignals {
                trend: Some(60),
                ..IntermediateSignals::default()
            },
        );
        let negative = matched(
            "RULE-NEGATIVE",
            EvidenceTier::EarlyCoreClassic,
            IntermediateSignals {
                trend: Some(-60),
                ..IntermediateSignals::default()
            },
        );
        assert!(matches!(
            aggregate_rule_matches(
                DivinationMethod::Liuyao,
                serde_json::json!({"fixture": true}),
                &[positive, negative],
            ),
            Err(RuleRuntimeError::ConflictingOutput { field, .. }) if field == "trend"
        ));
    }
}
