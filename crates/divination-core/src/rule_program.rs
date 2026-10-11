                                                                             
   
                                                                         
                                                                           
                                                                             
                          

use crate::{IntermediateSignals, RuleActionAdvice, RuleRecord, TimingWindow};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const RULE_CONDITION_SCHEMA_VERSION: u16 = 1;
pub const RULE_RESULT_SCHEMA_VERSION: u16 = 1;
pub const RULE_TIMING_SCHEMA_VERSION: u16 = 1;
pub const RULE_EVALUATION_OUTPUT_SCHEMA_VERSION: u16 = 1;
pub const RULE_PROGRAM_MAX_BYTES: usize = 32 * 1024;
pub const RULE_CONDITION_MAX_DEPTH: usize = 16;
pub const RULE_CONDITION_MAX_NODES: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RuleLiteral {
    Boolean(bool),
    Integer(i64),
    Text(String),
}

impl RuleLiteral {
    fn kind_name(&self) -> &'static str {
        match self {
            Self::Boolean(_) => "boolean",
            Self::Integer(_) => "integer",
            Self::Text(_) => "string",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum RulePredicate {
    All {
        terms: Vec<Self>,
    },
    Any {
        terms: Vec<Self>,
    },
    Not {
        term: Box<Self>,
    },
    Exists {
        path: String,
    },
    Eq {
        path: String,
        value: RuleLiteral,
    },
    NotEq {
        path: String,
        value: RuleLiteral,
    },
    In {
        path: String,
        values: Vec<RuleLiteral>,
    },
    Gt {
        path: String,
        value: i64,
    },
    Gte {
        path: String,
        value: i64,
    },
    Lt {
        path: String,
        value: i64,
    },
    Lte {
        path: String,
        value: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleConditionProgram {
    pub schema_version: u16,
    pub condition: RulePredicate,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RuleSignalOutput {
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
}

impl RuleSignalOutput {
    fn named_values(&self) -> [(&'static str, Option<i16>); 12] {
        [
            ("trend", self.trend),
            ("subject_strength", self.subject_strength),
            ("object_strength", self.object_strength),
            ("obstacle", self.obstacle),
            ("support", self.support),
            ("outcome_direction", self.outcome_direction),
            ("change_speed", self.change_speed),
            ("active_passive", self.active_passive),
            ("gain_loss", self.gain_loss),
            ("auspiciousness", self.auspiciousness),
            ("stability", self.stability),
            ("external_interference", self.external_interference),
        ]
    }

    fn into_intermediate(self, timing: Option<TimingWindow>) -> IntermediateSignals {
        IntermediateSignals {
            trend: self.trend,
            subject_strength: self.subject_strength,
            object_strength: self.object_strength,
            obstacle: self.obstacle,
            support: self.support,
            outcome_direction: self.outcome_direction,
            change_speed: self.change_speed,
            active_passive: self.active_passive,
            gain_loss: self.gain_loss,
            auspiciousness: self.auspiciousness,
            stability: self.stability,
            external_interference: self.external_interference,
            timing,
            option_scores: BTreeMap::new(),
            extra: BTreeMap::new(),
        }
    }
}

impl From<&IntermediateSignals> for RuleSignalOutput {
    fn from(signals: &IntermediateSignals) -> Self {
        Self {
            trend: signals.trend,
            subject_strength: signals.subject_strength,
            object_strength: signals.object_strength,
            obstacle: signals.obstacle,
            support: signals.support,
            outcome_direction: signals.outcome_direction,
            change_speed: signals.change_speed,
            active_passive: signals.active_passive,
            gain_loss: signals.gain_loss,
            auspiciousness: signals.auspiciousness,
            stability: signals.stability,
            external_interference: signals.external_interference,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleResultProgram {
    pub schema_version: u16,
    pub signals: RuleSignalOutput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleTimingProgram {
    pub schema_version: u16,
    pub window: TimingWindow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledRuleProgram {
    condition: RulePredicate,
    signals: RuleSignalOutput,
    timing: Option<TimingWindow>,
    action_advice: Option<RuleActionAdvice>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleProgramEvaluation {
    pub matched: bool,
    pub signals: Option<IntermediateSignals>,
    pub action_advice: Option<RuleActionAdvice>,
    pub trace: Vec<String>,
}

impl RuleProgramEvaluation {
    pub fn auditable_output(&self) -> RuleProgramOutput {
        RuleProgramOutput {
            schema_version: RULE_EVALUATION_OUTPUT_SCHEMA_VERSION,
            matched: self.matched,
            signals: self.signals.as_ref().map(RuleSignalOutput::from),
            timing: self.signals.as_ref().and_then(|signals| signals.timing),
            action_advice: self.action_advice.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleProgramOutput {
    pub schema_version: u16,
    pub matched: bool,
    pub signals: Option<RuleSignalOutput>,
    pub timing: Option<TimingWindow>,
    pub action_advice: Option<RuleActionAdvice>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RuleProgramError {
    #[error("规则字段 {field} 超过 {maximum} 字节上限")]
    ProgramTooLarge { field: String, maximum: usize },
    #[error("规则字段 {field} 不是有效的 v1 JSON 程序：{reason}")]
    InvalidJson { field: String, reason: String },
    #[error("规则字段 {field} 使用不支持的结构版本 {actual}，当前仅支持 {expected}")]
    UnsupportedSchema {
        field: String,
        actual: u16,
        expected: u16,
    },
    #[error("规则字段 {field} 结构无效：{reason}")]
    InvalidStructure { field: String, reason: String },
    #[error("条件输入 {path} 类型错误：期望 {expected}，实际 {actual}")]
    InputTypeMismatch {
        path: String,
        expected: String,
        actual: String,
    },
}

impl CompiledRuleProgram {
    pub fn evaluate(
        &self,
        input_snapshot: &Value,
    ) -> Result<RuleProgramEvaluation, RuleProgramError> {
        if !input_snapshot.is_object() {
            return Err(RuleProgramError::InvalidStructure {
                field: "input_snapshot".to_owned(),
                reason: "规则输入快照顶层必须是 JSON 对象".to_owned(),
            });
        }
        let mut trace = Vec::new();
        let matched = evaluate_predicate(&self.condition, input_snapshot, &mut trace)?;
        if !matched {
            trace.push("条件结果为 false；不产生信号、应期或行动建议".to_owned());
            return Ok(RuleProgramEvaluation {
                matched: false,
                signals: None,
                action_advice: None,
                trace,
            });
        }
        trace.push("条件结果为 true；输出已审核的标准化中间信号".to_owned());
        Ok(RuleProgramEvaluation {
            matched: true,
            signals: Some(self.signals.clone().into_intermediate(self.timing)),
            action_advice: self.action_advice.clone(),
            trace,
        })
    }
}

pub fn compile_rule_program(rule: &RuleRecord) -> Result<CompiledRuleProgram, RuleProgramError> {
    let condition: RuleConditionProgram = parse_program(
        "condition_expression",
        &rule.condition_expression,
        RULE_CONDITION_SCHEMA_VERSION,
    )?;
    let result: RuleResultProgram = parse_program(
        "result_expression",
        &rule.result_expression,
        RULE_RESULT_SCHEMA_VERSION,
    )?;
    let timing = rule
        .timing_expression
        .as_deref()
        .map(|raw| {
            parse_program::<RuleTimingProgram>("timing_expression", raw, RULE_TIMING_SCHEMA_VERSION)
                .map(|program| program.window)
        })
        .transpose()?;

    let mut nodes = 0;
    validate_predicate(&condition.condition, 1, &mut nodes)?;
    validate_signals(&result.signals)?;
    if !rule.action_metadata_valid() {
        return Err(RuleProgramError::InvalidStructure {
            field: "action_advice".to_owned(),
            reason: "行动建议三字段必须全有或全无，且行动键必须合法".to_owned(),
        });
    }

    Ok(CompiledRuleProgram {
        condition: condition.condition,
        signals: result.signals,
        timing,
        action_advice: rule.action_advice(),
    })
}

pub fn parse_rule_program_output(value: &Value) -> Result<RuleProgramOutput, RuleProgramError> {
    let output = serde_json::from_value::<RuleProgramOutput>(value.clone()).map_err(|error| {
        RuleProgramError::InvalidJson {
            field: "expected_output".to_owned(),
            reason: error.to_string(),
        }
    })?;
    if output.schema_version != RULE_EVALUATION_OUTPUT_SCHEMA_VERSION {
        return Err(RuleProgramError::UnsupportedSchema {
            field: "expected_output".to_owned(),
            actual: output.schema_version,
            expected: RULE_EVALUATION_OUTPUT_SCHEMA_VERSION,
        });
    }
    let shape_valid = if output.matched {
        output.signals.is_some()
    } else {
        output.signals.is_none() && output.timing.is_none() && output.action_advice.is_none()
    };
    if !shape_valid {
        return Err(RuleProgramError::InvalidStructure {
            field: "expected_output".to_owned(),
            reason: "matched=true 必须包含 signals；matched=false 不得包含信号、应期或建议"
                .to_owned(),
        });
    }
    if let Some(signals) = &output.signals {
        validate_signals(signals)?;
    }
    Ok(output)
}

trait VersionedProgram {
    fn schema_version(&self) -> u16;
}

impl VersionedProgram for RuleConditionProgram {
    fn schema_version(&self) -> u16 {
        self.schema_version
    }
}

impl VersionedProgram for RuleResultProgram {
    fn schema_version(&self) -> u16 {
        self.schema_version
    }
}

impl VersionedProgram for RuleTimingProgram {
    fn schema_version(&self) -> u16 {
        self.schema_version
    }
}

fn parse_program<T>(field: &str, raw: &str, expected: u16) -> Result<T, RuleProgramError>
where
    T: for<'de> Deserialize<'de> + VersionedProgram,
{
    if raw.len() > RULE_PROGRAM_MAX_BYTES {
        return Err(RuleProgramError::ProgramTooLarge {
            field: field.to_owned(),
            maximum: RULE_PROGRAM_MAX_BYTES,
        });
    }
    let parsed = serde_json::from_str::<T>(raw).map_err(|error| RuleProgramError::InvalidJson {
        field: field.to_owned(),
        reason: error.to_string(),
    })?;
    if parsed.schema_version() != expected {
        return Err(RuleProgramError::UnsupportedSchema {
            field: field.to_owned(),
            actual: parsed.schema_version(),
            expected,
        });
    }
    Ok(parsed)
}

fn validate_predicate(
    predicate: &RulePredicate,
    depth: usize,
    nodes: &mut usize,
) -> Result<(), RuleProgramError> {
    *nodes += 1;
    if *nodes > RULE_CONDITION_MAX_NODES {
        return invalid_condition(format!("条件节点超过 {RULE_CONDITION_MAX_NODES} 个上限"));
    }
    if depth > RULE_CONDITION_MAX_DEPTH {
        return invalid_condition(format!("条件嵌套超过 {RULE_CONDITION_MAX_DEPTH} 层上限"));
    }
    match predicate {
        RulePredicate::All { terms } | RulePredicate::Any { terms } => {
            if terms.is_empty() {
                return invalid_condition("all/any 至少需要一个子条件".to_owned());
            }
            for term in terms {
                validate_predicate(term, depth + 1, nodes)?;
            }
        }
        RulePredicate::Not { term } => validate_predicate(term, depth + 1, nodes)?,
        RulePredicate::Exists { path }
        | RulePredicate::Gt { path, .. }
        | RulePredicate::Gte { path, .. }
        | RulePredicate::Lt { path, .. }
        | RulePredicate::Lte { path, .. }
        | RulePredicate::Eq { path, .. }
        | RulePredicate::NotEq { path, .. } => validate_pointer(path)?,
        RulePredicate::In { path, values } => {
            validate_pointer(path)?;
            if values.is_empty() || values.len() > 64 {
                return invalid_condition("in 必须包含 1 至 64 个候选值".to_owned());
            }
            let kinds = values
                .iter()
                .map(RuleLiteral::kind_name)
                .collect::<BTreeSet<_>>();
            if kinds.len() != 1 {
                return invalid_condition("in 的候选值必须具有相同 JSON 类型".to_owned());
            }
            let unique = values.iter().collect::<BTreeSet<_>>();
            if unique.len() != values.len() {
                return invalid_condition("in 的候选值不得重复".to_owned());
            }
        }
    }
    match predicate {
        RulePredicate::Eq { value, .. } | RulePredicate::NotEq { value, .. } => {
            validate_literal(value)?
        }
        RulePredicate::In { values, .. } => {
            for value in values {
                validate_literal(value)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_pointer(path: &str) -> Result<(), RuleProgramError> {
    let valid = path.starts_with('/')
        && path.len() <= 256
        && !path.chars().any(char::is_control)
        && path.split('/').skip(1).all(|segment| {
            !segment.is_empty()
                && segment.as_bytes().iter().enumerate().all(|(index, byte)| {
                    *byte != b'~'
                        || segment
                            .as_bytes()
                            .get(index + 1)
                            .is_some_and(|next| matches!(next, b'0' | b'1'))
                })
        });
    if !valid {
        return invalid_condition(format!(
            "路径 {path:?} 必须是非空、无控制字符且转义合法的 JSON Pointer"
        ));
    }
    Ok(())
}

fn validate_literal(literal: &RuleLiteral) -> Result<(), RuleProgramError> {
    if let RuleLiteral::Text(text) = literal {
        if text.is_empty() || text.len() > 512 || text.chars().any(char::is_control) {
            return invalid_condition("字符串字面量必须为 1 至 512 字节且不含控制字符".to_owned());
        }
    }
    Ok(())
}

fn validate_signals(signals: &RuleSignalOutput) -> Result<(), RuleProgramError> {
    let values = signals.named_values();
    if values.iter().all(|(_, value)| value.is_none()) {
        return Err(RuleProgramError::InvalidStructure {
            field: "result_expression".to_owned(),
            reason: "至少需要一个标准化中间信号".to_owned(),
        });
    }
    if let Some((name, value)) = values.into_iter().find_map(|(name, value)| {
        value
            .filter(|score| !(-100..=100).contains(score))
            .map(|score| (name, score))
    }) {
        return Err(RuleProgramError::InvalidStructure {
            field: "result_expression".to_owned(),
            reason: format!("信号 {name} 的值 {value} 超出 -100 至 100"),
        });
    }
    Ok(())
}

fn invalid_condition<T>(reason: String) -> Result<T, RuleProgramError> {
    Err(RuleProgramError::InvalidStructure {
        field: "condition_expression".to_owned(),
        reason,
    })
}

fn evaluate_predicate(
    predicate: &RulePredicate,
    input: &Value,
    trace: &mut Vec<String>,
) -> Result<bool, RuleProgramError> {
    let outcome = match predicate {
        RulePredicate::All { terms } => {
            let mut all_match = true;
            for term in terms {
                all_match &= evaluate_predicate(term, input, trace)?;
            }
            all_match
        }
        RulePredicate::Any { terms } => {
            let mut any_match = false;
            for term in terms {
                any_match |= evaluate_predicate(term, input, trace)?;
            }
            any_match
        }
        RulePredicate::Not { term } => !evaluate_predicate(term, input, trace)?,
        RulePredicate::Exists { path } => {
            let matched = input.pointer(path).is_some();
            trace.push(format!("exists {path} => {matched}"));
            return Ok(matched);
        }
        RulePredicate::Eq { path, value } => compare_literal(path, value, input, false)?,
        RulePredicate::NotEq { path, value } => compare_literal(path, value, input, true)?,
        RulePredicate::In { path, values } => {
            let Some(actual) = input.pointer(path) else {
                trace.push(format!("in {path} => false（路径不存在）"));
                return Ok(false);
            };
            ensure_literal_type(path, &values[0], actual)?;
            values
                .iter()
                .any(|candidate| literal_equals(candidate, actual))
        }
        RulePredicate::Gt { path, value } => compare_integer(path, *value, input, |a, b| a > b)?,
        RulePredicate::Gte { path, value } => compare_integer(path, *value, input, |a, b| a >= b)?,
        RulePredicate::Lt { path, value } => compare_integer(path, *value, input, |a, b| a < b)?,
        RulePredicate::Lte { path, value } => compare_integer(path, *value, input, |a, b| a <= b)?,
    };
    trace.push(format!("{} => {outcome}", predicate_label(predicate)));
    Ok(outcome)
}

fn compare_literal(
    path: &str,
    expected: &RuleLiteral,
    input: &Value,
    invert: bool,
) -> Result<bool, RuleProgramError> {
    let Some(actual) = input.pointer(path) else {
        return Ok(false);
    };
    ensure_literal_type(path, expected, actual)?;
    Ok(literal_equals(expected, actual) != invert)
}

fn literal_equals(expected: &RuleLiteral, actual: &Value) -> bool {
    match expected {
        RuleLiteral::Boolean(value) => actual.as_bool() == Some(*value),
        RuleLiteral::Integer(value) => actual.as_i64() == Some(*value),
        RuleLiteral::Text(value) => actual.as_str() == Some(value),
    }
}

fn ensure_literal_type(
    path: &str,
    expected: &RuleLiteral,
    actual: &Value,
) -> Result<(), RuleProgramError> {
    let matches = match expected {
        RuleLiteral::Boolean(_) => actual.is_boolean(),
        RuleLiteral::Integer(_) => actual.as_i64().is_some(),
        RuleLiteral::Text(_) => actual.is_string(),
    };
    if !matches {
        return Err(RuleProgramError::InputTypeMismatch {
            path: path.to_owned(),
            expected: expected.kind_name().to_owned(),
            actual: json_type_name(actual).to_owned(),
        });
    }
    Ok(())
}

fn compare_integer(
    path: &str,
    expected: i64,
    input: &Value,
    compare: impl FnOnce(i64, i64) -> bool,
) -> Result<bool, RuleProgramError> {
    let Some(actual) = input.pointer(path) else {
        return Ok(false);
    };
    let Some(actual) = actual.as_i64() else {
        return Err(RuleProgramError::InputTypeMismatch {
            path: path.to_owned(),
            expected: "integer".to_owned(),
            actual: json_type_name(actual).to_owned(),
        });
    };
    Ok(compare(actual, expected))
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn predicate_label(predicate: &RulePredicate) -> String {
    match predicate {
        RulePredicate::All { .. } => "all".to_owned(),
        RulePredicate::Any { .. } => "any".to_owned(),
        RulePredicate::Not { .. } => "not".to_owned(),
        RulePredicate::Exists { path }
        | RulePredicate::Eq { path, .. }
        | RulePredicate::NotEq { path, .. }
        | RulePredicate::In { path, .. }
        | RulePredicate::Gt { path, .. }
        | RulePredicate::Gte { path, .. }
        | RulePredicate::Lt { path, .. }
        | RulePredicate::Lte { path, .. } => format!("{} {path}", predicate_op(predicate)),
    }
}

fn predicate_op(predicate: &RulePredicate) -> &'static str {
    match predicate {
        RulePredicate::All { .. } => "all",
        RulePredicate::Any { .. } => "any",
        RulePredicate::Not { .. } => "not",
        RulePredicate::Exists { .. } => "exists",
        RulePredicate::Eq { .. } => "eq",
        RulePredicate::NotEq { .. } => "not_eq",
        RulePredicate::In { .. } => "in",
        RulePredicate::Gt { .. } => "gt",
        RulePredicate::Gte { .. } => "gte",
        RulePredicate::Lt { .. } => "lt",
        RulePredicate::Lte { .. } => "lte",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ActionPolarity, AncientQuestionType, DivinationMethod, EvidenceTier, RuleStability,
    };

    fn test_rule() -> RuleRecord {
        RuleRecord {
            rule_id: "TEST-RULE-PROGRAM-001".to_owned(),
            method: DivinationMethod::Liuyao,
            applicable_question_types: vec![AncientQuestionType::ProjectOutcome],
            condition_expression: serde_json::json!({
                "schema_version": 1,
                "condition": {
                    "op": "all",
                    "terms": [
                        {"op": "eq", "path": "/chart/enabled", "value": true},
                        {"op": "gte", "path": "/chart/score", "value": 7},
                        {"op": "in", "path": "/chart/state", "values": ["open", "moving"]},
                        {"op": "exists", "path": "/chart/lines/0"}
                    ]
                }
            })
            .to_string(),
            result_expression: serde_json::json!({
                "schema_version": 1,
                "signals": {"trend": 60, "support": 40, "obstacle": -20}
            })
            .to_string(),
            timing_expression: Some(
                serde_json::json!({"schema_version": 1, "window": "within_one_month"}).to_string(),
            ),
            action_key: Some("project.verify_first".to_owned()),
            action_polarity: Some(ActionPolarity::Do),
            action_expression: Some("先核实条件。".to_owned()),
            priority: 10,
            evidence_tier: EvidenceTier::EarlyCoreClassic,
            historical_period: "测试".to_owned(),
            book_title: "测试".to_owned(),
            volume: "测试".to_owned(),
            chapter: "测试".to_owned(),
            original_text: "测试".to_owned(),
            edition: "测试".to_owned(),
            source_reference: "TEST".to_owned(),
            textual_variants: "无".to_owned(),
            stability: RuleStability::Stable,
            implementation_version: "test-v1".to_owned(),
            reviewed: true,
        }
    }

    #[test]
    fn compiles_and_evaluates_a_matching_program() {
        let compiled = compile_rule_program(&test_rule()).unwrap();
        let evaluation = compiled
            .evaluate(&serde_json::json!({
                "chart": {"enabled": true, "score": 8, "state": "open", "lines": [1]}
            }))
            .unwrap();
        assert!(evaluation.matched);
        let signals = evaluation.signals.as_ref().unwrap();
        assert_eq!(signals.trend, Some(60));
        assert_eq!(signals.support, Some(40));
        assert_eq!(signals.timing, Some(TimingWindow::WithinOneMonth));
        assert_eq!(
            evaluation.action_advice.as_ref().unwrap().rule_id,
            "TEST-RULE-PROGRAM-001"
        );
        assert!(evaluation
            .trace
            .iter()
            .any(|line| line.contains("/chart/score")));
        let output = evaluation.auditable_output();
        assert_eq!(
            parse_rule_program_output(&serde_json::to_value(&output).unwrap()).unwrap(),
            output
        );
    }

    #[test]
    fn missing_path_fails_closed_without_emitting_outputs() {
        let compiled = compile_rule_program(&test_rule()).unwrap();
        let evaluation = compiled
            .evaluate(&serde_json::json!({
                "chart": {"enabled": true, "score": 8, "state": "open", "lines": []}
            }))
            .unwrap();
        assert!(!evaluation.matched);
        assert!(evaluation.signals.is_none());
        assert!(evaluation.action_advice.is_none());
    }

    #[test]
    fn input_type_mismatch_is_an_error_not_a_false_match() {
        let compiled = compile_rule_program(&test_rule()).unwrap();
        assert!(matches!(
            compiled.evaluate(&serde_json::json!({
                "chart": {"enabled": true, "score": "8", "state": "open", "lines": [1]}
            })),
            Err(RuleProgramError::InputTypeMismatch { .. })
        ));
    }

    #[test]
    fn rejects_unknown_schema_fields_and_floating_point_literals() {
        let mut rule = test_rule();
        rule.condition_expression =
            r#"{"schema_version":1,"condition":{"op":"eq","path":"/x","value":1.5}}"#.to_owned();
        assert!(matches!(
            compile_rule_program(&rule),
            Err(RuleProgramError::InvalidJson { .. })
        ));

        let mut rule = test_rule();
        rule.result_expression =
            r#"{"schema_version":1,"signals":{"trend":10,"invented":1}}"#.to_owned();
        assert!(matches!(
            compile_rule_program(&rule),
            Err(RuleProgramError::InvalidJson { .. })
        ));
    }

    #[test]
    fn rejects_empty_or_oversized_condition_structures() {
        let mut rule = test_rule();
        rule.condition_expression =
            r#"{"schema_version":1,"condition":{"op":"all","terms":[]}}"#.to_owned();
        assert!(matches!(
            compile_rule_program(&rule),
            Err(RuleProgramError::InvalidStructure { .. })
        ));

        let mut rule = test_rule();
        rule.condition_expression = " ".repeat(RULE_PROGRAM_MAX_BYTES + 1);
        assert!(matches!(
            compile_rule_program(&rule),
            Err(RuleProgramError::ProgramTooLarge { .. })
        ));
    }

    #[test]
    fn rejects_invalid_pointers_signal_ranges_and_schema_versions() {
        let mut rule = test_rule();
        rule.condition_expression =
            r#"{"schema_version":1,"condition":{"op":"exists","path":"chart/x"}}"#.to_owned();
        assert!(matches!(
            compile_rule_program(&rule),
            Err(RuleProgramError::InvalidStructure { .. })
        ));

        let mut rule = test_rule();
        rule.result_expression =
            r#"{"schema_version":1,"signals":{"outcome_direction":101}}"#.to_owned();
        assert!(matches!(
            compile_rule_program(&rule),
            Err(RuleProgramError::InvalidStructure { .. })
        ));

        let mut rule = test_rule();
        rule.timing_expression =
            Some(r#"{"schema_version":2,"window":"within_one_month"}"#.to_owned());
        assert!(matches!(
            compile_rule_program(&rule),
            Err(RuleProgramError::UnsupportedSchema { .. })
        ));

        assert!(matches!(
            parse_rule_program_output(&serde_json::json!({
                "schema_version": 1,
                "matched": false,
                "signals": {"trend": 10},
                "timing": null,
                "action_advice": null
            })),
            Err(RuleProgramError::InvalidStructure { .. })
        ));
    }
}
