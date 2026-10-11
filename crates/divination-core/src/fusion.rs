use crate::{
    valid_action_key, ActionPolarity, DivinationMethod, EvidenceTier, IntermediateSignals,
    MethodResult, RuleActionAdvice, TimingWindow,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const INTERMEDIATE_SIGNAL_SCHEMA_VERSION: u16 = 1;
pub const ACTION_ADVICE_SCHEMA_VERSION: u16 = 1;
pub const FUSION_ALGORITHM_VERSION: &str = "jiuheng.signal_fusion.equal_method_weight.v2";
pub const SIGNAL_SCORE_MIN: i16 = -100;
pub const SIGNAL_SCORE_MAX: i16 = 100;
const DECISIVE_THRESHOLD: i16 = 15;
type SignalReader = fn(&IntermediateSignals) -> Option<i16>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FusionMode {
    GeneralOutcome,
    OptionComparison,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FusionOutcome {
    StrongPositive,
    Positive,
    Negative,
    StrongNegative,
    PreferredOption,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalDimensionAggregate {
    pub dimension: String,
    pub average_score: i16,
    pub contributing_methods: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MethodFusionContribution {
    pub method: DivinationMethod,
    pub composite_score: Option<i16>,
    pub decisive_signal_count: usize,
    pub evidence_tier: EvidenceTier,
    pub hit_rule_ids: Vec<String>,
    pub option_scores: BTreeMap<String, i16>,
                                                                              
                                                                          
    pub weight_basis: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictBreakAudit {
    pub used: bool,
    pub reason: String,
    pub selected_evidence_tier: Option<EvidenceTier>,
    pub selected_methods: Vec<DivinationMethod>,
    pub selected_score: Option<i16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionAdviceStatus {
    Available,
    PartiallyAvailable,
    Conflicted,
    #[default]
    NoVerifiedAdvice,
}

impl ActionAdviceStatus {
    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Available => "可用",
            Self::PartiallyAvailable => "部分可用",
            Self::Conflicted => "存在冲突，已隐藏",
            Self::NoVerifiedAdvice => "暂无可靠行动建议",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionAdviceItem {
    pub action_key: String,
    pub polarity: ActionPolarity,
    pub text: String,
    pub supporting_methods: Vec<DivinationMethod>,
    pub supporting_rule_ids: Vec<String>,
    pub evidence_tier: EvidenceTier,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionAdviceAudit {
    pub schema_version: u16,
    pub status: ActionAdviceStatus,
    pub items: Vec<ActionAdviceItem>,
    pub suppressed_action_keys: Vec<String>,
    pub notice: String,
    pub trace: Vec<String>,
}

impl Default for ActionAdviceAudit {
    fn default() -> Self {
        Self {
            schema_version: ACTION_ADVICE_SCHEMA_VERSION,
            status: ActionAdviceStatus::NoVerifiedAdvice,
            items: Vec::new(),
            suppressed_action_keys: Vec::new(),
            notice: "暂无通过古籍证据门禁且与命中 Rule ID 绑定的行动建议。".to_owned(),
            trace: vec!["没有可聚合的已核验行动建议；不根据结论分数编造建议".to_owned()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FusionDecision {
    pub algorithm_version: String,
    pub signal_schema_version: u16,
    pub mode: FusionMode,
    pub outcome: FusionOutcome,
    pub conclusion: String,
    pub selected_option: Option<String>,
    pub raw_equal_weight_score: i16,
    pub decision_score: i16,
    pub timing: TimingWindow,
    pub evidence_tier: EvidenceTier,
    #[serde(default)]
    pub action_advice: ActionAdviceAudit,
    pub participating_methods: usize,
    pub method_contributions: Vec<MethodFusionContribution>,
    pub dimension_aggregates: Vec<SignalDimensionAggregate>,
    pub option_aggregates: BTreeMap<String, i16>,
    pub conflict_break_used: bool,
    pub conflict_audit: ConflictBreakAudit,
    pub trace: Vec<String>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum FusionError {
    #[error("没有通过适用性与古籍证据门禁的术式，不能生成结论")]
    NoEligibleMethods,
    #[error("中间信号不足，不能生成结论")]
    InsufficientSignals,
    #[error("术式 {0} 未绑定已命中的放行 Rule ID，拒绝参与融合")]
    MissingReleasedRuleHit(String),
    #[error("术式 {0} 的古籍证据等级未核验，拒绝参与融合")]
    UnverifiedEvidence(String),
    #[error("同一术式重复提交融合结果：{0}")]
    DuplicateMethod(String),
    #[error("术式 {method} 的信号 {signal} 超出 -100 至 100")]
    SignalOutOfRange { method: String, signal: String },
    #[error("多选项比较至少需要两个非空且不重复的锁定选项")]
    InvalidOptions,
    #[error("术式 {0} 没有为全部锁定选项提供一一对应的分数")]
    IncompleteOptionScores(String),
    #[error("同权汇总形成无法由古籍证据等级破局的僵局")]
    UnresolvedConflict,
    #[error("术式 {method} 的行动建议无效：{reason}")]
    InvalidActionAdvice { method: String, reason: String },
}

pub fn fuse_equal_weight(results: &[MethodResult]) -> Result<FusionDecision, FusionError> {
    let prepared = prepare_results(results)?;
    fuse_general(prepared)
}

pub fn fuse_equal_weight_options(
    results: &[MethodResult],
    locked_options: &[String],
) -> Result<FusionDecision, FusionError> {
    let prepared = prepare_results(results)?;
    let options = normalize_options(locked_options)?;
    fuse_options(prepared, &options)
}

fn prepare_results(results: &[MethodResult]) -> Result<Vec<&MethodResult>, FusionError> {
    if results.is_empty() {
        return Err(FusionError::NoEligibleMethods);
    }
    let mut prepared = results.iter().collect::<Vec<_>>();
    prepared.sort_by_key(|result| result.method.key());
    let mut seen = BTreeSet::new();
    for result in &prepared {
        if !seen.insert(result.method.key()) {
            return Err(FusionError::DuplicateMethod(
                result.method.zh_name().to_owned(),
            ));
        }
        if result.hit_rule_ids.is_empty()
            || result
                .hit_rule_ids
                .iter()
                .any(|rule_id| rule_id.trim().is_empty())
        {
            return Err(FusionError::MissingReleasedRuleHit(
                result.method.zh_name().to_owned(),
            ));
        }
        if result.evidence_tier == EvidenceTier::Unverified {
            return Err(FusionError::UnverifiedEvidence(
                result.method.zh_name().to_owned(),
            ));
        }
        validate_action_advice(result)?;
        validate_signal_range(result)?;
    }
    Ok(prepared)
}

fn fuse_general(results: Vec<&MethodResult>) -> Result<FusionDecision, FusionError> {
    let mut contributions = Vec::with_capacity(results.len());
    let mut method_scores = Vec::with_capacity(results.len());
    for result in &results {
        let decisive = decisive_components(&result.signals);
        if decisive.is_empty() {
            return Err(FusionError::InsufficientSignals);
        }
        let score = rounded_average(decisive.iter().copied());
        method_scores.push((result.method, score, result.evidence_tier));
        contributions.push(method_contribution(
            result,
            Some(score),
            decisive.len(),
            results.len(),
        ));
    }

    let raw_score = rounded_average(method_scores.iter().map(|(_, score, _)| *score));
    let has_positive = method_scores
        .iter()
        .any(|(_, score, _)| *score >= DECISIVE_THRESHOLD);
    let has_negative = method_scores
        .iter()
        .any(|(_, score, _)| *score <= -DECISIVE_THRESHOLD);
    let difficult_conflict = has_positive && has_negative && raw_score.abs() < DECISIVE_THRESHOLD;
    let (decision_score, conflict_audit) = if difficult_conflict {
        break_directional_conflict(&method_scores)?
    } else {
        if raw_score.abs() < DECISIVE_THRESHOLD {
            return Err(FusionError::InsufficientSignals);
        }
        (
            raw_score,
            ConflictBreakAudit {
                used: false,
                reason: if has_positive && has_negative {
                    "存在方向分歧，但同权总分已形成明确方向；未使用古籍等级".to_owned()
                } else {
                    "同权总分方向明确；未使用古籍等级".to_owned()
                },
                selected_evidence_tier: None,
                selected_methods: Vec::new(),
                selected_score: None,
            },
        )
    };
    let (outcome, conclusion) = outcome_from_score(decision_score);
    let timing = timing_consensus(&results);
    let evidence_tier = weakest_evidence_tier(&results);
    let action_advice = aggregate_action_advice(&results);
    let dimension_aggregates = aggregate_dimensions(&results);
    let conflict_break_used = conflict_audit.used;
    let trace = vec![
        format!("融合算法：{FUSION_ALGORITHM_VERSION}"),
        format!(
            "{} 个术式各贡献一个方法分数，权重恒为 1/{}",
            results.len(),
            results.len()
        ),
        format!("同权原始总分：{raw_score}"),
        conflict_audit.reason.clone(),
        format!("裁决分：{decision_score}；时间：{}", timing.zh_name()),
        format!(
            "行动建议：{}；可用 {} 项；隐藏冲突 {} 项",
            action_advice.status.zh_name(),
            action_advice.items.len(),
            action_advice.suppressed_action_keys.len()
        ),
    ];

    Ok(FusionDecision {
        algorithm_version: FUSION_ALGORITHM_VERSION.to_owned(),
        signal_schema_version: INTERMEDIATE_SIGNAL_SCHEMA_VERSION,
        mode: FusionMode::GeneralOutcome,
        outcome,
        conclusion,
        selected_option: None,
        raw_equal_weight_score: raw_score,
        decision_score,
        timing,
        evidence_tier,
        action_advice,
        participating_methods: results.len(),
        method_contributions: contributions,
        dimension_aggregates,
        option_aggregates: BTreeMap::new(),
        conflict_break_used,
        conflict_audit,
        trace,
    })
}

fn fuse_options(
    results: Vec<&MethodResult>,
    options: &[String],
) -> Result<FusionDecision, FusionError> {
    let option_set = options.iter().map(String::as_str).collect::<BTreeSet<_>>();
    for result in &results {
        let submitted = result
            .signals
            .option_scores
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        if submitted != option_set {
            return Err(FusionError::IncompleteOptionScores(
                result.method.zh_name().to_owned(),
            ));
        }
    }

    let raw_sums = option_sums(&results, options);
    let raw_winners = highest_keys(&raw_sums);
    let (selected_option, conflict_audit) = if raw_winners.len() == 1 {
        (
            raw_winners[0].clone(),
            ConflictBreakAudit {
                used: false,
                reason: "各选项同权总分产生唯一最高项；未使用古籍等级".to_owned(),
                selected_evidence_tier: None,
                selected_methods: Vec::new(),
                selected_score: None,
            },
        )
    } else {
        break_option_conflict(&results, options)?
    };
    let option_aggregates = raw_sums
        .iter()
        .map(|(option, score)| (option.clone(), rounded_divide(*score, results.len())))
        .collect::<BTreeMap<_, _>>();
    let raw_selected_sum = *raw_sums
        .get(&selected_option)
        .expect("selected option came from locked options");
    let raw_score = rounded_divide(raw_selected_sum, results.len());
    let decision_score = conflict_audit.selected_score.unwrap_or(raw_score);
    let timing = timing_consensus(&results);
    let evidence_tier = weakest_evidence_tier(&results);
    let action_advice = aggregate_action_advice(&results);
    let contributions = results
        .iter()
        .map(|result| {
            let decisive = decisive_components(&result.signals);
            let composite =
                (!decisive.is_empty()).then(|| rounded_average(decisive.iter().copied()));
            method_contribution(result, composite, decisive.len(), results.len())
        })
        .collect();
    let conflict_break_used = conflict_audit.used;
    let trace = vec![
        format!("融合算法：{FUSION_ALGORITHM_VERSION}"),
        format!(
            "{} 个术式对每个锁定选项各贡献一次分数，术式权重恒为 1/{}",
            results.len(),
            results.len()
        ),
        conflict_audit.reason.clone(),
        format!(
            "唯一裁决选项：{selected_option}；时间：{}",
            timing.zh_name()
        ),
        format!(
            "行动建议：{}；可用 {} 项；隐藏冲突 {} 项",
            action_advice.status.zh_name(),
            action_advice.items.len(),
            action_advice.suppressed_action_keys.len()
        ),
    ];

    Ok(FusionDecision {
        algorithm_version: FUSION_ALGORITHM_VERSION.to_owned(),
        signal_schema_version: INTERMEDIATE_SIGNAL_SCHEMA_VERSION,
        mode: FusionMode::OptionComparison,
        outcome: FusionOutcome::PreferredOption,
        conclusion: format!("{selected_option} 最佳"),
        selected_option: Some(selected_option),
        raw_equal_weight_score: raw_score,
        decision_score,
        timing,
        evidence_tier,
        action_advice,
        participating_methods: results.len(),
        method_contributions: contributions,
        dimension_aggregates: aggregate_dimensions(&results),
        option_aggregates,
        conflict_break_used,
        conflict_audit,
        trace,
    })
}

fn validate_action_advice(result: &MethodResult) -> Result<(), FusionError> {
    let hit_rule_ids = result
        .hit_rule_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut advice_rule_ids = BTreeSet::new();
    for advice in &result.action_advice {
        let reason = if !hit_rule_ids.contains(advice.rule_id.as_str()) {
            Some(format!(
                "建议引用的 Rule ID {} 不在该术式命中清单中",
                advice.rule_id
            ))
        } else if !advice_rule_ids.insert(advice.rule_id.as_str()) {
            Some(format!("Rule ID {} 重复提供行动建议", advice.rule_id))
        } else if !valid_action_key(&advice.action_key) {
            Some(format!("行动键 {} 不是稳定的小写机器键", advice.action_key))
        } else if advice.text.trim().is_empty() {
            Some(format!("Rule ID {} 的行动建议文字为空", advice.rule_id))
        } else if advice.evidence_tier == EvidenceTier::Unverified {
            Some(format!(
                "Rule ID {} 的行动建议证据等级未核验",
                advice.rule_id
            ))
        } else {
            None
        };
        if let Some(reason) = reason {
            return Err(FusionError::InvalidActionAdvice {
                method: result.method.zh_name().to_owned(),
                reason,
            });
        }
    }
    Ok(())
}

fn aggregate_action_advice(results: &[&MethodResult]) -> ActionAdviceAudit {
    let mut by_key = BTreeMap::<String, Vec<(DivinationMethod, &RuleActionAdvice)>>::new();
    for result in results {
        for advice in &result.action_advice {
            by_key
                .entry(advice.action_key.clone())
                .or_default()
                .push((result.method, advice));
        }
    }
    if by_key.is_empty() {
        return ActionAdviceAudit::default();
    }

    let mut items = Vec::new();
    let mut suppressed_action_keys = Vec::new();
    let mut trace = Vec::new();
    for (action_key, candidates) in by_key {
        let polarities = candidates
            .iter()
            .map(|(_, advice)| advice.polarity)
            .collect::<BTreeSet<_>>();
        if polarities.len() != 1 {
            trace.push(format!(
                "行动键 {action_key} 同时存在建议与避免方向；整项隐藏，不使用多数票"
            ));
            suppressed_action_keys.push(action_key);
            continue;
        }
        let polarity = *polarities
            .first()
            .expect("non-empty candidates have one polarity");
        let mut by_text = BTreeMap::<String, Vec<(DivinationMethod, &RuleActionAdvice)>>::new();
        for (method, advice) in candidates {
            by_text
                .entry(advice.text.trim().to_owned())
                .or_default()
                .push((method, advice));
        }
        for (text, text_candidates) in by_text {
            let mut supporting_methods = text_candidates
                .iter()
                .map(|(method, _)| *method)
                .collect::<Vec<_>>();
            supporting_methods.sort_by_key(|method| method.key());
            supporting_methods.dedup_by_key(|method| method.key());
            let supporting_rule_ids = text_candidates
                .iter()
                .map(|(_, advice)| advice.rule_id.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let evidence_tier = text_candidates
                .iter()
                .map(|(_, advice)| advice.evidence_tier)
                .min_by_key(|tier| evidence_strength(*tier))
                .expect("text group is non-empty");
            trace.push(format!(
                "行动键 {action_key} 方向一致为{}；文字保持规则原文整理结果；来源 {} 条",
                polarity.zh_name(),
                supporting_rule_ids.len()
            ));
            items.push(ActionAdviceItem {
                action_key: action_key.clone(),
                polarity,
                text,
                supporting_methods,
                supporting_rule_ids,
                evidence_tier,
            });
        }
    }
    items.sort_by(|left, right| {
        left.action_key
            .cmp(&right.action_key)
            .then_with(|| left.polarity.cmp(&right.polarity))
            .then_with(|| left.text.cmp(&right.text))
    });
    let status = match (items.is_empty(), suppressed_action_keys.is_empty()) {
        (false, true) => ActionAdviceStatus::Available,
        (false, false) => ActionAdviceStatus::PartiallyAvailable,
        (true, false) => ActionAdviceStatus::Conflicted,
        (true, true) => ActionAdviceStatus::NoVerifiedAdvice,
    };
    let notice = match status {
        ActionAdviceStatus::Available => "以下行动建议均直接来自本次命中的已放行 Rule ID。",
        ActionAdviceStatus::PartiallyAvailable => {
            "仅显示方向一致的已核验行动建议；存在冲突的行动项已隐藏。"
        }
        ActionAdviceStatus::Conflicted => {
            "命中规则对行动方向存在冲突，系统已隐藏全部冲突项，不代替用户选择。"
        }
        ActionAdviceStatus::NoVerifiedAdvice => {
            "暂无通过古籍证据门禁且与命中 Rule ID 绑定的行动建议。"
        }
    }
    .to_owned();
    ActionAdviceAudit {
        schema_version: ACTION_ADVICE_SCHEMA_VERSION,
        status,
        items,
        suppressed_action_keys,
        notice,
        trace,
    }
}

fn method_contribution(
    result: &MethodResult,
    composite_score: Option<i16>,
    decisive_signal_count: usize,
    method_count: usize,
) -> MethodFusionContribution {
    MethodFusionContribution {
        method: result.method,
        composite_score,
        decisive_signal_count,
        evidence_tier: result.evidence_tier,
        hit_rule_ids: result.hit_rule_ids.clone(),
        option_scores: result.signals.option_scores.clone(),
        weight_basis: format!("1/{method_count}"),
    }
}

fn decisive_components(signals: &IntermediateSignals) -> Vec<i16> {
    let mut values = Vec::new();
    if let Some(value) = signals.trend {
        values.push(value);
    }
    if let (Some(subject), Some(object)) = (signals.subject_strength, signals.object_strength) {
        values.push((subject - object).clamp(SIGNAL_SCORE_MIN, SIGNAL_SCORE_MAX));
    }
    if let Some(value) = signals.obstacle {
        values.push(-value);
    }
    if let Some(value) = signals.support {
        values.push(value);
    }
    if let Some(value) = signals.outcome_direction {
        values.push(value);
    }
    if let Some(value) = signals.gain_loss {
        values.push(value);
    }
    if let Some(value) = signals.auspiciousness {
        values.push(value);
    }
    if let Some(value) = signals.external_interference {
        values.push(-value);
    }
    values
}

fn validate_signal_range(result: &MethodResult) -> Result<(), FusionError> {
    let named = [
        ("trend", result.signals.trend),
        ("subject_strength", result.signals.subject_strength),
        ("object_strength", result.signals.object_strength),
        ("obstacle", result.signals.obstacle),
        ("support", result.signals.support),
        ("outcome_direction", result.signals.outcome_direction),
        ("change_speed", result.signals.change_speed),
        ("active_passive", result.signals.active_passive),
        ("gain_loss", result.signals.gain_loss),
        ("auspiciousness", result.signals.auspiciousness),
        ("stability", result.signals.stability),
        (
            "external_interference",
            result.signals.external_interference,
        ),
    ];
    for (name, value) in named {
        if value.is_some_and(|score| !(SIGNAL_SCORE_MIN..=SIGNAL_SCORE_MAX).contains(&score)) {
            return Err(FusionError::SignalOutOfRange {
                method: result.method.zh_name().to_owned(),
                signal: name.to_owned(),
            });
        }
    }
    for (option, score) in &result.signals.option_scores {
        if !(SIGNAL_SCORE_MIN..=SIGNAL_SCORE_MAX).contains(score) {
            return Err(FusionError::SignalOutOfRange {
                method: result.method.zh_name().to_owned(),
                signal: format!("option_scores.{option}"),
            });
        }
    }
    Ok(())
}

fn aggregate_dimensions(results: &[&MethodResult]) -> Vec<SignalDimensionAggregate> {
    let dimensions: [(&str, SignalReader); 12] = [
        ("trend", |signals| signals.trend),
        ("subject_strength", |signals| signals.subject_strength),
        ("object_strength", |signals| signals.object_strength),
        ("obstacle", |signals| signals.obstacle),
        ("support", |signals| signals.support),
        ("outcome_direction", |signals| signals.outcome_direction),
        ("change_speed", |signals| signals.change_speed),
        ("active_passive", |signals| signals.active_passive),
        ("gain_loss", |signals| signals.gain_loss),
        ("auspiciousness", |signals| signals.auspiciousness),
        ("stability", |signals| signals.stability),
        ("external_interference", |signals| {
            signals.external_interference
        }),
    ];
    dimensions
        .into_iter()
        .filter_map(|(name, read)| {
            let values = results
                .iter()
                .filter_map(|result| read(&result.signals))
                .collect::<Vec<_>>();
            (!values.is_empty()).then(|| SignalDimensionAggregate {
                dimension: name.to_owned(),
                average_score: rounded_average(values.iter().copied()),
                contributing_methods: values.len(),
            })
        })
        .collect()
}

fn break_directional_conflict(
    scores: &[(DivinationMethod, i16, EvidenceTier)],
) -> Result<(i16, ConflictBreakAudit), FusionError> {
    let strongest = scores
        .iter()
        .filter(|(_, score, _)| score.abs() >= DECISIVE_THRESHOLD)
        .map(|(_, _, tier)| evidence_strength(*tier))
        .max()
        .ok_or(FusionError::UnresolvedConflict)?;
    let selected = scores
        .iter()
        .filter(|(_, score, tier)| {
            score.abs() >= DECISIVE_THRESHOLD && evidence_strength(*tier) == strongest
        })
        .collect::<Vec<_>>();
    let selected_score = rounded_average(selected.iter().map(|(_, score, _)| *score));
    if selected_score == 0 {
        return Err(FusionError::UnresolvedConflict);
    }
    let selected_tier = selected[0].2;
    let selected_methods = selected.iter().map(|(method, _, _)| *method).collect();
    Ok((
        selected_score,
        ConflictBreakAudit {
            used: true,
            reason: format!(
                "正反方向同权总分进入僵局，仅本次采用最高古籍证据等级 {:?} 破局",
                selected_tier
            ),
            selected_evidence_tier: Some(selected_tier),
            selected_methods,
            selected_score: Some(selected_score),
        },
    ))
}

fn break_option_conflict(
    results: &[&MethodResult],
    options: &[String],
) -> Result<(String, ConflictBreakAudit), FusionError> {
    let strongest = results
        .iter()
        .map(|result| evidence_strength(result.evidence_tier))
        .max()
        .ok_or(FusionError::UnresolvedConflict)?;
    let selected = results
        .iter()
        .copied()
        .filter(|result| evidence_strength(result.evidence_tier) == strongest)
        .collect::<Vec<_>>();
    let selected_sums = option_sums(&selected, options);
    let winners = highest_keys(&selected_sums);
    if winners.len() != 1 {
        return Err(FusionError::UnresolvedConflict);
    }
    let winner = winners[0].clone();
    let selected_score = rounded_divide(
        *selected_sums
            .get(&winner)
            .expect("winner came from selected sums"),
        selected.len(),
    );
    Ok((
        winner,
        ConflictBreakAudit {
            used: true,
            reason: format!(
                "选项同权总分并列，仅本次采用最高古籍证据等级 {:?} 破局",
                selected[0].evidence_tier
            ),
            selected_evidence_tier: Some(selected[0].evidence_tier),
            selected_methods: selected.iter().map(|result| result.method).collect(),
            selected_score: Some(selected_score),
        },
    ))
}

fn normalize_options(options: &[String]) -> Result<Vec<String>, FusionError> {
    if options.len() < 2 {
        return Err(FusionError::InvalidOptions);
    }
    let normalized = options
        .iter()
        .map(|option| option.trim().to_owned())
        .collect::<Vec<_>>();
    let unique = normalized.iter().collect::<BTreeSet<_>>();
    if normalized.iter().any(String::is_empty) || unique.len() != normalized.len() {
        return Err(FusionError::InvalidOptions);
    }
    Ok(normalized)
}

fn option_sums(results: &[&MethodResult], options: &[String]) -> BTreeMap<String, i32> {
    options
        .iter()
        .map(|option| {
            let sum = results
                .iter()
                .map(|result| i32::from(result.signals.option_scores[option]))
                .sum();
            (option.clone(), sum)
        })
        .collect()
}

fn highest_keys(values: &BTreeMap<String, i32>) -> Vec<String> {
    let highest = values.values().copied().max().unwrap_or(i32::MIN);
    values
        .iter()
        .filter(|(_, score)| **score == highest)
        .map(|(key, _)| key.clone())
        .collect()
}

fn timing_consensus(results: &[&MethodResult]) -> TimingWindow {
    let mut values = results
        .iter()
        .filter_map(|result| result.signals.timing)
        .filter(|timing| *timing != TimingWindow::Indeterminate)
        .collect::<Vec<_>>();
    if values.is_empty() {
        return TimingWindow::Indeterminate;
    }
    values.sort_by_key(|timing| timing_rank(*timing));
    values[values.len() / 2]
}

const fn timing_rank(timing: TimingWindow) -> u8 {
    match timing {
        TimingWindow::WithinOneHour => 0,
        TimingWindow::WithinThreeHours => 1,
        TimingWindow::Today => 2,
        TimingWindow::WithinThreeDays => 3,
        TimingWindow::WithinSevenDays => 4,
        TimingWindow::WithinHalfMonth => 5,
        TimingWindow::WithinOneMonth => 6,
        TimingWindow::WithinThreeMonths => 7,
        TimingWindow::WithinHalfYear => 8,
        TimingWindow::WithinOneYear => 9,
        TimingWindow::Indeterminate => 10,
    }
}

fn weakest_evidence_tier(results: &[&MethodResult]) -> EvidenceTier {
    results
        .iter()
        .map(|result| result.evidence_tier)
        .min_by_key(|tier| evidence_strength(*tier))
        .expect("non-empty results")
}

const fn evidence_strength(tier: EvidenceTier) -> u8 {
    match tier {
        EvidenceTier::EarlyCoreClassic => 3,
        EvidenceTier::EarlyAuthoritativeCommentary => 2,
        EvidenceTier::LaterAuthoritativeCommentary => 1,
        EvidenceTier::Unverified => 0,
    }
}

fn outcome_from_score(score: i16) -> (FusionOutcome, String) {
    if score >= 50 {
        (FusionOutcome::StrongPositive, "整体趋向明显正面".to_owned())
    } else if score > 0 {
        (FusionOutcome::Positive, "整体趋向正面".to_owned())
    } else if score <= -50 {
        (FusionOutcome::StrongNegative, "整体趋向明显负面".to_owned())
    } else {
        (FusionOutcome::Negative, "整体趋向负面".to_owned())
    }
}

fn rounded_average(values: impl Iterator<Item = i16>) -> i16 {
    let values = values.map(i32::from).collect::<Vec<_>>();
    let sum = values.iter().sum();
    rounded_divide(sum, values.len())
}

fn rounded_divide(sum: i32, count: usize) -> i16 {
    let denominator = i32::try_from(count).expect("method count fits i32");
    let rounded = if sum >= 0 {
        (sum + denominator / 2) / denominator
    } else {
        (sum - denominator / 2) / denominator
    };
    rounded as i16
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn result(
        method: DivinationMethod,
        tier: EvidenceTier,
        trend: Option<i16>,
        outcome: Option<i16>,
        timing: Option<TimingWindow>,
    ) -> MethodResult {
        MethodResult {
            method,
            signals: IntermediateSignals {
                trend,
                outcome_direction: outcome,
                timing,
                ..IntermediateSignals::default()
            },
            chart: json!({"fixture": true}),
            hit_rule_ids: vec![format!("TEST-{}", method.key())],
            action_advice: Vec::new(),
            evidence_tier: tier,
            trace: vec!["test fixture only".to_owned()],
        }
    }

    #[test]
    fn refuses_to_invent_a_result_from_no_methods() {
        assert_eq!(fuse_equal_weight(&[]), Err(FusionError::NoEligibleMethods));
    }

    #[test]
    fn gives_every_method_one_equal_composite_contribution() {
        let mut rich = result(
            DivinationMethod::Liuyao,
            EvidenceTier::EarlyCoreClassic,
            Some(60),
            Some(60),
            Some(TimingWindow::WithinThreeDays),
        );
        rich.signals.support = Some(60);
        rich.signals.gain_loss = Some(60);
        rich.signals.auspiciousness = Some(60);
        let sparse = result(
            DivinationMethod::QimenDunjia,
            EvidenceTier::EarlyCoreClassic,
            Some(-20),
            None,
            Some(TimingWindow::WithinSevenDays),
        );

        let decision = fuse_equal_weight(&[rich, sparse]).unwrap();
        assert_eq!(decision.raw_equal_weight_score, 20);
        assert_eq!(decision.method_contributions[0].weight_basis, "1/2");
        assert_eq!(decision.method_contributions[1].weight_basis, "1/2");
        assert!(!decision.conflict_break_used);
        assert_eq!(decision.timing, TimingWindow::WithinSevenDays);
    }

    #[test]
    fn evidence_tier_is_used_only_for_a_directional_deadlock() {
        let early = result(
            DivinationMethod::Liuyao,
            EvidenceTier::EarlyCoreClassic,
            Some(60),
            None,
            None,
        );
        let later = result(
            DivinationMethod::QimenDunjia,
            EvidenceTier::LaterAuthoritativeCommentary,
            Some(-60),
            None,
            None,
        );

        let decision = fuse_equal_weight(&[later, early]).unwrap();
        assert_eq!(decision.raw_equal_weight_score, 0);
        assert_eq!(decision.decision_score, 60);
        assert!(decision.conflict_break_used);
        assert_eq!(
            decision.conflict_audit.selected_evidence_tier,
            Some(EvidenceTier::EarlyCoreClassic)
        );
    }

    #[test]
    fn same_tier_exact_conflict_fails_closed() {
        let positive = result(
            DivinationMethod::Liuyao,
            EvidenceTier::EarlyCoreClassic,
            Some(60),
            None,
            None,
        );
        let negative = result(
            DivinationMethod::QimenDunjia,
            EvidenceTier::EarlyCoreClassic,
            Some(-60),
            None,
            None,
        );
        assert_eq!(
            fuse_equal_weight(&[positive, negative]),
            Err(FusionError::UnresolvedConflict)
        );
    }

    #[test]
    fn rejects_unverified_or_out_of_range_inputs() {
        let unverified = result(
            DivinationMethod::Liuyao,
            EvidenceTier::Unverified,
            Some(80),
            None,
            None,
        );
        assert!(matches!(
            fuse_equal_weight(&[unverified]),
            Err(FusionError::UnverifiedEvidence(_))
        ));

        let invalid = result(
            DivinationMethod::Liuyao,
            EvidenceTier::EarlyCoreClassic,
            Some(101),
            None,
            None,
        );
        assert!(matches!(
            fuse_equal_weight(&[invalid]),
            Err(FusionError::SignalOutOfRange { .. })
        ));
    }

    #[test]
    fn selects_a_unique_best_option_without_voting() {
        let mut first = result(
            DivinationMethod::Liuyao,
            EvidenceTier::EarlyCoreClassic,
            None,
            None,
            Some(TimingWindow::WithinOneMonth),
        );
        first.signals.option_scores = BTreeMap::from([
            ("A".to_owned(), 10),
            ("B".to_owned(), 70),
            ("C".to_owned(), 30),
        ]);
        let mut second = result(
            DivinationMethod::QimenDunjia,
            EvidenceTier::LaterAuthoritativeCommentary,
            None,
            None,
            Some(TimingWindow::WithinThreeMonths),
        );
        second.signals.option_scores = BTreeMap::from([
            ("A".to_owned(), 80),
            ("B".to_owned(), 50),
            ("C".to_owned(), 20),
        ]);

        let decision =
            fuse_equal_weight_options(&[second, first], &["A".into(), "B".into(), "C".into()])
                .unwrap();
        assert_eq!(decision.selected_option.as_deref(), Some("B"));
        assert_eq!(decision.conclusion, "B 最佳");
        assert!(!decision.conflict_break_used);
        assert_eq!(decision.timing, TimingWindow::WithinThreeMonths);
    }

    #[test]
    fn option_tie_uses_evidence_tier_only_as_the_breaker() {
        let mut early = result(
            DivinationMethod::Liuyao,
            EvidenceTier::EarlyCoreClassic,
            None,
            None,
            None,
        );
        early.signals.option_scores = BTreeMap::from([("A".to_owned(), 80), ("B".to_owned(), 20)]);
        let mut later = result(
            DivinationMethod::QimenDunjia,
            EvidenceTier::LaterAuthoritativeCommentary,
            None,
            None,
            None,
        );
        later.signals.option_scores = BTreeMap::from([("A".to_owned(), 20), ("B".to_owned(), 80)]);

        let decision =
            fuse_equal_weight_options(&[later, early], &["A".into(), "B".into()]).unwrap();
        assert_eq!(decision.selected_option.as_deref(), Some("A"));
        assert!(decision.conflict_break_used);
    }

    #[test]
    fn result_order_does_not_change_the_audit_or_decision() {
        let first = result(
            DivinationMethod::Liuyao,
            EvidenceTier::EarlyCoreClassic,
            Some(70),
            None,
            None,
        );
        let second = result(
            DivinationMethod::QimenDunjia,
            EvidenceTier::EarlyAuthoritativeCommentary,
            Some(30),
            None,
            None,
        );
        assert_eq!(
            fuse_equal_weight(&[first.clone(), second.clone()]).unwrap(),
            fuse_equal_weight(&[second, first]).unwrap()
        );
    }

    #[test]
    fn action_advice_is_deduplicated_and_keeps_rule_provenance() {
        let mut first = result(
            DivinationMethod::Liuyao,
            EvidenceTier::EarlyCoreClassic,
            Some(70),
            None,
            None,
        );
        first.action_advice.push(RuleActionAdvice {
            rule_id: "TEST-liuyao".to_owned(),
            action_key: "project.verify_before_commit".to_owned(),
            polarity: ActionPolarity::Do,
            text: "先核实关键条件，再推进不可逆步骤。".to_owned(),
            evidence_tier: EvidenceTier::EarlyCoreClassic,
        });
        let mut second = result(
            DivinationMethod::QimenDunjia,
            EvidenceTier::EarlyAuthoritativeCommentary,
            Some(30),
            None,
            None,
        );
        second.action_advice.push(RuleActionAdvice {
            rule_id: "TEST-qimen_dunjia".to_owned(),
            action_key: "project.verify_before_commit".to_owned(),
            polarity: ActionPolarity::Do,
            text: "先核实关键条件，再推进不可逆步骤。".to_owned(),
            evidence_tier: EvidenceTier::EarlyAuthoritativeCommentary,
        });

        let decision = fuse_equal_weight(&[second, first]).unwrap();
        assert_eq!(decision.action_advice.status, ActionAdviceStatus::Available);
        assert_eq!(decision.action_advice.items.len(), 1);
        assert_eq!(decision.action_advice.items[0].supporting_methods.len(), 2);
        assert_eq!(
            decision.action_advice.items[0].supporting_rule_ids,
            vec!["TEST-liuyao", "TEST-qimen_dunjia"]
        );
        assert_eq!(
            decision.action_advice.items[0].evidence_tier,
            EvidenceTier::EarlyAuthoritativeCommentary
        );
    }

    #[test]
    fn contradictory_action_directions_are_hidden_without_majority_voting() {
        let mut do_it = result(
            DivinationMethod::Liuyao,
            EvidenceTier::EarlyCoreClassic,
            Some(70),
            None,
            None,
        );
        do_it.action_advice.push(RuleActionAdvice {
            rule_id: "TEST-liuyao".to_owned(),
            action_key: "project.launch_now".to_owned(),
            polarity: ActionPolarity::Do,
            text: "现在推进。".to_owned(),
            evidence_tier: EvidenceTier::EarlyCoreClassic,
        });
        let mut avoid_it = result(
            DivinationMethod::QimenDunjia,
            EvidenceTier::LaterAuthoritativeCommentary,
            Some(30),
            None,
            None,
        );
        avoid_it.action_advice.push(RuleActionAdvice {
            rule_id: "TEST-qimen_dunjia".to_owned(),
            action_key: "project.launch_now".to_owned(),
            polarity: ActionPolarity::Avoid,
            text: "暂不推进。".to_owned(),
            evidence_tier: EvidenceTier::LaterAuthoritativeCommentary,
        });

        let decision = fuse_equal_weight(&[do_it, avoid_it]).unwrap();
        assert_eq!(
            decision.action_advice.status,
            ActionAdviceStatus::Conflicted
        );
        assert!(decision.action_advice.items.is_empty());
        assert_eq!(
            decision.action_advice.suppressed_action_keys,
            vec!["project.launch_now"]
        );
    }

    #[test]
    fn action_advice_must_bind_to_a_hit_rule_id() {
        let mut invalid = result(
            DivinationMethod::Liuyao,
            EvidenceTier::EarlyCoreClassic,
            Some(70),
            None,
            None,
        );
        invalid.action_advice.push(RuleActionAdvice {
            rule_id: "NOT-HIT".to_owned(),
            action_key: "project.pause".to_owned(),
            polarity: ActionPolarity::Avoid,
            text: "暂缓。".to_owned(),
            evidence_tier: EvidenceTier::EarlyCoreClassic,
        });
        assert!(matches!(
            fuse_equal_weight(&[invalid]),
            Err(FusionError::InvalidActionAdvice { .. })
        ));
    }

    #[test]
    fn old_fusion_json_without_action_advice_deserializes_fail_closed() {
        let decision = fuse_equal_weight(&[result(
            DivinationMethod::Liuyao,
            EvidenceTier::EarlyCoreClassic,
            Some(70),
            None,
            None,
        )])
        .unwrap();
        let mut json = serde_json::to_value(decision).unwrap();
        json.as_object_mut().unwrap().remove("action_advice");
        let legacy: FusionDecision = serde_json::from_value(json).unwrap();
        assert_eq!(
            legacy.action_advice.status,
            ActionAdviceStatus::NoVerifiedAdvice
        );
        assert!(legacy.action_advice.items.is_empty());
    }
}
