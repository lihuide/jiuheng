#![forbid(unsafe_code)]

mod sha256;

use chrono::{Datelike, Timelike, Utc};
use divination_core::{
    aggregate_rule_matches, assess_first_rain_scope, build_event_fingerprint,
    build_event_fingerprint_from_terms, chinese_astrology_structure_audit, civil_calendar_audit,
    compare_event_fingerprints, compile_rule_program, estimate_delta_t,
    five_visible_planet_geometric_observations, four_pillars_chart_candidate,
    four_pillars_structural_metadata, four_pillars_visible_stem_ten_gods_audit,
    four_pillars_visible_stem_ten_gods_zi_start_audit, fuse_equal_weight,
    fuse_equal_weight_options, hexagram_transformation, is_supported_yarrow_algorithm,
    jie_month_candidate, julian_day_tt_from_ut1, late_zi_dual_candidates, lichun_year_candidate,
    liuyao_eight_palace_audit, liuyao_hidden_spirits_audit, liuyao_najia_audit, liuyao_sanhe_audit,
    liuyao_six_relatives_audit, liuyao_six_spirits_audit, lunation_interval,
    lunisolar_date_candidate_from_structure, lunisolar_month_structure_for_civil_date,
    meihua_time_casting_audit, parse_rule_program_output, parse_rule_runtime_input_snapshot,
    qimen_ju_duty_identity_audit, qimen_midnight_boundary_safe, qizheng_siyu_identity_audit,
    sexagenary_day_for_fixed_date, sexagenary_hour_from_day_stem,
    sexagenary_year_for_astronomical_year_label, shichen_from_true_solar_time,
    solar_term_class_interval, solar_term_interval, solve_solar_term_near_bjt_date,
    true_solar_time_from_equation_of_time, validate_question_parameters, verify_liuyao_replay,
    verify_yarrow_replay, ziwei_life_body_audit, ziwei_life_body_audits_match, ActionAdviceAudit,
    ActionPolarity, AncientQuestionType, ApplicabilityStatus, AstronomyPrecision, BiologicalSex,
    BirthParameters, CalendarSystem, ChineseAstrologyStructureAudit, CivilCalendarAudit,
    Classification, CoordinatePrecision, DivinationMethod, EphemerisProvider, EventFingerprint,
    EventRevision, EvidenceTier, FourPillarsChartCandidate, FourPillarsStructuralMetadata,
    FourPillarsVisibleStemTenGodAudit, FusionDecision, GeoLocation, HexagramIdentity,
    HistoricalDateTime, IntermediateSignals, JieMonthCandidate, KeywordClassifier,
    LateZiDualCandidates, LichunYearCandidate, LiuyaoBranchPairRelationsAudit,
    LiuyaoCalendarMovingAudit, LiuyaoCalendarRelationsAudit, LiuyaoCast, LiuyaoEightPalaceAudit,
    LiuyaoFanyinFuyinAudit, LiuyaoHiddenSpiritsAudit, LiuyaoNaJiaAudit, LiuyaoProgressRetreatAudit,
    LiuyaoSanheAudit, LiuyaoSixRelativesAudit, LiuyaoSixSpiritsAudit, LiuyaoTranscript,
    LunationInterval, LunisolarDateCandidate, LunisolarMonthStructure, MeihuaTimeCastingAudit,
    MethodResult, PlanetaryGeometricObservation, QimenJuAudit, QizhengSiyuIdentityAudit,
    QuestionInput, RuleProgramEvaluation, RuleProgramOutput, RuleRecord, RuleRuntimeInputSnapshot,
    RuleRuntimeMatch, RuleStability, SexagenaryDesignation, ShichenBoundary, SolarTerm,
    SolarTermClass, SolarTermClassInterval, SpaSolarEphemeris, YarrowCast, YarrowTranscript,
    ZiweiLifeBodyAudit, EVENT_MATCH_ALGORITHM_VERSION, LIUYAO_ALGORITHM_VERSION,
    PILLAR_CANDIDATE_STATUS, SEXAGENARY_DAY_ALGORITHM_VERSION, SHICHEN_ALGORITHM_VERSION,
    SPA_EQUATION_OF_TIME_VERSION, ZI_HOUR_DAY_BOUNDARY_STATUS,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use thiserror::Error;
use uuid::Uuid;

const MIGRATION_0001: &str = include_str!("../migrations/0001_initial.sql");
const MIGRATION_0002: &str = include_str!("../migrations/0002_rule_evidence_gate.sql");
const MIGRATION_0003: &str = include_str!("../migrations/0003_runtime_rule_gate.sql");
const MIGRATION_0004: &str = include_str!("../migrations/0004_astronomy_snapshots.sql");
const MIGRATION_0005: &str = include_str!("../migrations/0005_location_provenance.sql");
const MIGRATION_0006: &str = include_str!("../migrations/0006_event_match_audit.sql");
const MIGRATION_0007: &str = include_str!("../migrations/0007_fusion_decisions.sql");
const MIGRATION_0008: &str = include_str!("../migrations/0008_action_advice.sql");
const MIGRATION_0009: &str = include_str!("../migrations/0009_rule_program_v1.sql");
const MIGRATION_0010: &str = include_str!("../migrations/0010_rule_runtime_integrity.sql");
const MIGRATION_0011: &str = include_str!("../migrations/0011_source_checksum_gate.sql");
const MIGRATION_0012: &str = include_str!("../migrations/0012_facsimile_byte_verification.sql");
const LOCATION_SEED: &str = include_str!("../../../data/locations.seed.json");
const STREET_SEED: &str = include_str!("../../../data/streets.seed.json");
const STREET_DATASET_VERSION: &str = "mca-regions-2025-12-31-slightlee-018603e0";
const RULE_SEED: &str = include_str!("../../../data/rules.seed.json");
const RULE_SEED_SCHEMA_VERSION: u16 = 3;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("数据库错误：{0}")]
    Database(#[from] rusqlite::Error),
    #[error("数据序列化错误：{0}")]
    Serialization(#[from] serde_json::Error),
    #[error("本地文件读取失败：{0}")]
    FileIo(#[from] std::io::Error),
    #[error("地点种子数据无效：{0}")]
    InvalidLocationSeed(String),
    #[error("随机起法材料完整性校验失败：{0}")]
    ArtifactIntegrity(String),
    #[error("规则证据门禁数据无效：{0}")]
    RuleGateValidation(String),
    #[error("内置规则包无效：{0}")]
    InvalidRuleSeed(String),
    #[error("天文审计快照失败：{0}")]
    AstronomySnapshot(String),
    #[error("占问参数校验失败：{0}")]
    ParameterValidation(String),
    #[error("综合裁决审计失败：{0}")]
    FusionAudit(String),
    #[error("规则程序无效：{0}")]
    RuleProgram(String),
    #[error("规则运行审计失败：{0}")]
    RuleRuntime(String),
}

pub struct Storage {
    connection: Connection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationSeedRow {
    pub administrative_code: String,
    pub code_system: String,
    pub province: String,
    pub city: String,
    pub district: Option<String>,
    pub street: Option<String>,
    pub longitude_deg: f64,
    pub latitude_deg: f64,
    pub coordinate_precision: CoordinatePrecision,
    pub source: String,
    pub source_record_id: String,
    pub source_url: String,
    pub source_license: String,
    pub source_modified_date: String,
    pub dataset_version: String,
}

#[derive(Debug, Clone, Deserialize)]
struct StreetSeedRow {
    administrative_code: String,
    province: String,
    city: String,
    district: String,
    street: String,
    coordinate_reference_code: String,
    match_basis: String,
}

#[derive(Debug, Clone)]
pub struct DuplicateEvent {
    pub question_id: Uuid,
    pub question_text: String,
    pub locked_at_bjt: String,
    pub algorithm_version: String,
    pub exact_digest: bool,
    pub score_percent: u8,
    pub match_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiuyaoArtifact {
    pub transcript: LiuyaoTranscript,
    pub cast: LiuyaoCast,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YarrowArtifact {
    pub transcript: YarrowTranscript,
    pub cast: YarrowCast,
}

#[derive(Debug, Clone)]
pub struct QuestionHistorySummary {
    pub question: QuestionInput,
    pub state: String,
    pub has_liuyao: bool,
    pub has_yarrow: bool,
    pub is_event_revision: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevisionLink {
    pub revision_id: Uuid,
    pub prior_question_id: Uuid,
    pub change_description: String,
    pub created_at_bjt: String,
    pub match_algorithm_version: Option<String>,
    pub match_exact_digest: Option<bool>,
    pub match_score_percent: Option<u8>,
    pub match_reasons: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct QuestionHistoryDetail {
    pub question: QuestionInput,
    pub ancient_question_type: AncientQuestionType,
    pub event_fingerprint_version: u16,
    pub event_fingerprint: String,
    pub normalized_terms: BTreeSet<String>,
    pub state: String,
    pub final_conclusion: Option<String>,
    pub final_timing: Option<String>,
    pub final_evidence_tier: Option<String>,
    pub final_action_advice: ActionAdviceAudit,
    pub created_at: String,
    pub revision: Option<RevisionLink>,
    pub later_revision_ids: Vec<Uuid>,
    pub astronomy_snapshot: Option<AstronomyAuditSnapshot>,
    pub engine_runs: Vec<EngineRunAudit>,
    pub fusion_decision: Option<FusionDecision>,
}

#[derive(Debug, Clone)]
pub struct RuleHitAudit {
    pub rule_id: String,
    pub input_snapshot: RuleRuntimeInputSnapshot,
    pub output_snapshot: RuleProgramOutput,
    pub evaluation_trace: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EngineRunAudit {
    pub method: DivinationMethod,
    pub implementation_version: String,
    pub status: ApplicabilityStatus,
    pub reasons: Vec<String>,
    pub chart: Option<serde_json::Value>,
    pub signals: Option<IntermediateSignals>,
    pub evidence_tier: Option<EvidenceTier>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub rule_hits: Vec<RuleHitAudit>,
}

pub const BIRTH_TIME_AUDIT_STATUS: &str = "mechanical_audit_not_natal_chart";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BirthTimeAuditSnapshot {
    pub audit_version: u16,
    pub status: String,
    pub sex: BiologicalSex,
    pub input_time_bjt: String,
    pub longitude_deg: f64,
    pub latitude_deg: f64,
    pub jd_ut1_approx: f64,
    pub delta_t_seconds: f64,
    pub delta_t_uncertainty_seconds: f64,
    pub delta_t_model_version: String,
    pub jd_tt: f64,
    pub equation_of_time_minutes: f64,
    pub equation_of_time_algorithm_version: String,
    pub longitude_correction_minutes: f64,
    pub true_solar_total_correction_minutes: f64,
    pub true_solar_time_bjt: String,
    pub true_solar_boundary_margin_minutes: f64,
    pub true_solar_boundary_safe: bool,
    #[serde(default)]
    pub civil_calendar_audit: Option<CivilCalendarAudit>,
    pub sexagenary_calendar_system: String,
    pub sexagenary_day_algorithm_version: String,
    pub sexagenary_day_bjt_midnight: SexagenaryDesignation,
    pub sexagenary_day_true_solar_midnight: SexagenaryDesignation,
    pub shichen_algorithm_version: String,
    pub true_solar_shichen: ShichenBoundary,
    pub sexagenary_hour_true_solar_midnight: SexagenaryDesignation,
    pub zi_hour_day_boundary_status: String,
    #[serde(default)]
    pub four_pillars_chart_candidate: Option<FourPillarsChartCandidate>,
    #[serde(default)]
    pub four_pillars_structural_metadata: Option<FourPillarsStructuralMetadata>,
    #[serde(default)]
    pub four_pillars_visible_stem_ten_gods: Option<FourPillarsVisibleStemTenGodAudit>,
    #[serde(default)]
    pub four_pillars_visible_stem_ten_gods_zi_start: Option<FourPillarsVisibleStemTenGodAudit>,
    #[serde(default)]
    pub late_zi_dual_candidates: Option<LateZiDualCandidates>,
    #[serde(default)]
    pub lunisolar_date_candidate: Option<LunisolarDateCandidate>,
    #[serde(default)]
    pub lunisolar_date_candidate_error: Option<String>,
    #[serde(default)]
    pub ziwei_life_body_audit: Option<ZiweiLifeBodyAudit>,
    #[serde(default)]
    pub ziwei_life_body_audit_error: Option<String>,
    #[serde(default)]
    pub qizheng_siyu_identity_audit: Option<QizhengSiyuIdentityAudit>,
    #[serde(default)]
    pub qizheng_five_planet_geometric_observations: Option<Vec<PlanetaryGeometricObservation>>,
    #[serde(default)]
    pub qizheng_five_planet_geometric_observation_error: Option<String>,
}

pub fn birth_time_audits_match(
    current: &BirthTimeAuditSnapshot,
    stored: &BirthTimeAuditSnapshot,
) -> bool {
    (current.audit_version == stored.audit_version
        || (current.audit_version == 13 && matches!(stored.audit_version, 1..=12)))
        && current.status == stored.status
        && current.sex == stored.sex
        && current.input_time_bjt == stored.input_time_bjt
        && close_f64(current.longitude_deg, stored.longitude_deg)
        && close_f64(current.latitude_deg, stored.latitude_deg)
        && close_f64(current.jd_ut1_approx, stored.jd_ut1_approx)
        && close_f64(current.delta_t_seconds, stored.delta_t_seconds)
        && close_f64(
            current.delta_t_uncertainty_seconds,
            stored.delta_t_uncertainty_seconds,
        )
        && current.delta_t_model_version == stored.delta_t_model_version
        && close_f64(current.jd_tt, stored.jd_tt)
        && close_f64(
            current.equation_of_time_minutes,
            stored.equation_of_time_minutes,
        )
        && current.equation_of_time_algorithm_version == stored.equation_of_time_algorithm_version
        && close_f64(
            current.longitude_correction_minutes,
            stored.longitude_correction_minutes,
        )
        && close_f64(
            current.true_solar_total_correction_minutes,
            stored.true_solar_total_correction_minutes,
        )
        && current.true_solar_time_bjt == stored.true_solar_time_bjt
        && close_f64(
            current.true_solar_boundary_margin_minutes,
            stored.true_solar_boundary_margin_minutes,
        )
        && current.true_solar_boundary_safe == stored.true_solar_boundary_safe
        && (stored.audit_version < 2 || current.civil_calendar_audit == stored.civil_calendar_audit)
        && current.sexagenary_calendar_system == stored.sexagenary_calendar_system
        && current.sexagenary_day_algorithm_version == stored.sexagenary_day_algorithm_version
        && current.sexagenary_day_bjt_midnight == stored.sexagenary_day_bjt_midnight
        && current.sexagenary_day_true_solar_midnight == stored.sexagenary_day_true_solar_midnight
        && current.shichen_algorithm_version == stored.shichen_algorithm_version
        && current.true_solar_shichen.branch == stored.true_solar_shichen.branch
        && close_f64(
            current.true_solar_shichen.minutes_to_boundary,
            stored.true_solar_shichen.minutes_to_boundary,
        )
        && close_f64(
            current.true_solar_shichen.boundary_margin_minutes,
            stored.true_solar_shichen.boundary_margin_minutes,
        )
        && current.true_solar_shichen.boundary_safe == stored.true_solar_shichen.boundary_safe
        && current.sexagenary_hour_true_solar_midnight == stored.sexagenary_hour_true_solar_midnight
        && current.zi_hour_day_boundary_status == stored.zi_hour_day_boundary_status
        && (stored.audit_version < 3
            || current.four_pillars_chart_candidate == stored.four_pillars_chart_candidate)
        && (stored.audit_version < 4
            || (current.four_pillars_structural_metadata
                == stored.four_pillars_structural_metadata
                && current.late_zi_dual_candidates == stored.late_zi_dual_candidates))
        && (stored.audit_version < 5
            || (current.lunisolar_date_candidate == stored.lunisolar_date_candidate
                && current.lunisolar_date_candidate_error == stored.lunisolar_date_candidate_error
                && match (
                    &current.ziwei_life_body_audit,
                    &stored.ziwei_life_body_audit,
                ) {
                    (Some(current), Some(stored)) => ziwei_life_body_audits_match(current, stored),
                    (None, None) => true,
                    _ => false,
                }
                && current.ziwei_life_body_audit_error == stored.ziwei_life_body_audit_error))
        && (stored.audit_version < 7
            || current.qizheng_siyu_identity_audit == stored.qizheng_siyu_identity_audit)
        && (stored.audit_version < 8
            || (planetary_geometric_observations_match(
                &current.qizheng_five_planet_geometric_observations,
                &stored.qizheng_five_planet_geometric_observations,
            ) && current.qizheng_five_planet_geometric_observation_error
                == stored.qizheng_five_planet_geometric_observation_error))
        && (stored.audit_version < 9
            || current.four_pillars_visible_stem_ten_gods
                == stored.four_pillars_visible_stem_ten_gods)
        && (stored.audit_version < 10
            || current.four_pillars_visible_stem_ten_gods_zi_start
                == stored.four_pillars_visible_stem_ten_gods_zi_start)
}

fn planetary_geometric_observations_match(
    current: &Option<Vec<PlanetaryGeometricObservation>>,
    stored: &Option<Vec<PlanetaryGeometricObservation>>,
) -> bool {
    match (current, stored) {
        (None, None) => true,
        (Some(current), Some(stored)) => {
            current.len() == stored.len()
                && current.iter().zip(stored).all(|(current, stored)| {
                    current.planet == stored.planet
                        && close_f64(
                            current.jd_tt_used_as_tdb_approx,
                            stored.jd_tt_used_as_tdb_approx,
                        )
                        && current
                            .heliocentric_j2000_ecliptic_au
                            .iter()
                            .zip(stored.heliocentric_j2000_ecliptic_au)
                            .all(|(left, right)| close_f64(*left, right))
                        && current
                            .earth_moon_barycenter_heliocentric_j2000_ecliptic_au
                            .iter()
                            .zip(stored.earth_moon_barycenter_heliocentric_j2000_ecliptic_au)
                            .all(|(left, right)| close_f64(*left, right))
                        && current
                            .planet_minus_earth_moon_barycenter_j2000_ecliptic_au
                            .iter()
                            .zip(stored.planet_minus_earth_moon_barycenter_j2000_ecliptic_au)
                            .all(|(left, right)| close_f64(*left, right))
                        && close_f64(
                            current.emb_relative_longitude_deg,
                            stored.emb_relative_longitude_deg,
                        )
                        && close_f64(
                            current.emb_relative_latitude_deg,
                            stored.emb_relative_latitude_deg,
                        )
                        && close_f64(
                            current.emb_relative_distance_au,
                            stored.emb_relative_distance_au,
                        )
                        && current.algorithm_version == stored.algorithm_version
                        && current.source_url == stored.source_url
                        && current.apparent_geocentric_position
                            == stored.apparent_geocentric_position
                        && current.historical_mansion_eligible == stored.historical_mansion_eligible
                        && current.divination_rule_eligible == stored.divination_rule_eligible
                })
        }
        _ => false,
    }
}

fn close_f64(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1.0e-12
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AstronomyAuditSnapshot {
    pub snapshot_version: u16,
    pub question_id: Uuid,
                                                                           
                                                                              
    pub ut1_basis: String,
    pub jd_ut1_approx: f64,
    pub delta_t_seconds: f64,
    pub delta_t_uncertainty_seconds: f64,
    pub delta_t_model_version: String,
    pub jd_tt: f64,
    pub solar_longitude_deg: f64,
    pub solar_latitude_deg: f64,
    pub solar_distance_au: f64,
    pub solar_precision: AstronomyPrecision,
    pub solar_estimated_error_arcmin: f64,
    pub solar_algorithm_version: String,
    #[serde(default)]
    pub equation_of_time_minutes: Option<f64>,
    #[serde(default)]
    pub equation_of_time_algorithm_version: Option<String>,
    #[serde(default)]
    pub longitude_correction_minutes: Option<f64>,
    #[serde(default)]
    pub true_solar_total_correction_minutes: Option<f64>,
    #[serde(default)]
    pub true_solar_time_bjt: Option<String>,
    #[serde(default)]
    pub true_solar_boundary_margin_minutes: Option<f64>,
    #[serde(default)]
    pub true_solar_boundary_safe: Option<bool>,
    #[serde(default)]
    pub civil_calendar_audit: Option<CivilCalendarAudit>,
                                                                            
                                                                                 
    #[serde(default)]
    pub sexagenary_calendar_system: Option<String>,
    #[serde(default)]
    pub sexagenary_day_algorithm_version: Option<String>,
    #[serde(default)]
    pub sexagenary_day_bjt_midnight: Option<SexagenaryDesignation>,
    #[serde(default)]
    pub sexagenary_day_true_solar_midnight: Option<SexagenaryDesignation>,
    #[serde(default)]
    pub shichen_algorithm_version: Option<String>,
    #[serde(default)]
    pub true_solar_shichen: Option<ShichenBoundary>,
                                                                              
                                                                              
    #[serde(default)]
    pub sexagenary_hour_true_solar_midnight: Option<SexagenaryDesignation>,
    #[serde(default)]
    pub zi_hour_day_boundary_status: Option<String>,
    #[serde(default)]
    pub solar_term_interval_algorithm_version: Option<String>,
    #[serde(default)]
    pub previous_solar_term: Option<SolarTerm>,
    #[serde(default)]
    pub previous_solar_term_jd_tt: Option<f64>,
    #[serde(default)]
    pub previous_solar_term_uncertainty_seconds: Option<f64>,
    #[serde(default)]
    pub previous_solar_term_iterations: Option<u32>,
    #[serde(default)]
    pub solar_term_boundary_margin_seconds: Option<f64>,
    #[serde(default)]
    pub seconds_since_previous_solar_term: Option<f64>,
    #[serde(default)]
    pub seconds_until_next_solar_term: Option<f64>,
    #[serde(default)]
    pub solar_term_boundary_safe: Option<bool>,
    #[serde(default)]
    pub jie_interval: Option<SolarTermClassInterval>,
    #[serde(default)]
    pub zhongqi_interval: Option<SolarTermClassInterval>,
    #[serde(default)]
    pub astronomical_year_label: Option<SexagenaryDesignation>,
    #[serde(default)]
    pub lichun_year_candidate: Option<LichunYearCandidate>,
    #[serde(default)]
    pub jie_month_candidate: Option<JieMonthCandidate>,
    #[serde(default)]
    pub pillar_candidate_status: Option<String>,
    #[serde(default)]
    pub birth_time_audit: Option<BirthTimeAuditSnapshot>,
                                                                            
                                                                         
    #[serde(default)]
    pub lunation_interval: Option<LunationInterval>,
                                                                            
                                                                             
    #[serde(default)]
    pub lunisolar_month_structure: Option<LunisolarMonthStructure>,
    #[serde(default)]
    pub lunisolar_month_structure_error: Option<String>,
                                                                            
                                                                             
    #[serde(default)]
    pub lunisolar_date_candidate: Option<LunisolarDateCandidate>,
    #[serde(default)]
    pub lunisolar_date_candidate_error: Option<String>,
                                                                             
                                                                     
    #[serde(default)]
    pub meihua_time_casting_audit: Option<MeihuaTimeCastingAudit>,
                                                                         
                                                                               
                                                                          
    #[serde(default)]
    pub chinese_astrology_structure_audit: Option<ChineseAstrologyStructureAudit>,
                                                                  
                                                                   
                                   
    #[serde(default)]
    pub qimen_ju_audit: Option<QimenJuAudit>,
    pub next_solar_term: SolarTerm,
    pub next_solar_term_jd_tt: f64,
    pub next_solar_term_uncertainty_seconds: f64,
    pub next_solar_term_iterations: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactReplayProof {
    pub method: DivinationMethod,
    pub algorithm_version: String,
    pub transcript_hash: String,
    pub operation_count: usize,
    pub primary_hexagram: HexagramIdentity,
    pub changed_hexagram: HexagramIdentity,
    pub liuyao_eight_palace_audit: Option<LiuyaoEightPalaceAudit>,
    pub liuyao_najia_audit: Option<LiuyaoNaJiaAudit>,
    pub liuyao_six_relatives_audit: Option<LiuyaoSixRelativesAudit>,
    pub liuyao_six_spirits_audit: Option<LiuyaoSixSpiritsAudit>,
    pub liuyao_hidden_spirits_audit: Option<LiuyaoHiddenSpiritsAudit>,
    pub liuyao_calendar_moving_audit: Option<LiuyaoCalendarMovingAudit>,
    pub liuyao_calendar_relations_audit: Option<LiuyaoCalendarRelationsAudit>,
    pub liuyao_branch_pair_relations_audit: Option<LiuyaoBranchPairRelationsAudit>,
    pub liuyao_progress_retreat_audit: Option<LiuyaoProgressRetreatAudit>,
    pub liuyao_fanyin_fuyin_audit: Option<LiuyaoFanyinFuyinAudit>,
    pub liuyao_sanhe_audit: Option<LiuyaoSanheAudit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryReplayAudit {
    pub question_id: Uuid,
    pub liuyao: Option<ArtifactReplayProof>,
    pub yarrow: Option<ArtifactReplayProof>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceEditionRecord {
    pub source_id: String,
    pub canonical_title: String,
    pub attributed_author: String,
    pub historical_period: String,
    pub edition_title: String,
    pub edition_type: EditionType,
    pub edition_year: String,
    pub holding_institution: String,
    pub catalog_identifier: String,
    pub facsimile_reference: String,
    pub digital_checksum: String,
    pub notes: String,
}

pub const FACSIMILE_VERIFIER_VERSION: &str = "jiuheng.sha256.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFacsimileVerificationRecord {
    pub source_id: String,
    pub digital_checksum: String,
    pub byte_length: u64,
    pub file_name: String,
    pub verifier_version: String,
    pub verified_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditionType {
    Facsimile,
    Collated,
    Critical,
    Transcription,
}

impl EditionType {
    const fn key(self) -> &'static str {
        match self {
            Self::Facsimile => "facsimile",
            Self::Collated => "collated",
            Self::Critical => "critical",
            Self::Transcription => "transcription",
        }
    }

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Facsimile => "影印本",
            Self::Collated => "校勘本",
            Self::Critical => "整理本",
            Self::Transcription => "电子转录",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcePassageRecord {
    pub passage_id: String,
    pub source_id: String,
    pub volume: String,
    pub chapter: String,
    pub folio_or_page: String,
    pub original_text: String,
    pub normalized_text: String,
    pub textual_variants: String,
    pub transcription_status: TranscriptionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptionStatus {
    Unreviewed,
    Verified,
    Disputed,
}

impl TranscriptionStatus {
    const fn key(self) -> &'static str {
        match self {
            Self::Unreviewed => "unreviewed",
            Self::Verified => "verified",
            Self::Disputed => "disputed",
        }
    }

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Unreviewed => "未复核",
            Self::Verified => "已核对",
            Self::Disputed => "有争议",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceRelation {
    Primary,
    Commentary,
    Variant,
    Counterevidence,
}

impl SourceRelation {
    const fn key(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Commentary => "commentary",
            Self::Variant => "variant",
            Self::Counterevidence => "counterevidence",
        }
    }

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Primary => "主原文",
            Self::Commentary => "注本",
            Self::Variant => "异文",
            Self::Counterevidence => "反证",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewKind {
    Bibliography,
    Transcription,
    Logic,
    Implementation,
}

impl ReviewKind {
    pub const ALL: [Self; 4] = [
        Self::Bibliography,
        Self::Transcription,
        Self::Logic,
        Self::Implementation,
    ];

    const fn key(self) -> &'static str {
        match self {
            Self::Bibliography => "bibliography",
            Self::Transcription => "transcription",
            Self::Logic => "logic",
            Self::Implementation => "implementation",
        }
    }

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Bibliography => "书目复核",
            Self::Transcription => "转录复核",
            Self::Logic => "逻辑复核",
            Self::Implementation => "实现复核",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewDecision {
    Approved,
    Rejected,
    NeedsWork,
}

impl ReviewDecision {
    const fn key(self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::NeedsWork => "needs_work",
        }
    }

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Approved => "通过",
            Self::Rejected => "否决",
            Self::NeedsWork => "需补充",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleTestCaseRecord {
    pub test_case_id: String,
    pub rule_id: String,
    pub name: String,
    pub input: serde_json::Value,
    pub expected_output: serde_json::Value,
    pub boundary_class: String,
    pub source_basis: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleTestRunRecord {
    pub run_id: String,
    pub rule_id: String,
    pub rule_content_hash: String,
    pub implementation_version: String,
    pub test_suite_hash: String,
    pub passed_count: u32,
    pub failed_count: u32,
    pub total_count: u32,
    pub executed_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleTestSuiteExecution {
    pub run: RuleTestRunRecord,
    pub failed_case_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleSourceLinkSeed {
    pub rule_id: String,
    pub passage_id: String,
    pub relation: SourceRelation,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleReviewSeed {
    pub review_id: String,
    pub rule_id: String,
    pub kind: ReviewKind,
    pub decision: ReviewDecision,
    pub reviewer: String,
    pub evidence_notes: String,
    pub rule_content_hash: String,
    pub evidence_bundle_hash: String,
    pub reviewed_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleTestRunSeed {
    pub run_id: String,
    pub rule_id: String,
    pub rule_content_hash: String,
    pub implementation_version: String,
    pub test_suite_hash: String,
    pub passed_count: u32,
    pub failed_count: u32,
    pub total_count: u32,
    pub environment: serde_json::Value,
    pub executed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSeedBundle {
    pub schema_version: u16,
    pub bundle_id: String,
    #[serde(default)]
    pub sources: Vec<SourceEditionRecord>,
    #[serde(default)]
    pub facsimile_verifications: Vec<SourceFacsimileVerificationRecord>,
    #[serde(default)]
    pub passages: Vec<SourcePassageRecord>,
    #[serde(default)]
    pub rules: Vec<RuleRecord>,
    #[serde(default)]
    pub links: Vec<RuleSourceLinkSeed>,
    #[serde(default)]
    pub reviews: Vec<RuleReviewSeed>,
    #[serde(default)]
    pub test_cases: Vec<RuleTestCaseRecord>,
    #[serde(default)]
    pub test_runs: Vec<RuleTestRunSeed>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleGateCheck {
    pub key: String,
    pub label: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleGateReport {
    pub rule_id: String,
    pub rule_content_hash: String,
    pub rule_fields_ready: bool,
    pub primary_source_ready: bool,
    pub reviews_ready: bool,
    pub tests_ready: bool,
    pub checks: Vec<RuleGateCheck>,
    pub blocking_reasons: Vec<String>,
}

impl RuleGateReport {
    pub const fn may_execute(&self) -> bool {
        self.rule_fields_ready
            && self.primary_source_ready
            && self.reviews_ready
            && self.tests_ready
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuleEvidenceSummary {
    pub total_rules: u64,
    pub executable_rules: u64,
    pub source_editions: u64,
    pub facsimile_verifications: u64,
    pub source_passages: u64,
    pub reviews: u64,
    pub enabled_test_cases: u64,
    pub test_runs: u64,
}

#[derive(Debug, Clone)]
pub struct RuleSourceEvidence {
    pub relation: SourceRelation,
    pub link_notes: String,
    pub edition: SourceEditionRecord,
    pub passage: SourcePassageRecord,
    pub facsimile_verification: Option<SourceFacsimileVerificationRecord>,
}

#[derive(Debug, Clone)]
pub struct RuleEvidenceDossier {
    pub rule: RuleRecord,
    pub rule_content_hash: String,
    pub calculated_rule_content_hash: String,
    pub rule_content_hash_matches: bool,
    pub evidence_bundle_hash: String,
    pub calculated_evidence_bundle_hash: String,
    pub evidence_bundle_hash_matches: bool,
    pub test_suite_hash: String,
    pub calculated_test_suite_hash: String,
    pub test_suite_hash_matches: bool,
    pub gate: RuleGateReport,
    pub sources: Vec<RuleSourceEvidence>,
    pub reviews: Vec<RuleReviewSeed>,
    pub test_cases: Vec<RuleTestCaseRecord>,
    pub test_runs: Vec<RuleTestRunSeed>,
}

#[derive(Debug, Clone)]
pub struct ReleasedRuleRecord {
    pub rule: RuleRecord,
    pub content_hash: String,
}

#[derive(Debug, Clone)]
pub struct RuleMethodExecution {
    pub method: DivinationMethod,
    pub status: ApplicabilityStatus,
    pub reasons: Vec<String>,
    pub evaluated_rule_count: usize,
    pub matched_rule_ids: Vec<String>,
    pub result: Option<MethodResult>,
}

impl ReleasedRuleRecord {
    pub fn evaluate(
        &self,
        input_snapshot: &serde_json::Value,
    ) -> Result<RuleProgramEvaluation, StorageError> {
        compile_rule_program(&self.rule)
            .map_err(|error| {
                StorageError::RuleProgram(format!("规则 {}：{error}", self.rule.rule_id))
            })?
            .evaluate(input_snapshot)
            .map_err(|error| {
                StorageError::RuleProgram(format!("规则 {}：{error}", self.rule.rule_id))
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleGateListItem {
    pub rule_id: String,
    pub method: DivinationMethod,
    pub book_title: String,
    pub chapter: String,
    pub action_key: Option<String>,
    pub action_polarity: Option<ActionPolarity>,
    pub action_expression: Option<String>,
    pub report: RuleGateReport,
}

#[derive(Debug)]
struct StoredRuleRow {
    rule_id: String,
    method: String,
    applicable_question_types_json: String,
    condition_expression: String,
    result_expression: String,
    timing_expression: Option<String>,
    action_key: Option<String>,
    action_polarity: Option<String>,
    action_expression: Option<String>,
    priority: i32,
    evidence_tier: String,
    historical_period: String,
    book_title: String,
    volume: String,
    chapter: String,
    original_text: String,
    edition: String,
    source_reference: String,
    textual_variants: String,
    stability: String,
    implementation_version: String,
    reviewed: bool,
    content_hash: String,
}

#[derive(Debug)]
struct StoredRuleSourceEvidenceRow {
    relation: String,
    link_notes: String,
    passage_id: String,
    source_id: String,
    passage_volume: String,
    passage_chapter: String,
    folio_or_page: String,
    passage_original_text: String,
    normalized_text: String,
    passage_textual_variants: String,
    transcription_status: String,
    canonical_title: String,
    attributed_author: String,
    source_historical_period: String,
    edition_title: String,
    edition_type: String,
    edition_year: String,
    holding_institution: String,
    catalog_identifier: String,
    facsimile_reference: String,
    source_digital_checksum: String,
    source_notes: String,
    verification_checksum: Option<String>,
    verification_byte_length: Option<i64>,
    verification_file_name: Option<String>,
    verification_version: Option<String>,
    verification_time: Option<String>,
}

impl Storage {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let connection = Connection::open(path)?;
        let mut storage = Self { connection };
        storage.initialize()?;
        Ok(storage)
    }

    pub fn open_in_memory() -> Result<Self, StorageError> {
        let connection = Connection::open_in_memory()?;
        let mut storage = Self { connection };
        storage.initialize()?;
        Ok(storage)
    }

    fn initialize(&mut self) -> Result<(), StorageError> {
        self.connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;",
        )?;
        let applied = self
            .connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='schema_migrations')",
                [],
                |row| row.get::<_, bool>(0),
            )?;
        let has_v1 = if applied {
            self.connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 1)",
                [],
                |row| row.get::<_, bool>(0),
            )?
        } else {
            false
        };
        if !has_v1 {
            let tx = self.connection.transaction()?;
            tx.execute_batch(MIGRATION_0001)?;
            tx.execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES(1, ?1)",
                [Utc::now().to_rfc3339()],
            )?;
            tx.commit()?;
        }
        let has_v2 = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 2)",
            [],
            |row| row.get::<_, bool>(0),
        )?;
        if !has_v2 {
            let tx = self.connection.transaction()?;
            tx.execute_batch(MIGRATION_0002)?;
            tx.execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES(2, ?1)",
                [Utc::now().to_rfc3339()],
            )?;
            tx.commit()?;
        }
        let has_v3 = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 3)",
            [],
            |row| row.get::<_, bool>(0),
        )?;
        if !has_v3 {
            let tx = self.connection.transaction()?;
            tx.execute_batch(MIGRATION_0003)?;
            tx.execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES(3, ?1)",
                [Utc::now().to_rfc3339()],
            )?;
            tx.commit()?;
        }
        let has_v4 = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 4)",
            [],
            |row| row.get::<_, bool>(0),
        )?;
        if !has_v4 {
            let tx = self.connection.transaction()?;
            tx.execute_batch(MIGRATION_0004)?;
            tx.execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES(4, ?1)",
                [Utc::now().to_rfc3339()],
            )?;
            tx.commit()?;
        }
        let has_v5 = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 5)",
            [],
            |row| row.get::<_, bool>(0),
        )?;
        if !has_v5 {
            let tx = self.connection.transaction()?;
            tx.execute_batch(MIGRATION_0005)?;
            tx.execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES(5, ?1)",
                [Utc::now().to_rfc3339()],
            )?;
            tx.commit()?;
        }
        let has_v6 = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 6)",
            [],
            |row| row.get::<_, bool>(0),
        )?;
        if !has_v6 {
            let tx = self.connection.transaction()?;
            tx.execute_batch(MIGRATION_0006)?;
            tx.execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES(6, ?1)",
                [Utc::now().to_rfc3339()],
            )?;
            tx.commit()?;
        }
        let has_v7 = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 7)",
            [],
            |row| row.get::<_, bool>(0),
        )?;
        if !has_v7 {
            let tx = self.connection.transaction()?;
            tx.execute_batch(MIGRATION_0007)?;
            tx.execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES(7, ?1)",
                [Utc::now().to_rfc3339()],
            )?;
            tx.commit()?;
        }
        let has_v8 = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 8)",
            [],
            |row| row.get::<_, bool>(0),
        )?;
        if !has_v8 {
            let tx = self.connection.transaction()?;
            tx.execute_batch(MIGRATION_0008)?;
            tx.execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES(8, ?1)",
                [Utc::now().to_rfc3339()],
            )?;
            tx.commit()?;
        }
        let has_v9 = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 9)",
            [],
            |row| row.get::<_, bool>(0),
        )?;
        if !has_v9 {
            let tx = self.connection.transaction()?;
            tx.execute_batch(MIGRATION_0009)?;
            tx.execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES(9, ?1)",
                [Utc::now().to_rfc3339()],
            )?;
            tx.commit()?;
        }
        let has_v10 = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 10)",
            [],
            |row| row.get::<_, bool>(0),
        )?;
        if !has_v10 {
            let tx = self.connection.transaction()?;
            tx.execute_batch(MIGRATION_0010)?;
            tx.execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES(10, ?1)",
                [Utc::now().to_rfc3339()],
            )?;
            tx.commit()?;
        }
        let has_v11 = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 11)",
            [],
            |row| row.get::<_, bool>(0),
        )?;
        if !has_v11 {
            let tx = self.connection.transaction()?;
            tx.execute_batch(MIGRATION_0011)?;
            tx.execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES(11, ?1)",
                [Utc::now().to_rfc3339()],
            )?;
            tx.commit()?;
        }
        let has_v12 = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 12)",
            [],
            |row| row.get::<_, bool>(0),
        )?;
        if !has_v12 {
            let tx = self.connection.transaction()?;
            tx.execute_batch(MIGRATION_0012)?;
            tx.execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES(12, ?1)",
                [Utc::now().to_rfc3339()],
            )?;
            tx.commit()?;
        }
        self.seed_locations()?;
        self.seed_streets()?;
        self.seed_reviewed_rules()?;
        Ok(())
    }

    fn seed_locations(&mut self) -> Result<(), StorageError> {
        let rows: Vec<LocationSeedRow> = serde_json::from_str(LOCATION_SEED)?;
        if rows.iter().any(|row| row.administrative_code.is_empty()) {
            return Err(StorageError::InvalidLocationSeed(
                "行政区划代码不能为空".to_owned(),
            ));
        }
        let tx = self.connection.transaction()?;
        for row in rows {
            tx.execute(
                "INSERT INTO locations(
                    administrative_code, province, city, district, street,
                    longitude_deg, latitude_deg, coordinate_precision, source, dataset_version,
                    code_system, source_record_id, source_url, source_license,
                    source_modified_date
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                           ?11, ?12, ?13, ?14, ?15)
                 ON CONFLICT(administrative_code) DO UPDATE SET
                    province=excluded.province, city=excluded.city, district=excluded.district,
                    street=excluded.street, longitude_deg=excluded.longitude_deg,
                    latitude_deg=excluded.latitude_deg,
                    coordinate_precision=excluded.coordinate_precision,
                    source=excluded.source, dataset_version=excluded.dataset_version,
                    code_system=excluded.code_system,
                    source_record_id=excluded.source_record_id,
                    source_url=excluded.source_url,
                    source_license=excluded.source_license,
                    source_modified_date=excluded.source_modified_date",
                params![
                    row.administrative_code,
                    row.province,
                    row.city,
                    row.district,
                    row.street,
                    row.longitude_deg,
                    row.latitude_deg,
                    precision_key(row.coordinate_precision),
                    row.source,
                    row.dataset_version,
                    row.code_system,
                    row.source_record_id,
                    row.source_url,
                    row.source_license,
                    row.source_modified_date,
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    fn seed_streets(&mut self) -> Result<(), StorageError> {
        let rows: Vec<StreetSeedRow> = serde_json::from_str(STREET_SEED)?;
        if rows.len() < 34_000 {
            return Err(StorageError::InvalidLocationSeed(
                "街道种子少于经过核验的预期下限".to_owned(),
            ));
        }
        let seeded: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM locations
             WHERE code_system = 'mca_administrative_code'
               AND dataset_version = ?1",
            [STREET_DATASET_VERSION],
            |row| row.get(0),
        )?;
        if seeded == rows.len() as i64 {
            return Ok(());
        }
        let tx = self.connection.transaction()?;
        {
            let mut insert = tx.prepare(
                "INSERT INTO locations(
                    administrative_code, province, city, district, street,
                    longitude_deg, latitude_deg, coordinate_precision,
                    source, dataset_version, code_system, source_record_id,
                    source_url, source_license, source_modified_date
                 ) SELECT ?1, ?2, ?3, ?4, ?5,
                          longitude_deg, latitude_deg, 'district_center',
                          ?6, ?7, 'mca_administrative_code', ?8,
                          ?9, 'MIT (regions-data); CC-BY-4.0 (GeoNames)', '2025-12-31'
                   FROM locations WHERE administrative_code = ?10
                 ON CONFLICT(administrative_code) DO UPDATE SET
                    province=excluded.province, city=excluded.city,
                    district=excluded.district, street=excluded.street,
                    longitude_deg=excluded.longitude_deg,
                    latitude_deg=excluded.latitude_deg,
                    coordinate_precision=excluded.coordinate_precision,
                    source=excluded.source, dataset_version=excluded.dataset_version,
                    code_system=excluded.code_system,
                    source_record_id=excluded.source_record_id,
                    source_url=excluded.source_url,
                    source_license=excluded.source_license,
                    source_modified_date=excluded.source_modified_date",
            )?;
            for row in rows {
                if row.administrative_code.len() != 9
                    || !row
                        .administrative_code
                        .chars()
                        .all(|char| char.is_ascii_digit())
                    || row.administrative_code.starts_with("000000")
                    || row.province.trim().is_empty()
                    || row.city.trim().is_empty()
                    || row.district.trim().is_empty()
                    || row.street.trim().is_empty()
                    || !row.coordinate_reference_code.starts_with("GN:")
                    || !matches!(
                        row.match_basis.as_str(),
                        "exact_hierarchy" | "unique_province_district"
                    )
                {
                    return Err(StorageError::InvalidLocationSeed(
                        "街道代码、层级或坐标关联不合法".to_owned(),
                    ));
                }
                let source = format!(
                    "MCA administrative name; GeoNames district center; match={} (not a street center)",
                    row.match_basis
                );
                let reference = format!(
                    "MCA:{};{}",
                    row.administrative_code, row.coordinate_reference_code
                );
                let changed = insert.execute(params![
                    format!("MCA:{}", row.administrative_code),
                    row.province,
                    row.city,
                    row.district,
                    row.street,
                    source,
                    STREET_DATASET_VERSION,
                    reference,
                    "https://github.com/slightlee/regions-data/blob/018603e0ef0124577807deb692a21040f9fcdd20/data/processed/regions_20260304_173812.json",
                    row.coordinate_reference_code,
                ])?;
                if changed != 1 {
                    return Err(StorageError::InvalidLocationSeed(
                        "街道缺少可验证的区县中心坐标".to_owned(),
                    ));
                }
            }
        }
        tx.commit()?;
        Ok(())
    }

    fn seed_reviewed_rules(&self) -> Result<(), StorageError> {
        let bundle: RuleSeedBundle = serde_json::from_str(RULE_SEED)
            .map_err(|error| StorageError::InvalidRuleSeed(format!("JSON 无法解析：{error}")))?;
        self.import_rule_seed_bundle(&bundle)
    }

    fn import_rule_seed_bundle(&self, bundle: &RuleSeedBundle) -> Result<(), StorageError> {
        if bundle.schema_version != RULE_SEED_SCHEMA_VERSION {
            return Err(StorageError::InvalidRuleSeed(format!(
                "不支持的规则包结构版本：{}",
                bundle.schema_version
            )));
        }
        if bundle.bundle_id.trim().is_empty() {
            return Err(StorageError::InvalidRuleSeed(
                "规则包 bundle_id 不能为空".to_owned(),
            ));
        }
        validate_unique_seed_ids(bundle)?;

        self.connection
            .execute_batch("SAVEPOINT import_reviewed_rule_seed")?;
        let import_result = (|| {
            for source in &bundle.sources {
                self.upsert_source_edition(source)?;
            }
            for verification in &bundle.facsimile_verifications {
                self.import_source_facsimile_verification(verification)?;
            }
            for passage in &bundle.passages {
                self.upsert_source_passage(passage)?;
            }
            for rule in &bundle.rules {
                self.upsert_rule(rule)?;
            }
            for link in &bundle.links {
                self.link_rule_source(&link.rule_id, &link.passage_id, link.relation, &link.notes)?;
            }
            for test_case in &bundle.test_cases {
                self.upsert_rule_test_case(test_case)?;
            }
            for review in &bundle.reviews {
                self.import_rule_review_seed(review)?;
            }
            for run in &bundle.test_runs {
                self.import_rule_test_run_seed(run)?;
            }
            Ok::<(), StorageError>(())
        })();
        match import_result {
            Ok(()) => {
                self.connection
                    .execute_batch("RELEASE import_reviewed_rule_seed")?;
                Ok(())
            }
            Err(error) => {
                self.connection.execute_batch(
                    "ROLLBACK TO import_reviewed_rule_seed; RELEASE import_reviewed_rule_seed;",
                )?;
                Err(error)
            }
        }
    }

    pub fn locations(&self) -> Result<Vec<GeoLocation>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT province, city, district, street, longitude_deg, latitude_deg, coordinate_precision
             FROM locations WHERE enabled = 1 ORDER BY province, city, district, street",
        )?;
        let rows = statement.query_map([], |row| {
            let precision: String = row.get(6)?;
            Ok(GeoLocation {
                country_code: "CN".to_owned(),
                province: row.get(0)?,
                city: row.get(1)?,
                district: row.get(2)?,
                street: row.get(3)?,
                longitude_deg: row.get(4)?,
                latitude_deg: row.get(5)?,
                coordinate_precision: match precision.as_str() {
                    "street_center" => CoordinatePrecision::StreetCenter,
                    "district_center" => CoordinatePrecision::DistrictCenter,
                    _ => CoordinatePrecision::CityCenter,
                },
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StorageError::from)
    }

    pub fn upsert_rule(&self, rule: &RuleRecord) -> Result<String, StorageError> {
        if rule.rule_id.trim().is_empty() {
            return Err(StorageError::RuleGateValidation(
                "Rule ID 不能为空".to_owned(),
            ));
        }
        if !rule.action_metadata_valid() {
            return Err(StorageError::RuleGateValidation(
                "行动建议必须同时提供合法 action_key、action_polarity 与非空 action_expression，或三者全部留空"
                    .to_owned(),
            ));
        }
        compile_rule_program(rule).map_err(|error| {
            StorageError::RuleProgram(format!("规则 {}：{error}", rule.rule_id))
        })?;
        let now = Utc::now().to_rfc3339();
        let content_hash = canonical_rule_hash(rule)?;
        self.connection.execute(
            "INSERT INTO rules(
                rule_id, method, applicable_question_types_json, condition_expression,
                result_expression, timing_expression, action_key, action_polarity,
                action_expression, priority, evidence_tier,
                historical_period, book_title, volume, chapter, original_text,
                edition, source_reference, textual_variants, stability,
                implementation_version, reviewed, content_hash, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                       ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?24)
             ON CONFLICT(rule_id) DO UPDATE SET
                method=excluded.method,
                applicable_question_types_json=excluded.applicable_question_types_json,
                condition_expression=excluded.condition_expression,
                result_expression=excluded.result_expression,
                timing_expression=excluded.timing_expression,
                action_key=excluded.action_key,
                action_polarity=excluded.action_polarity,
                action_expression=excluded.action_expression,
                priority=excluded.priority,
                evidence_tier=excluded.evidence_tier,
                historical_period=excluded.historical_period,
                book_title=excluded.book_title,
                volume=excluded.volume,
                chapter=excluded.chapter,
                original_text=excluded.original_text,
                edition=excluded.edition,
                source_reference=excluded.source_reference,
                textual_variants=excluded.textual_variants,
                stability=excluded.stability,
                implementation_version=excluded.implementation_version,
                reviewed=excluded.reviewed,
                content_hash=excluded.content_hash,
                updated_at=excluded.updated_at",
            params![
                rule.rule_id,
                rule.method.key(),
                serde_json::to_string(&rule.applicable_question_types)?,
                rule.condition_expression,
                rule.result_expression,
                rule.timing_expression,
                rule.action_key,
                rule.action_polarity.map(action_polarity_key),
                rule.action_expression,
                rule.priority,
                evidence_tier_key(rule.evidence_tier),
                rule.historical_period,
                rule.book_title,
                rule.volume,
                rule.chapter,
                rule.original_text,
                rule.edition,
                rule.source_reference,
                rule.textual_variants,
                stability_key(rule.stability),
                rule.implementation_version,
                rule.reviewed,
                content_hash,
                now,
            ],
        )?;
        Ok(content_hash)
    }

    pub fn upsert_source_edition(&self, source: &SourceEditionRecord) -> Result<(), StorageError> {
        let now = Utc::now().to_rfc3339();
        self.connection.execute(
            "INSERT INTO source_editions(
                source_id, canonical_title, attributed_author, historical_period,
                edition_title, edition_type, edition_year, holding_institution,
                catalog_identifier, facsimile_reference, digital_checksum, notes,
                created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?13)
             ON CONFLICT(source_id) DO UPDATE SET
                canonical_title=excluded.canonical_title,
                attributed_author=excluded.attributed_author,
                historical_period=excluded.historical_period,
                edition_title=excluded.edition_title,
                edition_type=excluded.edition_type,
                edition_year=excluded.edition_year,
                holding_institution=excluded.holding_institution,
                catalog_identifier=excluded.catalog_identifier,
                facsimile_reference=excluded.facsimile_reference,
                digital_checksum=excluded.digital_checksum,
                notes=excluded.notes,
                updated_at=excluded.updated_at",
            params![
                source.source_id,
                source.canonical_title,
                source.attributed_author,
                source.historical_period,
                source.edition_title,
                source.edition_type.key(),
                source.edition_year,
                source.holding_institution,
                source.catalog_identifier,
                source.facsimile_reference,
                source.digital_checksum,
                source.notes,
                now,
            ],
        )?;
        for rule_id in self.rule_ids_for_source(&source.source_id)? {
            self.refresh_rule_evidence_state(&rule_id)?;
        }
        Ok(())
    }

    pub fn verify_source_facsimile_file(
        &self,
        source_id: &str,
        path: &Path,
    ) -> Result<SourceFacsimileVerificationRecord, StorageError> {
        if source_id.trim().is_empty() {
            return Err(StorageError::RuleGateValidation(
                "影印文件验证必须指定 source_id".to_owned(),
            ));
        }
        let expected_checksum = self.connection.query_row(
            "SELECT digital_checksum FROM source_editions WHERE source_id = ?1",
            [source_id],
            |row| row.get::<_, String>(0),
        )?;
        if !is_canonical_sha256(&expected_checksum) {
            return Err(StorageError::RuleGateValidation(format!(
                "版本 {source_id} 尚未登记规范 SHA-256"
            )));
        }
        let file_name = path
            .file_name()
            .filter(|name| !name.is_empty())
            .map(|name| name.to_string_lossy().into_owned())
            .ok_or_else(|| {
                StorageError::RuleGateValidation("影印文件缺少可记录的文件名".to_owned())
            })?;
        let mut file = std::fs::File::open(path)?;
        let (digest, byte_length) = sha256::digest_reader(&mut file)?;
        if byte_length == 0 {
            return Err(StorageError::RuleGateValidation(
                "影印文件不能为空文件".to_owned(),
            ));
        }
        let actual_checksum = format!("sha256:{}", sha256::lowercase_hex(&digest));
        if actual_checksum != expected_checksum {
            return Err(StorageError::RuleGateValidation(format!(
                "影印文件 SHA-256 与版本 {source_id} 的登记值不一致"
            )));
        }
        let verification = SourceFacsimileVerificationRecord {
            source_id: source_id.to_owned(),
            digital_checksum: actual_checksum,
            byte_length,
            file_name,
            verifier_version: FACSIMILE_VERIFIER_VERSION.to_owned(),
            verified_at: Utc::now().to_rfc3339(),
        };
        self.import_source_facsimile_verification(&verification)?;
        Ok(verification)
    }

    fn import_source_facsimile_verification(
        &self,
        verification: &SourceFacsimileVerificationRecord,
    ) -> Result<(), StorageError> {
        if verification.source_id.trim().is_empty()
            || !is_canonical_sha256(&verification.digital_checksum)
            || verification.byte_length == 0
            || verification.byte_length > i64::MAX as u64
            || verification.file_name.trim().is_empty()
            || verification.verifier_version != FACSIMILE_VERIFIER_VERSION
            || chrono::DateTime::parse_from_rfc3339(&verification.verified_at).is_err()
        {
            return Err(StorageError::RuleGateValidation(
                "影印文件验证记录缺少规范校验和、正数字节长度、文件名、验证器版本或 RFC 3339 时间"
                    .to_owned(),
            ));
        }
        let expected_checksum = self.connection.query_row(
            "SELECT digital_checksum FROM source_editions WHERE source_id = ?1",
            [&verification.source_id],
            |row| row.get::<_, String>(0),
        )?;
        if expected_checksum != verification.digital_checksum {
            return Err(StorageError::RuleGateValidation(format!(
                "影印验证记录与版本 {} 的 SHA-256 不一致",
                verification.source_id
            )));
        }
        self.connection.execute(
            "INSERT INTO source_facsimile_verifications(
                source_id, digital_checksum, byte_length, file_name,
                verifier_version, verified_at
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(source_id) DO UPDATE SET
                digital_checksum=excluded.digital_checksum,
                byte_length=excluded.byte_length,
                file_name=excluded.file_name,
                verifier_version=excluded.verifier_version,
                verified_at=excluded.verified_at",
            params![
                verification.source_id,
                verification.digital_checksum,
                verification.byte_length as i64,
                verification.file_name,
                verification.verifier_version,
                verification.verified_at,
            ],
        )?;
        for rule_id in self.rule_ids_for_source(&verification.source_id)? {
            self.refresh_rule_evidence_state(&rule_id)?;
        }
        Ok(())
    }

    pub fn upsert_source_passage(
        &self,
        passage: &SourcePassageRecord,
    ) -> Result<String, StorageError> {
        let now = Utc::now().to_rfc3339();
        let content_hash = hash_serializable(&(
            &passage.source_id,
            &passage.volume,
            &passage.chapter,
            &passage.folio_or_page,
            &passage.original_text,
            &passage.normalized_text,
            &passage.textual_variants,
            passage.transcription_status.key(),
        ))?;
        self.connection.execute(
            "INSERT INTO source_passages(
                passage_id, source_id, volume, chapter, folio_or_page,
                original_text, normalized_text, textual_variants,
                transcription_status, content_hash, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)
             ON CONFLICT(passage_id) DO UPDATE SET
                source_id=excluded.source_id,
                volume=excluded.volume,
                chapter=excluded.chapter,
                folio_or_page=excluded.folio_or_page,
                original_text=excluded.original_text,
                normalized_text=excluded.normalized_text,
                textual_variants=excluded.textual_variants,
                transcription_status=excluded.transcription_status,
                content_hash=excluded.content_hash,
                updated_at=excluded.updated_at",
            params![
                passage.passage_id,
                passage.source_id,
                passage.volume,
                passage.chapter,
                passage.folio_or_page,
                passage.original_text,
                passage.normalized_text,
                passage.textual_variants,
                passage.transcription_status.key(),
                content_hash,
                now,
            ],
        )?;
        for rule_id in self.rule_ids_for_passage(&passage.passage_id)? {
            self.refresh_rule_evidence_state(&rule_id)?;
        }
        Ok(content_hash)
    }

    pub fn link_rule_source(
        &self,
        rule_id: &str,
        passage_id: &str,
        relation: SourceRelation,
        notes: &str,
    ) -> Result<(), StorageError> {
        self.connection.execute(
            "INSERT INTO rule_source_links(rule_id, passage_id, relation, notes, created_at)
             VALUES(?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(rule_id, passage_id, relation) DO UPDATE SET notes=excluded.notes",
            params![
                rule_id,
                passage_id,
                relation.key(),
                notes,
                Utc::now().to_rfc3339()
            ],
        )?;
        self.refresh_rule_evidence_state(rule_id)?;
        Ok(())
    }

    pub fn record_rule_review(
        &self,
        rule_id: &str,
        kind: ReviewKind,
        decision: ReviewDecision,
        reviewer: &str,
        evidence_notes: &str,
    ) -> Result<(), StorageError> {
        if reviewer.trim().is_empty() || evidence_notes.trim().is_empty() {
            return Err(StorageError::RuleGateValidation(
                "复核人和复核说明均不能为空".to_owned(),
            ));
        }
        let rule_content_hash = self.rule_content_hash(rule_id)?;
        let evidence_bundle_hash = self.current_evidence_bundle_hash(rule_id)?;
        self.connection.execute(
            "INSERT INTO rule_reviews(
                review_id, rule_id, review_kind, decision, reviewer,
                evidence_notes, rule_content_hash, evidence_bundle_hash, reviewed_at
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(rule_id, review_kind, reviewer, rule_content_hash, evidence_bundle_hash)
             DO UPDATE SET
                decision=excluded.decision,
                evidence_notes=excluded.evidence_notes,
                reviewed_at=excluded.reviewed_at",
            params![
                Uuid::new_v4().to_string(),
                rule_id,
                kind.key(),
                decision.key(),
                reviewer,
                evidence_notes,
                rule_content_hash,
                evidence_bundle_hash,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn import_rule_review_seed(&self, review: &RuleReviewSeed) -> Result<(), StorageError> {
        if review.review_id.trim().is_empty()
            || review.reviewer.trim().is_empty()
            || review.evidence_notes.trim().is_empty()
        {
            return Err(StorageError::InvalidRuleSeed(format!(
                "规则 {} 的复核记录缺少 ID、复核人或说明",
                review.rule_id
            )));
        }
        let current_rule_hash = self.rule_content_hash(&review.rule_id)?;
        let current_evidence_hash = self.current_evidence_bundle_hash(&review.rule_id)?;
        if current_rule_hash != review.rule_content_hash
            || current_evidence_hash != review.evidence_bundle_hash
        {
            return Err(StorageError::InvalidRuleSeed(format!(
                "复核 {} 未绑定当前规则正文与文献依据",
                review.review_id
            )));
        }
        self.connection.execute(
            "INSERT INTO rule_reviews(
                review_id, rule_id, review_kind, decision, reviewer,
                evidence_notes, rule_content_hash, evidence_bundle_hash, reviewed_at
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(review_id) DO NOTHING",
            params![
                review.review_id,
                review.rule_id,
                review.kind.key(),
                review.decision.key(),
                review.reviewer,
                review.evidence_notes,
                review.rule_content_hash,
                review.evidence_bundle_hash,
                review.reviewed_at,
            ],
        )?;
        let stored = self.connection.query_row(
            "SELECT rule_id, review_kind, decision, reviewer, evidence_notes,
                    rule_content_hash, evidence_bundle_hash, reviewed_at
             FROM rule_reviews WHERE review_id = ?1",
            [&review.review_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                ))
            },
        )?;
        let expected = (
            review.rule_id.clone(),
            review.kind.key().to_owned(),
            review.decision.key().to_owned(),
            review.reviewer.clone(),
            review.evidence_notes.clone(),
            review.rule_content_hash.clone(),
            review.evidence_bundle_hash.clone(),
            review.reviewed_at.clone(),
        );
        if stored != expected {
            return Err(StorageError::InvalidRuleSeed(format!(
                "复核 ID {} 与数据库既有记录冲突",
                review.review_id
            )));
        }
        Ok(())
    }

    pub fn upsert_rule_test_case(
        &self,
        test_case: &RuleTestCaseRecord,
    ) -> Result<String, StorageError> {
        if test_case.test_case_id.trim().is_empty()
            || test_case.rule_id.trim().is_empty()
            || test_case.name.trim().is_empty()
            || test_case.boundary_class.trim().is_empty()
            || test_case.source_basis.trim().is_empty()
        {
            return Err(StorageError::RuleGateValidation(
                "规则测试用例的 ID、Rule ID、名称、边界分类和出处依据均不能为空".to_owned(),
            ));
        }
        if !test_case.input.is_object() {
            return Err(StorageError::RuleProgram(format!(
                "测试用例 {} 的输入快照顶层必须是 JSON 对象",
                test_case.test_case_id
            )));
        }
        parse_rule_program_output(&test_case.expected_output).map_err(|error| {
            StorageError::RuleProgram(format!(
                "测试用例 {} 的预期输出无效：{error}",
                test_case.test_case_id
            ))
        })?;
        let now = Utc::now().to_rfc3339();
        self.connection.execute(
            "INSERT INTO rule_test_cases(
                test_case_id, rule_id, name, input_json, expected_output_json,
                boundary_class, source_basis, enabled, created_at, updated_at
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)
             ON CONFLICT(test_case_id) DO UPDATE SET
                rule_id=excluded.rule_id,
                name=excluded.name,
                input_json=excluded.input_json,
                expected_output_json=excluded.expected_output_json,
                boundary_class=excluded.boundary_class,
                source_basis=excluded.source_basis,
                enabled=excluded.enabled,
                updated_at=excluded.updated_at",
            params![
                test_case.test_case_id,
                test_case.rule_id,
                test_case.name,
                serde_json::to_string(&test_case.input)?,
                serde_json::to_string(&test_case.expected_output)?,
                test_case.boundary_class,
                test_case.source_basis,
                test_case.enabled,
                now,
            ],
        )?;
        self.refresh_rule_test_suite_state(&test_case.rule_id)
    }

    pub fn delete_rule_test_case(&self, test_case_id: &str) -> Result<bool, StorageError> {
        let rule_id = self
            .connection
            .query_row(
                "SELECT rule_id FROM rule_test_cases WHERE test_case_id = ?1",
                [test_case_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        let Some(rule_id) = rule_id else {
            return Ok(false);
        };
        self.connection.execute(
            "DELETE FROM rule_test_cases WHERE test_case_id = ?1",
            [test_case_id],
        )?;
        self.refresh_rule_test_suite_state(&rule_id)?;
        Ok(true)
    }

    fn record_rule_test_run(
        &self,
        rule_id: &str,
        passed_count: u32,
        failed_count: u32,
        environment: &serde_json::Value,
    ) -> Result<RuleTestRunRecord, StorageError> {
        let (test_suite_hash, enabled_case_count) = self.current_test_suite_state(rule_id)?;
        let total_count = passed_count
            .checked_add(failed_count)
            .ok_or_else(|| StorageError::RuleGateValidation("测试数量溢出".to_owned()))?;
        if enabled_case_count == 0 || u64::from(total_count) != enabled_case_count {
            return Err(StorageError::RuleGateValidation(format!(
                "测试结果数量 {total_count} 与当前启用用例数 {enabled_case_count} 不一致"
            )));
        }
        let (rule_content_hash, implementation_version) = self.connection.query_row(
            "SELECT content_hash, implementation_version FROM rules WHERE rule_id = ?1",
            [rule_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )?;
        let record = RuleTestRunRecord {
            run_id: Uuid::new_v4().to_string(),
            rule_id: rule_id.to_owned(),
            rule_content_hash,
            implementation_version,
            test_suite_hash,
            passed_count,
            failed_count,
            total_count,
            executed_at: Utc::now().to_rfc3339(),
        };
        self.connection.execute(
            "INSERT INTO rule_test_runs(
                run_id, rule_id, rule_content_hash, implementation_version,
                test_suite_hash, passed_count, failed_count, total_count,
                environment_json, executed_at
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                record.run_id,
                record.rule_id,
                record.rule_content_hash,
                record.implementation_version,
                record.test_suite_hash,
                record.passed_count,
                record.failed_count,
                record.total_count,
                serde_json::to_string(environment)?,
                record.executed_at,
            ],
        )?;
        Ok(record)
    }

    pub fn run_rule_test_suite(
        &self,
        rule_id: &str,
        environment: &serde_json::Value,
    ) -> Result<RuleTestSuiteExecution, StorageError> {
        let (passed_count, failed_count, failed_case_ids) =
            self.evaluate_rule_test_suite(rule_id)?;
        let run = self.record_rule_test_run(rule_id, passed_count, failed_count, environment)?;
        Ok(RuleTestSuiteExecution {
            run,
            failed_case_ids,
        })
    }

    fn evaluate_rule_test_suite(
        &self,
        rule_id: &str,
    ) -> Result<(u32, u32, Vec<String>), StorageError> {
        let rule = self.rule_for_program(rule_id)?;
        let compiled = compile_rule_program(&rule).map_err(|error| {
            StorageError::RuleProgram(format!("规则 {}：{error}", rule.rule_id))
        })?;
        let mut statement = self.connection.prepare(
            "SELECT test_case_id, input_json, expected_output_json
             FROM rule_test_cases
             WHERE rule_id = ?1 AND enabled = 1
             ORDER BY test_case_id",
        )?;
        let rows = statement.query_map([rule_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        let cases = rows.collect::<Result<Vec<_>, _>>()?;
        if cases.is_empty() {
            return Err(StorageError::RuleGateValidation(format!(
                "规则 {rule_id} 没有启用的测试用例"
            )));
        }

        let mut passed_count = 0_u32;
        let mut failed_case_ids = Vec::new();
        for (test_case_id, input_json, expected_output_json) in cases {
            let input: serde_json::Value = serde_json::from_str(&input_json)?;
            let expected_value: serde_json::Value = serde_json::from_str(&expected_output_json)?;
            let expected = parse_rule_program_output(&expected_value).map_err(|error| {
                StorageError::RuleProgram(format!(
                    "测试用例 {test_case_id} 的预期输出无效：{error}"
                ))
            })?;
            let actual = compiled
                .evaluate(&input)
                .map_err(|error| {
                    StorageError::RuleProgram(format!("测试用例 {test_case_id} 执行失败：{error}"))
                })?
                .auditable_output();
            if actual == expected {
                passed_count = passed_count
                    .checked_add(1)
                    .ok_or_else(|| StorageError::RuleGateValidation("测试数量溢出".to_owned()))?;
            } else {
                failed_case_ids.push(test_case_id);
            }
        }
        let failed_count = u32::try_from(failed_case_ids.len())
            .map_err(|_| StorageError::RuleGateValidation("测试数量溢出".to_owned()))?;
        Ok((passed_count, failed_count, failed_case_ids))
    }

    fn import_rule_test_run_seed(&self, run: &RuleTestRunSeed) -> Result<(), StorageError> {
        if run.run_id.trim().is_empty() || run.total_count == 0 {
            return Err(StorageError::InvalidRuleSeed(format!(
                "规则 {} 的测试运行缺少 ID 或没有测试用例",
                run.rule_id
            )));
        }
        if run.passed_count.checked_add(run.failed_count) != Some(run.total_count) {
            return Err(StorageError::InvalidRuleSeed(format!(
                "测试运行 {} 的通过、失败与总数不一致",
                run.run_id
            )));
        }
        let (current_suite_hash, enabled_count) = self.current_test_suite_state(&run.rule_id)?;
        let (current_rule_hash, current_implementation_version) = self.connection.query_row(
            "SELECT content_hash, implementation_version FROM rules WHERE rule_id = ?1",
            [&run.rule_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )?;
        if current_rule_hash != run.rule_content_hash
            || current_implementation_version != run.implementation_version
            || current_suite_hash != run.test_suite_hash
            || enabled_count != u64::from(run.total_count)
        {
            return Err(StorageError::InvalidRuleSeed(format!(
                "测试运行 {} 未绑定当前规则、实现版本和测试套件",
                run.run_id
            )));
        }
        let (actual_passed, actual_failed, failed_case_ids) =
            self.evaluate_rule_test_suite(&run.rule_id)?;
        if actual_passed != run.passed_count || actual_failed != run.failed_count {
            return Err(StorageError::InvalidRuleSeed(format!(
                "测试运行 {} 的声明结果与内置 v1 规则执行器不一致；失败用例：{}",
                run.run_id,
                if failed_case_ids.is_empty() {
                    "无".to_owned()
                } else {
                    failed_case_ids.join(",")
                }
            )));
        }
        let environment_json = serde_json::to_string(&run.environment)?;
        self.connection.execute(
            "INSERT INTO rule_test_runs(
                run_id, rule_id, rule_content_hash, implementation_version,
                test_suite_hash, passed_count, failed_count, total_count,
                environment_json, executed_at
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(run_id) DO NOTHING",
            params![
                run.run_id,
                run.rule_id,
                run.rule_content_hash,
                run.implementation_version,
                run.test_suite_hash,
                run.passed_count,
                run.failed_count,
                run.total_count,
                environment_json,
                run.executed_at,
            ],
        )?;
        let stored = self.connection.query_row(
            "SELECT rule_id, rule_content_hash, implementation_version,
                    test_suite_hash, passed_count, failed_count, total_count,
                    environment_json, executed_at
             FROM rule_test_runs WHERE run_id = ?1",
            [&run.run_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, u32>(4)?,
                    row.get::<_, u32>(5)?,
                    row.get::<_, u32>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                ))
            },
        )?;
        let expected = (
            run.rule_id.clone(),
            run.rule_content_hash.clone(),
            run.implementation_version.clone(),
            run.test_suite_hash.clone(),
            run.passed_count,
            run.failed_count,
            run.total_count,
            serde_json::to_string(&run.environment)?,
            run.executed_at.clone(),
        );
        if stored != expected {
            return Err(StorageError::InvalidRuleSeed(format!(
                "测试运行 ID {} 与数据库既有记录冲突",
                run.run_id
            )));
        }
        Ok(())
    }

    pub fn rule_gate_report(&self, rule_id: &str) -> Result<Option<RuleGateReport>, StorageError> {
        let row = self
            .connection
            .query_row(
                "SELECT rule_content_hash, rule_fields_ready, primary_source_ready,
                        reviews_ready, tests_ready
                 FROM rule_release_readiness WHERE rule_id = ?1",
                [rule_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, bool>(1)?,
                        row.get::<_, bool>(2)?,
                        row.get::<_, bool>(3)?,
                        row.get::<_, bool>(4)?,
                    ))
                },
            )
            .optional()?;
        let Some((content_hash, fields, source, reviews, tests)) = row else {
            return Ok(None);
        };
        let mut blocking_reasons = Vec::new();
        if !fields {
            blocking_reasons.push("规则字段、稳定性或证据等级尚未达到发布条件".to_owned());
        }
        if !source {
            blocking_reasons.push(
                "缺少已核对的主版本原文、馆藏编号、影印定位或实际文件字节校验记录".to_owned(),
            );
        }
        if !reviews {
            blocking_reasons.push("书目、转录、逻辑、实现四类复核尚未全部通过".to_owned());
        }
        if !tests {
            blocking_reasons.push("当前规则与测试套件尚无完整零失败测试记录".to_owned());
        }
        let checks =
            self.rule_gate_checks(rule_id, &content_hash, fields, source, reviews, tests)?;
        Ok(Some(RuleGateReport {
            rule_id: rule_id.to_owned(),
            rule_content_hash: content_hash,
            rule_fields_ready: fields,
            primary_source_ready: source,
            reviews_ready: reviews,
            tests_ready: tests,
            checks,
            blocking_reasons,
        }))
    }

    fn rule_gate_checks(
        &self,
        rule_id: &str,
        content_hash: &str,
        fields_ready: bool,
        primary_source_ready: bool,
        reviews_ready: bool,
        tests_ready: bool,
    ) -> Result<Vec<RuleGateCheck>, StorageError> {
        let mut checks = Vec::new();
        let (
            reviewed,
            stability,
            evidence_tier,
            original_text,
            source_reference,
            implementation_version,
        ) = self.connection.query_row(
            "SELECT reviewed, stability, evidence_tier, original_text,
                    source_reference, implementation_version
             FROM rules WHERE rule_id = ?1",
            [rule_id],
            |row| {
                Ok((
                    row.get::<_, bool>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )?;
        checks.push(RuleGateCheck {
            key: "fields.reviewed".to_owned(),
            label: "规则审核标记".to_owned(),
            passed: reviewed,
            detail: if reviewed {
                "规则已明确标记为审核完成".to_owned()
            } else {
                "reviewed 尚未置为 true".to_owned()
            },
        });
        checks.push(RuleGateCheck {
            key: "fields.stability".to_owned(),
            label: "规则稳定性".to_owned(),
            passed: stability == "stable",
            detail: format!("当前 stability={stability}"),
        });
        checks.push(RuleGateCheck {
            key: "fields.evidence_tier".to_owned(),
            label: "古籍证据等级".to_owned(),
            passed: evidence_tier != "unverified",
            detail: format!("当前 evidence_tier={evidence_tier}"),
        });
        checks.push(RuleGateCheck {
            key: "fields.original_text".to_owned(),
            label: "规则原文".to_owned(),
            passed: !original_text.trim().is_empty(),
            detail: if original_text.trim().is_empty() {
                "规则记录缺少原文".to_owned()
            } else {
                format!("已保存 {} 个 Unicode 字符", original_text.chars().count())
            },
        });
        checks.push(RuleGateCheck {
            key: "fields.source_reference".to_owned(),
            label: "规则出处定位".to_owned(),
            passed: !source_reference.trim().is_empty(),
            detail: if source_reference.trim().is_empty() {
                "规则记录缺少出处定位".to_owned()
            } else {
                source_reference.trim().to_owned()
            },
        });
        let program_check = self.rule_for_program(rule_id).and_then(|rule| {
            compile_rule_program(&rule)
                .map_err(|error| StorageError::RuleProgram(format!("规则 {rule_id}：{error}")))
        });
        checks.push(RuleGateCheck {
            key: "fields.program".to_owned(),
            label: "确定性规则程序".to_owned(),
            passed: program_check.is_ok(),
            detail: match program_check {
                Ok(_) => "条件、结果、应期与行动字段可按当前程序版本编译".to_owned(),
                Err(error) => format!("编译失败：{error}"),
            },
        });
        checks.push(RuleGateCheck {
            key: "fields.release_view".to_owned(),
            label: "规则字段总门禁".to_owned(),
            passed: fields_ready,
            detail: if fields_ready {
                "数据库发布视图确认字段层通过".to_owned()
            } else {
                "数据库发布视图仍拒绝字段层".to_owned()
            },
        });

        let primary_links: u64 = self.connection.query_row(
            "SELECT COUNT(*) FROM rule_source_links
             WHERE rule_id = ?1 AND relation = 'primary'",
            [rule_id],
            |row| row.get(0),
        )?;
        let verified_passages: u64 = self.connection.query_row(
            "SELECT COUNT(*)
             FROM rule_source_links l
             JOIN source_passages p ON p.passage_id = l.passage_id
             WHERE l.rule_id = ?1 AND l.relation = 'primary'
               AND p.transcription_status = 'verified'
               AND length(trim(p.folio_or_page)) > 0
               AND length(trim(p.original_text)) > 0",
            [rule_id],
            |row| row.get(0),
        )?;
        let qualified_editions: u64 = self.connection.query_row(
            "SELECT COUNT(*)
             FROM rule_source_links l
             JOIN source_passages p ON p.passage_id = l.passage_id
             JOIN source_editions e ON e.source_id = p.source_id
             WHERE l.rule_id = ?1 AND l.relation = 'primary'
               AND e.edition_type IN ('facsimile', 'collated', 'critical')
               AND length(trim(e.catalog_identifier)) > 0
               AND length(trim(e.facsimile_reference)) > 0
               AND length(e.digital_checksum) = 71
               AND substr(e.digital_checksum, 1, 7) = 'sha256:'
               AND substr(e.digital_checksum, 8) NOT GLOB '*[^0-9a-f]*'",
            [rule_id],
            |row| row.get(0),
        )?;
        let matching_verifications: u64 = self.connection.query_row(
            "SELECT COUNT(*)
             FROM rule_source_links l
             JOIN source_passages p ON p.passage_id = l.passage_id
             JOIN source_editions e ON e.source_id = p.source_id
             JOIN source_facsimile_verifications v ON v.source_id = e.source_id
             WHERE l.rule_id = ?1 AND l.relation = 'primary'
               AND v.digital_checksum = e.digital_checksum
               AND v.byte_length > 0
               AND v.verifier_version = 'jiuheng.sha256.v1'
               AND length(trim(v.file_name)) > 0
               AND length(trim(v.verified_at)) > 0",
            [rule_id],
            |row| row.get(0),
        )?;
        let evidence_bundle_hash = self
            .connection
            .query_row(
                "SELECT evidence_bundle_hash FROM rule_evidence_state WHERE rule_id = ?1",
                [rule_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .unwrap_or_default();
        for (key, label, count, missing) in [
            (
                "source.primary_link",
                "主原文关联",
                primary_links,
                "没有 relation=primary 的规则—原文关联",
            ),
            (
                "source.verified_passage",
                "已核卷页原文",
                verified_passages,
                "没有同时具备卷页、原文和 verified 转录状态的主片段",
            ),
            (
                "source.qualified_edition",
                "合格版本书目",
                qualified_editions,
                "没有同时具备馆藏号、影印定位和规范 SHA-256 的影印/校勘/整理本",
            ),
            (
                "source.facsimile_bytes",
                "影印文件字节验证",
                matching_verifications,
                "没有与版本 SHA-256 完全匹配的本地文件字节验证记录",
            ),
        ] {
            checks.push(RuleGateCheck {
                key: key.to_owned(),
                label: label.to_owned(),
                passed: count > 0,
                detail: if count > 0 {
                    format!("匹配记录 {count} 条")
                } else {
                    missing.to_owned()
                },
            });
        }
        checks.push(RuleGateCheck {
            key: "source.evidence_bundle".to_owned(),
            label: "证据包哈希".to_owned(),
            passed: !evidence_bundle_hash.is_empty(),
            detail: if evidence_bundle_hash.is_empty() {
                "当前规则的证据包尚未刷新为非空哈希".to_owned()
            } else {
                evidence_bundle_hash.clone()
            },
        });
        checks.push(RuleGateCheck {
            key: "source.release_view".to_owned(),
            label: "主来源总门禁".to_owned(),
            passed: primary_source_ready,
            detail: if primary_source_ready {
                "数据库发布视图确认至少一条完整主来源链通过".to_owned()
            } else {
                "尚无一条主来源链同时满足全部条件".to_owned()
            },
        });

        for kind in ReviewKind::ALL {
            let approved: u64 = self.connection.query_row(
                "SELECT COUNT(*)
                 FROM rule_reviews v
                 JOIN rule_evidence_state s ON s.rule_id = v.rule_id
                 WHERE v.rule_id = ?1
                   AND v.rule_content_hash = ?2
                   AND v.evidence_bundle_hash = s.evidence_bundle_hash
                   AND length(s.evidence_bundle_hash) > 0
                   AND v.review_kind = ?3
                   AND v.decision = 'approved'",
                params![rule_id, content_hash, kind.key()],
                |row| row.get(0),
            )?;
            let label = match kind {
                ReviewKind::Bibliography => "书目复核",
                ReviewKind::Transcription => "转录复核",
                ReviewKind::Logic => "逻辑复核",
                ReviewKind::Implementation => "实现复核",
            };
            checks.push(RuleGateCheck {
                key: format!("review.{}", kind.key()),
                label: label.to_owned(),
                passed: approved > 0,
                detail: if approved > 0 {
                    format!("当前规则与证据包已有 {approved} 条 approved 记录")
                } else {
                    "缺少绑定当前规则哈希与当前证据包哈希的 approved 记录".to_owned()
                },
            });
        }
        let rejected_reviews: u64 = self.connection.query_row(
            "SELECT COUNT(*)
             FROM rule_reviews v
             JOIN rule_evidence_state s ON s.rule_id = v.rule_id
             WHERE v.rule_id = ?1
               AND v.rule_content_hash = ?2
               AND v.evidence_bundle_hash = s.evidence_bundle_hash
               AND v.decision = 'rejected'",
            params![rule_id, content_hash],
            |row| row.get(0),
        )?;
        checks.push(RuleGateCheck {
            key: "review.no_rejection".to_owned(),
            label: "无否决复核".to_owned(),
            passed: rejected_reviews == 0,
            detail: if rejected_reviews == 0 {
                "当前规则与证据包没有 rejected 记录".to_owned()
            } else {
                format!("存在 {rejected_reviews} 条 rejected 记录")
            },
        });
        checks.push(RuleGateCheck {
            key: "review.release_view".to_owned(),
            label: "四类复核总门禁".to_owned(),
            passed: reviews_ready,
            detail: if reviews_ready {
                "数据库发布视图确认四类复核通过且无否决".to_owned()
            } else {
                "数据库发布视图仍拒绝复核层".to_owned()
            },
        });

        let enabled_cases: u64 = self.connection.query_row(
            "SELECT COUNT(*) FROM rule_test_cases WHERE rule_id = ?1 AND enabled = 1",
            [rule_id],
            |row| row.get(0),
        )?;
        checks.push(RuleGateCheck {
            key: "tests.enabled_cases".to_owned(),
            label: "启用测试用例".to_owned(),
            passed: enabled_cases > 0,
            detail: if enabled_cases > 0 {
                format!("当前启用 {enabled_cases} 条用例")
            } else {
                "没有启用的测试用例".to_owned()
            },
        });
        let suite_state = self
            .connection
            .query_row(
                "SELECT test_suite_hash, enabled_case_count
                 FROM rule_test_suite_state WHERE rule_id = ?1",
                [rule_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?)),
            )
            .optional()?;
        let suite_ready = suite_state.as_ref().is_some_and(|(hash, count)| {
            !hash.is_empty() && *count == enabled_cases && enabled_cases > 0
        });
        checks.push(RuleGateCheck {
            key: "tests.suite_hash".to_owned(),
            label: "当前测试套件哈希".to_owned(),
            passed: suite_ready,
            detail: match suite_state {
                Some((hash, count)) if suite_ready => {
                    format!("{hash}；用例数 {count}")
                }
                Some((hash, count)) => format!(
                    "套件状态未闭合：hash={}，登记用例数 {}，实际启用 {}",
                    if hash.is_empty() { "空" } else { &hash },
                    count,
                    enabled_cases
                ),
                None => "没有测试套件状态记录".to_owned(),
            },
        });
        let passing_current_run: u64 = self.connection.query_row(
            "SELECT COUNT(*)
             FROM rule_test_runs t
             JOIN rule_test_suite_state s ON s.rule_id = t.rule_id
             WHERE t.rule_id = ?1
               AND t.rule_content_hash = ?2
               AND t.implementation_version = ?3
               AND t.test_suite_hash = s.test_suite_hash
               AND t.failed_count = 0
               AND t.total_count = s.enabled_case_count
               AND t.executed_at = (
                   SELECT MAX(t2.executed_at) FROM rule_test_runs t2
                   WHERE t2.rule_id = t.rule_id
                     AND t2.rule_content_hash = t.rule_content_hash
                     AND t2.implementation_version = t.implementation_version
               )",
            params![rule_id, content_hash, implementation_version],
            |row| row.get(0),
        )?;
        checks.push(RuleGateCheck {
            key: "tests.current_zero_failure_run".to_owned(),
            label: "当前版本零失败运行".to_owned(),
            passed: passing_current_run > 0,
            detail: if passing_current_run > 0 {
                "最新当前规则/实现/套件运行覆盖全部启用用例且 0 失败".to_owned()
            } else {
                "没有匹配当前规则哈希、实现版本和套件哈希的最新零失败运行".to_owned()
            },
        });
        checks.push(RuleGateCheck {
            key: "tests.release_view".to_owned(),
            label: "测试总门禁".to_owned(),
            passed: tests_ready,
            detail: if tests_ready {
                "数据库发布视图确认测试层通过".to_owned()
            } else {
                "数据库发布视图仍拒绝测试层".to_owned()
            },
        });
        Ok(checks)
    }

    pub fn rule_evidence_summary(&self) -> Result<RuleEvidenceSummary, StorageError> {
        Ok(RuleEvidenceSummary {
            total_rules: self.table_count("rules")?,
            executable_rules: self.connection.query_row(
                "SELECT COUNT(*) FROM rule_release_readiness
                 WHERE rule_fields_ready = 1 AND primary_source_ready = 1
                   AND reviews_ready = 1 AND tests_ready = 1",
                [],
                |row| row.get(0),
            )?,
            source_editions: self.table_count("source_editions")?,
            facsimile_verifications: self.table_count("source_facsimile_verifications")?,
            source_passages: self.table_count("source_passages")?,
            reviews: self.table_count("rule_reviews")?,
            enabled_test_cases: self.connection.query_row(
                "SELECT COUNT(*) FROM rule_test_cases WHERE enabled = 1",
                [],
                |row| row.get(0),
            )?,
            test_runs: self.table_count("rule_test_runs")?,
        })
    }

    pub fn released_rules(
        &self,
        method: DivinationMethod,
        question_type: &AncientQuestionType,
    ) -> Result<Vec<ReleasedRuleRecord>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT r.rule_id, r.method, r.applicable_question_types_json,
                    r.condition_expression, r.result_expression, r.timing_expression,
                    r.action_key, r.action_polarity, r.action_expression,
                    r.priority, r.evidence_tier, r.historical_period, r.book_title,
                    r.volume, r.chapter, r.original_text, r.edition,
                    r.source_reference, r.textual_variants, r.stability,
                    r.implementation_version, r.reviewed, r.content_hash
             FROM rules r
             JOIN rule_release_readiness gate ON gate.rule_id = r.rule_id
             WHERE r.method = ?1
               AND gate.rule_fields_ready = 1
               AND gate.primary_source_ready = 1
               AND gate.reviews_ready = 1
               AND gate.tests_ready = 1
             ORDER BY r.priority DESC, r.rule_id",
        )?;
        let rows = statement.query_map([method.key()], |row| {
            Ok(StoredRuleRow {
                rule_id: row.get(0)?,
                method: row.get(1)?,
                applicable_question_types_json: row.get(2)?,
                condition_expression: row.get(3)?,
                result_expression: row.get(4)?,
                timing_expression: row.get(5)?,
                action_key: row.get(6)?,
                action_polarity: row.get(7)?,
                action_expression: row.get(8)?,
                priority: row.get(9)?,
                evidence_tier: row.get(10)?,
                historical_period: row.get(11)?,
                book_title: row.get(12)?,
                volume: row.get(13)?,
                chapter: row.get(14)?,
                original_text: row.get(15)?,
                edition: row.get(16)?,
                source_reference: row.get(17)?,
                textual_variants: row.get(18)?,
                stability: row.get(19)?,
                implementation_version: row.get(20)?,
                reviewed: row.get(21)?,
                content_hash: row.get(22)?,
            })
        })?;
        let mut released = Vec::new();
        for row in rows {
            let stored = row?;
            let content_hash = stored.content_hash.clone();
            let rule = stored.into_rule()?;
            if !rule.may_execute() {
                return Err(StorageError::RuleGateValidation(format!(
                    "规则 {} 的运行时字段不满足发布条件",
                    rule.rule_id
                )));
            }
            compile_rule_program(&rule).map_err(|error| {
                StorageError::RuleProgram(format!("规则 {}：{error}", rule.rule_id))
            })?;
            let recalculated = canonical_rule_hash(&rule)?;
            if recalculated != content_hash {
                return Err(StorageError::ArtifactIntegrity(format!(
                    "规则 {} 的正文哈希与存储索引不一致",
                    rule.rule_id
                )));
            }
            if rule.applicable_question_types.contains(question_type) {
                released.push(ReleasedRuleRecord { rule, content_hash });
            }
        }
        Ok(released)
    }

                                                                           
                                                                              
                                                                           
                                                                             
    pub fn execute_and_store_rule_method(
        &mut self,
        question_id: Uuid,
        method: DivinationMethod,
        implementation_version: &str,
        chart: &serde_json::Value,
    ) -> Result<RuleMethodExecution, StorageError> {
        if !valid_runtime_identifier(implementation_version) {
            return Err(StorageError::RuleRuntime(
                "术式实现版本必须为 1 至 128 个小写 ASCII 字母、数字、点、下划线或连字符"
                    .to_owned(),
            ));
        }
        if self.load_fusion_decision(question_id)?.is_some() {
            return Err(StorageError::RuleRuntime(
                "占问已经形成不可变综合裁决，不能追加术式运行".to_owned(),
            ));
        }
        let existing: bool = self.connection.query_row(
            "SELECT EXISTS(
                SELECT 1 FROM engine_runs WHERE question_id = ?1 AND method = ?2
             )",
            params![question_id.to_string(), method.key()],
            |row| row.get(0),
        )?;
        if existing {
            return Err(StorageError::RuleRuntime(format!(
                "{} 已经为本占问留下术式运行记录，禁止覆盖或重复执行",
                method.zh_name()
            )));
        }

        let (question_json, ancient_type_json, locked) = self
            .connection
            .query_row(
                "SELECT parameters_json, ancient_question_type, locked
                 FROM questions WHERE id = ?1",
                [question_id.to_string()],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, bool>(2)?,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| StorageError::RuleRuntime("占问记录不存在".to_owned()))?;
        if !locked {
            return Err(StorageError::RuleRuntime(
                "占问尚未锁定，不能执行规则".to_owned(),
            ));
        }
        let question: QuestionInput = serde_json::from_str(&question_json)?;
        if question.id != question_id {
            return Err(StorageError::RuleRuntime(
                "占问参数中的 Question ID 与数据库索引不一致".to_owned(),
            ));
        }
        let ancient_type: AncientQuestionType = serde_json::from_str(&ancient_type_json)?;
        if ancient_type == AncientQuestionType::WeatherRain {
            let scope = assess_first_rain_scope(&question);
            if !scope.in_research_scope {
                return Err(StorageError::RuleRuntime(format!(
                    "天气雨雪旧分类不能直接触发降雨规则：{}",
                    scope.reasons.join("；")
                )));
            }
        }
        let input =
            RuleRuntimeInputSnapshot::new(method, ancient_type.clone(), question, chart.clone())
                .map_err(|error| StorageError::RuleRuntime(error.to_string()))?;
        let input_value = input
            .to_value()
            .map_err(|error| StorageError::RuleRuntime(error.to_string()))?;
        let input_json = serde_json::to_string(&input_value)?;
        let chart_json = serde_json::to_string(chart)?;

        let released = self.released_rules(method, &ancient_type)?;
        let evaluated_rule_count = released.len();
        let mut matches = Vec::new();
        for item in &released {
            let evaluation = item.evaluate(&input_value)?;
            if evaluation.matched {
                matches.push(RuleRuntimeMatch {
                    rule_id: item.rule.rule_id.clone(),
                    evidence_tier: item.rule.evidence_tier,
                    evaluation,
                });
            }
        }

        let result = if matches.is_empty() {
            None
        } else {
            Some(
                aggregate_rule_matches(method, chart.clone(), &matches)
                    .map_err(|error| StorageError::RuleRuntime(error.to_string()))?,
            )
        };
        let status = if result.is_some() {
            ApplicabilityStatus::Applicable
        } else {
            ApplicabilityStatus::InsufficientEvidence
        };
        let reasons = if released.is_empty() {
            vec!["当前术式和古代问题类型没有已放行规则；失败关闭".to_owned()]
        } else if matches.is_empty() {
            vec!["已放行规则均未命中当前不可变输入快照；本术式不参与".to_owned()]
        } else {
            vec![format!(
                "{} 条已放行规则命中并通过无冲突术内合并",
                matches.len()
            )]
        };
        let reasons_json = serde_json::to_string(&reasons)?;
        let signals_json = result
            .as_ref()
            .map(|value| serde_json::to_string(&value.signals))
            .transpose()?;
        let evidence_tier = result
            .as_ref()
            .map(|value| evidence_tier_key(value.evidence_tier));
        let executed_at = Utc::now().to_rfc3339();

        let tx = self.connection.transaction()?;
        tx.execute(
            "INSERT INTO engine_runs(
                question_id, method, implementation_version,
                applicability_status, applicability_reasons_json,
                chart_json, signals_json, evidence_tier, started_at, completed_at
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
            params![
                question_id.to_string(),
                method.key(),
                implementation_version,
                applicability_status_key(status),
                reasons_json,
                chart_json,
                signals_json,
                evidence_tier,
                executed_at,
            ],
        )?;
        let engine_run_id = tx.last_insert_rowid();
        for item in &matches {
            tx.execute(
                "INSERT INTO rule_hits(
                    engine_run_id, rule_id, input_snapshot_json,
                    output_snapshot_json, evaluation_trace_json
                 ) VALUES(?1, ?2, ?3, ?4, ?5)",
                params![
                    engine_run_id,
                    item.rule_id,
                    input_json,
                    serde_json::to_string(&item.evaluation.auditable_output())?,
                    serde_json::to_string(&item.evaluation.trace)?,
                ],
            )?;
        }
        tx.commit()?;

        Ok(RuleMethodExecution {
            method,
            status,
            reasons,
            evaluated_rule_count,
            matched_rule_ids: matches.iter().map(|item| item.rule_id.clone()).collect(),
            result,
        })
    }

    pub fn list_rule_gate_reports(&self) -> Result<Vec<RuleGateListItem>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT rule_id, method, book_title, chapter,
                    action_key, action_polarity, action_expression
             FROM rules ORDER BY method, priority DESC, rule_id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
            ))
        })?;
        let mut items = Vec::new();
        for row in rows {
            let (
                rule_id,
                method,
                book_title,
                chapter,
                action_key,
                action_polarity,
                action_expression,
            ) = row?;
            let report = self.rule_gate_report(&rule_id)?.ok_or_else(|| {
                StorageError::RuleGateValidation(format!("规则 {rule_id} 缺少发布门禁视图记录"))
            })?;
            items.push(RuleGateListItem {
                rule_id,
                method: method_from_key(&method)?,
                book_title,
                chapter,
                action_key,
                action_polarity: action_polarity
                    .as_deref()
                    .map(action_polarity_from_key)
                    .transpose()?,
                action_expression,
                report,
            });
        }
        Ok(items)
    }

    pub fn rule_evidence_dossier(
        &self,
        rule_id: &str,
    ) -> Result<Option<RuleEvidenceDossier>, StorageError> {
        let stored_state = self
            .connection
            .query_row(
                "SELECT r.content_hash,
                        COALESCE(e.evidence_bundle_hash, ''),
                        COALESCE(t.test_suite_hash, '')
                 FROM rules r
                 LEFT JOIN rule_evidence_state e ON e.rule_id = r.rule_id
                 LEFT JOIN rule_test_suite_state t ON t.rule_id = r.rule_id
                 WHERE r.rule_id = ?1",
                [rule_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()?;
        let Some((rule_content_hash, evidence_bundle_hash, test_suite_hash)) = stored_state else {
            return Ok(None);
        };

        let rule = self.rule_for_program(rule_id)?;
        let calculated_rule_content_hash = canonical_rule_hash(&rule)?;
        let calculated_evidence_bundle_hash =
            calculate_evidence_bundle_hash(&self.connection, rule_id)?;
        let (calculated_test_suite_hash, _) =
            calculate_test_suite_state(&self.connection, rule_id)?;
        let gate = self.rule_gate_report(rule_id)?.ok_or_else(|| {
            StorageError::RuleGateValidation(format!("规则 {rule_id} 缺少发布门禁视图记录"))
        })?;

        let mut source_statement = self.connection.prepare(
            "SELECT l.relation, l.notes,
                    p.passage_id, p.source_id, p.volume, p.chapter,
                    p.folio_or_page, p.original_text, p.normalized_text,
                    p.textual_variants, p.transcription_status,
                    e.canonical_title, e.attributed_author, e.historical_period,
                    e.edition_title, e.edition_type, e.edition_year,
                    e.holding_institution, e.catalog_identifier,
                    e.facsimile_reference, e.digital_checksum, e.notes,
                    v.digital_checksum, v.byte_length, v.file_name,
                    v.verifier_version, v.verified_at
             FROM rule_source_links l
             JOIN source_passages p ON p.passage_id = l.passage_id
             JOIN source_editions e ON e.source_id = p.source_id
             LEFT JOIN source_facsimile_verifications v ON v.source_id = e.source_id
             WHERE l.rule_id = ?1
             ORDER BY l.relation, e.source_id, p.passage_id",
        )?;
        let source_rows = source_statement.query_map([rule_id], |row| {
            Ok(StoredRuleSourceEvidenceRow {
                relation: row.get(0)?,
                link_notes: row.get(1)?,
                passage_id: row.get(2)?,
                source_id: row.get(3)?,
                passage_volume: row.get(4)?,
                passage_chapter: row.get(5)?,
                folio_or_page: row.get(6)?,
                passage_original_text: row.get(7)?,
                normalized_text: row.get(8)?,
                passage_textual_variants: row.get(9)?,
                transcription_status: row.get(10)?,
                canonical_title: row.get(11)?,
                attributed_author: row.get(12)?,
                source_historical_period: row.get(13)?,
                edition_title: row.get(14)?,
                edition_type: row.get(15)?,
                edition_year: row.get(16)?,
                holding_institution: row.get(17)?,
                catalog_identifier: row.get(18)?,
                facsimile_reference: row.get(19)?,
                source_digital_checksum: row.get(20)?,
                source_notes: row.get(21)?,
                verification_checksum: row.get(22)?,
                verification_byte_length: row.get(23)?,
                verification_file_name: row.get(24)?,
                verification_version: row.get(25)?,
                verification_time: row.get(26)?,
            })
        })?;
        let mut sources = Vec::new();
        for row in source_rows {
            sources.push(row?.into_evidence()?);
        }

        let mut review_statement = self.connection.prepare(
            "SELECT review_id, review_kind, decision, reviewer, evidence_notes,
                    rule_content_hash, evidence_bundle_hash, reviewed_at
             FROM rule_reviews WHERE rule_id = ?1
             ORDER BY reviewed_at DESC, review_kind, review_id",
        )?;
        let review_rows = review_statement.query_map([rule_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
            ))
        })?;
        let mut reviews = Vec::new();
        for row in review_rows {
            let (
                review_id,
                kind,
                decision,
                reviewer,
                evidence_notes,
                review_rule_hash,
                review_evidence_hash,
                reviewed_at,
            ) = row?;
            reviews.push(RuleReviewSeed {
                review_id,
                rule_id: rule_id.to_owned(),
                kind: review_kind_from_key(&kind)?,
                decision: review_decision_from_key(&decision)?,
                reviewer,
                evidence_notes,
                rule_content_hash: review_rule_hash,
                evidence_bundle_hash: review_evidence_hash,
                reviewed_at,
            });
        }

        let mut case_statement = self.connection.prepare(
            "SELECT test_case_id, name, input_json, expected_output_json,
                    boundary_class, source_basis, enabled
             FROM rule_test_cases WHERE rule_id = ?1 ORDER BY test_case_id",
        )?;
        let case_rows = case_statement.query_map([rule_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, bool>(6)?,
            ))
        })?;
        let mut test_cases = Vec::new();
        for row in case_rows {
            let (test_case_id, name, input, expected_output, boundary_class, source_basis, enabled) =
                row?;
            test_cases.push(RuleTestCaseRecord {
                test_case_id,
                rule_id: rule_id.to_owned(),
                name,
                input: serde_json::from_str(&input)?,
                expected_output: serde_json::from_str(&expected_output)?,
                boundary_class,
                source_basis,
                enabled,
            });
        }

        let mut run_statement = self.connection.prepare(
            "SELECT run_id, rule_content_hash, implementation_version,
                    test_suite_hash, passed_count, failed_count, total_count,
                    environment_json, executed_at
             FROM rule_test_runs WHERE rule_id = ?1
             ORDER BY executed_at DESC, run_id",
        )?;
        let run_rows = run_statement.query_map([rule_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, u32>(4)?,
                row.get::<_, u32>(5)?,
                row.get::<_, u32>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
            ))
        })?;
        let mut test_runs = Vec::new();
        for row in run_rows {
            let (
                run_id,
                run_rule_hash,
                implementation_version,
                run_suite_hash,
                passed_count,
                failed_count,
                total_count,
                environment,
                executed_at,
            ) = row?;
            test_runs.push(RuleTestRunSeed {
                run_id,
                rule_id: rule_id.to_owned(),
                rule_content_hash: run_rule_hash,
                implementation_version,
                test_suite_hash: run_suite_hash,
                passed_count,
                failed_count,
                total_count,
                environment: serde_json::from_str(&environment)?,
                executed_at,
            });
        }

        Ok(Some(RuleEvidenceDossier {
            rule,
            rule_content_hash: rule_content_hash.clone(),
            calculated_rule_content_hash: calculated_rule_content_hash.clone(),
            rule_content_hash_matches: rule_content_hash == calculated_rule_content_hash,
            evidence_bundle_hash: evidence_bundle_hash.clone(),
            calculated_evidence_bundle_hash: calculated_evidence_bundle_hash.clone(),
            evidence_bundle_hash_matches: evidence_bundle_hash == calculated_evidence_bundle_hash,
            test_suite_hash: test_suite_hash.clone(),
            calculated_test_suite_hash: calculated_test_suite_hash.clone(),
            test_suite_hash_matches: test_suite_hash == calculated_test_suite_hash,
            gate,
            sources,
            reviews,
            test_cases,
            test_runs,
        }))
    }

    fn rule_content_hash(&self, rule_id: &str) -> Result<String, StorageError> {
        self.connection
            .query_row(
                "SELECT content_hash FROM rules WHERE rule_id = ?1",
                [rule_id],
                |row| row.get(0),
            )
            .map_err(StorageError::from)
    }

    fn rule_for_program(&self, rule_id: &str) -> Result<RuleRecord, StorageError> {
        let stored = self.connection.query_row(
            "SELECT rule_id, method, applicable_question_types_json,
                    condition_expression, result_expression, timing_expression,
                    action_key, action_polarity, action_expression,
                    priority, evidence_tier, historical_period, book_title,
                    volume, chapter, original_text, edition,
                    source_reference, textual_variants, stability,
                    implementation_version, reviewed, content_hash
             FROM rules WHERE rule_id = ?1",
            [rule_id],
            |row| {
                Ok(StoredRuleRow {
                    rule_id: row.get(0)?,
                    method: row.get(1)?,
                    applicable_question_types_json: row.get(2)?,
                    condition_expression: row.get(3)?,
                    result_expression: row.get(4)?,
                    timing_expression: row.get(5)?,
                    action_key: row.get(6)?,
                    action_polarity: row.get(7)?,
                    action_expression: row.get(8)?,
                    priority: row.get(9)?,
                    evidence_tier: row.get(10)?,
                    historical_period: row.get(11)?,
                    book_title: row.get(12)?,
                    volume: row.get(13)?,
                    chapter: row.get(14)?,
                    original_text: row.get(15)?,
                    edition: row.get(16)?,
                    source_reference: row.get(17)?,
                    textual_variants: row.get(18)?,
                    stability: row.get(19)?,
                    implementation_version: row.get(20)?,
                    reviewed: row.get(21)?,
                    content_hash: row.get(22)?,
                })
            },
        )?;
        stored.into_rule()
    }

    fn refresh_rule_evidence_state(&self, rule_id: &str) -> Result<String, StorageError> {
        let evidence_bundle_hash = calculate_evidence_bundle_hash(&self.connection, rule_id)?;
        self.connection.execute(
            "INSERT INTO rule_evidence_state(rule_id, evidence_bundle_hash, updated_at)
             VALUES(?1, ?2, ?3)
             ON CONFLICT(rule_id) DO UPDATE SET
                evidence_bundle_hash=excluded.evidence_bundle_hash,
                updated_at=excluded.updated_at",
            params![rule_id, evidence_bundle_hash, Utc::now().to_rfc3339()],
        )?;
        Ok(evidence_bundle_hash)
    }

    fn current_evidence_bundle_hash(&self, rule_id: &str) -> Result<String, StorageError> {
        let row = self
            .connection
            .query_row(
                "SELECT evidence_bundle_hash FROM rule_evidence_state WHERE rule_id = ?1",
                [rule_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        match row {
            Some(hash) if !hash.is_empty() => Ok(hash),
            _ => self.refresh_rule_evidence_state(rule_id),
        }
    }

    fn rule_ids_for_passage(&self, passage_id: &str) -> Result<Vec<String>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT DISTINCT rule_id FROM rule_source_links
             WHERE passage_id = ?1 ORDER BY rule_id",
        )?;
        let rows = statement.query_map([passage_id], |row| row.get::<_, String>(0))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StorageError::from)
    }

    fn rule_ids_for_source(&self, source_id: &str) -> Result<Vec<String>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT DISTINCT l.rule_id
             FROM rule_source_links l
             JOIN source_passages p ON p.passage_id = l.passage_id
             WHERE p.source_id = ?1 ORDER BY l.rule_id",
        )?;
        let rows = statement.query_map([source_id], |row| row.get::<_, String>(0))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StorageError::from)
    }

    fn refresh_rule_test_suite_state(&self, rule_id: &str) -> Result<String, StorageError> {
        let (test_suite_hash, enabled_case_count) =
            calculate_test_suite_state(&self.connection, rule_id)?;
        self.connection.execute(
            "INSERT INTO rule_test_suite_state(rule_id, test_suite_hash, enabled_case_count, updated_at)
             VALUES(?1, ?2, ?3, ?4)
             ON CONFLICT(rule_id) DO UPDATE SET
                test_suite_hash=excluded.test_suite_hash,
                enabled_case_count=excluded.enabled_case_count,
                updated_at=excluded.updated_at",
            params![
                rule_id,
                test_suite_hash,
                enabled_case_count,
                Utc::now().to_rfc3339()
            ],
        )?;
        Ok(test_suite_hash)
    }

    fn current_test_suite_state(&self, rule_id: &str) -> Result<(String, u64), StorageError> {
        let row = self
            .connection
            .query_row(
                "SELECT test_suite_hash, enabled_case_count
                 FROM rule_test_suite_state WHERE rule_id = ?1",
                [rule_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?)),
            )
            .optional()?;
        match row {
            Some((hash, count)) if !hash.is_empty() => Ok((hash, count)),
            _ => {
                self.refresh_rule_test_suite_state(rule_id)?;
                self.connection
                    .query_row(
                        "SELECT test_suite_hash, enabled_case_count
                     FROM rule_test_suite_state WHERE rule_id = ?1",
                        [rule_id],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .map_err(StorageError::from)
            }
        }
    }

    fn table_count(&self, table: &str) -> Result<u64, StorageError> {
        let sql = match table {
            "rules" => "SELECT COUNT(*) FROM rules",
            "source_editions" => "SELECT COUNT(*) FROM source_editions",
            "source_facsimile_verifications" => {
                "SELECT COUNT(*) FROM source_facsimile_verifications"
            }
            "source_passages" => "SELECT COUNT(*) FROM source_passages",
            "rule_reviews" => "SELECT COUNT(*) FROM rule_reviews",
            "rule_test_runs" => "SELECT COUNT(*) FROM rule_test_runs",
            _ => {
                return Err(StorageError::RuleGateValidation(
                    "拒绝查询未知数据表".to_owned(),
                ))
            }
        };
        self.connection
            .query_row(sql, [], |row| row.get(0))
            .map_err(StorageError::from)
    }

    pub fn find_duplicate_event(
        &self,
        fingerprint: &EventFingerprint,
    ) -> Result<Option<DuplicateEvent>, StorageError> {
        let exact = self
            .connection
            .query_row(
                "SELECT id, question_text, locked_at_bjt FROM questions
                 WHERE event_fingerprint = ?1
                 ORDER BY created_at DESC LIMIT 1",
                [&fingerprint.digest_hex],
                |row| {
                    let id: String = row.get(0)?;
                    Ok(DuplicateEvent {
                        question_id: Uuid::parse_str(&id).map_err(|error| {
                            rusqlite::Error::FromSqlConversionFailure(
                                0,
                                rusqlite::types::Type::Text,
                                Box::new(error),
                            )
                        })?,
                        question_text: row.get(1)?,
                        locked_at_bjt: row.get(2)?,
                        algorithm_version: EVENT_MATCH_ALGORITHM_VERSION.to_owned(),
                        exact_digest: true,
                        score_percent: 100,
                        match_reasons: vec!["事件指纹完全一致".to_owned()],
                    })
                },
            )
            .optional()
            .map_err(StorageError::from)?;
        if exact.is_some() {
            return Ok(exact);
        }

        let ancient_type_json = serde_json::to_string(&fingerprint.ancient_type)?;
        let mut statement = self.connection.prepare(
            "SELECT id, question_text, locked_at_bjt, parameters_json,
                    normalized_terms_json
             FROM questions
             WHERE ancient_question_type = ?1
             ORDER BY created_at DESC",
        )?;
        let rows = statement.query_map([ancient_type_json], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;

        for row in rows {
            let (id, question_text, locked_at_bjt, parameters_json, normalized_terms_json) = row?;
            let previous_question: QuestionInput = serde_json::from_str(&parameters_json)?;
            let previous_terms: BTreeSet<String> = serde_json::from_str(&normalized_terms_json)?;
            let previous = build_event_fingerprint_from_terms(
                &previous_question,
                fingerprint.ancient_type.clone(),
                previous_terms,
            );
            let report = compare_event_fingerprints(fingerprint, &previous);
            if report.same_event {
                return Ok(Some(DuplicateEvent {
                    question_id: parse_uuid(&id)?,
                    question_text,
                    locked_at_bjt,
                    algorithm_version: report.algorithm_version,
                    exact_digest: report.exact_digest,
                    score_percent: report.score_percent,
                    match_reasons: report.reasons,
                }));
            }
        }
        Ok(None)
    }

    pub fn insert_locked_question(
        &mut self,
        question: &QuestionInput,
        classification: &Classification,
        fingerprint: &EventFingerprint,
        revision: Option<&EventRevision>,
    ) -> Result<(), StorageError> {
        let validation = validate_question_parameters(question);
        if !validation.can_lock {
            return Err(StorageError::ParameterValidation(
                validation.errors.join("；"),
            ));
        }
        let expected_classification = KeywordClassifier::default()
            .classify_detailed(question.category, &question.text)
            .map_err(|error| {
                StorageError::ParameterValidation(format!("问题分类无唯一结果：{error:?}"))
            })?;
        if classification != &expected_classification {
            return Err(StorageError::ParameterValidation(
                "问题分类与已锁定类别或原文不一致".to_owned(),
            ));
        }
        if fingerprint != &build_event_fingerprint(question, classification) {
            return Err(StorageError::ParameterValidation(
                "事件指纹与已锁定问题及分类不一致".to_owned(),
            ));
        }
        let astronomy_snapshot = calculate_astronomy_snapshot(question)?;
        let astronomy_snapshot_json = serde_json::to_string(&astronomy_snapshot)?;
        let astronomy_snapshot_hash = blake3::hash(astronomy_snapshot_json.as_bytes())
            .to_hex()
            .to_string();
        let created_at = Utc::now().to_rfc3339();
        let tx = self.connection.transaction()?;
        tx.execute(
            "INSERT INTO questions(
                id, category, subcategory, ancient_question_type, question_text,
                subject_text, object_text, target_event, options_json, parameters_json,
                location_json, divination_time_bjt, target_time_text, locked_at_bjt,
                event_fingerprint_version, event_fingerprint, normalized_terms_json,
                locked, state, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                       ?13, ?14, ?15, ?16, ?17, 1, 'not_executable', ?18)",
            params![
                question.id.to_string(),
                question.category.key(),
                question.subcategory,
                serde_json::to_string(&classification.ancient_type)?,
                question.text,
                question.subject,
                question.object,
                question.target_event,
                serde_json::to_string(&question.options)?,
                serde_json::to_string(question)?,
                serde_json::to_string(&question.location)?,
                question.divination_time_bjt.to_rfc3339(),
                question.target_time_text,
                question.locked_at_bjt.to_rfc3339(),
                fingerprint.version,
                fingerprint.digest_hex,
                serde_json::to_string(&fingerprint.canonical_terms)?,
                created_at,
            ],
        )?;

        tx.execute(
            "INSERT INTO astronomy_snapshots(
                question_id, snapshot_version, snapshot_json, content_hash, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                question.id.to_string(),
                astronomy_snapshot.snapshot_version,
                astronomy_snapshot_json,
                astronomy_snapshot_hash,
                Utc::now().to_rfc3339(),
            ],
        )?;

        if let Some(revision) = revision {
            let match_algorithm_version = revision
                .match_report
                .as_ref()
                .map(|report| report.algorithm_version.clone());
            let match_exact_digest = revision
                .match_report
                .as_ref()
                .map(|report| i64::from(report.exact_digest));
            let match_score_percent = revision
                .match_report
                .as_ref()
                .map(|report| i64::from(report.score_percent));
            let match_reasons_json = revision
                .match_report
                .as_ref()
                .map(|report| serde_json::to_string(&report.reasons))
                .transpose()?;
            tx.execute(
                "INSERT INTO event_revisions(
                    id, prior_question_id, new_question_id, change_description, created_at_bjt,
                    match_algorithm_version, match_exact_digest, match_score_percent,
                    match_reasons_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    revision.id.to_string(),
                    revision.prior_question_id.to_string(),
                    question.id.to_string(),
                    revision.change_description,
                    revision.created_at_bjt.to_rfc3339(),
                    match_algorithm_version,
                    match_exact_digest,
                    match_score_percent,
                    match_reasons_json,
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn load_astronomy_snapshot(
        &self,
        question_id: Uuid,
    ) -> Result<Option<AstronomyAuditSnapshot>, StorageError> {
        let row = self
            .connection
            .query_row(
                "SELECT snapshot_json, content_hash
                 FROM astronomy_snapshots WHERE question_id = ?1",
                [question_id.to_string()],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?;
        let Some((snapshot_json, stored_hash)) = row else {
            return Ok(None);
        };
        let snapshot: AstronomyAuditSnapshot = serde_json::from_str(&snapshot_json)?;
        if snapshot.question_id != question_id {
            return Err(StorageError::AstronomySnapshot(
                "快照 Question ID 与查询目标不一致".to_owned(),
            ));
        }
        let actual_hash = blake3::hash(snapshot_json.as_bytes()).to_hex().to_string();
        if actual_hash != stored_hash {
            return Err(StorageError::AstronomySnapshot(
                "快照内容哈希不一致".to_owned(),
            ));
        }
        Ok(Some(snapshot))
    }

    pub fn load_fusion_decision(
        &self,
        question_id: Uuid,
    ) -> Result<Option<FusionDecision>, StorageError> {
        let row = self
            .connection
            .query_row(
                "SELECT algorithm_version, signal_schema_version,
                        decision_json, content_hash
                 FROM fusion_decisions WHERE question_id = ?1",
                [question_id.to_string()],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, u16>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .optional()?;
        let Some((algorithm_version, signal_schema_version, decision_json, stored_hash)) = row
        else {
            return Ok(None);
        };
        let actual_hash = blake3::hash(decision_json.as_bytes()).to_hex().to_string();
        if actual_hash != stored_hash {
            return Err(StorageError::FusionAudit(
                "融合快照内容哈希不一致".to_owned(),
            ));
        }
        let decision: FusionDecision = serde_json::from_str(&decision_json)?;
        if decision.algorithm_version != algorithm_version
            || decision.signal_schema_version != signal_schema_version
        {
            return Err(StorageError::FusionAudit(
                "融合快照版本索引与固化内容不一致".to_owned(),
            ));
        }
        Ok(Some(decision))
    }

                                                                           
                                                                         
                                                                              
    pub fn fuse_and_store_question(
        &mut self,
        question_id: Uuid,
    ) -> Result<FusionDecision, StorageError> {
        if let Some(existing) = self.load_fusion_decision(question_id)? {
            return Ok(existing);
        }
        let (question_json, ancient_type_json) = self
            .connection
            .query_row(
                "SELECT parameters_json, ancient_question_type
                 FROM questions WHERE id = ?1",
                [question_id.to_string()],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?
            .ok_or_else(|| StorageError::FusionAudit("占问记录不存在".to_owned()))?;
        let question: QuestionInput = serde_json::from_str(&question_json)?;
        let ancient_type: AncientQuestionType = serde_json::from_str(&ancient_type_json)?;
        if ancient_type == AncientQuestionType::WeatherRain {
            let scope = assess_first_rain_scope(&question);
            if !scope.in_research_scope {
                return Err(StorageError::FusionAudit(format!(
                    "天气旧分类未通过降雨范围复核，禁止生成结论：{}",
                    scope.reasons.join("；")
                )));
            }
        }
        let results = self.load_verified_fusion_inputs(question_id, &ancient_type, &question)?;
        let decision = if question.options.is_empty() {
            fuse_equal_weight(&results)
        } else {
            fuse_equal_weight_options(&results, &question.options)
        }
        .map_err(|error| StorageError::FusionAudit(error.to_string()))?;

        let decision_json = serde_json::to_string(&decision)?;
        let content_hash = blake3::hash(decision_json.as_bytes()).to_hex().to_string();
        let created_at = Utc::now().to_rfc3339();
        let tx = self.connection.transaction()?;
        tx.execute(
            "INSERT INTO fusion_decisions(
                question_id, algorithm_version, signal_schema_version,
                decision_json, content_hash, created_at
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                question_id.to_string(),
                decision.algorithm_version,
                decision.signal_schema_version,
                decision_json,
                content_hash,
                created_at,
            ],
        )?;
        for (sequence, trace) in decision.trace.iter().enumerate() {
            tx.execute(
                "INSERT INTO decision_traces(
                    question_id, stage, sequence_no, payload_json, created_at
                 ) VALUES(?1, 'fusion', ?2, ?3, ?4)",
                params![
                    question_id.to_string(),
                    sequence as i64,
                    serde_json::to_string(trace)?,
                    Utc::now().to_rfc3339(),
                ],
            )?;
        }
        tx.execute(
            "UPDATE questions
             SET state = 'completed', final_conclusion = ?2,
                 final_timing = ?3, final_evidence_tier = ?4,
                 final_action_advice_json = ?5
             WHERE id = ?1",
            params![
                question_id.to_string(),
                decision.conclusion,
                decision.timing.zh_name(),
                evidence_tier_key(decision.evidence_tier),
                serde_json::to_string(&decision.action_advice)?,
            ],
        )?;
        tx.commit()?;
        Ok(decision)
    }

    fn load_verified_fusion_inputs(
        &self,
        question_id: Uuid,
        ancient_type: &AncientQuestionType,
        question: &QuestionInput,
    ) -> Result<Vec<MethodResult>, StorageError> {
        type EngineRunRow = (i64, String, String, String, String);
        let rows = {
            let mut statement = self.connection.prepare(
                "SELECT id, method, chart_json, signals_json, evidence_tier
                 FROM engine_runs
                 WHERE question_id = ?1
                   AND applicability_status = 'applicable'
                   AND completed_at IS NOT NULL
                   AND chart_json IS NOT NULL
                   AND signals_json IS NOT NULL
                   AND evidence_tier IS NOT NULL
                 ORDER BY method",
            )?;
            let collected = statement
                .query_map([question_id.to_string()], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                })?
                .collect::<Result<Vec<EngineRunRow>, _>>()?;
            collected
        };

        let mut output = Vec::new();
        for (run_id, method_key, chart_json, signals_json, stored_tier_key) in rows {
            let method = method_from_key(&method_key)?;
            let released = self.released_rules(method, ancient_type)?;
            let released_by_id = released
                .into_iter()
                .map(|item| (item.rule.rule_id.clone(), item))
                .collect::<BTreeMap<_, _>>();
            let hit_rows = {
                let mut statement = self.connection.prepare(
                    "SELECT rule_id, input_snapshot_json, output_snapshot_json,
                            evaluation_trace_json
                     FROM rule_hits WHERE engine_run_id = ?1 ORDER BY rule_id",
                )?;
                let collected = statement
                    .query_map([run_id], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                        ))
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                collected
            };
            if hit_rows.is_empty()
                || hit_rows
                    .iter()
                    .any(|(rule_id, _, _, _)| !released_by_id.contains_key(rule_id))
            {
                continue;
            }
            let stored_chart: serde_json::Value = serde_json::from_str(&chart_json)?;
            let expected_input = RuleRuntimeInputSnapshot::new(
                method,
                ancient_type.clone(),
                question.clone(),
                stored_chart.clone(),
            )
            .map_err(|error| StorageError::FusionAudit(error.to_string()))?
            .to_value()
            .map_err(|error| StorageError::FusionAudit(error.to_string()))?;
            let mut expected_evaluations = BTreeMap::new();
            for (rule_id, released_rule) in &released_by_id {
                let evaluation = released_rule.evaluate(&expected_input)?;
                if evaluation.matched {
                    expected_evaluations.insert(rule_id.clone(), evaluation);
                }
            }
            let stored_hit_ids = hit_rows
                .iter()
                .map(|(rule_id, _, _, _)| rule_id.clone())
                .collect::<BTreeSet<_>>();
            let expected_hit_ids = expected_evaluations
                .keys()
                .cloned()
                .collect::<BTreeSet<_>>();
            if stored_hit_ids != expected_hit_ids {
                return Err(StorageError::FusionAudit(format!(
                    "术式 {} 的保存 Rule ID 集合不等于当前输入的完整命中集合",
                    method.zh_name()
                )));
            }
            let mut verified_matches = Vec::with_capacity(hit_rows.len());
            for (rule_id, input_snapshot_json, output_snapshot_json, evaluation_trace_json) in
                hit_rows
            {
                let released_rule = &released_by_id[&rule_id];
                let input_value: serde_json::Value = serde_json::from_str(&input_snapshot_json)?;
                parse_rule_runtime_input_snapshot(&input_value)
                    .map_err(|error| StorageError::FusionAudit(error.to_string()))?;
                if input_value != expected_input {
                    return Err(StorageError::FusionAudit(format!(
                        "规则 {rule_id} 的输入快照与锁定占问、术式或排盘不一致"
                    )));
                }
                let evaluation = expected_evaluations[&rule_id].clone();
                let stored_output_value: serde_json::Value =
                    serde_json::from_str(&output_snapshot_json)?;
                let stored_output = parse_rule_program_output(&stored_output_value)
                    .map_err(|error| StorageError::FusionAudit(error.to_string()))?;
                if stored_output != evaluation.auditable_output() {
                    return Err(StorageError::FusionAudit(format!(
                        "规则 {rule_id} 的输出快照与重新求值结果不一致"
                    )));
                }
                let stored_trace: Vec<String> = serde_json::from_str(&evaluation_trace_json)?;
                if stored_trace != evaluation.trace {
                    return Err(StorageError::FusionAudit(format!(
                        "规则 {rule_id} 的求值轨迹与重新求值结果不一致"
                    )));
                }
                verified_matches.push(RuleRuntimeMatch {
                    rule_id,
                    evidence_tier: released_rule.rule.evidence_tier,
                    evaluation,
                });
            }
            let derived = aggregate_rule_matches(method, stored_chart, &verified_matches)
                .map_err(|error| StorageError::FusionAudit(error.to_string()))?;
            let stored_signals: IntermediateSignals = serde_json::from_str(&signals_json)?;
            if stored_signals != derived.signals {
                return Err(StorageError::FusionAudit(format!(
                    "术式 {} 的保存信号不能由规则命中重新构造",
                    method.zh_name()
                )));
            }
            let stored_tier = evidence_tier_from_key(&stored_tier_key)?;
            if stored_tier != derived.evidence_tier {
                return Err(StorageError::FusionAudit(format!(
                    "术式 {} 的证据等级与当前放行 Rule ID 不一致",
                    method.zh_name()
                )));
            }
            output.push(derived);
        }
        Ok(output)
    }

    pub fn question_count(&self) -> Result<u64, StorageError> {
        self.connection
            .query_row("SELECT COUNT(*) FROM questions", [], |row| row.get(0))
            .map_err(StorageError::from)
    }

    pub fn recent_questions(
        &self,
        requested_limit: usize,
    ) -> Result<Vec<QuestionHistorySummary>, StorageError> {
        let limit = requested_limit.clamp(1, 500) as i64;
        let mut statement = self.connection.prepare(
            "SELECT q.parameters_json, q.state,
                    EXISTS(SELECT 1 FROM random_artifacts a
                           WHERE a.question_id = q.id AND a.method = 'liuyao'),
                    EXISTS(SELECT 1 FROM random_artifacts a
                           WHERE a.question_id = q.id AND a.method = 'yarrow_stalk'),
                    EXISTS(SELECT 1 FROM event_revisions r WHERE r.new_question_id = q.id)
             FROM questions q
             ORDER BY q.created_at DESC, q.rowid DESC
             LIMIT ?1",
        )?;
        let rows = statement.query_map([limit], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, bool>(2)?,
                row.get::<_, bool>(3)?,
                row.get::<_, bool>(4)?,
            ))
        })?;
        let mut output = Vec::new();
        for row in rows {
            let (question_json, state, has_liuyao, has_yarrow, is_event_revision) = row?;
            output.push(QuestionHistorySummary {
                question: serde_json::from_str(&question_json)?,
                state,
                has_liuyao,
                has_yarrow,
                is_event_revision,
            });
        }
        Ok(output)
    }

    pub fn load_engine_run_audits(
        &self,
        question_id: Uuid,
    ) -> Result<Vec<EngineRunAudit>, StorageError> {
        type Row = (
            i64,
            String,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            String,
            Option<String>,
        );
        let rows = {
            let mut statement = self.connection.prepare(
                "SELECT id, method, implementation_version, applicability_status,
                        applicability_reasons_json, chart_json, signals_json,
                        evidence_tier, started_at, completed_at
                 FROM engine_runs WHERE question_id = ?1 ORDER BY method",
            )?;
            let collected = statement
                .query_map([question_id.to_string()], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                        row.get(8)?,
                        row.get(9)?,
                    ))
                })?
                .collect::<Result<Vec<Row>, _>>()?;
            collected
        };
        let mut audits = Vec::with_capacity(rows.len());
        for (
            run_id,
            method_key,
            implementation_version,
            status_key,
            reasons_json,
            chart_json,
            signals_json,
            evidence_tier_key,
            started_at,
            completed_at,
        ) in rows
        {
            let hit_rows = {
                let mut statement = self.connection.prepare(
                    "SELECT rule_id, input_snapshot_json, output_snapshot_json,
                            evaluation_trace_json
                     FROM rule_hits WHERE engine_run_id = ?1 ORDER BY rule_id",
                )?;
                let collected = statement
                    .query_map([run_id], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                        ))
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                collected
            };
            let mut rule_hits = Vec::with_capacity(hit_rows.len());
            for (rule_id, input_json, output_json, trace_json) in hit_rows {
                let input_value: serde_json::Value = serde_json::from_str(&input_json)?;
                let output_value: serde_json::Value = serde_json::from_str(&output_json)?;
                rule_hits.push(RuleHitAudit {
                    rule_id,
                    input_snapshot: parse_rule_runtime_input_snapshot(&input_value)
                        .map_err(|error| StorageError::RuleRuntime(error.to_string()))?,
                    output_snapshot: parse_rule_program_output(&output_value)
                        .map_err(|error| StorageError::RuleRuntime(error.to_string()))?,
                    evaluation_trace: serde_json::from_str(&trace_json)?,
                });
            }
            audits.push(EngineRunAudit {
                method: method_from_key(&method_key)?,
                implementation_version,
                status: applicability_status_from_key(&status_key)?,
                reasons: serde_json::from_str(&reasons_json)?,
                chart: chart_json
                    .as_deref()
                    .map(serde_json::from_str)
                    .transpose()?,
                signals: signals_json
                    .as_deref()
                    .map(serde_json::from_str)
                    .transpose()?,
                evidence_tier: evidence_tier_key
                    .as_deref()
                    .map(evidence_tier_from_key)
                    .transpose()?,
                started_at,
                completed_at,
                rule_hits,
            });
        }
        Ok(audits)
    }

    pub fn load_question_detail(
        &self,
        question_id: Uuid,
    ) -> Result<Option<QuestionHistoryDetail>, StorageError> {
        type DetailRow = (
            String,
            String,
            u16,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            String,
        );
        let row: Option<DetailRow> = self
            .connection
            .query_row(
                "SELECT parameters_json, ancient_question_type,
                        event_fingerprint_version, event_fingerprint,
                        normalized_terms_json, state, final_conclusion,
                        final_timing, final_evidence_tier,
                        final_action_advice_json, created_at
                 FROM questions WHERE id = ?1",
                [question_id.to_string()],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                        row.get(8)?,
                        row.get(9)?,
                        row.get(10)?,
                    ))
                },
            )
            .optional()?;
        let Some((
            question_json,
            ancient_type_json,
            event_fingerprint_version,
            event_fingerprint,
            normalized_terms_json,
            state,
            final_conclusion,
            final_timing,
            final_evidence_tier,
            final_action_advice_json,
            created_at,
        )) = row
        else {
            return Ok(None);
        };

        let revision = self
            .connection
            .query_row(
                "SELECT id, prior_question_id, change_description, created_at_bjt,
                        match_algorithm_version, match_exact_digest,
                        match_score_percent, match_reasons_json
                 FROM event_revisions WHERE new_question_id = ?1",
                [question_id.to_string()],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<i64>>(5)?,
                        row.get::<_, Option<i64>>(6)?,
                        row.get::<_, Option<String>>(7)?,
                    ))
                },
            )
            .optional()?
            .map(
                |(
                    revision_id,
                    prior_question_id,
                    change_description,
                    created_at_bjt,
                    match_algorithm_version,
                    match_exact_digest,
                    match_score_percent,
                    match_reasons_json,
                )| {
                    let match_score_percent = match_score_percent
                        .map(|score| {
                            u8::try_from(score).map_err(|_| {
                                StorageError::ArtifactIntegrity(format!(
                                    "EVENT_REVISION 匹配分数越界：{score}"
                                ))
                            })
                        })
                        .transpose()?;
                    let match_reasons = match match_reasons_json {
                        Some(json) => serde_json::from_str(&json)?,
                        None => Vec::new(),
                    };
                    Ok::<RevisionLink, StorageError>(RevisionLink {
                        revision_id: parse_uuid(&revision_id)?,
                        prior_question_id: parse_uuid(&prior_question_id)?,
                        change_description,
                        created_at_bjt,
                        match_algorithm_version,
                        match_exact_digest: match_exact_digest.map(|value| value != 0),
                        match_score_percent,
                        match_reasons,
                    })
                },
            )
            .transpose()?;

        let mut statement = self.connection.prepare(
            "SELECT new_question_id FROM event_revisions
             WHERE prior_question_id = ?1 ORDER BY created_at_bjt",
        )?;
        let later_rows =
            statement.query_map([question_id.to_string()], |row| row.get::<_, String>(0))?;
        let mut later_revision_ids = Vec::new();
        for later in later_rows {
            later_revision_ids.push(parse_uuid(&later?)?);
        }

        let astronomy_snapshot = self.load_astronomy_snapshot(question_id)?;
        let engine_runs = self.load_engine_run_audits(question_id)?;
        let fusion_decision = self.load_fusion_decision(question_id)?;
        let final_action_advice = final_action_advice_json
            .as_deref()
            .map(serde_json::from_str::<ActionAdviceAudit>)
            .transpose()?
            .unwrap_or_default();
        if let Some(decision) = &fusion_decision {
            if decision.action_advice != final_action_advice {
                return Err(StorageError::FusionAudit(
                    "结果页行动建议与不可变融合快照不一致".to_owned(),
                ));
            }
        }

        Ok(Some(QuestionHistoryDetail {
            question: serde_json::from_str(&question_json)?,
            ancient_question_type: serde_json::from_str(&ancient_type_json)?,
            event_fingerprint_version,
            event_fingerprint,
            normalized_terms: serde_json::from_str(&normalized_terms_json)?,
            state,
            final_conclusion,
            final_timing,
            final_evidence_tier,
            final_action_advice,
            created_at,
            revision,
            later_revision_ids,
            astronomy_snapshot,
            engine_runs,
            fusion_decision,
        }))
    }

    pub fn audit_history_replay(
        &self,
        question_id: Uuid,
    ) -> Result<HistoryReplayAudit, StorageError> {
        let astronomy_snapshot = self.load_astronomy_snapshot(question_id)?;
        let day_stem = astronomy_snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.sexagenary_day_bjt_midnight)
            .map(|day| day.stem);
        let liuyao = match self.load_liuyao_artifact(question_id)? {
            Some(artifact) => {
                let transformation =
                    hexagram_transformation(artifact.cast.primary_bits, artifact.cast.changed_bits)
                        .map_err(|error| StorageError::ArtifactIntegrity(error.to_string()))?;
                let sanhe = match astronomy_snapshot.as_ref().and_then(|snapshot| {
                    snapshot
                        .jie_month_candidate
                        .as_ref()
                        .zip(snapshot.sexagenary_day_bjt_midnight)
                }) {
                    Some((month, day)) => Some(
                        liuyao_sanhe_audit(&artifact.cast, month, day)
                            .map_err(|error| StorageError::ArtifactIntegrity(error.to_string()))?,
                    ),
                    None => None,
                };
                let fanyin_fuyin = sanhe.as_ref().map(|audit| audit.fanyin_fuyin.clone());
                let progress_retreat = fanyin_fuyin
                    .as_ref()
                    .map(|audit| audit.progress_retreat.clone());
                let branch_pair_relations = progress_retreat
                    .as_ref()
                    .map(|audit| audit.branch_pair_relations.clone());
                let calendar_relations = branch_pair_relations
                    .as_ref()
                    .map(|audit| audit.calendar_relations.clone());
                let calendar_moving = calendar_relations
                    .as_ref()
                    .map(|audit| audit.calendar_moving.clone());
                Some(ArtifactReplayProof {
                    method: DivinationMethod::Liuyao,
                    algorithm_version: artifact.cast.algorithm_version,
                    transcript_hash: artifact.cast.transcript_hash,
                    operation_count: artifact.transcript.events.len(),
                    primary_hexagram: transformation.primary,
                    changed_hexagram: transformation.changed,
                    liuyao_eight_palace_audit: Some(
                        liuyao_eight_palace_audit(artifact.cast.primary_bits)
                            .map_err(|error| StorageError::ArtifactIntegrity(error.to_string()))?,
                    ),
                    liuyao_najia_audit: Some(
                        liuyao_najia_audit(artifact.cast.primary_bits)
                            .map_err(|error| StorageError::ArtifactIntegrity(error.to_string()))?,
                    ),
                    liuyao_six_relatives_audit: Some(
                        liuyao_six_relatives_audit(artifact.cast.primary_bits)
                            .map_err(|error| StorageError::ArtifactIntegrity(error.to_string()))?,
                    ),
                    liuyao_six_spirits_audit: day_stem
                        .map(|stem| liuyao_six_spirits_audit(artifact.cast.primary_bits, stem))
                        .transpose()
                        .map_err(|error| StorageError::ArtifactIntegrity(error.to_string()))?,
                    liuyao_hidden_spirits_audit: Some(
                        liuyao_hidden_spirits_audit(artifact.cast.primary_bits)
                            .map_err(|error| StorageError::ArtifactIntegrity(error.to_string()))?,
                    ),
                    liuyao_calendar_moving_audit: calendar_moving,
                    liuyao_calendar_relations_audit: calendar_relations,
                    liuyao_branch_pair_relations_audit: branch_pair_relations,
                    liuyao_progress_retreat_audit: progress_retreat,
                    liuyao_fanyin_fuyin_audit: fanyin_fuyin,
                    liuyao_sanhe_audit: sanhe,
                })
            }
            None => None,
        };
        let yarrow = match self.load_yarrow_artifact(question_id)? {
            Some(artifact) => {
                let transformation =
                    hexagram_transformation(artifact.cast.primary_bits, artifact.cast.changed_bits)
                        .map_err(|error| StorageError::ArtifactIntegrity(error.to_string()))?;
                Some(ArtifactReplayProof {
                    method: DivinationMethod::YarrowStalk,
                    algorithm_version: artifact.cast.algorithm_version,
                    transcript_hash: artifact.cast.transcript_hash,
                    operation_count: artifact.transcript.left_splits.len(),
                    primary_hexagram: transformation.primary,
                    changed_hexagram: transformation.changed,
                    liuyao_eight_palace_audit: None,
                    liuyao_najia_audit: None,
                    liuyao_six_relatives_audit: None,
                    liuyao_six_spirits_audit: None,
                    liuyao_hidden_spirits_audit: None,
                    liuyao_calendar_moving_audit: None,
                    liuyao_calendar_relations_audit: None,
                    liuyao_branch_pair_relations_audit: None,
                    liuyao_progress_retreat_audit: None,
                    liuyao_fanyin_fuyin_audit: None,
                    liuyao_sanhe_audit: None,
                })
            }
            None => None,
        };
        Ok(HistoryReplayAudit {
            question_id,
            liuyao,
            yarrow,
        })
    }

    pub fn save_liuyao_artifact(
        &self,
        question_id: Uuid,
        transcript: &LiuyaoTranscript,
        cast: &LiuyaoCast,
    ) -> Result<(), StorageError> {
        if transcript.question_id != question_id || cast.question_id != question_id {
            return Err(StorageError::ArtifactIntegrity(
                "Question ID 与六爻转录不一致".to_owned(),
            ));
        }
        verify_liuyao_replay(transcript, cast)
            .map_err(|error| StorageError::ArtifactIntegrity(error.to_string()))?;
        self.insert_random_artifact(
            question_id,
            DivinationMethod::Liuyao,
            "three_coins",
            LIUYAO_ALGORITHM_VERSION,
            &cast.master_seed_hex,
            serde_json::to_string(transcript)?,
            serde_json::to_string(cast)?,
        )
    }

    pub fn load_liuyao_artifact(
        &self,
        question_id: Uuid,
    ) -> Result<Option<LiuyaoArtifact>, StorageError> {
        let Some((version, seed_hex, transcript_json, cast_json)) =
            self.load_random_artifact(question_id, DivinationMethod::Liuyao)?
        else {
            return Ok(None);
        };
        if version != LIUYAO_ALGORITHM_VERSION {
            return Err(StorageError::ArtifactIntegrity(format!(
                "不支持的六爻起法版本：{version}"
            )));
        }
        let transcript: LiuyaoTranscript = serde_json::from_str(&transcript_json)?;
        let cast: LiuyaoCast = serde_json::from_str(&cast_json)?;
        if cast.master_seed_hex != seed_hex {
            return Err(StorageError::ArtifactIntegrity(
                "六爻主种子与数据库索引不一致".to_owned(),
            ));
        }
        verify_liuyao_replay(&transcript, &cast)
            .map_err(|error| StorageError::ArtifactIntegrity(error.to_string()))?;
        Ok(Some(LiuyaoArtifact { transcript, cast }))
    }

    pub fn save_yarrow_artifact(
        &self,
        question_id: Uuid,
        transcript: &YarrowTranscript,
        cast: &YarrowCast,
    ) -> Result<(), StorageError> {
        verify_yarrow_replay(transcript, cast)
            .map_err(|error| StorageError::ArtifactIntegrity(error.to_string()))?;
        self.insert_random_artifact(
            question_id,
            DivinationMethod::YarrowStalk,
            "eighteen_changes",
            &cast.algorithm_version,
            &cast.transcript_hash,
            serde_json::to_string(transcript)?,
            serde_json::to_string(cast)?,
        )
    }

    pub fn load_yarrow_artifact(
        &self,
        question_id: Uuid,
    ) -> Result<Option<YarrowArtifact>, StorageError> {
        let Some((version, seed_hex, transcript_json, cast_json)) =
            self.load_random_artifact(question_id, DivinationMethod::YarrowStalk)?
        else {
            return Ok(None);
        };
        if !is_supported_yarrow_algorithm(&version) {
            return Err(StorageError::ArtifactIntegrity(format!(
                "不支持的蓍草起法版本：{version}"
            )));
        }
        let transcript: YarrowTranscript = serde_json::from_str(&transcript_json)?;
        let cast: YarrowCast = serde_json::from_str(&cast_json)?;
        if transcript.algorithm_version != version || cast.algorithm_version != version {
            return Err(StorageError::ArtifactIntegrity(
                "蓍草算法版本与数据库索引不一致".to_owned(),
            ));
        }
        if cast.transcript_hash != seed_hex {
            return Err(StorageError::ArtifactIntegrity(
                "蓍草转录哈希与数据库索引不一致".to_owned(),
            ));
        }
        verify_yarrow_replay(&transcript, &cast)
            .map_err(|error| StorageError::ArtifactIntegrity(error.to_string()))?;
        Ok(Some(YarrowArtifact { transcript, cast }))
    }

    #[allow(clippy::too_many_arguments)]
    fn insert_random_artifact(
        &self,
        question_id: Uuid,
        method: DivinationMethod,
        algorithm: &str,
        algorithm_version: &str,
        seed_hex: &str,
        transcript_json: String,
        generated_values_json: String,
    ) -> Result<(), StorageError> {
        self.connection.execute(
            "INSERT INTO random_artifacts(
                question_id, method, algorithm, algorithm_version, seed_hex,
                entropy_events_json, generated_values_json, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                question_id.to_string(),
                method.key(),
                algorithm,
                algorithm_version,
                seed_hex,
                transcript_json,
                generated_values_json,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn load_random_artifact(
        &self,
        question_id: Uuid,
        method: DivinationMethod,
    ) -> Result<Option<(String, String, String, String)>, StorageError> {
        self.connection
            .query_row(
                "SELECT algorithm_version, seed_hex, entropy_events_json, generated_values_json
                 FROM random_artifacts WHERE question_id = ?1 AND method = ?2",
                params![question_id.to_string(), method.key()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(StorageError::from)
    }
}

impl StoredRuleRow {
    fn into_rule(self) -> Result<RuleRecord, StorageError> {
        Ok(RuleRecord {
            rule_id: self.rule_id,
            method: method_from_key(&self.method)?,
            applicable_question_types: serde_json::from_str(&self.applicable_question_types_json)?,
            condition_expression: self.condition_expression,
            result_expression: self.result_expression,
            timing_expression: self.timing_expression,
            action_key: self.action_key,
            action_polarity: self
                .action_polarity
                .as_deref()
                .map(action_polarity_from_key)
                .transpose()?,
            action_expression: self.action_expression,
            priority: self.priority,
            evidence_tier: evidence_tier_from_key(&self.evidence_tier)?,
            historical_period: self.historical_period,
            book_title: self.book_title,
            volume: self.volume,
            chapter: self.chapter,
            original_text: self.original_text,
            edition: self.edition,
            source_reference: self.source_reference,
            textual_variants: self.textual_variants,
            stability: stability_from_key(&self.stability)?,
            implementation_version: self.implementation_version,
            reviewed: self.reviewed,
        })
    }
}

impl StoredRuleSourceEvidenceRow {
    fn into_evidence(self) -> Result<RuleSourceEvidence, StorageError> {
        let facsimile_verification = match (
            self.verification_checksum,
            self.verification_byte_length,
            self.verification_file_name,
            self.verification_version,
            self.verification_time,
        ) {
            (None, None, None, None, None) => None,
            (
                Some(digital_checksum),
                Some(byte_length),
                Some(file_name),
                Some(verifier_version),
                Some(verified_at),
            ) => {
                let byte_length = u64::try_from(byte_length).map_err(|_| {
                    StorageError::RuleGateValidation(
                        "影印文件验证记录包含非法负数字节数".to_owned(),
                    )
                })?;
                Some(SourceFacsimileVerificationRecord {
                    source_id: self.source_id.clone(),
                    digital_checksum,
                    byte_length,
                    file_name,
                    verifier_version,
                    verified_at,
                })
            }
            _ => {
                return Err(StorageError::RuleGateValidation(
                    "影印文件验证记录字段不完整".to_owned(),
                ));
            }
        };
        Ok(RuleSourceEvidence {
            relation: source_relation_from_key(&self.relation)?,
            link_notes: self.link_notes,
            edition: SourceEditionRecord {
                source_id: self.source_id.clone(),
                canonical_title: self.canonical_title,
                attributed_author: self.attributed_author,
                historical_period: self.source_historical_period,
                edition_title: self.edition_title,
                edition_type: edition_type_from_key(&self.edition_type)?,
                edition_year: self.edition_year,
                holding_institution: self.holding_institution,
                catalog_identifier: self.catalog_identifier,
                facsimile_reference: self.facsimile_reference,
                digital_checksum: self.source_digital_checksum,
                notes: self.source_notes,
            },
            passage: SourcePassageRecord {
                passage_id: self.passage_id,
                source_id: self.source_id,
                volume: self.passage_volume,
                chapter: self.passage_chapter,
                folio_or_page: self.folio_or_page,
                original_text: self.passage_original_text,
                normalized_text: self.normalized_text,
                textual_variants: self.passage_textual_variants,
                transcription_status: transcription_status_from_key(&self.transcription_status)?,
            },
            facsimile_verification,
        })
    }
}

fn validate_unique_seed_ids(bundle: &RuleSeedBundle) -> Result<(), StorageError> {
    ensure_unique_seed_ids(
        "source_id",
        bundle.sources.iter().map(|item| item.source_id.as_str()),
    )?;
    ensure_unique_seed_ids(
        "facsimile_verification.source_id",
        bundle
            .facsimile_verifications
            .iter()
            .map(|item| item.source_id.as_str()),
    )?;
    ensure_unique_seed_ids(
        "passage_id",
        bundle.passages.iter().map(|item| item.passage_id.as_str()),
    )?;
    ensure_unique_seed_ids(
        "rule_id",
        bundle.rules.iter().map(|item| item.rule_id.as_str()),
    )?;
    ensure_unique_seed_ids(
        "review_id",
        bundle.reviews.iter().map(|item| item.review_id.as_str()),
    )?;
    ensure_unique_seed_ids(
        "test_case_id",
        bundle
            .test_cases
            .iter()
            .map(|item| item.test_case_id.as_str()),
    )?;
    ensure_unique_seed_ids(
        "run_id",
        bundle.test_runs.iter().map(|item| item.run_id.as_str()),
    )?;
    let link_ids = bundle.links.iter().map(|item| {
        format!(
            "{}|{}|{}",
            item.rule_id,
            item.passage_id,
            item.relation.key()
        )
    });
    ensure_unique_seed_ids("rule_source_link", link_ids)?;
    Ok(())
}

fn ensure_unique_seed_ids<T, I>(label: &str, values: I) -> Result<(), StorageError>
where
    T: AsRef<str>,
    I: IntoIterator<Item = T>,
{
    let mut seen = BTreeSet::new();
    for value in values {
        let value = value.as_ref();
        if value.trim().is_empty() {
            return Err(StorageError::InvalidRuleSeed(format!("{label} 不能为空")));
        }
        if !seen.insert(value.to_owned()) {
            return Err(StorageError::InvalidRuleSeed(format!(
                "{label} 重复：{value}"
            )));
        }
    }
    Ok(())
}

pub fn canonical_rule_hash(rule: &RuleRecord) -> Result<String, StorageError> {
    hash_serializable(rule)
}

fn hash_serializable<T: Serialize + ?Sized>(value: &T) -> Result<String, StorageError> {
    let bytes = serde_json::to_vec(value)?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn is_canonical_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value.as_bytes()[7..]
            .iter()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
}

fn calculate_evidence_bundle_hash(
    connection: &Connection,
    rule_id: &str,
) -> Result<String, StorageError> {
    let mut statement = connection.prepare(
        "SELECT l.relation, l.notes,
                p.passage_id, p.content_hash, p.transcription_status,
                p.volume, p.chapter, p.folio_or_page, p.original_text,
                p.normalized_text, p.textual_variants,
                e.source_id, e.canonical_title, e.attributed_author,
                e.historical_period, e.edition_title, e.edition_type,
                e.edition_year, e.holding_institution, e.catalog_identifier,
                e.facsimile_reference, e.digital_checksum, e.notes,
                COALESCE(v.digital_checksum, ''),
                COALESCE(CAST(v.byte_length AS TEXT), ''),
                COALESCE(v.file_name, ''), COALESCE(v.verifier_version, ''),
                COALESCE(v.verified_at, '')
         FROM rule_source_links l
         JOIN source_passages p ON p.passage_id = l.passage_id
         JOIN source_editions e ON e.source_id = p.source_id
         LEFT JOIN source_facsimile_verifications v ON v.source_id = e.source_id
         WHERE l.rule_id = ?1
         ORDER BY l.relation, p.passage_id",
    )?;
    let rows = statement.query_map([rule_id], |row| {
        let mut values = Vec::with_capacity(28);
        for index in 0..28 {
            values.push(row.get::<_, String>(index)?);
        }
        Ok(values)
    })?;
    let evidence_rows = rows.collect::<Result<Vec<_>, _>>()?;
    hash_serializable(&evidence_rows)
}

fn calculate_test_suite_state(
    connection: &Connection,
    rule_id: &str,
) -> Result<(String, u64), StorageError> {
    let mut statement = connection.prepare(
        "SELECT test_case_id, name, input_json, expected_output_json,
                boundary_class, source_basis
         FROM rule_test_cases
         WHERE rule_id = ?1 AND enabled = 1
         ORDER BY test_case_id",
    )?;
    let rows = statement.query_map([rule_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
        ))
    })?;
    let cases = rows.collect::<Result<Vec<_>, _>>()?;
    let count = cases.len() as u64;
    Ok((hash_serializable(&cases)?, count))
}

const fn action_polarity_key(polarity: ActionPolarity) -> &'static str {
    match polarity {
        ActionPolarity::Do => "do",
        ActionPolarity::Avoid => "avoid",
    }
}

fn action_polarity_from_key(value: &str) -> Result<ActionPolarity, StorageError> {
    match value {
        "do" => Ok(ActionPolarity::Do),
        "avoid" => Ok(ActionPolarity::Avoid),
        _ => Err(StorageError::RuleGateValidation(format!(
            "未知行动建议方向：{value}"
        ))),
    }
}

fn edition_type_from_key(value: &str) -> Result<EditionType, StorageError> {
    match value {
        "facsimile" => Ok(EditionType::Facsimile),
        "collated" => Ok(EditionType::Collated),
        "critical" => Ok(EditionType::Critical),
        "transcription" => Ok(EditionType::Transcription),
        _ => Err(StorageError::RuleGateValidation(format!(
            "未知版本类型：{value}"
        ))),
    }
}

fn transcription_status_from_key(value: &str) -> Result<TranscriptionStatus, StorageError> {
    match value {
        "unreviewed" => Ok(TranscriptionStatus::Unreviewed),
        "verified" => Ok(TranscriptionStatus::Verified),
        "disputed" => Ok(TranscriptionStatus::Disputed),
        _ => Err(StorageError::RuleGateValidation(format!(
            "未知转录状态：{value}"
        ))),
    }
}

fn source_relation_from_key(value: &str) -> Result<SourceRelation, StorageError> {
    match value {
        "primary" => Ok(SourceRelation::Primary),
        "commentary" => Ok(SourceRelation::Commentary),
        "variant" => Ok(SourceRelation::Variant),
        "counterevidence" => Ok(SourceRelation::Counterevidence),
        _ => Err(StorageError::RuleGateValidation(format!(
            "未知来源关系：{value}"
        ))),
    }
}

fn review_kind_from_key(value: &str) -> Result<ReviewKind, StorageError> {
    match value {
        "bibliography" => Ok(ReviewKind::Bibliography),
        "transcription" => Ok(ReviewKind::Transcription),
        "logic" => Ok(ReviewKind::Logic),
        "implementation" => Ok(ReviewKind::Implementation),
        _ => Err(StorageError::RuleGateValidation(format!(
            "未知复核类型：{value}"
        ))),
    }
}

fn review_decision_from_key(value: &str) -> Result<ReviewDecision, StorageError> {
    match value {
        "approved" => Ok(ReviewDecision::Approved),
        "rejected" => Ok(ReviewDecision::Rejected),
        "needs_work" => Ok(ReviewDecision::NeedsWork),
        _ => Err(StorageError::RuleGateValidation(format!(
            "未知复核结论：{value}"
        ))),
    }
}

fn evidence_tier_key(tier: EvidenceTier) -> &'static str {
    match tier {
        EvidenceTier::EarlyCoreClassic => "early_core_classic",
        EvidenceTier::EarlyAuthoritativeCommentary => "early_authoritative_commentary",
        EvidenceTier::LaterAuthoritativeCommentary => "later_authoritative_commentary",
        EvidenceTier::Unverified => "unverified",
    }
}

fn evidence_tier_from_key(value: &str) -> Result<EvidenceTier, StorageError> {
    match value {
        "early_core_classic" => Ok(EvidenceTier::EarlyCoreClassic),
        "early_authoritative_commentary" => Ok(EvidenceTier::EarlyAuthoritativeCommentary),
        "later_authoritative_commentary" => Ok(EvidenceTier::LaterAuthoritativeCommentary),
        "unverified" => Ok(EvidenceTier::Unverified),
        _ => Err(StorageError::RuleGateValidation(format!(
            "未知古籍证据等级：{value}"
        ))),
    }
}

const fn applicability_status_key(status: ApplicabilityStatus) -> &'static str {
    match status {
        ApplicabilityStatus::Applicable => "applicable",
        ApplicabilityStatus::InsufficientEvidence => "insufficient_evidence",
        ApplicabilityStatus::MissingParameters => "missing_parameters",
        ApplicabilityStatus::UnsupportedQuestionType => "unsupported_question_type",
        ApplicabilityStatus::NotImplemented => "not_implemented",
    }
}

fn applicability_status_from_key(value: &str) -> Result<ApplicabilityStatus, StorageError> {
    match value {
        "applicable" => Ok(ApplicabilityStatus::Applicable),
        "insufficient_evidence" => Ok(ApplicabilityStatus::InsufficientEvidence),
        "missing_parameters" => Ok(ApplicabilityStatus::MissingParameters),
        "unsupported_question_type" => Ok(ApplicabilityStatus::UnsupportedQuestionType),
        "not_implemented" => Ok(ApplicabilityStatus::NotImplemented),
        _ => Err(StorageError::RuleRuntime(format!(
            "未知术式适用性状态：{value}"
        ))),
    }
}

fn valid_runtime_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.as_bytes().iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn stability_key(stability: RuleStability) -> &'static str {
    match stability {
        RuleStability::Stable => "stable",
        RuleStability::Disputed => "disputed",
        RuleStability::Fragmentary => "fragmentary",
        RuleStability::Unreviewed => "unreviewed",
    }
}

fn stability_from_key(value: &str) -> Result<RuleStability, StorageError> {
    match value {
        "stable" => Ok(RuleStability::Stable),
        "disputed" => Ok(RuleStability::Disputed),
        "fragmentary" => Ok(RuleStability::Fragmentary),
        "unreviewed" => Ok(RuleStability::Unreviewed),
        _ => Err(StorageError::RuleGateValidation(format!(
            "未知规则稳定性：{value}"
        ))),
    }
}

fn method_from_key(value: &str) -> Result<DivinationMethod, StorageError> {
    DivinationMethod::ALL
        .into_iter()
        .find(|method| method.key() == value)
        .ok_or_else(|| StorageError::RuleGateValidation(format!("未知术式：{value}")))
}

fn precision_key(precision: CoordinatePrecision) -> &'static str {
    match precision {
        CoordinatePrecision::CityCenter => "city_center",
        CoordinatePrecision::DistrictCenter => "district_center",
        CoordinatePrecision::StreetCenter => "street_center",
    }
}

fn civil_calendar_audit_for_bjt(
    value: chrono::DateTime<chrono::FixedOffset>,
) -> Result<CivilCalendarAudit, StorageError> {
    civil_calendar_audit(
        HistoricalDateTime {
            year: value.year(),
            month: value.month() as u8,
            day: value.day() as u8,
            hour: value.hour() as u8,
            minute: value.minute() as u8,
            second: value.second() as u8,
        },
        CalendarSystem::Gregorian,
    )
    .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))
}

pub fn calculate_birth_time_audit_snapshot(
    birth: &BirthParameters,
) -> Result<BirthTimeAuditSnapshot, StorageError> {
    let jd_ut1_approx = birth.date_time_bjt.timestamp_millis() as f64 / 86_400_000.0 + 2_440_587.5;
    let delta_t = estimate_delta_t(
        birth.date_time_bjt.year(),
        birth.date_time_bjt.month() as u8,
    )
    .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let jd_tt = julian_day_tt_from_ut1(jd_ut1_approx, delta_t.seconds)
        .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let provider = SpaSolarEphemeris;
    let solar = provider
        .calculate(jd_tt)
        .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let boundary_margin_minutes = 5.0;
    let (true_solar_time, correction) = true_solar_time_from_equation_of_time(
        birth.date_time_bjt,
        birth.location.longitude_deg,
        solar.equation_of_time_minutes,
        boundary_margin_minutes,
    )
    .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let bjt_day = sexagenary_day_for_fixed_date(
        birth.date_time_bjt,
        divination_core::CalendarSystem::Gregorian,
    )
    .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let true_solar_day =
        sexagenary_day_for_fixed_date(true_solar_time, divination_core::CalendarSystem::Gregorian)
            .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let shichen = shichen_from_true_solar_time(true_solar_time, boundary_margin_minutes)
        .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let hour = sexagenary_hour_from_day_stem(true_solar_day.stem, shichen.branch);
    let civil_calendar_audit = civil_calendar_audit_for_bjt(birth.date_time_bjt)?;
    let solar_term_boundary_margin_seconds = 300.0;
    let jie_interval = solar_term_class_interval(
        &provider,
        jd_tt,
        SolarTermClass::Jie,
        solar_term_boundary_margin_seconds,
    )
    .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let civil_year = birth.date_time_bjt.year();
    let lichun =
        solve_solar_term_near_bjt_date(&provider, SolarTerm::Lichun, civil_year, 2, 4, 7.0)
            .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let year_candidate = lichun_year_candidate(
        civil_year,
        jd_tt,
        &lichun,
        solar_term_boundary_margin_seconds,
    )
    .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let month_candidate = jie_month_candidate(year_candidate.designation.stem, &jie_interval)
        .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let four_pillars = four_pillars_chart_candidate(
        &year_candidate,
        &month_candidate,
        bjt_day,
        true_solar_day,
        shichen,
        hour,
        correction.boundary_safe,
        true_solar_time.hour() == 23,
    )
    .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let four_pillars_structural_metadata = four_pillars_structural_metadata(&four_pillars);
    let four_pillars_visible_stem_ten_gods =
        four_pillars_visible_stem_ten_gods_audit(&four_pillars);
    let four_pillars_visible_stem_ten_gods_zi_start =
        four_pillars_visible_stem_ten_gods_zi_start_audit(&four_pillars);
    let late_zi_dual_candidates = late_zi_dual_candidates(&four_pillars);
    let bjt_shichen = shichen_from_true_solar_time(birth.date_time_bjt, boundary_margin_minutes)
        .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let lunar_boundary_margin_seconds = 300.0;
    let lunisolar_target = HistoricalDateTime {
        year: birth.date_time_bjt.year(),
        month: birth.date_time_bjt.month() as u8,
        day: birth.date_time_bjt.day() as u8,
        hour: birth.date_time_bjt.hour() as u8,
        minute: birth.date_time_bjt.minute() as u8,
        second: birth.date_time_bjt.second() as u8,
    };
    let (lunisolar_date_candidate, lunisolar_date_candidate_error) =
        match lunisolar_month_structure_for_civil_date(
            &provider,
            lunisolar_target,
            lunar_boundary_margin_seconds,
        ) {
            Ok(structure) => {
                match lunisolar_date_candidate_from_structure(&structure, lunisolar_target) {
                    Ok(candidate) => (Some(candidate), None),
                    Err(error) => (None, Some(error.to_string())),
                }
            }
            Err(error) => (None, Some(error.to_string())),
        };
    let (ziwei_life_body_audit, ziwei_life_body_audit_error) = match lunisolar_date_candidate
        .as_ref()
    {
        Some(date) => match ziwei_life_body_audit(date, &year_candidate, bjt_shichen, shichen) {
            Ok(audit) => (Some(audit), None),
            Err(error) => (None, Some(error.to_string())),
        },
        None => (
            None,
            Some("出生农历日期候选未形成，紫微命身宫机械层自动跳过".to_owned()),
        ),
    };

    let (
        qizheng_five_planet_geometric_observations,
        qizheng_five_planet_geometric_observation_error,
    ) = match five_visible_planet_geometric_observations(jd_tt) {
        Ok(observations) => (Some(observations), None),
        Err(error) => (None, Some(error.to_string())),
    };

    Ok(BirthTimeAuditSnapshot {
        audit_version: 13,
        status: BIRTH_TIME_AUDIT_STATUS.to_owned(),
        sex: birth.sex,
        input_time_bjt: birth.date_time_bjt.to_rfc3339(),
        longitude_deg: birth.location.longitude_deg,
        latitude_deg: birth.location.latitude_deg,
        jd_ut1_approx,
        delta_t_seconds: delta_t.seconds,
        delta_t_uncertainty_seconds: delta_t.estimated_uncertainty_seconds,
        delta_t_model_version: delta_t.model_version,
        jd_tt,
        equation_of_time_minutes: solar.equation_of_time_minutes,
        equation_of_time_algorithm_version: SPA_EQUATION_OF_TIME_VERSION.to_owned(),
        longitude_correction_minutes: correction.longitude_correction_minutes,
        true_solar_total_correction_minutes: correction.total_correction_minutes,
        true_solar_time_bjt: true_solar_time.to_rfc3339(),
        true_solar_boundary_margin_minutes: boundary_margin_minutes,
        true_solar_boundary_safe: correction.boundary_safe,
        civil_calendar_audit: Some(civil_calendar_audit),
        sexagenary_calendar_system: "proleptic_gregorian_iso_input.v1".to_owned(),
        sexagenary_day_algorithm_version: SEXAGENARY_DAY_ALGORITHM_VERSION.to_owned(),
        sexagenary_day_bjt_midnight: bjt_day,
        sexagenary_day_true_solar_midnight: true_solar_day,
        shichen_algorithm_version: SHICHEN_ALGORITHM_VERSION.to_owned(),
        true_solar_shichen: shichen,
        sexagenary_hour_true_solar_midnight: hour,
        zi_hour_day_boundary_status: ZI_HOUR_DAY_BOUNDARY_STATUS.to_owned(),
        four_pillars_chart_candidate: Some(four_pillars),
        four_pillars_structural_metadata: Some(four_pillars_structural_metadata),
        four_pillars_visible_stem_ten_gods: Some(four_pillars_visible_stem_ten_gods),
        four_pillars_visible_stem_ten_gods_zi_start,
        late_zi_dual_candidates,
        lunisolar_date_candidate,
        lunisolar_date_candidate_error,
        ziwei_life_body_audit,
        ziwei_life_body_audit_error,
        qizheng_siyu_identity_audit: Some(qizheng_siyu_identity_audit()),
        qizheng_five_planet_geometric_observations,
        qizheng_five_planet_geometric_observation_error,
    })
}

fn calculate_astronomy_snapshot(
    question: &QuestionInput,
) -> Result<AstronomyAuditSnapshot, StorageError> {
    let jd_ut1_approx =
        question.divination_time_bjt.timestamp_millis() as f64 / 86_400_000.0 + 2_440_587.5;
    let delta_t = estimate_delta_t(
        question.divination_time_bjt.year(),
        question.divination_time_bjt.month() as u8,
    )
    .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let jd_tt = julian_day_tt_from_ut1(jd_ut1_approx, delta_t.seconds)
        .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let provider = SpaSolarEphemeris;
    let solar = provider
        .sun_geocentric_ecliptic(jd_tt)
        .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let solar_coordinates = provider
        .calculate(jd_tt)
        .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let boundary_margin_minutes = 5.0;
    let (true_solar_time, true_solar_correction) = true_solar_time_from_equation_of_time(
        question.divination_time_bjt,
        question.location.longitude_deg,
        solar_coordinates.equation_of_time_minutes,
        boundary_margin_minutes,
    )
    .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let sexagenary_day_bjt = sexagenary_day_for_fixed_date(
        question.divination_time_bjt,
        divination_core::CalendarSystem::Gregorian,
    )
    .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let sexagenary_day_true_solar =
        sexagenary_day_for_fixed_date(true_solar_time, divination_core::CalendarSystem::Gregorian)
            .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let beijing_shichen =
        shichen_from_true_solar_time(question.divination_time_bjt, boundary_margin_minutes)
            .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let sexagenary_hour_bjt_midnight =
        sexagenary_hour_from_day_stem(sexagenary_day_bjt.stem, beijing_shichen.branch);
    let true_solar_shichen = shichen_from_true_solar_time(true_solar_time, boundary_margin_minutes)
        .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let sexagenary_hour_true_solar_midnight =
        sexagenary_hour_from_day_stem(sexagenary_day_true_solar.stem, true_solar_shichen.branch);
    let solar_term_boundary_margin_seconds = 300.0;
    let term_interval = solar_term_interval(&provider, jd_tt, solar_term_boundary_margin_seconds)
        .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let jie_interval = solar_term_class_interval(
        &provider,
        jd_tt,
        SolarTermClass::Jie,
        solar_term_boundary_margin_seconds,
    )
    .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let zhongqi_interval = solar_term_class_interval(
        &provider,
        jd_tt,
        SolarTermClass::Zhongqi,
        solar_term_boundary_margin_seconds,
    )
    .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let civil_year = question.divination_time_bjt.year();
    let lichun =
        solve_solar_term_near_bjt_date(&provider, SolarTerm::Lichun, civil_year, 2, 4, 7.0)
            .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let lichun_year = lichun_year_candidate(
        civil_year,
        jd_tt,
        &lichun,
        solar_term_boundary_margin_seconds,
    )
    .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let month_candidate = jie_month_candidate(lichun_year.designation.stem, &jie_interval)
        .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let birth_time_audit = question
        .birth
        .as_ref()
        .map(calculate_birth_time_audit_snapshot)
        .transpose()?;
    let civil_calendar_audit = civil_calendar_audit_for_bjt(question.divination_time_bjt)?;
    let lunar_boundary_margin_seconds = 300.0;
    let lunar_interval = lunation_interval(jd_tt, lunar_boundary_margin_seconds)
        .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?;
    let lunisolar_target = HistoricalDateTime {
        year: question.divination_time_bjt.year(),
        month: question.divination_time_bjt.month() as u8,
        day: question.divination_time_bjt.day() as u8,
        hour: question.divination_time_bjt.hour() as u8,
        minute: question.divination_time_bjt.minute() as u8,
        second: question.divination_time_bjt.second() as u8,
    };
    let (
        lunisolar_month_structure,
        lunisolar_month_structure_error,
        lunisolar_date_candidate,
        lunisolar_date_candidate_error,
    ) = match lunisolar_month_structure_for_civil_date(
        &provider,
        lunisolar_target,
        lunar_boundary_margin_seconds,
    ) {
        Ok(structure) => {
            match lunisolar_date_candidate_from_structure(&structure, lunisolar_target) {
                Ok(candidate) => (Some(structure), None, Some(candidate), None),
                Err(error) => (Some(structure), None, None, Some(error.to_string())),
            }
        }
        Err(error) => (None, Some(error.to_string()), None, None),
    };
    let meihua_time_casting_audit = lunisolar_date_candidate
        .as_ref()
        .map(|date| {
            meihua_time_casting_audit(date, beijing_shichen, true_solar_shichen)
                .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))
        })
        .transpose()?;

    let chinese_astrology_structure_audit = chinese_astrology_structure_audit();
    let qimen_day_boundary_margin_minutes = 5.0;
    let qimen_ju_audit = qimen_ju_duty_identity_audit(
        &term_interval,
        sexagenary_day_bjt,
        qimen_midnight_boundary_safe(
            question.divination_time_bjt,
            qimen_day_boundary_margin_minutes,
        )
        .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?,
        sexagenary_hour_bjt_midnight,
        beijing_shichen.boundary_safe,
        sexagenary_day_true_solar,
        qimen_midnight_boundary_safe(true_solar_time, qimen_day_boundary_margin_minutes)
            .map_err(|error| StorageError::AstronomySnapshot(error.to_string()))?,
        sexagenary_hour_true_solar_midnight,
        true_solar_shichen.boundary_safe,
    );

    Ok(AstronomyAuditSnapshot {
        snapshot_version: 29,
        question_id: question.id,
        ut1_basis: "utc_approximation_without_versioned_eop.v1".to_owned(),
        jd_ut1_approx,
        delta_t_seconds: delta_t.seconds,
        delta_t_uncertainty_seconds: delta_t.estimated_uncertainty_seconds,
        delta_t_model_version: delta_t.model_version,
        jd_tt,
        solar_longitude_deg: solar.longitude_deg,
        solar_latitude_deg: solar.latitude_deg,
        solar_distance_au: solar.distance_au,
        solar_precision: solar.precision,
        solar_estimated_error_arcmin: solar.estimated_error_arcmin,
        solar_algorithm_version: solar.algorithm_version,
        equation_of_time_minutes: Some(solar_coordinates.equation_of_time_minutes),
        equation_of_time_algorithm_version: Some(SPA_EQUATION_OF_TIME_VERSION.to_owned()),
        longitude_correction_minutes: Some(true_solar_correction.longitude_correction_minutes),
        true_solar_total_correction_minutes: Some(true_solar_correction.total_correction_minutes),
        true_solar_time_bjt: Some(true_solar_time.to_rfc3339()),
        true_solar_boundary_margin_minutes: Some(boundary_margin_minutes),
        true_solar_boundary_safe: Some(true_solar_correction.boundary_safe),
        civil_calendar_audit: Some(civil_calendar_audit),
        sexagenary_calendar_system: Some("proleptic_gregorian_iso_input.v1".to_owned()),
        sexagenary_day_algorithm_version: Some(SEXAGENARY_DAY_ALGORITHM_VERSION.to_owned()),
        sexagenary_day_bjt_midnight: Some(sexagenary_day_bjt),
        sexagenary_day_true_solar_midnight: Some(sexagenary_day_true_solar),
        shichen_algorithm_version: Some(SHICHEN_ALGORITHM_VERSION.to_owned()),
        true_solar_shichen: Some(true_solar_shichen),
        sexagenary_hour_true_solar_midnight: Some(sexagenary_hour_true_solar_midnight),
        zi_hour_day_boundary_status: Some(ZI_HOUR_DAY_BOUNDARY_STATUS.to_owned()),
        solar_term_interval_algorithm_version: Some(term_interval.algorithm_version),
        previous_solar_term: Some(term_interval.previous.term),
        previous_solar_term_jd_tt: Some(term_interval.previous.jd_tt),
        previous_solar_term_uncertainty_seconds: Some(
            term_interval.previous.combined_uncertainty_seconds,
        ),
        previous_solar_term_iterations: Some(term_interval.previous.iterations),
        solar_term_boundary_margin_seconds: Some(solar_term_boundary_margin_seconds),
        seconds_since_previous_solar_term: Some(term_interval.seconds_since_previous),
        seconds_until_next_solar_term: Some(term_interval.seconds_until_next),
        solar_term_boundary_safe: Some(term_interval.boundary_safe),
        jie_interval: Some(jie_interval),
        zhongqi_interval: Some(zhongqi_interval),
        astronomical_year_label: Some(sexagenary_year_for_astronomical_year_label(civil_year)),
        lichun_year_candidate: Some(lichun_year),
        jie_month_candidate: Some(month_candidate),
        pillar_candidate_status: Some(PILLAR_CANDIDATE_STATUS.to_owned()),
        birth_time_audit,
        lunation_interval: Some(lunar_interval),
        lunisolar_month_structure,
        lunisolar_month_structure_error,
        lunisolar_date_candidate,
        lunisolar_date_candidate_error,
        meihua_time_casting_audit,
        chinese_astrology_structure_audit: Some(chinese_astrology_structure_audit),
        qimen_ju_audit: Some(qimen_ju_audit),
        next_solar_term: term_interval.next.term,
        next_solar_term_jd_tt: term_interval.next.jd_tt,
        next_solar_term_uncertainty_seconds: term_interval.next.combined_uncertainty_seconds,
        next_solar_term_iterations: term_interval.next.iterations,
    })
}

fn parse_uuid(value: &str) -> Result<Uuid, StorageError> {
    Uuid::parse_str(value).map_err(|error| {
        StorageError::ArtifactIntegrity(format!("数据库中的 UUID 无效：{value}（{error}）"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, TimeZone};
    use divination_core::{
        build_event_fingerprint, ActionAdviceStatus, KeywordClassifier, QuestionCategory, Trigram,
    };

    fn sample_question() -> QuestionInput {
        let zone = FixedOffset::east_opt(8 * 3600).unwrap();
        let now = zone.with_ymd_and_hms(2026, 8, 30, 12, 45, 0).unwrap();
        QuestionInput {
            id: Uuid::new_v4(),
            category: QuestionCategory::Weather,
            subcategory: "降雨".to_owned(),
            text: "一小时后会下雨吗？".to_owned(),
            subject: "深圳市南山区".to_owned(),
            object: "降雨".to_owned(),
            target_event: "下雨".to_owned(),
            result_terms: vec!["雨".to_owned()],
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
            divination_time_bjt: now,
            target_time_text: "一小时后".to_owned(),
            birth: None,
            locked_at_bjt: now,
        }
    }

    #[test]
    fn migrates_and_seeds_locations() {
        let storage = Storage::open_in_memory().unwrap();
        assert!(storage.locations().unwrap().len() >= 37_000);
        let schema_version: u32 = storage
            .connection
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(schema_version, 12);
        let attributed_rows: u64 = storage
            .connection
            .query_row(
                "SELECT COUNT(*) FROM locations
                 WHERE code_system = 'geonames_id'
                   AND source_license = 'CC-BY-4.0'
                   AND length(source_record_id) > 0
                   AND length(source_modified_date) = 10",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(attributed_rows >= 3_000);
        let attributed_streets: u64 = storage
            .connection
            .query_row(
                "SELECT COUNT(*) FROM locations
                 WHERE code_system = 'mca_administrative_code'
                   AND coordinate_precision = 'district_center'
                   AND street IS NOT NULL
                   AND source_record_id LIKE 'MCA:%;GN:%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(attributed_streets, 34_535);
        let (precision, coordinate_ref): (String, String) = storage
            .connection
            .query_row(
                "SELECT coordinate_precision, source_record_id FROM locations
                 WHERE administrative_code = 'MCA:440305007'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(precision, "district_center");
        assert_eq!(coordinate_ref, "MCA:440305007;GN:6571327");
        assert_eq!(
            storage.rule_evidence_summary().unwrap(),
            RuleEvidenceSummary::default()
        );
    }

    #[test]
    fn upgrades_a_v5_database_with_event_match_fusion_and_action_advice() {
        let connection = Connection::open_in_memory().unwrap();
        for (version, migration) in [
            (1, MIGRATION_0001),
            (2, MIGRATION_0002),
            (3, MIGRATION_0003),
            (4, MIGRATION_0004),
            (5, MIGRATION_0005),
        ] {
            connection.execute_batch(migration).unwrap();
            connection
                .execute(
                    "INSERT INTO schema_migrations(version, applied_at) VALUES(?1, ?2)",
                    params![version, Utc::now().to_rfc3339()],
                )
                .unwrap();
        }

        let mut storage = Storage { connection };
        storage.initialize().unwrap();
        let schema_version: u32 = storage
            .connection
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(schema_version, 12);

        let mut statement = storage
            .connection
            .prepare("PRAGMA table_info(event_revisions)")
            .unwrap();
        let columns = statement
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<BTreeSet<_>, _>>()
            .unwrap();
        for required in [
            "match_algorithm_version",
            "match_exact_digest",
            "match_score_percent",
            "match_reasons_json",
        ] {
            assert!(columns.contains(required), "missing column {required}");
        }
        let fusion_table_exists: bool = storage
            .connection
            .query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM sqlite_master
                    WHERE type = 'table' AND name = 'fusion_decisions'
                 )",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(fusion_table_exists);
        let mut statement = storage
            .connection
            .prepare("PRAGMA table_info(rules)")
            .unwrap();
        let rule_columns = statement
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<BTreeSet<_>, _>>()
            .unwrap();
        for required in ["action_key", "action_polarity", "action_expression"] {
            assert!(rule_columns.contains(required), "missing column {required}");
        }
        let question_action_column: bool = storage
            .connection
            .query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM pragma_table_info('questions')
                    WHERE name = 'final_action_advice_json'
                 )",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(question_action_column);
    }

    #[test]
    fn v8_and_v9_upgrade_revoke_legacy_review_and_expression_state() {
        let connection = Connection::open_in_memory().unwrap();
        for (version, migration) in [
            (1, MIGRATION_0001),
            (2, MIGRATION_0002),
            (3, MIGRATION_0003),
            (4, MIGRATION_0004),
            (5, MIGRATION_0005),
            (6, MIGRATION_0006),
            (7, MIGRATION_0007),
        ] {
            connection.execute_batch(migration).unwrap();
            connection
                .execute(
                    "INSERT INTO schema_migrations(version, applied_at) VALUES(?1, ?2)",
                    params![version, Utc::now().to_rfc3339()],
                )
                .unwrap();
        }
        connection
            .execute(
                "INSERT INTO rules(
                    rule_id, method, applicable_question_types_json,
                    condition_expression, result_expression, timing_expression,
                    priority, evidence_tier, historical_period, book_title,
                    volume, chapter, original_text, edition, source_reference,
                    textual_variants, stability, implementation_version,
                    reviewed, content_hash, created_at, updated_at
                 ) VALUES(
                    'LEGACY-RULE', 'liuyao', '[\"project_outcome\"]',
                    'legacy condition', 'legacy result', NULL,
                    1, 'early_core_classic', 'test', 'test',
                    'test', 'test', 'test', 'test', 'test',
                    'none', 'stable', 'legacy-v1',
                    1, 'legacy-hash', ?1, ?1
                 )",
                [Utc::now().to_rfc3339()],
            )
            .unwrap();

        let mut storage = Storage { connection };
        storage.initialize().unwrap();
        let reviewed: bool = storage
            .connection
            .query_row(
                "SELECT reviewed FROM rules WHERE rule_id = 'LEGACY-RULE'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(!reviewed);
        assert!(!storage
            .rule_gate_report("LEGACY-RULE")
            .unwrap()
            .unwrap()
            .may_execute());
    }

    #[test]
    fn persists_and_hash_verifies_astronomy_snapshot_with_locked_question() {
        let mut storage = Storage::open_in_memory().unwrap();
        let mut question = sample_question();
        question.category = QuestionCategory::Career;
        question.subcategory = "项目成败".to_owned();
        question.text = "这个项目能否按计划完成？".to_owned();
        question.target_event = "项目按计划完成".to_owned();
        let classification = KeywordClassifier::default()
            .classify(question.category, &question.text)
            .unwrap();
        let fingerprint = build_event_fingerprint(&question, &classification);
        storage
            .insert_locked_question(&question, &classification, &fingerprint, None)
            .unwrap();

        let snapshot = storage
            .load_astronomy_snapshot(question.id)
            .unwrap()
            .unwrap();
        assert_eq!(snapshot.question_id, question.id);
        assert_eq!(snapshot.snapshot_version, 29);
        assert_eq!(snapshot.birth_time_audit, None);
        let chinese_astrology = snapshot
            .chinese_astrology_structure_audit
            .as_ref()
            .expect("twenty-eight mansion structure is persisted");
        assert_eq!(chinese_astrology.audit_version, 2);
        assert_eq!(chinese_astrology.three_enclosures.len(), 3);
        assert_eq!(chinese_astrology.mansions.len(), 28);
        assert_eq!(chinese_astrology.seven_luminaries.len(), 7);
        assert_eq!(chinese_astrology.mansions[0].short_zh_name, "角");
        assert_eq!(chinese_astrology.mansions[27].short_zh_name, "轸");
        assert!(!chinese_astrology.angular_boundary_model_selected);
        assert!(!chinese_astrology.celestial_positions_calculated);
        assert!(!chinese_astrology.interpretation_eligible);
        assert!(!chinese_astrology.rule_eligible);
        let qimen = snapshot
            .qimen_ju_audit
            .as_ref()
            .expect("Qimen mechanical ju audit is persisted");
        assert_eq!(qimen.audit_version, 4);
        assert_eq!(qimen.active_solar_term, SolarTerm::Chushu);
        assert_eq!(qimen.dun.zh_name(), "阴遁");
        assert_eq!(qimen.received_bureau_numbers, None);
        assert!(qimen.table_entry_evidence_conflict);
        assert_eq!(qimen.beijing_midnight_candidate.earth_plate, None);
        assert_eq!(qimen.true_solar_midnight_candidate.earth_plate, None);
        assert!(qimen
            .beijing_midnight_candidate
            .hour_xun_candidate
            .is_some());
        assert!(qimen
            .true_solar_midnight_candidate
            .hour_xun_candidate
            .is_some());
        assert!(qimen
            .beijing_midnight_candidate
            .hour_xun_candidate
            .as_ref()
            .unwrap()
            .duty_identity_candidate
            .is_none());
        assert_eq!(qimen.selected_day_convention, None);
        assert_eq!(qimen.selected_hour_convention, None);
        assert_eq!(qimen.transmission_calendar_adjustment_selected, None);
        assert!(!qimen.rule_eligible);
        let lunation = snapshot
            .lunation_interval
            .as_ref()
            .expect("lunation interval is persisted");
        assert!(lunation.previous_new_moon.jd_tt < snapshot.jd_tt);
        assert!(snapshot.jd_tt < lunation.next_new_moon.jd_tt);
        assert!(lunation.previous_new_moon.jd_tt < lunation.full_moon.jd_tt);
        assert!(lunation.full_moon.jd_tt < lunation.next_new_moon.jd_tt);
        assert_eq!(
            lunation.status,
            "astronomical_phase_audit_not_chinese_calendar.v1"
        );
        let month_structure = snapshot
            .lunisolar_month_structure
            .as_ref()
            .expect("lunisolar month structure is persisted");
        assert!(matches!(month_structure.month_count, 12 | 13));
        assert_eq!(month_structure.months.first().unwrap().month_number, 11);
        assert_eq!(month_structure.months.last().unwrap().month_number, 10);
        assert_eq!(
            month_structure.status,
            "audit_candidate_not_complete_chinese_calendar.v1"
        );
        assert_eq!(snapshot.lunisolar_month_structure_error, None);
        let lunar_date = snapshot
            .lunisolar_date_candidate
            .as_ref()
            .expect("lunisolar date candidate is persisted");
        assert_eq!(lunar_date.nominal_lunar_year, 2026);
        assert_eq!(lunar_date.month_number, 7);
        assert!(!lunar_date.is_leap_month);
        assert_eq!(lunar_date.day_number, 18);
        assert!(matches!(lunar_date.month_length_days, 29 | 30));
        assert_eq!(
            lunar_date.status,
            "modern_official_candidate_not_historical_calendar_or_divination_rule.v1"
        );
        assert_eq!(snapshot.lunisolar_date_candidate_error, None);
        let meihua = snapshot
            .meihua_time_casting_audit
            .as_ref()
            .expect("Meihua time-casting audit is persisted");
        assert_eq!(meihua.audit_version, 1);
        assert_eq!(meihua.year_branch.zh_name(), "午");
        assert!(meihua.candidates_identical);
        assert_eq!(meihua.selected_hour_convention, None);
        assert_eq!(
            meihua
                .beijing_time_candidate
                .as_ref()
                .unwrap()
                .cast
                .compact_zh(),
            "上坤 下艮 · 3爻动 · 000100→000000"
        );
        let raw = meihua
            .beijing_time_candidate
            .as_ref()
            .unwrap()
            .cast
            .mutual_raw_audit()
            .unwrap();
        assert_eq!(raw.primary_lower_from_lines_2_3_4, Trigram::Kan);
        assert_eq!(raw.primary_upper_from_lines_3_4_5, Trigram::Zhen);
        assert_eq!(raw.changed_lower_from_lines_2_3_4, Trigram::Kun);
        assert_eq!(raw.changed_upper_from_lines_3_4_5, Trigram::Kun);
        assert_eq!(raw.selected_received_basis, None);
        assert!(!raw.rule_eligible);
        assert!(!meihua.rule_eligible);
        let calendar_audit = snapshot
            .civil_calendar_audit
            .as_ref()
            .expect("civil calendar views are persisted");
        assert_eq!(
            calendar_audit.algorithm_version,
            divination_core::CIVIL_CALENDAR_AUDIT_VERSION
        );
        assert_eq!(calendar_audit.proleptic_gregorian.year, 2026);
        assert_eq!(calendar_audit.julian.year, 2026);
        assert_eq!(calendar_audit.julian.month, 8);
        assert_eq!(calendar_audit.julian.day, 17);
        assert_eq!(
            snapshot.solar_algorithm_version,
            "jiuheng.sun.spa_report.v1"
        );
        assert_eq!(
            snapshot.equation_of_time_algorithm_version.as_deref(),
            Some("jiuheng.equation_of_time.spa_report_appendix_a.v1")
        );
        assert!(snapshot.equation_of_time_minutes.is_some());
        assert!(snapshot.longitude_correction_minutes.is_some());
        assert!(snapshot.true_solar_time_bjt.is_some());
        assert_eq!(snapshot.true_solar_boundary_margin_minutes, Some(5.0));
        assert!(snapshot.true_solar_boundary_safe.is_some());
        assert_eq!(
            snapshot.sexagenary_day_algorithm_version.as_deref(),
            Some("jiuheng.sexagenary_day.hko_2024_02_10_anchor.v1")
        );
        assert_eq!(
            snapshot
                .sexagenary_day_bjt_midnight
                .expect("BJT day is persisted")
                .zh_name(),
            snapshot
                .sexagenary_day_true_solar_midnight
                .expect("true-solar day is persisted")
                .zh_name()
        );
        assert_eq!(
            snapshot
                .true_solar_shichen
                .expect("true-solar shichen is persisted")
                .branch
                .zh_name(),
            "午"
        );
        assert_eq!(
            snapshot.zi_hour_day_boundary_status.as_deref(),
            Some("unresolved_tradition_difference_not_rule_eligible")
        );
        assert_eq!(
            snapshot.solar_term_interval_algorithm_version.as_deref(),
            Some("jiuheng.solar_term_interval.bidirectional_bisection.v1")
        );
        assert!(snapshot.previous_solar_term_jd_tt.unwrap() < snapshot.jd_tt);
        assert_eq!(snapshot.solar_term_boundary_margin_seconds, Some(300.0));
        assert_eq!(snapshot.solar_term_boundary_safe, Some(true));
        let jie = snapshot.jie_interval.expect("Jie interval is persisted");
        let zhongqi = snapshot
            .zhongqi_interval
            .expect("Zhongqi interval is persisted");
        assert_eq!(jie.class, SolarTermClass::Jie);
        assert_eq!(zhongqi.class, SolarTermClass::Zhongqi);
        assert_eq!(jie.previous.term.class(), SolarTermClass::Jie);
        assert_eq!(jie.next.term.class(), SolarTermClass::Jie);
        assert_eq!(zhongqi.previous.term.class(), SolarTermClass::Zhongqi);
        assert_eq!(zhongqi.next.term.class(), SolarTermClass::Zhongqi);
        assert_eq!(
            jie.algorithm_version,
            "jiuheng.solar_term_class_interval.jie_zhongqi.v1"
        );
        assert_eq!(
            snapshot
                .astronomical_year_label
                .expect("year label is persisted")
                .zh_name(),
            "丙午"
        );
        let year_candidate = snapshot
            .lichun_year_candidate
            .expect("Lichun year candidate is persisted");
        assert_eq!(year_candidate.designation.zh_name(), "丙午");
        assert!(year_candidate.boundary_safe);
        assert_eq!(
            snapshot
                .jie_month_candidate
                .expect("Jie month candidate is persisted")
                .designation
                .zh_name(),
            "丙申"
        );
        assert_eq!(
            snapshot.pillar_candidate_status.as_deref(),
            Some("audit_only_convention_not_selected_by_runtime_rule")
        );
        assert!(snapshot.next_solar_term_jd_tt > snapshot.jd_tt);
    }

    #[test]
    fn persists_qimen_earth_plate_hour_xun_and_duty_identity_for_a_safe_term() {
        let mut question = sample_question();
        let zone = FixedOffset::east_opt(8 * 3600).unwrap();
        let time = zone.with_ymd_and_hms(2026, 12, 23, 12, 45, 0).unwrap();
        question.divination_time_bjt = time;
        question.locked_at_bjt = time;

        let snapshot = calculate_astronomy_snapshot(&question).unwrap();
        assert_eq!(snapshot.snapshot_version, 29);
        let audit = snapshot.qimen_ju_audit.expect("qimen audit");
        assert_eq!(audit.active_solar_term, SolarTerm::Dongzhi);
        assert_eq!(audit.received_bureau_numbers, Some([1, 7, 4]));
        for candidate in [
            audit.beijing_midnight_candidate,
            audit.true_solar_midnight_candidate,
        ] {
            let bureau_number = candidate.bureau_number.unwrap();
            let hour_xun = candidate
                .hour_xun_candidate
                .as_ref()
                .expect("hour xun candidate");
            assert!(hour_xun.xun_head_palace.is_some());
            let identity = hour_xun
                .duty_identity_candidate
                .as_ref()
                .expect("duty identity candidate");
            assert_eq!(identity.duty_chief_hourly_palace, None);
            assert_eq!(identity.duty_envoy_hourly_palace, None);
            assert!(!identity.heaven_plate_calculated);
            assert!(!identity.interpretation_eligible);
            assert!(!identity.rule_eligible);
            assert!(!hour_xun.interpretation_eligible);
            assert!(!hour_xun.rule_eligible);
            let plate = candidate.earth_plate.expect("earth plate candidate");
            assert_eq!(plate.bureau_number, bureau_number);
            assert_eq!(plate.cells_by_palace_number.len(), 9);
            assert!(plate.earth_plate_complete);
            assert!(!plate.interpretation_eligible);
            assert!(!plate.rule_eligible);
        }
    }

    #[test]
    fn storage_rejects_invalid_birth_parameters_even_if_the_ui_is_bypassed() {
        let mut storage = Storage::open_in_memory().unwrap();
        let mut question = sample_question();
        question.category = QuestionCategory::LongTermFate;
        question.birth = Some(BirthParameters {
            sex: BiologicalSex::Male,
            date_time_bjt: FixedOffset::east_opt(8 * 3600)
                .unwrap()
                .with_ymd_and_hms(2030, 1, 1, 0, 0, 0)
                .unwrap(),
            location: question.location.clone(),
        });
        let classification = KeywordClassifier::default()
            .classify(question.category, &question.text)
            .unwrap();
        let fingerprint = build_event_fingerprint(&question, &classification);
        let error = storage
            .insert_locked_question(&question, &classification, &fingerprint, None)
            .unwrap_err();
        assert!(matches!(error, StorageError::ParameterValidation(_)));
        assert_eq!(storage.question_count().unwrap(), 0);
    }

    #[test]
    fn astronomy_snapshot_v1_without_true_solar_fields_remains_readable() {
        let question = sample_question();
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 1;
        let mut value = serde_json::to_value(snapshot).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .remove("chinese_astrology_structure_audit");
        value.as_object_mut().unwrap().remove("qimen_ju_audit");
        let object = value.as_object_mut().unwrap();
        for key in [
            "equation_of_time_minutes",
            "equation_of_time_algorithm_version",
            "longitude_correction_minutes",
            "true_solar_total_correction_minutes",
            "true_solar_time_bjt",
            "true_solar_boundary_margin_minutes",
            "true_solar_boundary_safe",
            "sexagenary_calendar_system",
            "sexagenary_day_algorithm_version",
            "sexagenary_day_bjt_midnight",
            "sexagenary_day_true_solar_midnight",
            "shichen_algorithm_version",
            "true_solar_shichen",
            "sexagenary_hour_true_solar_midnight",
            "zi_hour_day_boundary_status",
            "solar_term_interval_algorithm_version",
            "previous_solar_term",
            "previous_solar_term_jd_tt",
            "previous_solar_term_uncertainty_seconds",
            "previous_solar_term_iterations",
            "solar_term_boundary_margin_seconds",
            "seconds_since_previous_solar_term",
            "seconds_until_next_solar_term",
            "solar_term_boundary_safe",
            "jie_interval",
            "zhongqi_interval",
            "astronomical_year_label",
            "lichun_year_candidate",
            "jie_month_candidate",
            "pillar_candidate_status",
            "birth_time_audit",
            "civil_calendar_audit",
            "lunation_interval",
            "lunisolar_month_structure",
            "lunisolar_month_structure_error",
            "lunisolar_date_candidate",
            "lunisolar_date_candidate_error",
        ] {
            object.remove(key);
        }

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 1);
        assert_eq!(legacy.equation_of_time_minutes, None);
        assert_eq!(legacy.true_solar_time_bjt, None);
        assert_eq!(legacy.true_solar_boundary_safe, None);
        assert_eq!(legacy.sexagenary_day_bjt_midnight, None);
        assert_eq!(legacy.true_solar_shichen, None);
    }

    #[test]
    fn astronomy_snapshot_v2_without_sexagenary_fields_remains_readable() {
        let question = sample_question();
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 2;
        let mut value = serde_json::to_value(snapshot).unwrap();
        let object = value.as_object_mut().unwrap();
        for key in [
            "sexagenary_calendar_system",
            "sexagenary_day_algorithm_version",
            "sexagenary_day_bjt_midnight",
            "sexagenary_day_true_solar_midnight",
            "shichen_algorithm_version",
            "true_solar_shichen",
            "sexagenary_hour_true_solar_midnight",
            "zi_hour_day_boundary_status",
            "solar_term_interval_algorithm_version",
            "previous_solar_term",
            "previous_solar_term_jd_tt",
            "previous_solar_term_uncertainty_seconds",
            "previous_solar_term_iterations",
            "solar_term_boundary_margin_seconds",
            "seconds_since_previous_solar_term",
            "seconds_until_next_solar_term",
            "solar_term_boundary_safe",
            "jie_interval",
            "zhongqi_interval",
            "astronomical_year_label",
            "lichun_year_candidate",
            "jie_month_candidate",
            "pillar_candidate_status",
            "birth_time_audit",
            "civil_calendar_audit",
            "lunation_interval",
            "lunisolar_month_structure",
            "lunisolar_month_structure_error",
            "lunisolar_date_candidate",
            "lunisolar_date_candidate_error",
        ] {
            object.remove(key);
        }

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 2);
        assert!(legacy.equation_of_time_minutes.is_some());
        assert!(legacy.true_solar_time_bjt.is_some());
        assert_eq!(legacy.sexagenary_day_bjt_midnight, None);
        assert_eq!(legacy.true_solar_shichen, None);
        assert_eq!(legacy.previous_solar_term, None);
        assert_eq!(legacy.solar_term_boundary_safe, None);
    }

    #[test]
    fn astronomy_snapshot_v3_without_bidirectional_solar_term_fields_remains_readable() {
        let question = sample_question();
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 3;
        let mut value = serde_json::to_value(snapshot).unwrap();
        let object = value.as_object_mut().unwrap();
        for key in [
            "solar_term_interval_algorithm_version",
            "previous_solar_term",
            "previous_solar_term_jd_tt",
            "previous_solar_term_uncertainty_seconds",
            "previous_solar_term_iterations",
            "solar_term_boundary_margin_seconds",
            "seconds_since_previous_solar_term",
            "seconds_until_next_solar_term",
            "solar_term_boundary_safe",
            "jie_interval",
            "zhongqi_interval",
            "astronomical_year_label",
            "lichun_year_candidate",
            "jie_month_candidate",
            "pillar_candidate_status",
            "birth_time_audit",
            "civil_calendar_audit",
            "lunation_interval",
            "lunisolar_month_structure",
            "lunisolar_month_structure_error",
            "lunisolar_date_candidate",
            "lunisolar_date_candidate_error",
        ] {
            object.remove(key);
        }

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 3);
        assert!(legacy.sexagenary_day_bjt_midnight.is_some());
        assert!(legacy.true_solar_shichen.is_some());
        assert_eq!(legacy.previous_solar_term, None);
        assert_eq!(legacy.solar_term_boundary_safe, None);
        assert_eq!(legacy.jie_interval, None);
        assert_eq!(legacy.zhongqi_interval, None);
    }

    #[test]
    fn astronomy_snapshot_v4_without_jie_zhongqi_fields_remains_readable() {
        let question = sample_question();
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 4;
        let mut value = serde_json::to_value(snapshot).unwrap();
        let object = value.as_object_mut().unwrap();
        for key in [
            "jie_interval",
            "zhongqi_interval",
            "astronomical_year_label",
            "lichun_year_candidate",
            "jie_month_candidate",
            "pillar_candidate_status",
            "birth_time_audit",
            "civil_calendar_audit",
            "lunation_interval",
            "lunisolar_month_structure",
            "lunisolar_month_structure_error",
            "lunisolar_date_candidate",
            "lunisolar_date_candidate_error",
        ] {
            object.remove(key);
        }

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 4);
        assert!(legacy.previous_solar_term.is_some());
        assert!(legacy.solar_term_boundary_safe.is_some());
        assert_eq!(legacy.jie_interval, None);
        assert_eq!(legacy.zhongqi_interval, None);
        assert_eq!(legacy.astronomical_year_label, None);
        assert_eq!(legacy.lichun_year_candidate, None);
    }

    #[test]
    fn astronomy_snapshot_v5_without_pillar_candidates_remains_readable() {
        let question = sample_question();
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 5;
        let mut value = serde_json::to_value(snapshot).unwrap();
        let object = value.as_object_mut().unwrap();
        for key in [
            "astronomical_year_label",
            "lichun_year_candidate",
            "jie_month_candidate",
            "pillar_candidate_status",
            "birth_time_audit",
            "civil_calendar_audit",
            "lunation_interval",
            "lunisolar_month_structure",
            "lunisolar_month_structure_error",
            "lunisolar_date_candidate",
            "lunisolar_date_candidate_error",
        ] {
            object.remove(key);
        }

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 5);
        assert!(legacy.jie_interval.is_some());
        assert!(legacy.zhongqi_interval.is_some());
        assert_eq!(legacy.astronomical_year_label, None);
        assert_eq!(legacy.lichun_year_candidate, None);
        assert_eq!(legacy.jie_month_candidate, None);
    }

    #[test]
    fn astronomy_snapshot_v6_without_birth_audit_remains_readable() {
        let question = sample_question();
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 6;
        let mut value = serde_json::to_value(snapshot).unwrap();
        let object = value.as_object_mut().unwrap();
        object.remove("birth_time_audit");
        object.remove("civil_calendar_audit");
        object.remove("lunation_interval");
        object.remove("lunisolar_month_structure");
        object.remove("lunisolar_month_structure_error");
        object.remove("lunisolar_date_candidate");
        object.remove("lunisolar_date_candidate_error");

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 6);
        assert!(legacy.lichun_year_candidate.is_some());
        assert!(legacy.jie_month_candidate.is_some());
        assert_eq!(legacy.birth_time_audit, None);
        assert_eq!(legacy.civil_calendar_audit, None);
    }

    #[test]
    fn astronomy_snapshot_v7_without_civil_calendar_views_remains_readable() {
        let question = sample_question();
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 7;
        let mut value = serde_json::to_value(snapshot).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .remove("civil_calendar_audit");
        value.as_object_mut().unwrap().remove("lunation_interval");
        value
            .as_object_mut()
            .unwrap()
            .remove("lunisolar_month_structure");
        value
            .as_object_mut()
            .unwrap()
            .remove("lunisolar_month_structure_error");
        value
            .as_object_mut()
            .unwrap()
            .remove("lunisolar_date_candidate");
        value
            .as_object_mut()
            .unwrap()
            .remove("lunisolar_date_candidate_error");

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 7);
        assert_eq!(legacy.civil_calendar_audit, None);
        assert_eq!(legacy.lunation_interval, None);
    }

    #[test]
    fn astronomy_snapshot_v8_without_lunation_interval_remains_readable() {
        let question = sample_question();
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 8;
        let mut value = serde_json::to_value(snapshot).unwrap();
        value.as_object_mut().unwrap().remove("lunation_interval");
        value
            .as_object_mut()
            .unwrap()
            .remove("lunisolar_month_structure");
        value
            .as_object_mut()
            .unwrap()
            .remove("lunisolar_month_structure_error");
        value
            .as_object_mut()
            .unwrap()
            .remove("lunisolar_date_candidate");
        value
            .as_object_mut()
            .unwrap()
            .remove("lunisolar_date_candidate_error");

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 8);
        assert!(legacy.civil_calendar_audit.is_some());
        assert_eq!(legacy.lunation_interval, None);
        assert_eq!(legacy.lunisolar_month_structure, None);
        assert_eq!(legacy.lunisolar_month_structure_error, None);
        assert_eq!(legacy.lunisolar_date_candidate, None);
        assert_eq!(legacy.lunisolar_date_candidate_error, None);
    }

    #[test]
    fn astronomy_snapshot_v10_without_lunisolar_date_candidate_remains_readable() {
        let question = sample_question();
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 10;
        let mut value = serde_json::to_value(snapshot).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .remove("lunisolar_date_candidate");
        value
            .as_object_mut()
            .unwrap()
            .remove("lunisolar_date_candidate_error");

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 10);
        assert!(legacy.lunisolar_month_structure.is_some());
        assert_eq!(legacy.lunisolar_date_candidate, None);
        assert_eq!(legacy.lunisolar_date_candidate_error, None);
    }

    #[test]
    fn astronomy_snapshot_v11_before_birth_chart_upgrade_remains_readable() {
        let question = sample_question();
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 11;
        let value = serde_json::to_value(snapshot).unwrap();

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 11);
        assert!(legacy.lunisolar_month_structure.is_some());
        assert!(legacy.lunisolar_date_candidate.is_some());
        assert_eq!(legacy.birth_time_audit, None);
    }

    #[test]
    fn astronomy_snapshot_v12_before_structural_metadata_upgrade_remains_readable() {
        let mut question = sample_question();
        question.birth = Some(BirthParameters {
            sex: BiologicalSex::Male,
            date_time_bjt: question.divination_time_bjt,
            location: question.location.clone(),
        });
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 12;
        snapshot
            .birth_time_audit
            .as_mut()
            .expect("birth audit")
            .audit_version = 3;
        let mut value = serde_json::to_value(snapshot).unwrap();
        let birth = value
            .as_object_mut()
            .unwrap()
            .get_mut("birth_time_audit")
            .unwrap()
            .as_object_mut()
            .unwrap();
        birth.remove("four_pillars_structural_metadata");
        birth.remove("late_zi_dual_candidates");

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 12);
        let audit = legacy.birth_time_audit.expect("legacy birth audit");
        assert_eq!(audit.audit_version, 3);
        assert!(audit.four_pillars_chart_candidate.is_some());
        assert_eq!(audit.four_pillars_structural_metadata, None);
        assert_eq!(audit.late_zi_dual_candidates, None);
    }

    #[test]
    fn astronomy_snapshot_v13_without_meihua_time_cast_remains_readable() {
        let question = sample_question();
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 13;
        let mut value = serde_json::to_value(snapshot).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .remove("meihua_time_casting_audit");

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 13);
        assert!(legacy.lunisolar_date_candidate.is_some());
        assert_eq!(legacy.meihua_time_casting_audit, None);
    }

    #[test]
    fn astronomy_snapshot_v14_before_ziwei_birth_upgrade_remains_readable() {
        let mut question = sample_question();
        question.category = QuestionCategory::LongTermFate;
        question.birth = Some(BirthParameters {
            sex: BiologicalSex::Male,
            date_time_bjt: question.divination_time_bjt,
            location: question.location.clone(),
        });
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 14;
        snapshot
            .birth_time_audit
            .as_mut()
            .expect("birth audit")
            .audit_version = 4;
        let mut value = serde_json::to_value(snapshot).unwrap();
        let birth = value
            .as_object_mut()
            .unwrap()
            .get_mut("birth_time_audit")
            .unwrap()
            .as_object_mut()
            .unwrap();
        birth.remove("lunisolar_date_candidate");
        birth.remove("lunisolar_date_candidate_error");
        birth.remove("ziwei_life_body_audit");
        birth.remove("ziwei_life_body_audit_error");

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 14);
        let audit = legacy.birth_time_audit.expect("legacy birth audit");
        assert_eq!(audit.audit_version, 4);
        assert!(audit.four_pillars_chart_candidate.is_some());
        assert!(audit.four_pillars_structural_metadata.is_some());
        assert_eq!(audit.lunisolar_date_candidate, None);
        assert_eq!(audit.lunisolar_date_candidate_error, None);
        assert_eq!(audit.ziwei_life_body_audit, None);
        assert_eq!(audit.ziwei_life_body_audit_error, None);
        assert!(birth_time_audits_match(
            &calculate_birth_time_audit_snapshot(question.birth.as_ref().unwrap()).unwrap(),
            &audit
        ));
    }

    #[test]
    fn astronomy_snapshot_v16_without_mansion_catalog_remains_readable() {
        let question = sample_question();
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 16;
        let mut value = serde_json::to_value(snapshot).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .remove("chinese_astrology_structure_audit");
        value.as_object_mut().unwrap().remove("qimen_ju_audit");

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 16);
        assert_eq!(legacy.chinese_astrology_structure_audit, None);
        assert_eq!(legacy.qimen_ju_audit, None);
        assert!(legacy.meihua_time_casting_audit.is_some());
    }

    #[test]
    fn astronomy_snapshot_v17_without_qimen_ju_audit_remains_readable() {
        let question = sample_question();
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 17;
        let mut value = serde_json::to_value(snapshot).unwrap();
        value.as_object_mut().unwrap().remove("qimen_ju_audit");

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 17);
        assert!(legacy.chinese_astrology_structure_audit.is_some());
        assert_eq!(legacy.qimen_ju_audit, None);
    }

    #[test]
    fn astronomy_snapshot_v21_with_mansion_only_catalog_remains_readable() {
        let question = sample_question();
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 21;
        let audit = snapshot
            .chinese_astrology_structure_audit
            .as_mut()
            .expect("Chinese astrology audit");
        audit.audit_version = 1;
        audit.algorithm_version =
            divination_core::TWENTY_EIGHT_MANSION_CATALOG_ALGORITHM_VERSION.to_owned();
        audit.status =
            "mechanical_received_identity_catalog_no_sky_boundary_or_interpretation.v1".to_owned();
        audit.three_enclosures.clear();
        audit.seven_luminaries.clear();
        audit.enclosure_boundary_model_selected = false;
        audit.blocked_reasons = vec![
            "historical_determinative_star_and_angular_boundary_model_not_selected".to_owned(),
            "sun_moon_five_planets_guest_star_positions_not_calculated".to_owned(),
            "omen_text_edition_passage_conditions_and_counterexamples_not_reviewed".to_owned(),
        ];

        let mut value = serde_json::to_value(snapshot).unwrap();
        let audit = value["chinese_astrology_structure_audit"]
            .as_object_mut()
            .unwrap();
        audit.remove("three_enclosures");
        audit.remove("seven_luminaries");
        audit.remove("enclosure_boundary_model_selected");

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 21);
        let audit = legacy
            .chinese_astrology_structure_audit
            .expect("legacy mansion catalog");
        assert_eq!(audit.audit_version, 1);
        assert_eq!(audit.mansions.len(), 28);
        assert!(audit.three_enclosures.is_empty());
        assert!(audit.seven_luminaries.is_empty());
        assert!(!audit.enclosure_boundary_model_selected);
    }

    #[test]
    fn astronomy_snapshot_v20_before_qimen_duty_identity_remains_readable() {
        let mut question = sample_question();
        let zone = FixedOffset::east_opt(8 * 3600).unwrap();
        let time = zone.with_ymd_and_hms(2026, 12, 23, 12, 45, 0).unwrap();
        question.divination_time_bjt = time;
        question.locked_at_bjt = time;
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 20;
        let qimen = snapshot.qimen_ju_audit.as_mut().expect("qimen audit");
        qimen.audit_version = 3;
        qimen.status = divination_core::QIMEN_JU_HOUR_CANDIDATE_STATUS.to_owned();
        for candidate in [
            &mut qimen.beijing_midnight_candidate,
            &mut qimen.true_solar_midnight_candidate,
        ] {
            let hour = candidate
                .hour_xun_candidate
                .as_mut()
                .expect("hour xun candidate");
            hour.audit_version = 1;
            hour.status = divination_core::QIMEN_HOUR_XUN_CANDIDATE_STATUS.to_owned();
        }
        let mut value = serde_json::to_value(snapshot).unwrap();
        let qimen = value
            .as_object_mut()
            .unwrap()
            .get_mut("qimen_ju_audit")
            .unwrap()
            .as_object_mut()
            .unwrap();
        for candidate_name in [
            "beijing_midnight_candidate",
            "true_solar_midnight_candidate",
        ] {
            qimen
                .get_mut(candidate_name)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .get_mut("hour_xun_candidate")
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove("duty_identity_candidate");
        }

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 20);
        let qimen = legacy.qimen_ju_audit.expect("legacy qimen audit");
        assert_eq!(qimen.audit_version, 3);
        for candidate in [
            qimen.beijing_midnight_candidate,
            qimen.true_solar_midnight_candidate,
        ] {
            let hour = candidate.hour_xun_candidate.expect("legacy hour xun");
            assert_eq!(hour.audit_version, 1);
            assert_eq!(hour.duty_identity_candidate, None);
        }
    }

    #[test]
    fn astronomy_snapshot_v19_before_qimen_hour_xun_remains_readable() {
        let mut question = sample_question();
        let zone = FixedOffset::east_opt(8 * 3600).unwrap();
        let time = zone.with_ymd_and_hms(2026, 12, 23, 12, 45, 0).unwrap();
        question.divination_time_bjt = time;
        question.locked_at_bjt = time;
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 19;
        let qimen = snapshot.qimen_ju_audit.as_mut().expect("qimen audit");
        qimen.audit_version = 2;
        qimen.status = divination_core::QIMEN_JU_CANDIDATE_STATUS.to_owned();
        let mut value = serde_json::to_value(snapshot).unwrap();
        let qimen = value
            .as_object_mut()
            .unwrap()
            .get_mut("qimen_ju_audit")
            .unwrap()
            .as_object_mut()
            .unwrap();
        qimen.remove("selected_hour_convention");
        for candidate_name in [
            "beijing_midnight_candidate",
            "true_solar_midnight_candidate",
        ] {
            qimen
                .get_mut(candidate_name)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove("hour_xun_candidate");
        }

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 19);
        let qimen = legacy.qimen_ju_audit.expect("legacy qimen audit");
        assert_eq!(qimen.audit_version, 2);
        assert!(qimen.beijing_midnight_candidate.earth_plate.is_some());
        assert_eq!(qimen.beijing_midnight_candidate.hour_xun_candidate, None);
        assert_eq!(qimen.true_solar_midnight_candidate.hour_xun_candidate, None);
        assert_eq!(qimen.selected_hour_convention, None);
    }

    #[test]
    fn astronomy_snapshot_v18_before_qimen_earth_plate_remains_readable() {
        let question = sample_question();
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 18;
        let qimen = snapshot.qimen_ju_audit.as_mut().expect("qimen audit");
        qimen.audit_version = 1;
        qimen.status =
            "mechanical_ju_candidate_no_plate_stars_gates_or_interpretation.v1".to_owned();
        let mut value = serde_json::to_value(snapshot).unwrap();
        let qimen = value
            .as_object_mut()
            .unwrap()
            .get_mut("qimen_ju_audit")
            .unwrap()
            .as_object_mut()
            .unwrap();
        qimen.remove("selected_hour_convention");
        for candidate_name in [
            "beijing_midnight_candidate",
            "true_solar_midnight_candidate",
        ] {
            qimen
                .get_mut(candidate_name)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove("earth_plate");
            qimen
                .get_mut(candidate_name)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove("hour_xun_candidate");
        }

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 18);
        let qimen = legacy.qimen_ju_audit.expect("legacy qimen audit");
        assert_eq!(qimen.audit_version, 1);
        assert_eq!(qimen.beijing_midnight_candidate.earth_plate, None);
        assert_eq!(qimen.true_solar_midnight_candidate.earth_plate, None);
        assert_eq!(qimen.beijing_midnight_candidate.hour_xun_candidate, None);
        assert_eq!(qimen.selected_hour_convention, None);
    }

    #[test]
    fn astronomy_snapshot_v15_before_ziwei_bureau_upgrade_remains_readable() {
        let mut question = sample_question();
        question.category = QuestionCategory::LongTermFate;
        question.birth = Some(BirthParameters {
            sex: BiologicalSex::Male,
            date_time_bjt: question.divination_time_bjt,
            location: question.location.clone(),
        });
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 15;
        snapshot
            .birth_time_audit
            .as_mut()
            .expect("birth audit")
            .audit_version = 5;
        let mut value = serde_json::to_value(snapshot).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .remove("chinese_astrology_structure_audit");
        value.as_object_mut().unwrap().remove("qimen_ju_audit");
        let ziwei = value
            .as_object_mut()
            .unwrap()
            .get_mut("birth_time_audit")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .get_mut("ziwei_life_body_audit")
            .unwrap()
            .as_object_mut()
            .unwrap();
        ziwei.insert("audit_version".to_owned(), serde_json::json!(1));
        ziwei.insert(
            "algorithm_version".to_owned(),
            serde_json::json!("jiuheng.ziwei.received_month_hour_life_body_twelve_palaces.v1"),
        );
        ziwei.insert(
            "status".to_owned(),
            serde_json::json!(
                "mechanical_palaces_only_no_stars_interpretation_or_runtime_signals.v1"
            ),
        );
        ziwei.remove("year_stem_convention_candidates");
        ziwei.remove("year_stem_conventions_identical");
        ziwei.remove("selected_year_stem_convention");
        ziwei
            .get_mut("blocked_reasons")
            .unwrap()
            .as_array_mut()
            .unwrap()
            .retain(|reason| {
                reason != "nominal_lunar_year_vs_lichun_birth_year_stem_basis_not_selected"
                    && reason != "received_nayin_text_not_bound_to_reviewed_primary_facsimile_pages"
            });

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 15);
        let audit = legacy.birth_time_audit.expect("legacy birth audit");
        assert_eq!(audit.audit_version, 5);
        let legacy_ziwei = audit
            .ziwei_life_body_audit
            .as_ref()
            .expect("legacy Ziwei audit");
        assert_eq!(legacy_ziwei.audit_version, 1);
        assert!(legacy_ziwei.year_stem_convention_candidates.is_empty());
        assert_eq!(legacy_ziwei.selected_year_stem_convention, None);
        assert!(birth_time_audits_match(
            &calculate_birth_time_audit_snapshot(question.birth.as_ref().unwrap()).unwrap(),
            &audit
        ));
    }

    #[test]
    fn astronomy_snapshot_v26_before_ziwei_star_anchor_remains_readable() {
        let mut question = sample_question();
        question.category = QuestionCategory::LongTermFate;
        question.birth = Some(BirthParameters {
            sex: BiologicalSex::Male,
            date_time_bjt: question.divination_time_bjt,
            location: question.location.clone(),
        });
        let current =
            calculate_birth_time_audit_snapshot(question.birth.as_ref().unwrap()).unwrap();
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 26;
        let audit = snapshot.birth_time_audit.as_mut().unwrap();
        audit.audit_version = 10;
        let ziwei = audit.ziwei_life_body_audit.as_mut().unwrap();
        ziwei.audit_version = 2;
        ziwei.star_anchor_candidates.clear();
        ziwei.algorithm_version =
            "jiuheng.ziwei.received_life_body_palaces_and_bureau.v2".to_owned();
        ziwei.status =
            "mechanical_palaces_and_bureau_no_stars_interpretation_or_runtime_signals.v2"
                .to_owned();
        ziwei.blocked_reasons.retain(|reason| {
            reason != "fourteen_main_stars_and_interpretation_layers_not_implemented"
                && reason != "ziwei_star_anchor_transcription_not_bound_to_reviewed_facsimile_pages"
                && reason != "fourteen_main_star_placements_not_bound_to_reviewed_facsimile_pages"
                && reason
                    != "auxiliary_stars_four_transformations_limits_and_interpretation_not_implemented"
        });
        ziwei
            .blocked_reasons
            .push("main_star_and_interpretation_layers_not_implemented".to_owned());
        let mut value = serde_json::to_value(snapshot).unwrap();
        value["birth_time_audit"]["ziwei_life_body_audit"]
            .as_object_mut()
            .unwrap()
            .remove("star_anchor_candidates");
        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        let stored = legacy.birth_time_audit.unwrap();
        assert_eq!(legacy.snapshot_version, 26);
        assert!(birth_time_audits_match(&current, &stored));
        let mut changed = current.clone();
        let branch = &mut changed
            .ziwei_life_body_audit
            .as_mut()
            .unwrap()
            .star_anchor_candidates[0]
            .anchor
            .ziwei_branch;
        *branch = if *branch == divination_core::EarthlyBranch::Zi {
            divination_core::EarthlyBranch::Chou
        } else {
            divination_core::EarthlyBranch::Zi
        };
        assert!(!birth_time_audits_match(&current, &changed));
    }

    #[test]
    fn astronomy_snapshot_v27_before_fourteen_stars_remains_readable() {
        let mut question = sample_question();
        question.category = QuestionCategory::LongTermFate;
        question.birth = Some(BirthParameters {
            sex: BiologicalSex::Male,
            date_time_bjt: question.divination_time_bjt,
            location: question.location.clone(),
        });
        let current =
            calculate_birth_time_audit_snapshot(question.birth.as_ref().unwrap()).unwrap();
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 27;
        let audit = snapshot.birth_time_audit.as_mut().unwrap();
        audit.audit_version = 11;
        let ziwei = audit.ziwei_life_body_audit.as_mut().unwrap();
        ziwei.audit_version = 3;
        ziwei.algorithm_version =
            "jiuheng.ziwei.received_life_body_palaces_bureau_star_anchor.v3".to_owned();
        ziwei.status =
            "mechanical_palaces_bureau_ziwei_anchor_no_interpretation_or_runtime_signals.v3"
                .to_owned();
        ziwei.blocked_reasons.retain(|reason| {
            reason != "fourteen_main_star_placements_not_bound_to_reviewed_facsimile_pages"
                && reason
                    != "auxiliary_stars_four_transformations_limits_and_interpretation_not_implemented"
        });
        ziwei
            .blocked_reasons
            .push("fourteen_main_stars_and_interpretation_layers_not_implemented".to_owned());
        let mut value = serde_json::to_value(snapshot).unwrap();
        for candidate in value["birth_time_audit"]["ziwei_life_body_audit"]
            ["star_anchor_candidates"]
            .as_array_mut()
            .unwrap()
        {
            candidate.as_object_mut().unwrap().remove("major_stars");
        }
        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 27);
        let stored = legacy.birth_time_audit.unwrap();
        assert!(stored
            .ziwei_life_body_audit
            .as_ref()
            .unwrap()
            .star_anchor_candidates
            .iter()
            .all(|candidate| candidate.major_stars.is_none()));
        assert!(birth_time_audits_match(&current, &stored));
        let mut changed = current.clone();
        let branch = &mut changed
            .ziwei_life_body_audit
            .as_mut()
            .unwrap()
            .star_anchor_candidates[0]
            .major_stars
            .as_mut()
            .unwrap()
            .placements[0]
            .branch;
        *branch = if *branch == divination_core::EarthlyBranch::Zi {
            divination_core::EarthlyBranch::Chou
        } else {
            divination_core::EarthlyBranch::Zi
        };
        assert!(!birth_time_audits_match(&current, &changed));
    }

    #[test]
    fn astronomy_snapshot_v28_before_four_auxiliaries_remains_readable() {
        let mut question = sample_question();
        question.category = QuestionCategory::LongTermFate;
        question.birth = Some(BirthParameters {
            sex: BiologicalSex::Male,
            date_time_bjt: question.divination_time_bjt,
            location: question.location.clone(),
        });
        let current =
            calculate_birth_time_audit_snapshot(question.birth.as_ref().unwrap()).unwrap();
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 28;
        let birth_audit = snapshot.birth_time_audit.as_mut().unwrap();
        birth_audit.audit_version = 12;
        let ziwei = birth_audit.ziwei_life_body_audit.as_mut().unwrap();
        ziwei.audit_version = 4;
        ziwei.algorithm_version =
            "jiuheng.ziwei.received_life_body_palaces_bureau_fourteen_stars.v4".to_owned();
        ziwei.status =
            "mechanical_palaces_bureau_fourteen_stars_no_interpretation_or_runtime_signals.v4"
                .to_owned();
        ziwei
            .star_anchor_candidates
            .iter_mut()
            .for_each(|candidate| candidate.auxiliary_stars = None);
        let mut value = serde_json::to_value(snapshot).unwrap();
        for candidate in value["birth_time_audit"]["ziwei_life_body_audit"]
            ["star_anchor_candidates"]
            .as_array_mut()
            .unwrap()
        {
            candidate.as_object_mut().unwrap().remove("auxiliary_stars");
        }
        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 28);
        let stored = legacy.birth_time_audit.unwrap();
        assert!(stored
            .ziwei_life_body_audit
            .as_ref()
            .unwrap()
            .star_anchor_candidates
            .iter()
            .all(|candidate| candidate.auxiliary_stars.is_none()));
        assert!(birth_time_audits_match(&current, &stored));
    }

    #[test]
    fn astronomy_snapshot_v22_before_qizheng_identity_remains_readable() {
        let mut question = sample_question();
        question.category = QuestionCategory::LongTermFate;
        question.birth = Some(BirthParameters {
            sex: BiologicalSex::Male,
            date_time_bjt: question.divination_time_bjt,
            location: question.location.clone(),
        });
        let current =
            calculate_birth_time_audit_snapshot(question.birth.as_ref().expect("birth parameters"))
                .unwrap();
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 22;
        snapshot
            .birth_time_audit
            .as_mut()
            .expect("birth audit")
            .audit_version = 6;
        let mut value = serde_json::to_value(snapshot).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .get_mut("birth_time_audit")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove("qizheng_siyu_identity_audit");

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 22);
        let audit = legacy.birth_time_audit.expect("legacy birth audit");
        assert_eq!(audit.audit_version, 6);
        assert_eq!(audit.qizheng_siyu_identity_audit, None);
        assert!(birth_time_audits_match(&current, &audit));
    }

    #[test]
    fn astronomy_snapshot_v23_without_five_planet_vectors_remains_readable() {
        let mut question = sample_question();
        question.category = QuestionCategory::LongTermFate;
        question.birth = Some(BirthParameters {
            sex: BiologicalSex::Male,
            date_time_bjt: question.divination_time_bjt,
            location: question.location.clone(),
        });
        let current =
            calculate_birth_time_audit_snapshot(question.birth.as_ref().expect("birth parameters"))
                .unwrap();
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 23;
        snapshot.birth_time_audit.as_mut().unwrap().audit_version = 7;
        let mut value = serde_json::to_value(snapshot).unwrap();
        let birth_object = value["birth_time_audit"].as_object_mut().unwrap();
        birth_object.remove("qizheng_five_planet_geometric_observations");
        birth_object.remove("qizheng_five_planet_geometric_observation_error");
        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        let audit = legacy.birth_time_audit.expect("legacy birth audit");
        assert_eq!(legacy.snapshot_version, 23);
        assert_eq!(audit.audit_version, 7);
        assert_eq!(audit.qizheng_five_planet_geometric_observations, None);
        assert_eq!(audit.qizheng_five_planet_geometric_observation_error, None);
        assert!(birth_time_audits_match(&current, &audit));
    }

    #[test]
    fn astronomy_snapshot_v24_without_visible_stem_ten_gods_remains_readable() {
        let mut question = sample_question();
        question.category = QuestionCategory::LongTermFate;
        question.birth = Some(BirthParameters {
            sex: BiologicalSex::Male,
            date_time_bjt: question.divination_time_bjt,
            location: question.location.clone(),
        });
        let current =
            calculate_birth_time_audit_snapshot(question.birth.as_ref().expect("birth parameters"))
                .unwrap();
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 24;
        snapshot.birth_time_audit.as_mut().unwrap().audit_version = 8;
        let mut value = serde_json::to_value(snapshot).unwrap();
        value["birth_time_audit"]
            .as_object_mut()
            .unwrap()
            .remove("four_pillars_visible_stem_ten_gods");

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        let audit = legacy.birth_time_audit.expect("legacy birth audit");
        assert_eq!(legacy.snapshot_version, 24);
        assert_eq!(audit.audit_version, 8);
        assert_eq!(audit.four_pillars_visible_stem_ten_gods, None);
        assert!(birth_time_audits_match(&current, &audit));
    }

    #[test]
    fn astronomy_snapshot_v25_without_parallel_late_zi_ten_gods_remains_readable() {
        let mut question = sample_question();
        question.category = QuestionCategory::LongTermFate;
        question.birth = Some(BirthParameters {
            sex: BiologicalSex::Female,
            date_time_bjt: FixedOffset::east_opt(8 * 3_600)
                .unwrap()
                .with_ymd_and_hms(1990, 5, 17, 23, 50, 0)
                .unwrap(),
            location: question.location.clone(),
        });
        let current =
            calculate_birth_time_audit_snapshot(question.birth.as_ref().expect("birth parameters"))
                .unwrap();
        assert!(current
            .four_pillars_visible_stem_ten_gods_zi_start
            .is_some());
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 25;
        snapshot.birth_time_audit.as_mut().unwrap().audit_version = 9;
        let mut value = serde_json::to_value(snapshot).unwrap();
        value["birth_time_audit"]
            .as_object_mut()
            .unwrap()
            .remove("four_pillars_visible_stem_ten_gods_zi_start");

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        let audit = legacy.birth_time_audit.expect("legacy birth audit");
        assert_eq!(legacy.snapshot_version, 25);
        assert_eq!(audit.audit_version, 9);
        assert!(audit.four_pillars_visible_stem_ten_gods.is_some());
        assert_eq!(audit.four_pillars_visible_stem_ten_gods_zi_start, None);
        assert!(birth_time_audits_match(&current, &audit));
    }

    #[test]
    fn astronomy_snapshot_v9_without_lunisolar_month_structure_remains_readable() {
        let question = sample_question();
        let mut snapshot = calculate_astronomy_snapshot(&question).unwrap();
        snapshot.snapshot_version = 9;
        let mut value = serde_json::to_value(snapshot).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .remove("lunisolar_month_structure");
        value
            .as_object_mut()
            .unwrap()
            .remove("lunisolar_month_structure_error");
        value
            .as_object_mut()
            .unwrap()
            .remove("lunisolar_date_candidate");
        value
            .as_object_mut()
            .unwrap()
            .remove("lunisolar_date_candidate_error");

        let legacy: AstronomyAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.snapshot_version, 9);
        assert!(legacy.lunation_interval.is_some());
        assert_eq!(legacy.lunisolar_month_structure, None);
        assert_eq!(legacy.lunisolar_month_structure_error, None);
        assert_eq!(legacy.lunisolar_date_candidate, None);
        assert_eq!(legacy.lunisolar_date_candidate_error, None);
    }

    #[test]
    fn persists_birth_true_solar_time_and_calendar_views_in_current_mechanical_audit() {
        let mut storage = Storage::open_in_memory().unwrap();
        let mut question = sample_question();
        question.category = QuestionCategory::LongTermFate;
        question.subcategory = "长期趋势".to_owned();
        question.text = "未来长期趋势如何？".to_owned();
        question.target_event = "长期趋势".to_owned();
        question.birth = Some(BirthParameters {
            sex: BiologicalSex::Female,
            date_time_bjt: FixedOffset::east_opt(8 * 3600)
                .unwrap()
                .with_ymd_and_hms(1990, 5, 17, 8, 30, 0)
                .unwrap(),
            location: question.location.clone(),
        });
        let classification = KeywordClassifier::default()
            .classify(question.category, &question.text)
            .unwrap();
        let fingerprint = build_event_fingerprint(&question, &classification);
        storage
            .insert_locked_question(&question, &classification, &fingerprint, None)
            .unwrap();

        let snapshot = storage
            .load_astronomy_snapshot(question.id)
            .unwrap()
            .unwrap();
        assert_eq!(snapshot.snapshot_version, 29);
        let audit = snapshot.birth_time_audit.expect("birth audit is persisted");
        assert_eq!(audit.audit_version, 13);
        assert_eq!(audit.status, BIRTH_TIME_AUDIT_STATUS);
        assert_eq!(audit.sex, BiologicalSex::Female);
        assert_eq!(audit.input_time_bjt, "1990-05-17T08:30:00+08:00");
        assert_eq!(
            audit.sexagenary_calendar_system,
            "proleptic_gregorian_iso_input.v1"
        );
        assert_eq!(audit.true_solar_boundary_margin_minutes, 5.0);
        assert!(audit.jd_tt > audit.jd_ut1_approx);
        let birth_calendar = audit
            .civil_calendar_audit
            .as_ref()
            .expect("birth calendar views are persisted");
        assert_eq!(birth_calendar.proleptic_gregorian.year, 1990);
        assert_eq!(birth_calendar.julian.month, 5);
        assert_eq!(birth_calendar.julian.day, 4);
        let chart = audit
            .four_pillars_chart_candidate
            .as_ref()
            .expect("four-pillars chart candidate is persisted");
        assert_eq!(chart.compact_zh(), "庚午 辛巳 壬午 甲辰");
        assert!(!chart.late_zi_hour_day_ambiguous);
        assert!(chart.mechanical_boundaries_safe);
        assert!(!chart.rule_eligible);
        assert_eq!(
            chart.status,
            "chart_candidate_only_no_fortune_rules_or_runtime_signals.v1"
        );
        let metadata = audit
            .four_pillars_structural_metadata
            .as_ref()
            .expect("stem-branch structural metadata is persisted");
        assert_eq!(
            metadata.compact_zh(),
            "庚（阳金）午（阳火）  辛（阴金）巳（阴火）  壬（阳水）午（阳火）  甲（阳木）辰（阳土）"
        );
        assert_eq!(
            metadata.status,
            "structural_metadata_only_not_strength_or_fortune_rule.v1"
        );
        let visible_ten_gods = audit
            .four_pillars_visible_stem_ten_gods
            .as_ref()
            .expect("visible-stem ten-god identities are persisted");
        assert_eq!(visible_ten_gods.day_master_stem.zh_name(), "壬");
        assert_eq!(visible_ten_gods.observations.len(), 4);
        assert_eq!(
            visible_ten_gods
                .observations
                .iter()
                .map(|item| {
                    item.ten_god_identity
                        .map(|identity| identity.zh_name())
                        .unwrap_or("日主")
                })
                .collect::<Vec<_>>(),
            vec!["偏印", "正印", "日主", "食神"]
        );
        assert!(!visible_ten_gods.hidden_stems_included);
        assert!(!visible_ten_gods.strength_assessed);
        assert!(!visible_ten_gods.pattern_selected);
        assert!(!visible_ten_gods.interpretation_eligible);
        assert!(!visible_ten_gods.rule_eligible);
        assert_eq!(audit.four_pillars_visible_stem_ten_gods_zi_start, None);
        assert_eq!(audit.late_zi_dual_candidates, None);
        let birth_lunar = audit
            .lunisolar_date_candidate
            .as_ref()
            .expect("birth lunisolar date candidate is persisted");
        assert_eq!(birth_lunar.nominal_lunar_year, 1990);
        assert!(birth_lunar.date_assignment_safe);
        assert_eq!(audit.lunisolar_date_candidate_error, None);
        let ziwei = audit
            .ziwei_life_body_audit
            .as_ref()
            .expect("Ziwei life/body palace audit is persisted");
        assert_eq!(ziwei.audit_version, 5);
        assert_eq!(ziwei.selected_hour_convention, None);
        assert_eq!(ziwei.selected_year_stem_convention, None);
        assert_eq!(ziwei.year_stem_convention_candidates.len(), 2);
        assert_eq!(ziwei.star_anchor_candidates.len(), 4);
        assert!(ziwei.star_anchor_candidates.iter().all(|candidate| {
            candidate
                .major_stars
                .as_ref()
                .is_some_and(|stars| stars.placements.len() == 14 && !stars.rule_eligible)
        }));
        assert!(ziwei.year_stem_conventions_identical);
        assert_eq!(ziwei.beijing_time_candidate.chart.palaces.len(), 12);
        assert_eq!(ziwei.true_solar_time_candidate.chart.palaces.len(), 12);
        for year in &ziwei.year_stem_convention_candidates {
            assert_eq!(year.beijing_time_candidate.branch_stems.len(), 12);
            assert_eq!(year.true_solar_time_candidate.branch_stems.len(), 12);
            assert!(!year.beijing_time_candidate.interpretation_eligible);
            assert!(!year.beijing_time_candidate.rule_eligible);
        }
        assert!(!ziwei.rule_eligible);
        assert_eq!(audit.ziwei_life_body_audit_error, None);
        let qizheng = audit
            .qizheng_siyu_identity_audit
            .as_ref()
            .expect("Qizheng Siyu identity audit is persisted");
        assert_eq!(qizheng.seven_governors.len(), 7);
        assert_eq!(qizheng.four_residuals.len(), 4);
        assert!(!qizheng.residual_astronomical_roles_selected);
        assert!(!qizheng.positions_calculated);
        assert!(!qizheng.natal_chart_generated);
        assert!(!qizheng.interpretation_eligible);
        assert!(!qizheng.rule_eligible);
        let planets = audit
            .qizheng_five_planet_geometric_observations
            .as_ref()
            .expect("five research vectors are persisted");
        assert_eq!(planets.len(), 5);
        assert!(planets
            .iter()
            .all(|planet| !planet.historical_mansion_eligible && !planet.divination_rule_eligible));
        assert_eq!(audit.qizheng_five_planet_geometric_observation_error, None);
        let recomputed =
            calculate_birth_time_audit_snapshot(question.birth.as_ref().unwrap()).unwrap();
        assert!(birth_time_audits_match(&recomputed, &audit));
        let mut changed = audit.clone();
        changed.true_solar_total_correction_minutes += 1.0e-6;
        assert!(!birth_time_audits_match(&recomputed, &changed));
        let mut changed_planet = audit.clone();
        changed_planet
            .qizheng_five_planet_geometric_observations
            .as_mut()
            .unwrap()[0]
            .emb_relative_longitude_deg += 0.01;
        assert!(!birth_time_audits_match(&recomputed, &changed_planet));
        let mut changed_ten_god = audit.clone();
        changed_ten_god
            .four_pillars_visible_stem_ten_gods
            .as_mut()
            .unwrap()
            .observations[0]
            .ten_god_identity = Some(divination_core::TenGodIdentity::DirectOfficer);
        assert!(!birth_time_audits_match(&recomputed, &changed_ten_god));
    }

    #[test]
    fn birth_audit_v1_without_calendar_views_remains_readable() {
        let question = sample_question();
        let birth = BirthParameters {
            sex: BiologicalSex::Male,
            date_time_bjt: question.divination_time_bjt,
            location: question.location,
        };
        let mut audit = calculate_birth_time_audit_snapshot(&birth).unwrap();
        audit.audit_version = 1;
        let mut value = serde_json::to_value(audit).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .remove("civil_calendar_audit");
        value
            .as_object_mut()
            .unwrap()
            .remove("four_pillars_chart_candidate");
        value
            .as_object_mut()
            .unwrap()
            .remove("four_pillars_structural_metadata");
        value
            .as_object_mut()
            .unwrap()
            .remove("late_zi_dual_candidates");

        let legacy: BirthTimeAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.audit_version, 1);
        assert_eq!(legacy.civil_calendar_audit, None);
        assert_eq!(legacy.four_pillars_chart_candidate, None);
        assert_eq!(legacy.four_pillars_structural_metadata, None);
        assert_eq!(legacy.late_zi_dual_candidates, None);
        assert!(birth_time_audits_match(
            &calculate_birth_time_audit_snapshot(&birth).unwrap(),
            &legacy
        ));
    }

    #[test]
    fn late_zi_birth_keeps_day_ownership_unresolved_and_blocks_safety() {
        let question = sample_question();
        let birth = BirthParameters {
            sex: BiologicalSex::Female,
            date_time_bjt: FixedOffset::east_opt(8 * 3_600)
                .unwrap()
                .with_ymd_and_hms(1990, 5, 17, 23, 50, 0)
                .unwrap(),
            location: question.location,
        };
        let audit = calculate_birth_time_audit_snapshot(&birth).unwrap();
        let dual = audit
            .late_zi_dual_candidates
            .as_ref()
            .expect("late Zi persists both day-boundary candidates");
        assert_eq!(dual.selected_convention, None);
        assert_eq!(
            (dual.midnight_boundary.day_pillar.cycle_index + 1) % 60,
            dual.zi_start_boundary.day_pillar.cycle_index
        );
        assert_eq!(dual.midnight_boundary.day_pillar.zh_name(), "壬午");
        assert_eq!(dual.zi_start_boundary.day_pillar.zh_name(), "癸未");
        let midnight_ten_gods = audit
            .four_pillars_visible_stem_ten_gods
            .as_ref()
            .expect("midnight visible-stem identities");
        let zi_start_ten_gods = audit
            .four_pillars_visible_stem_ten_gods_zi_start
            .as_ref()
            .expect("Zi-start visible-stem identities");
        assert_eq!(
            midnight_ten_gods.day_master_stem,
            dual.midnight_boundary.day_pillar.stem
        );
        assert_eq!(
            zi_start_ten_gods.day_master_stem,
            dual.zi_start_boundary.day_pillar.stem
        );
        assert_eq!(
            midnight_ten_gods.observations[3].stem,
            dual.midnight_boundary.hour_pillar.stem
        );
        assert_eq!(
            zi_start_ten_gods.observations[3].stem,
            dual.zi_start_boundary.hour_pillar.stem
        );
        assert_ne!(
            midnight_ten_gods.compact_zh(),
            zi_start_ten_gods.compact_zh()
        );
        assert!(!zi_start_ten_gods.rule_eligible);
        let mut altered = audit.clone();
        altered
            .four_pillars_visible_stem_ten_gods_zi_start
            .as_mut()
            .unwrap()
            .observations[0]
            .ten_god_identity = Some(divination_core::TenGodIdentity::Peer);
        assert!(!birth_time_audits_match(&audit, &altered));
        let chart = audit.four_pillars_chart_candidate.unwrap();
        assert_eq!(chart.true_solar_shichen.branch.zh_name(), "子");
        assert!(chart.late_zi_hour_day_ambiguous);
        assert!(!chart.mechanical_boundaries_safe);
        assert!(!chart.rule_eligible);
    }

    #[test]
    fn birth_audit_v2_without_four_pillars_chart_remains_readable() {
        let question = sample_question();
        let birth = BirthParameters {
            sex: BiologicalSex::Male,
            date_time_bjt: question.divination_time_bjt,
            location: question.location,
        };
        let mut audit = calculate_birth_time_audit_snapshot(&birth).unwrap();
        audit.audit_version = 2;
        let mut value = serde_json::to_value(audit).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .remove("four_pillars_chart_candidate");
        value
            .as_object_mut()
            .unwrap()
            .remove("four_pillars_structural_metadata");
        value
            .as_object_mut()
            .unwrap()
            .remove("late_zi_dual_candidates");

        let legacy: BirthTimeAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.audit_version, 2);
        assert!(legacy.civil_calendar_audit.is_some());
        assert_eq!(legacy.four_pillars_chart_candidate, None);
        assert_eq!(legacy.four_pillars_structural_metadata, None);
        assert_eq!(legacy.late_zi_dual_candidates, None);
        assert!(birth_time_audits_match(
            &calculate_birth_time_audit_snapshot(&birth).unwrap(),
            &legacy
        ));
    }

    #[test]
    fn birth_audit_v3_without_structural_metadata_remains_readable() {
        let question = sample_question();
        let birth = BirthParameters {
            sex: BiologicalSex::Male,
            date_time_bjt: question.divination_time_bjt,
            location: question.location,
        };
        let mut audit = calculate_birth_time_audit_snapshot(&birth).unwrap();
        audit.audit_version = 3;
        let mut value = serde_json::to_value(audit).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .remove("four_pillars_structural_metadata");
        value
            .as_object_mut()
            .unwrap()
            .remove("late_zi_dual_candidates");

        let legacy: BirthTimeAuditSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.audit_version, 3);
        assert!(legacy.four_pillars_chart_candidate.is_some());
        assert_eq!(legacy.four_pillars_structural_metadata, None);
        assert_eq!(legacy.late_zi_dual_candidates, None);
        assert!(birth_time_audits_match(
            &calculate_birth_time_audit_snapshot(&birth).unwrap(),
            &legacy
        ));
    }

    #[test]
    fn astronomy_snapshot_rejects_tampered_persisted_bytes() {
        let mut storage = Storage::open_in_memory().unwrap();
        let question = sample_question();
        let classification = KeywordClassifier::default()
            .classify(question.category, &question.text)
            .unwrap();
        let fingerprint = build_event_fingerprint(&question, &classification);
        storage
            .insert_locked_question(&question, &classification, &fingerprint, None)
            .unwrap();
        storage
            .connection
            .execute_batch("DROP TRIGGER prevent_astronomy_snapshot_update")
            .unwrap();
        storage
            .connection
            .execute(
                "UPDATE astronomy_snapshots SET snapshot_json = snapshot_json || ' '
                 WHERE question_id = ?1",
                [question.id.to_string()],
            )
            .unwrap();

        assert!(matches!(
            storage.load_astronomy_snapshot(question.id),
            Err(StorageError::AstronomySnapshot(_))
        ));
    }

    fn sample_rule() -> RuleRecord {
        RuleRecord {
            rule_id: "TEST-ONLY-LIUYAO-001".to_owned(),
            method: DivinationMethod::Liuyao,
            applicable_question_types: vec![AncientQuestionType::ProjectOutcome],
            condition_expression: serde_json::json!({
                "schema_version": 1,
                "condition": {"op": "eq", "path": "/chart/fixture", "value": true}
            })
            .to_string(),
            result_expression: serde_json::json!({
                "schema_version": 1,
                "signals": {"trend": 60, "outcome_direction": 60}
            })
            .to_string(),
            timing_expression: Some(
                serde_json::json!({"schema_version": 1, "window": "within_one_month"}).to_string(),
            ),
            action_key: Some("project.verify_before_commit".to_owned()),
            action_polarity: Some(ActionPolarity::Do),
            action_expression: Some("先核实关键条件，再推进不可逆步骤。".to_owned()),
            priority: 100,
            evidence_tier: EvidenceTier::EarlyCoreClassic,
            historical_period: "仅供自动化测试".to_owned(),
            book_title: "测试资料，不属于产品规则库".to_owned(),
            volume: "测试卷".to_owned(),
            chapter: "测试篇".to_owned(),
            original_text: "这是门禁自动化测试原文，不得作为占断规则。".to_owned(),
            edition: "测试版本".to_owned(),
            source_reference: "TEST-FIXTURE-ONLY".to_owned(),
            textual_variants: "无".to_owned(),
            stability: RuleStability::Stable,
            implementation_version: "test-1".to_owned(),
            reviewed: true,
        }
    }

    #[test]
    fn rule_action_metadata_is_all_or_nothing_and_hash_bound() {
        let storage = Storage::open_in_memory().unwrap();
        let mut partial = sample_rule();
        partial.action_expression = None;
        assert!(matches!(
            storage.upsert_rule(&partial),
            Err(StorageError::RuleGateValidation(_))
        ));

        let mut released = release_sample_rule(&storage);
        assert!(storage
            .rule_gate_report(&released.rule_id)
            .unwrap()
            .unwrap()
            .may_execute());
        released.action_expression = Some("修改后的行动文字必须重新复核。".to_owned());
        storage.upsert_rule(&released).unwrap();
        let report = storage
            .rule_gate_report(&released.rule_id)
            .unwrap()
            .unwrap();
        assert!(report.rule_fields_ready);
        assert!(!report.reviews_ready);
        assert!(!report.tests_ready);
        assert!(!report.may_execute());
    }

    #[test]
    fn rule_import_rejects_free_form_or_malformed_programs() {
        let storage = Storage::open_in_memory().unwrap();
        let mut rule = sample_rule();
        rule.condition_expression = "fixture.input == true".to_owned();
        assert!(matches!(
            storage.upsert_rule(&rule),
            Err(StorageError::RuleProgram(_))
        ));

        let mut rule = sample_rule();
        rule.result_expression = serde_json::json!({
            "schema_version": 1,
            "signals": {"trend": 101}
        })
        .to_string();
        assert!(matches!(
            storage.upsert_rule(&rule),
            Err(StorageError::RuleProgram(_))
        ));
        assert_eq!(
            storage.rule_evidence_summary().unwrap(),
            RuleEvidenceSummary::default()
        );
    }

    #[test]
    fn released_rule_executes_only_its_compiled_json_program() {
        let storage = Storage::open_in_memory().unwrap();
        let rule = release_sample_rule(&storage);
        let released = storage
            .released_rules(
                DivinationMethod::Liuyao,
                &AncientQuestionType::ProjectOutcome,
            )
            .unwrap();
        assert_eq!(released.len(), 1);

        let matched = released[0]
            .evaluate(&serde_json::json!({"chart": {"fixture": true}}))
            .unwrap();
        assert!(matched.matched);
        let signals = matched.signals.unwrap();
        assert_eq!(signals.trend, Some(60));
        assert_eq!(signals.outcome_direction, Some(60));
        assert_eq!(
            signals.timing,
            Some(divination_core::TimingWindow::WithinOneMonth)
        );
        assert_eq!(matched.action_advice.unwrap().rule_id, rule.rule_id);

        let unmatched = released[0]
            .evaluate(&serde_json::json!({"chart": {"fixture": false}}))
            .unwrap();
        assert!(!unmatched.matched);
        assert!(unmatched.signals.is_none());
        assert!(unmatched.action_advice.is_none());
    }

    fn sample_source_edition() -> SourceEditionRecord {
        SourceEditionRecord {
            source_id: "TEST-SOURCE-001".to_owned(),
            canonical_title: "测试资料".to_owned(),
            attributed_author: "测试作者".to_owned(),
            historical_period: "测试时期".to_owned(),
            edition_title: "门禁测试影印本".to_owned(),
            edition_type: EditionType::Facsimile,
            edition_year: "2026".to_owned(),
            holding_institution: "测试馆藏".to_owned(),
            catalog_identifier: "TEST-CATALOG-001".to_owned(),
            facsimile_reference: "TEST-FACSIMILE-P1".to_owned(),
                                                                       
            digital_checksum:
                "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".to_owned(),
            notes: "仅供内存自动化测试".to_owned(),
        }
    }

    fn sample_source_passage() -> SourcePassageRecord {
        SourcePassageRecord {
            passage_id: "TEST-PASSAGE-001".to_owned(),
            source_id: "TEST-SOURCE-001".to_owned(),
            volume: "测试卷".to_owned(),
            chapter: "测试篇".to_owned(),
            folio_or_page: "第1页".to_owned(),
            original_text: "这是门禁自动化测试原文，不得作为占断规则。".to_owned(),
            normalized_text: "这是门禁自动化测试原文不得作为占断规则".to_owned(),
            textual_variants: "无".to_owned(),
            transcription_status: TranscriptionStatus::Verified,
        }
    }

    fn sample_facsimile_verification() -> SourceFacsimileVerificationRecord {
        SourceFacsimileVerificationRecord {
            source_id: "TEST-SOURCE-001".to_owned(),
            digital_checksum:
                "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".to_owned(),
            byte_length: 3,
            file_name: "synthetic-abc.bin".to_owned(),
            verifier_version: FACSIMILE_VERIFIER_VERSION.to_owned(),
            verified_at: "2026-08-30T12:00:00+08:00".to_owned(),
        }
    }

    fn add_sample_source_and_reviews(storage: &Storage, rule_id: &str) {
        storage
            .upsert_source_edition(&sample_source_edition())
            .unwrap();
        storage
            .import_source_facsimile_verification(&sample_facsimile_verification())
            .unwrap();
        storage
            .upsert_source_passage(&sample_source_passage())
            .unwrap();
        storage
            .link_rule_source(
                rule_id,
                "TEST-PASSAGE-001",
                SourceRelation::Primary,
                "自动化测试主原文",
            )
            .unwrap();
        for kind in ReviewKind::ALL {
            storage
                .record_rule_review(
                    rule_id,
                    kind,
                    ReviewDecision::Approved,
                    "automated-gate-test",
                    "仅证明门禁状态转换，不证明任何真实术数规则",
                )
                .unwrap();
        }
    }

    fn sample_test_case(id: &str, boundary_class: &str) -> RuleTestCaseRecord {
        let input = serde_json::json!({"chart": {"fixture": true}});
        let expected_output = serde_json::to_value(
            compile_rule_program(&sample_rule())
                .unwrap()
                .evaluate(&input)
                .unwrap()
                .auditable_output(),
        )
        .unwrap();
        RuleTestCaseRecord {
            test_case_id: id.to_owned(),
            rule_id: "TEST-ONLY-LIUYAO-001".to_owned(),
            name: format!("门禁用例 {id}"),
            input,
            expected_output,
            boundary_class: boundary_class.to_owned(),
            source_basis: "TEST-FIXTURE-ONLY".to_owned(),
            enabled: true,
        }
    }

    fn release_sample_rule(storage: &Storage) -> RuleRecord {
        let rule = sample_rule();
        storage.upsert_rule(&rule).unwrap();
        add_sample_source_and_reviews(storage, &rule.rule_id);
        storage
            .upsert_rule_test_case(&sample_test_case("TEST-CASE-001", "ordinary"))
            .unwrap();
        let execution = storage
            .run_rule_test_suite(&rule.rule_id, &serde_json::json!({"target": "test"}))
            .unwrap();
        assert_eq!(execution.run.passed_count, 1);
        assert_eq!(execution.run.failed_count, 0);
        assert!(execution.failed_case_ids.is_empty());
        assert!(storage
            .rule_gate_report(&rule.rule_id)
            .unwrap()
            .unwrap()
            .may_execute());
        rule
    }

    fn sample_reviewed_seed_bundle() -> RuleSeedBundle {
        let builder = Storage::open_in_memory().unwrap();
        let rule = sample_rule();
        let source = sample_source_edition();
        let passage = sample_source_passage();
        let test_case = sample_test_case("TEST-CASE-001", "ordinary");
        builder.upsert_source_edition(&source).unwrap();
        let verification = sample_facsimile_verification();
        builder
            .import_source_facsimile_verification(&verification)
            .unwrap();
        builder.upsert_source_passage(&passage).unwrap();
        builder.upsert_rule(&rule).unwrap();
        builder
            .link_rule_source(
                &rule.rule_id,
                &passage.passage_id,
                SourceRelation::Primary,
                "自动化测试主原文",
            )
            .unwrap();
        builder.upsert_rule_test_case(&test_case).unwrap();
        let rule_content_hash = builder.rule_content_hash(&rule.rule_id).unwrap();
        let evidence_bundle_hash = builder.current_evidence_bundle_hash(&rule.rule_id).unwrap();
        let (test_suite_hash, _) = builder.current_test_suite_state(&rule.rule_id).unwrap();
        let reviews = ReviewKind::ALL
            .into_iter()
            .map(|kind| RuleReviewSeed {
                review_id: format!("TEST-REVIEW-{}", kind.key()),
                rule_id: rule.rule_id.clone(),
                kind,
                decision: ReviewDecision::Approved,
                reviewer: "automated-seed-test".to_owned(),
                evidence_notes: "仅测试规则包哈希绑定，不证明真实术数规则".to_owned(),
                rule_content_hash: rule_content_hash.clone(),
                evidence_bundle_hash: evidence_bundle_hash.clone(),
                reviewed_at: "2026-08-30T12:00:00+08:00".to_owned(),
            })
            .collect();
        RuleSeedBundle {
            schema_version: RULE_SEED_SCHEMA_VERSION,
            bundle_id: "test-reviewed-seed-bundle".to_owned(),
            sources: vec![source],
            facsimile_verifications: vec![verification],
            passages: vec![passage.clone()],
            rules: vec![rule.clone()],
            links: vec![RuleSourceLinkSeed {
                rule_id: rule.rule_id.clone(),
                passage_id: passage.passage_id,
                relation: SourceRelation::Primary,
                notes: "自动化测试主原文".to_owned(),
            }],
            reviews,
            test_cases: vec![test_case],
            test_runs: vec![RuleTestRunSeed {
                run_id: "TEST-SEED-RUN-001".to_owned(),
                rule_id: rule.rule_id,
                rule_content_hash,
                implementation_version: rule.implementation_version,
                test_suite_hash,
                passed_count: 1,
                failed_count: 0,
                total_count: 1,
                environment: serde_json::json!({"target": "seed-import-test"}),
                executed_at: "2026-08-30T12:30:00+08:00".to_owned(),
            }],
        }
    }

    #[test]
    fn reviewed_rule_seed_import_is_hash_bound_atomic_and_idempotent() {
        let storage = Storage::open_in_memory().unwrap();
        let bundle = sample_reviewed_seed_bundle();
        storage.import_rule_seed_bundle(&bundle).unwrap();
        let summary = storage.rule_evidence_summary().unwrap();
        assert_eq!(summary.total_rules, 1);
        assert_eq!(summary.executable_rules, 1);
        assert_eq!(summary.facsimile_verifications, 1);
        assert_eq!(summary.reviews, 4);
        assert_eq!(summary.test_runs, 1);

        storage.import_rule_seed_bundle(&bundle).unwrap();
        assert_eq!(storage.rule_evidence_summary().unwrap(), summary);

        let mut false_test_claim = bundle.clone();
        false_test_claim.test_runs[0].passed_count = 0;
        false_test_claim.test_runs[0].failed_count = 1;
        assert!(matches!(
            Storage::open_in_memory()
                .unwrap()
                .import_rule_seed_bundle(&false_test_claim),
            Err(StorageError::InvalidRuleSeed(_))
        ));

        let mut invalid_bundle = bundle.clone();
        invalid_bundle.reviews[0].rule_content_hash = "tampered".to_owned();
        assert!(matches!(
            storage.import_rule_seed_bundle(&invalid_bundle),
            Err(StorageError::InvalidRuleSeed(_))
        ));
        assert_eq!(storage.rule_evidence_summary().unwrap(), summary);
    }

    #[test]
    fn automatic_rule_test_runner_records_real_failures_and_keeps_gate_closed() {
        let storage = Storage::open_in_memory().unwrap();
        let rule = sample_rule();
        storage.upsert_rule(&rule).unwrap();
        add_sample_source_and_reviews(&storage, &rule.rule_id);
        let mut mismatched = sample_test_case("TEST-CASE-MISMATCH", "ordinary");
        mismatched.expected_output["signals"]["trend"] = serde_json::json!(59);
        storage.upsert_rule_test_case(&mismatched).unwrap();

        let execution = storage
            .run_rule_test_suite(
                &rule.rule_id,
                &serde_json::json!({"target": "automatic-test"}),
            )
            .unwrap();
        assert_eq!(execution.run.passed_count, 0);
        assert_eq!(execution.run.failed_count, 1);
        assert_eq!(
            execution.failed_case_ids,
            vec!["TEST-CASE-MISMATCH".to_owned()]
        );
        assert!(
            !storage
                .rule_gate_report(&rule.rule_id)
                .unwrap()
                .unwrap()
                .tests_ready
        );
    }

    #[test]
    fn rule_seed_rejects_duplicate_ids_before_writing() {
        let storage = Storage::open_in_memory().unwrap();
        let mut bundle: RuleSeedBundle = serde_json::from_str(RULE_SEED).unwrap();
        let rule = sample_rule();
        bundle.rules = vec![rule.clone(), rule];
        assert!(matches!(
            storage.import_rule_seed_bundle(&bundle),
            Err(StorageError::InvalidRuleSeed(_))
        ));
        assert_eq!(
            storage.rule_evidence_summary().unwrap(),
            RuleEvidenceSummary::default()
        );
    }

    #[test]
    fn rule_gate_requires_every_layer_and_rule_edits_invalidate_approvals() {
        let storage = Storage::open_in_memory().unwrap();
        let mut rule = sample_rule();
        let first_hash = storage.upsert_rule(&rule).unwrap();
        let fields_only = storage.rule_gate_report(&rule.rule_id).unwrap().unwrap();
        assert!(fields_only.rule_fields_ready);
        assert!(!fields_only.primary_source_ready);
        assert!(!fields_only.reviews_ready);
        assert!(!fields_only.tests_ready);
        assert!(!fields_only.may_execute());
        let failed_keys = fields_only
            .checks
            .iter()
            .filter(|check| !check.passed)
            .map(|check| check.key.as_str())
            .collect::<BTreeSet<_>>();
        assert!(failed_keys.contains("source.primary_link"));
        assert!(failed_keys.contains("source.verified_passage"));
        assert!(failed_keys.contains("source.qualified_edition"));
        assert!(failed_keys.contains("source.facsimile_bytes"));
        assert!(failed_keys.contains("review.bibliography"));
        assert!(failed_keys.contains("review.transcription"));
        assert!(failed_keys.contains("review.logic"));
        assert!(failed_keys.contains("review.implementation"));
        assert!(failed_keys.contains("tests.enabled_cases"));
        assert!(failed_keys.contains("tests.suite_hash"));
        assert!(failed_keys.contains("tests.current_zero_failure_run"));

        add_sample_source_and_reviews(&storage, &rule.rule_id);
        let reviewed = storage.rule_gate_report(&rule.rule_id).unwrap().unwrap();
        assert!(reviewed.primary_source_ready);
        assert!(reviewed.reviews_ready);
        assert!(!reviewed.tests_ready);
        assert!(reviewed
            .checks
            .iter()
            .filter(|check| check.key.starts_with("source.") || check.key.starts_with("review."))
            .all(|check| check.passed));
        assert!(
            !reviewed
                .checks
                .iter()
                .find(|check| check.key == "tests.enabled_cases")
                .unwrap()
                .passed
        );

        storage
            .upsert_rule_test_case(&sample_test_case("TEST-CASE-001", "ordinary"))
            .unwrap();
        assert!(
            !storage
                .rule_gate_report(&rule.rule_id)
                .unwrap()
                .unwrap()
                .tests_ready
        );
        storage
            .record_rule_test_run(&rule.rule_id, 1, 0, &serde_json::json!({"target": "test"}))
            .unwrap();
        let released = storage.rule_gate_report(&rule.rule_id).unwrap().unwrap();
        assert!(released.may_execute());
        assert!(released.blocking_reasons.is_empty());
        assert!(released.checks.iter().all(|check| check.passed));
        assert_eq!(storage.rule_evidence_summary().unwrap().executable_rules, 1);
        let runtime_rules = storage
            .released_rules(
                DivinationMethod::Liuyao,
                &AncientQuestionType::ProjectOutcome,
            )
            .unwrap();
        assert_eq!(runtime_rules.len(), 1);
        assert_eq!(runtime_rules[0].rule.rule_id, rule.rule_id);
        assert!(storage
            .released_rules(DivinationMethod::Liuyao, &AncientQuestionType::WeatherRain,)
            .unwrap()
            .is_empty());

        rule.result_expression = serde_json::json!({
            "schema_version": 1,
            "signals": {"trend": 40, "outcome_direction": 40}
        })
        .to_string();
        let changed_hash = storage.upsert_rule(&rule).unwrap();
        assert_ne!(first_hash, changed_hash);
        let invalidated = storage.rule_gate_report(&rule.rule_id).unwrap().unwrap();
        assert!(invalidated.rule_fields_ready);
        assert!(invalidated.primary_source_ready);
        assert!(!invalidated.reviews_ready);
        assert!(!invalidated.tests_ready);
        assert!(!invalidated.may_execute());
        assert!(storage
            .released_rules(
                DivinationMethod::Liuyao,
                &AncientQuestionType::ProjectOutcome,
            )
            .unwrap()
            .is_empty());
    }

    #[test]
    fn rule_evidence_dossier_reconstructs_every_layer_and_detects_stale_hashes() {
        let storage = Storage::open_in_memory().unwrap();
        assert!(storage
            .rule_evidence_dossier("MISSING-RULE")
            .unwrap()
            .is_none());

        let rule = sample_rule();
        storage.upsert_rule(&rule).unwrap();
        add_sample_source_and_reviews(&storage, &rule.rule_id);
        storage
            .upsert_rule_test_case(&sample_test_case("TEST-CASE-001", "ordinary"))
            .unwrap();
        storage
            .run_rule_test_suite(
                &rule.rule_id,
                &serde_json::json!({"target": "dossier-test"}),
            )
            .unwrap();

        let changes_before_read = storage.connection.total_changes();
        let dossier = storage
            .rule_evidence_dossier(&rule.rule_id)
            .unwrap()
            .unwrap();
        assert_eq!(storage.connection.total_changes(), changes_before_read);
        assert_eq!(dossier.rule.rule_id, rule.rule_id);
        assert!(dossier.rule_content_hash_matches);
        assert!(dossier.evidence_bundle_hash_matches);
        assert!(dossier.test_suite_hash_matches);
        assert!(dossier.gate.may_execute());
        assert_eq!(dossier.sources.len(), 1);
        assert_eq!(dossier.sources[0].relation, SourceRelation::Primary);
        assert_eq!(dossier.sources[0].edition, sample_source_edition());
        assert_eq!(dossier.sources[0].passage, sample_source_passage());
        assert_eq!(
            dossier.sources[0].facsimile_verification,
            Some(sample_facsimile_verification())
        );
        assert_eq!(dossier.reviews.len(), ReviewKind::ALL.len());
        assert!(dossier.reviews.iter().all(|review| {
            review.rule_content_hash == dossier.rule_content_hash
                && review.evidence_bundle_hash == dossier.evidence_bundle_hash
                && review.decision == ReviewDecision::Approved
        }));
        assert_eq!(dossier.test_cases.len(), 1);
        assert_eq!(dossier.test_cases[0].test_case_id, "TEST-CASE-001");
        assert_eq!(dossier.test_runs.len(), 1);
        assert_eq!(
            dossier.test_runs[0].environment,
            serde_json::json!({"target": "dossier-test"})
        );

        storage
            .connection
            .execute(
                "UPDATE rule_test_suite_state SET test_suite_hash = 'tampered' WHERE rule_id = ?1",
                [&rule.rule_id],
            )
            .unwrap();
        let changes_before_stale_read = storage.connection.total_changes();
        let stale = storage
            .rule_evidence_dossier(&rule.rule_id)
            .unwrap()
            .unwrap();
        assert_eq!(
            storage.connection.total_changes(),
            changes_before_stale_read
        );
        assert!(!stale.test_suite_hash_matches);
        assert_eq!(stale.test_suite_hash, "tampered");
        assert_ne!(stale.calculated_test_suite_hash, "tampered");
        assert!(!stale.gate.tests_ready);
        assert!(!stale.gate.may_execute());
    }

    #[test]
    fn changed_test_suite_cannot_reuse_an_old_passing_run() {
        let storage = Storage::open_in_memory().unwrap();
        let rule = sample_rule();
        storage.upsert_rule(&rule).unwrap();
        add_sample_source_and_reviews(&storage, &rule.rule_id);
        storage
            .upsert_rule_test_case(&sample_test_case("TEST-CASE-001", "ordinary"))
            .unwrap();
        storage
            .record_rule_test_run(&rule.rule_id, 1, 0, &serde_json::json!({"target": "test"}))
            .unwrap();
        assert!(storage
            .rule_gate_report(&rule.rule_id)
            .unwrap()
            .unwrap()
            .may_execute());

        storage
            .upsert_rule_test_case(&sample_test_case("TEST-CASE-002", "boundary"))
            .unwrap();
        assert!(
            !storage
                .rule_gate_report(&rule.rule_id)
                .unwrap()
                .unwrap()
                .tests_ready
        );
        storage
            .record_rule_test_run(&rule.rule_id, 2, 0, &serde_json::json!({"target": "test"}))
            .unwrap();
        assert!(storage
            .rule_gate_report(&rule.rule_id)
            .unwrap()
            .unwrap()
            .may_execute());

        storage
            .connection
            .execute(
                "UPDATE rule_test_cases SET expected_output_json = '{\"tampered\":true}'
                 WHERE test_case_id = 'TEST-CASE-002'",
                [],
            )
            .unwrap();
        assert!(
            !storage
                .rule_gate_report(&rule.rule_id)
                .unwrap()
                .unwrap()
                .tests_ready
        );
    }

    #[test]
    fn changed_source_evidence_invalidates_existing_reviews() {
        let storage = Storage::open_in_memory().unwrap();
        let rule = sample_rule();
        storage.upsert_rule(&rule).unwrap();
        add_sample_source_and_reviews(&storage, &rule.rule_id);
        storage
            .upsert_rule_test_case(&sample_test_case("TEST-CASE-001", "ordinary"))
            .unwrap();
        storage
            .record_rule_test_run(&rule.rule_id, 1, 0, &serde_json::json!({"target": "test"}))
            .unwrap();
        assert!(storage
            .rule_gate_report(&rule.rule_id)
            .unwrap()
            .unwrap()
            .may_execute());

        storage
            .connection
            .execute(
                "UPDATE source_editions SET digital_checksum = 'sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824'
                 WHERE source_id = 'TEST-SOURCE-001'",
                [],
            )
            .unwrap();
        let fail_closed = storage.rule_gate_report(&rule.rule_id).unwrap().unwrap();
        assert!(!fail_closed.primary_source_ready);
        assert!(!fail_closed.reviews_ready);

        storage.refresh_rule_evidence_state(&rule.rule_id).unwrap();
        let refreshed_source = storage.rule_gate_report(&rule.rule_id).unwrap().unwrap();
        assert!(!refreshed_source.primary_source_ready);
        assert!(!refreshed_source.reviews_ready);
        assert!(!refreshed_source.may_execute());
    }

    #[test]
    fn primary_source_gate_rejects_textual_checksums_and_unlocated_transcriptions() {
        let storage = Storage::open_in_memory().unwrap();
        let rule = sample_rule();
        storage.upsert_rule(&rule).unwrap();
        let mut source = sample_source_edition();
        let mut passage = sample_source_passage();
        storage.upsert_source_edition(&source).unwrap();
        storage.upsert_source_passage(&passage).unwrap();
        storage
            .link_rule_source(
                &rule.rule_id,
                &passage.passage_id,
                SourceRelation::Primary,
                "自动化测试主原文",
            )
            .unwrap();
        let is_ready = || {
            storage
                .rule_gate_report(&rule.rule_id)
                .unwrap()
                .unwrap()
                .primary_source_ready
        };
        assert!(!is_ready());
        storage
            .import_source_facsimile_verification(&sample_facsimile_verification())
            .unwrap();
        assert!(is_ready());

        source.digital_checksum = "sha256:unverified".to_owned();
        storage.upsert_source_edition(&source).unwrap();
        assert!(!is_ready());
        let checksum_mismatch = storage.rule_gate_report(&rule.rule_id).unwrap().unwrap();
        assert!(
            !checksum_mismatch
                .checks
                .iter()
                .find(|check| check.key == "source.qualified_edition")
                .unwrap()
                .passed
        );
        assert!(
            !checksum_mismatch
                .checks
                .iter()
                .find(|check| check.key == "source.facsimile_bytes")
                .unwrap()
                .passed
        );
        source.digital_checksum = format!("sha256:{}g", "0".repeat(63));
        storage.upsert_source_edition(&source).unwrap();
        assert!(!is_ready());
        source.digital_checksum = sample_source_edition().digital_checksum;
        storage.upsert_source_edition(&source).unwrap();
        assert!(is_ready());

        source.edition_type = EditionType::Transcription;
        storage.upsert_source_edition(&source).unwrap();
        assert!(!is_ready());
        source.edition_type = EditionType::Facsimile;
        storage.upsert_source_edition(&source).unwrap();
        assert!(is_ready());

        passage.folio_or_page = " ".to_owned();
        storage.upsert_source_passage(&passage).unwrap();
        assert!(!is_ready());
        passage.folio_or_page = "第1页".to_owned();
        passage.original_text.clear();
        storage.upsert_source_passage(&passage).unwrap();
        assert!(!is_ready());
        passage = sample_source_passage();
        storage.upsert_source_passage(&passage).unwrap();
        assert!(is_ready());
    }

    #[test]
    fn facsimile_verifier_hashes_actual_bytes_and_rejects_a_mismatch() {
        let storage = Storage::open_in_memory().unwrap();
        storage
            .upsert_source_edition(&sample_source_edition())
            .unwrap();
        let matching_path =
            std::env::temp_dir().join(format!("jiuheng-facsimile-match-{}.bin", Uuid::new_v4()));
        let mismatch_path =
            std::env::temp_dir().join(format!("jiuheng-facsimile-mismatch-{}.bin", Uuid::new_v4()));
        std::fs::write(&matching_path, b"abc").unwrap();
        std::fs::write(&mismatch_path, b"abd").unwrap();

        let verified = storage
            .verify_source_facsimile_file("TEST-SOURCE-001", &matching_path)
            .unwrap();
        assert_eq!(verified.byte_length, 3);
        assert_eq!(verified.verifier_version, FACSIMILE_VERIFIER_VERSION);
        assert_eq!(
            verified.digital_checksum,
            sample_source_edition().digital_checksum
        );
        assert!(matches!(
            storage.verify_source_facsimile_file("TEST-SOURCE-001", &mismatch_path),
            Err(StorageError::RuleGateValidation(_))
        ));
        let count: u64 = storage
            .connection
            .query_row(
                "SELECT COUNT(*) FROM source_facsimile_verifications",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        std::fs::remove_file(matching_path).unwrap();
        std::fs::remove_file(mismatch_path).unwrap();
    }

    #[test]
    fn adding_byte_verification_invalidates_reviews_bound_to_checksum_only_evidence() {
        let storage = Storage::open_in_memory().unwrap();
        let rule = sample_rule();
        storage.upsert_rule(&rule).unwrap();
        storage
            .upsert_source_edition(&sample_source_edition())
            .unwrap();
        storage
            .upsert_source_passage(&sample_source_passage())
            .unwrap();
        storage
            .link_rule_source(
                &rule.rule_id,
                "TEST-PASSAGE-001",
                SourceRelation::Primary,
                "自动化测试主原文",
            )
            .unwrap();
        for kind in ReviewKind::ALL {
            storage
                .record_rule_review(
                    &rule.rule_id,
                    kind,
                    ReviewDecision::Approved,
                    "checksum-only-review-test",
                    "仅测试新增字节验证后的复核失效",
                )
                .unwrap();
        }
        let checksum_only = storage.rule_gate_report(&rule.rule_id).unwrap().unwrap();
        assert!(!checksum_only.primary_source_ready);
        assert!(checksum_only.reviews_ready);

        storage
            .import_source_facsimile_verification(&sample_facsimile_verification())
            .unwrap();
        let byte_verified = storage.rule_gate_report(&rule.rule_id).unwrap().unwrap();
        assert!(byte_verified.primary_source_ready);
        assert!(!byte_verified.reviews_ready);
        assert!(!byte_verified.may_execute());
    }

    #[test]
    fn v11_and_v12_gates_reject_a_legacy_textual_checksum() {
        let connection = Connection::open_in_memory().unwrap();
        for (version, migration) in [
            (1, MIGRATION_0001),
            (2, MIGRATION_0002),
            (3, MIGRATION_0003),
            (4, MIGRATION_0004),
            (5, MIGRATION_0005),
            (6, MIGRATION_0006),
            (7, MIGRATION_0007),
            (8, MIGRATION_0008),
            (9, MIGRATION_0009),
            (10, MIGRATION_0010),
            (11, MIGRATION_0011),
        ] {
            connection.execute_batch(migration).unwrap();
            connection
                .execute(
                    "INSERT INTO schema_migrations(version, applied_at) VALUES(?1, ?2)",
                    params![version, Utc::now().to_rfc3339()],
                )
                .unwrap();
        }
        let mut storage = Storage { connection };
        storage.initialize().unwrap();
        let rule = sample_rule();
        storage.upsert_rule(&rule).unwrap();
        let mut source = sample_source_edition();
        source.digital_checksum = "sha256:legacy-free-text".to_owned();
        storage.upsert_source_edition(&source).unwrap();
        let passage = sample_source_passage();
        storage.upsert_source_passage(&passage).unwrap();
        storage
            .link_rule_source(
                &rule.rule_id,
                &passage.passage_id,
                SourceRelation::Primary,
                "自动化测试主原文",
            )
            .unwrap();
        for kind in ReviewKind::ALL {
            storage
                .record_rule_review(
                    &rule.rule_id,
                    kind,
                    ReviewDecision::Approved,
                    "legacy-gate-test",
                    "仅测试迁移撤销，不证明真实术数规则",
                )
                .unwrap();
        }
        storage
            .upsert_rule_test_case(&sample_test_case("TEST-CASE-001", "ordinary"))
            .unwrap();
        storage
            .record_rule_test_run(&rule.rule_id, 1, 0, &serde_json::json!({"target": "test"}))
            .unwrap();
        let gate = storage.rule_gate_report(&rule.rule_id).unwrap().unwrap();
        assert!(!gate.primary_source_ready);
        assert!(!gate.may_execute());
        assert!(storage
            .released_rules(
                DivinationMethod::Liuyao,
                &AncientQuestionType::ProjectOutcome,
            )
            .unwrap()
            .is_empty());
    }

    #[test]
    fn database_rejects_rule_hits_until_the_rule_is_released() {
        let mut storage = Storage::open_in_memory().unwrap();
        let rule = sample_rule();
        storage.upsert_rule(&rule).unwrap();

        let question = sample_question();
        let classification = KeywordClassifier::default()
            .classify(question.category, &question.text)
            .unwrap();
        let fingerprint = build_event_fingerprint(&question, &classification);
        storage
            .insert_locked_question(&question, &classification, &fingerprint, None)
            .unwrap();
        let input = RuleRuntimeInputSnapshot::new(
            DivinationMethod::Liuyao,
            classification.ancient_type.clone(),
            question.clone(),
            serde_json::json!({"fixture": true}),
        )
        .unwrap()
        .to_value()
        .unwrap();
        let evaluation = compile_rule_program(&rule)
            .unwrap()
            .evaluate(&input)
            .unwrap();
        let signals = evaluation.signals.as_ref().unwrap();
        storage
            .connection
            .execute(
                "INSERT INTO engine_runs(
                    question_id, method, implementation_version,
                    applicability_status, applicability_reasons_json,
                    chart_json, signals_json, evidence_tier, started_at, completed_at
                 ) VALUES(?1, 'liuyao', 'test-1', 'applicable', '[]',
                          ?2, ?3, 'early_core_classic', ?4, ?4)",
                params![
                    question.id.to_string(),
                    serde_json::json!({"fixture": true}).to_string(),
                    serde_json::to_string(signals).unwrap(),
                    Utc::now().to_rfc3339()
                ],
            )
            .unwrap();
        let engine_run_id = storage.connection.last_insert_rowid();

        let input_json = serde_json::to_string(&input).unwrap();
        let output_json = serde_json::to_string(&evaluation.auditable_output()).unwrap();
        let trace_json = serde_json::to_string(&evaluation.trace).unwrap();

        let blocked = storage.connection.execute(
            "INSERT INTO rule_hits(
                engine_run_id, rule_id, input_snapshot_json,
                output_snapshot_json, evaluation_trace_json
             ) VALUES(?1, ?2, ?3, ?4, ?5)",
            params![
                engine_run_id,
                rule.rule_id,
                input_json,
                output_json,
                trace_json
            ],
        );
        assert!(blocked.is_err());

        add_sample_source_and_reviews(&storage, &rule.rule_id);
        storage
            .upsert_rule_test_case(&sample_test_case("TEST-CASE-001", "ordinary"))
            .unwrap();
        storage
            .record_rule_test_run(&rule.rule_id, 1, 0, &serde_json::json!({"target": "test"}))
            .unwrap();
        storage
            .connection
            .execute(
                "INSERT INTO rule_hits(
                    engine_run_id, rule_id, input_snapshot_json,
                    output_snapshot_json, evaluation_trace_json
                 ) VALUES(?1, ?2, ?3, ?4, ?5)",
                params![
                    engine_run_id,
                    rule.rule_id,
                    input_json,
                    output_json,
                    trace_json
                ],
            )
            .unwrap();
    }

    #[test]
    fn empty_runtime_rule_database_cannot_create_a_fusion_decision() {
        let mut storage = Storage::open_in_memory().unwrap();
        let question = sample_question();
        let classification = KeywordClassifier::default()
            .classify(question.category, &question.text)
            .unwrap();
        let fingerprint = build_event_fingerprint(&question, &classification);
        storage
            .insert_locked_question(&question, &classification, &fingerprint, None)
            .unwrap();

        let error = storage.fuse_and_store_question(question.id).unwrap_err();
        assert!(matches!(error, StorageError::FusionAudit(_)));
        assert!(storage.load_fusion_decision(question.id).unwrap().is_none());
        let state: String = storage
            .connection
            .query_row(
                "SELECT state FROM questions WHERE id = ?1",
                [question.id.to_string()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(state, "not_executable");
    }

    #[test]
    fn empty_rule_method_run_is_persisted_as_non_participating_and_cannot_be_repeated() {
        let mut storage = Storage::open_in_memory().unwrap();
        let question = sample_question();
        let classification = KeywordClassifier::default()
            .classify(question.category, &question.text)
            .unwrap();
        let fingerprint = build_event_fingerprint(&question, &classification);
        storage
            .insert_locked_question(&question, &classification, &fingerprint, None)
            .unwrap();

        let execution = storage
            .execute_and_store_rule_method(
                question.id,
                DivinationMethod::Liuyao,
                "jiuheng.test_chart.v1",
                &serde_json::json!({"fixture": true}),
            )
            .unwrap();
        assert_eq!(execution.method, DivinationMethod::Liuyao);
        assert_eq!(execution.status, ApplicabilityStatus::InsufficientEvidence);
        assert_eq!(execution.evaluated_rule_count, 0);
        assert!(execution.matched_rule_ids.is_empty());
        assert!(execution.result.is_none());
        assert!(execution.reasons[0].contains("没有已放行规则"));

        assert!(matches!(
            storage.execute_and_store_rule_method(
                question.id,
                DivinationMethod::Liuyao,
                "jiuheng.test_chart.v1",
                &serde_json::json!({"fixture": true}),
            ),
            Err(StorageError::RuleRuntime(_))
        ));
        assert!(matches!(
            storage.fuse_and_store_question(question.id),
            Err(StorageError::FusionAudit(_))
        ));
    }

    #[test]
    fn broad_weather_classification_cannot_execute_a_rain_rule_or_fuse() {
        for (text, event, time) in [
            ("明天会不会刮风？", "刮风", "一小时后"),
            ("明天会不会下雨？", "下雨", "近期"),
            ("明天会不会下雨或下雪？", "下雨或下雪", "一小时后"),
        ] {
            let mut storage = Storage::open_in_memory().unwrap();
            let mut question = sample_question();
            question.text = text.to_owned();
            question.target_event = event.to_owned();
            question.target_time_text = time.to_owned();
            let classification = KeywordClassifier::default()
                .classify(question.category, &question.text)
                .unwrap();
            assert_eq!(
                classification.ancient_type,
                AncientQuestionType::WeatherRain
            );
            let fingerprint = build_event_fingerprint(&question, &classification);
            storage
                .insert_locked_question(&question, &classification, &fingerprint, None)
                .unwrap();

            assert!(matches!(
                storage.execute_and_store_rule_method(
                    question.id,
                    DivinationMethod::Liuyao,
                    "test-1",
                    &serde_json::json!({"fixture": true}),
                ),
                Err(StorageError::RuleRuntime(_))
            ));
            let run_count: i64 = storage
                .connection
                .query_row(
                    "SELECT COUNT(*) FROM engine_runs WHERE question_id = ?1",
                    [question.id.to_string()],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(run_count, 0);
            assert!(matches!(
                storage.fuse_and_store_question(question.id),
                Err(StorageError::FusionAudit(_))
            ));
        }
    }

    #[test]
    fn a_rule_hit_invalidated_before_fusion_is_excluded() {
        let mut storage = Storage::open_in_memory().unwrap();
        let mut rule = release_sample_rule(&storage);
        let mut question = sample_question();
        question.category = QuestionCategory::Career;
        question.text = "这个项目能否按计划完成？".to_owned();
        question.target_event = "项目按计划完成".to_owned();
        let classification = KeywordClassifier::default()
            .classify(question.category, &question.text)
            .unwrap();
        let fingerprint = build_event_fingerprint(&question, &classification);
        storage
            .insert_locked_question(&question, &classification, &fingerprint, None)
            .unwrap();
        let execution = storage
            .execute_and_store_rule_method(
                question.id,
                DivinationMethod::Liuyao,
                "test-1",
                &serde_json::json!({"fixture": true}),
            )
            .unwrap();
        assert_eq!(execution.matched_rule_ids, vec![rule.rule_id.clone()]);

        rule.result_expression = serde_json::json!({
            "schema_version": 1,
            "signals": {"trend": 20, "outcome_direction": 20}
        })
        .to_string();
        storage.upsert_rule(&rule).unwrap();
        assert!(!storage
            .rule_gate_report(&rule.rule_id)
            .unwrap()
            .unwrap()
            .may_execute());

        assert!(matches!(
            storage.fuse_and_store_question(question.id),
            Err(StorageError::FusionAudit(_))
        ));
        assert!(storage.load_fusion_decision(question.id).unwrap().is_none());
    }

    #[test]
    fn fusion_uses_only_currently_released_rule_hits_and_persists_an_immutable_audit() {
        let mut storage = Storage::open_in_memory().unwrap();
        let rule = release_sample_rule(&storage);
        let mut question = sample_question();
        question.category = QuestionCategory::Career;
        question.subcategory = "项目成败".to_owned();
        question.text = "这个项目能否按计划完成？".to_owned();
        question.target_event = "项目按计划完成".to_owned();
        question.result_terms = vec!["项目".to_owned(), "完成".to_owned()];
        let classification = KeywordClassifier::default()
            .classify(question.category, &question.text)
            .unwrap();
        assert_eq!(
            classification.ancient_type,
            AncientQuestionType::ProjectOutcome
        );
        let fingerprint = build_event_fingerprint(&question, &classification);
        storage
            .insert_locked_question(&question, &classification, &fingerprint, None)
            .unwrap();

        let execution = storage
            .execute_and_store_rule_method(
                question.id,
                DivinationMethod::Liuyao,
                "test-1",
                &serde_json::json!({"fixture": true}),
            )
            .unwrap();
        assert_eq!(execution.status, ApplicabilityStatus::Applicable);
        assert_eq!(execution.evaluated_rule_count, 1);
        assert_eq!(execution.matched_rule_ids, vec![rule.rule_id.clone()]);
        assert_eq!(execution.result.as_ref().unwrap().signals.trend, Some(60));
        let engine_run_id: i64 = storage
            .connection
            .query_row(
                "SELECT id FROM engine_runs WHERE question_id = ?1 AND method = 'liuyao'",
                [question.id.to_string()],
                |row| row.get(0),
            )
            .unwrap();
        assert!(storage
            .connection
            .execute(
                "UPDATE engine_runs SET signals_json = '{}' WHERE id = ?1",
                [engine_run_id],
            )
            .is_err());
        assert!(storage
            .connection
            .execute(
                "UPDATE rule_hits SET output_snapshot_json = '{}' WHERE engine_run_id = ?1",
                [engine_run_id],
            )
            .is_err());

        let decision = storage.fuse_and_store_question(question.id).unwrap();
        assert_eq!(decision.participating_methods, 1);
        assert_eq!(decision.raw_equal_weight_score, 60);
        assert_eq!(decision.timing.zh_name(), "1个月内");
        assert_eq!(decision.action_advice.status, ActionAdviceStatus::Available);
        assert_eq!(decision.action_advice.items.len(), 1);
        assert_eq!(
            decision.action_advice.items[0].supporting_rule_ids,
            vec![rule.rule_id.clone()]
        );
        assert_eq!(
            storage.load_fusion_decision(question.id).unwrap(),
            Some(decision.clone())
        );
        assert_eq!(
            storage.fuse_and_store_question(question.id).unwrap(),
            decision
        );
        let detail = storage.load_question_detail(question.id).unwrap().unwrap();
        assert_eq!(detail.state, "completed");
        assert_eq!(detail.final_conclusion.as_deref(), Some("整体趋向明显正面"));
        assert_eq!(detail.final_timing.as_deref(), Some("1个月内"));
        assert_eq!(detail.final_action_advice, decision.action_advice);
        assert_eq!(detail.engine_runs.len(), 1);
        assert_eq!(detail.engine_runs[0].method, DivinationMethod::Liuyao);
        assert_eq!(detail.engine_runs[0].rule_hits.len(), 1);
        assert_eq!(detail.engine_runs[0].rule_hits[0].rule_id, rule.rule_id);
        assert_eq!(detail.fusion_decision, Some(decision));
        assert!(storage
            .connection
            .execute(
                "UPDATE fusion_decisions SET decision_json = '{}' WHERE question_id = ?1",
                [question.id.to_string()],
            )
            .is_err());
        storage
            .connection
            .execute(
                "UPDATE questions SET final_action_advice_json = ?2 WHERE id = ?1",
                params![
                    question.id.to_string(),
                    serde_json::to_string(&ActionAdviceAudit::default()).unwrap()
                ],
            )
            .unwrap();
        assert!(matches!(
            storage.load_question_detail(question.id),
            Err(StorageError::FusionAudit(_))
        ));
    }

    #[test]
    fn fusion_re_evaluates_rule_hits_after_database_trigger_is_bypassed() {
        let mut storage = Storage::open_in_memory().unwrap();
        let rule = release_sample_rule(&storage);
        let mut question = sample_question();
        question.category = QuestionCategory::Career;
        question.subcategory = "项目成败".to_owned();
        question.text = "这个项目能否按计划完成？".to_owned();
        question.target_event = "项目按计划完成".to_owned();
        let classification = KeywordClassifier::default()
            .classify(question.category, &question.text)
            .unwrap();
        let fingerprint = build_event_fingerprint(&question, &classification);
        storage
            .insert_locked_question(&question, &classification, &fingerprint, None)
            .unwrap();
        storage
            .execute_and_store_rule_method(
                question.id,
                DivinationMethod::Liuyao,
                "test-1",
                &serde_json::json!({"fixture": true}),
            )
            .unwrap();

        storage
            .connection
            .execute_batch("DROP TRIGGER prevent_rule_hit_update")
            .unwrap();
        storage
            .connection
            .execute(
                "UPDATE rule_hits
                 SET output_snapshot_json = json_set(
                     output_snapshot_json, '$.signals.trend', 59
                 )
                 WHERE rule_id = ?1",
                [rule.rule_id],
            )
            .unwrap();
        let error = storage.fuse_and_store_question(question.id).unwrap_err();
        assert!(matches!(error, StorageError::FusionAudit(_)));
        assert!(storage.load_fusion_decision(question.id).unwrap().is_none());
    }

    #[test]
    fn fusion_rejects_an_incomplete_subset_of_matching_rules() {
        let mut storage = Storage::open_in_memory().unwrap();
        let first = release_sample_rule(&storage);
        let mut second = sample_rule();
        second.rule_id = "TEST-ONLY-LIUYAO-002".to_owned();
        second.result_expression = serde_json::json!({
            "schema_version": 1,
            "signals": {"support": 40}
        })
        .to_string();
        second.action_key = Some("project.preserve_fallback".to_owned());
        storage.upsert_rule(&second).unwrap();
        add_sample_source_and_reviews(&storage, &second.rule_id);
        let input = serde_json::json!({"chart": {"fixture": true}});
        let expected_output = serde_json::to_value(
            compile_rule_program(&second)
                .unwrap()
                .evaluate(&input)
                .unwrap()
                .auditable_output(),
        )
        .unwrap();
        storage
            .upsert_rule_test_case(&RuleTestCaseRecord {
                test_case_id: "TEST-CASE-SECOND".to_owned(),
                rule_id: second.rule_id.clone(),
                name: "第二条合并规则".to_owned(),
                input,
                expected_output,
                boundary_class: "ordinary".to_owned(),
                source_basis: "TEST-FIXTURE-ONLY".to_owned(),
                enabled: true,
            })
            .unwrap();
        storage
            .run_rule_test_suite(&second.rule_id, &serde_json::json!({"target": "test"}))
            .unwrap();

        let mut question = sample_question();
        question.category = QuestionCategory::Career;
        question.subcategory = "项目成败".to_owned();
        question.text = "这个项目能否按计划完成？".to_owned();
        question.target_event = "项目按计划完成".to_owned();
        let classification = KeywordClassifier::default()
            .classify(question.category, &question.text)
            .unwrap();
        let fingerprint = build_event_fingerprint(&question, &classification);
        storage
            .insert_locked_question(&question, &classification, &fingerprint, None)
            .unwrap();
        let execution = storage
            .execute_and_store_rule_method(
                question.id,
                DivinationMethod::Liuyao,
                "test-1",
                &serde_json::json!({"fixture": true}),
            )
            .unwrap();
        assert_eq!(
            execution.matched_rule_ids,
            vec![first.rule_id, second.rule_id.clone()]
        );
        assert_eq!(execution.result.unwrap().signals.support, Some(40));

        storage
            .connection
            .execute_batch("DROP TRIGGER prevent_rule_hit_delete")
            .unwrap();
        storage
            .connection
            .execute("DELETE FROM rule_hits WHERE rule_id = ?1", [second.rule_id])
            .unwrap();
        let error = storage.fuse_and_store_question(question.id).unwrap_err();
        assert!(matches!(error, StorageError::FusionAudit(_)));
        assert!(storage.load_fusion_decision(question.id).unwrap().is_none());
    }

    #[test]
    fn detects_same_event_after_first_lock() {
        let mut storage = Storage::open_in_memory().unwrap();
        let question = sample_question();
        let classification = KeywordClassifier::default()
            .classify(question.category, &question.text)
            .unwrap();
        let fingerprint = build_event_fingerprint(&question, &classification);
        assert!(storage
            .find_duplicate_event(&fingerprint)
            .unwrap()
            .is_none());
        storage
            .insert_locked_question(&question, &classification, &fingerprint, None)
            .unwrap();
        assert!(storage
            .connection
            .execute(
                "UPDATE questions SET normalized_terms_json = '[]' WHERE id = ?1",
                [question.id.to_string()],
            )
            .is_err());
        let duplicate = storage.find_duplicate_event(&fingerprint).unwrap().unwrap();
        assert_eq!(duplicate.question_id, question.id);
        assert!(duplicate.exact_digest);
        assert_eq!(duplicate.score_percent, 100);
    }

    #[test]
    fn detects_paraphrased_event_without_rewriting_stored_fingerprint() {
        let mut storage = Storage::open_in_memory().unwrap();
        let mut first = sample_question();
        first.target_event = "南山区一小时后是否下雨".to_owned();
        let classification = KeywordClassifier::default()
            .classify(first.category, &first.text)
            .unwrap();
        let fingerprint = build_event_fingerprint(&first, &classification);
        storage
            .insert_locked_question(&first, &classification, &fingerprint, None)
            .unwrap();

        let mut paraphrase = first.clone();
        paraphrase.id = Uuid::new_v4();
        paraphrase.text = "南山区一小时后有没有雨？".to_owned();
        paraphrase.target_event = "南山区一小时后有没有雨".to_owned();
        assert_ne!(
            fingerprint.digest_hex,
            build_event_fingerprint(&paraphrase, &classification).digest_hex
        );

        let duplicate = storage
            .find_duplicate_event(&build_event_fingerprint(&paraphrase, &classification))
            .unwrap()
            .unwrap();
        assert_eq!(duplicate.question_id, first.id);
        assert!(!duplicate.exact_digest);
        assert!(duplicate.score_percent >= 75);
        assert_eq!(duplicate.algorithm_version, EVENT_MATCH_ALGORITHM_VERSION);
    }

    #[test]
    fn duplicate_rules_do_not_block_a_different_target_time() {
        let mut storage = Storage::open_in_memory().unwrap();
        let first = sample_question();
        let classification = KeywordClassifier::default()
            .classify(first.category, &first.text)
            .unwrap();
        let fingerprint = build_event_fingerprint(&first, &classification);
        storage
            .insert_locked_question(&first, &classification, &fingerprint, None)
            .unwrap();

        let mut later = first.clone();
        later.id = Uuid::new_v4();
        later.target_time_text = "七天后".to_owned();
        later.text = "七天后会下雨吗？".to_owned();
        assert!(storage
            .find_duplicate_event(&build_event_fingerprint(&later, &classification))
            .unwrap()
            .is_none());
    }

    fn entropy_event(sequence: u8) -> divination_core::EntropyEvent {
        divination_core::EntropyEvent {
            sequence,
            wall_clock_unix_nanos: 1_788_068_725_000_000_000 + sequence as i64,
            monotonic_nanos: 10_000 + sequence as u64,
            press_duration_nanos: 100_000 + sequence as u64,
            delta_since_previous_nanos: 200_000 + sequence as u64,
            click_x_micropoints: 100 + sequence as i32,
            click_y_micropoints: 200 + sequence as i32,
            pointer_path: Vec::new(),
        }
    }

    #[test]
    fn persists_and_verifies_liuyao_artifact() {
        let mut storage = Storage::open_in_memory().unwrap();
        let question = sample_question();
        let classification = KeywordClassifier::default()
            .classify(question.category, &question.text)
            .unwrap();
        let fingerprint = build_event_fingerprint(&question, &classification);
        storage
            .insert_locked_question(&question, &classification, &fingerprint, None)
            .unwrap();

        let mut transcript = LiuyaoTranscript::new(question.id, [0x42; 32]);
        for sequence in 0..6 {
            transcript.push_event(entropy_event(sequence)).unwrap();
        }
        let cast = transcript.finish().unwrap();
        storage
            .save_liuyao_artifact(question.id, &transcript, &cast)
            .unwrap();
        let loaded = storage.load_liuyao_artifact(question.id).unwrap().unwrap();
        assert_eq!(loaded.transcript, transcript);
        assert_eq!(loaded.cast, cast);
    }

    #[test]
    fn persists_and_verifies_yarrow_artifact() {
        let mut storage = Storage::open_in_memory().unwrap();
        let question = sample_question();
        let classification = KeywordClassifier::default()
            .classify(question.category, &question.text)
            .unwrap();
        let fingerprint = build_event_fingerprint(&question, &classification);
        storage
            .insert_locked_question(&question, &classification, &fingerprint, None)
            .unwrap();

        let mut session = divination_core::YarrowSession::default();
        while !session.is_complete() {
            session.apply_split(session.active_stalks() / 2).unwrap();
        }
        let transcript = session.transcript().clone();
        let cast = session.finish().unwrap();
        storage
            .save_yarrow_artifact(question.id, &transcript, &cast)
            .unwrap();
        let loaded = storage.load_yarrow_artifact(question.id).unwrap().unwrap();
        assert_eq!(loaded.transcript, transcript);
        assert_eq!(loaded.cast, cast);
    }

    #[test]
    fn history_returns_locked_input_revision_chain_and_replay_proofs() {
        let mut storage = Storage::open_in_memory().unwrap();
        let first = sample_question();
        let classification = KeywordClassifier::default()
            .classify(first.category, &first.text)
            .unwrap();
        let fingerprint = build_event_fingerprint(&first, &classification);
        storage
            .insert_locked_question(&first, &classification, &fingerprint, None)
            .unwrap();

        let mut liuyao = LiuyaoTranscript::new(first.id, [0x33; 32]);
        for sequence in 0..6 {
            liuyao.push_event(entropy_event(sequence)).unwrap();
        }
        let liuyao_cast = liuyao.finish().unwrap();
        storage
            .save_liuyao_artifact(first.id, &liuyao, &liuyao_cast)
            .unwrap();

        let mut second = first.clone();
        second.id = Uuid::new_v4();
        second.locked_at_bjt += chrono::Duration::minutes(20);
        let revision = EventRevision {
            id: Uuid::new_v4(),
            prior_question_id: first.id,
            change_description: "项目交付日期发生实质变化".to_owned(),
            created_at_bjt: second.locked_at_bjt,
            match_report: Some(divination_core::EventMatchReport {
                algorithm_version: EVENT_MATCH_ALGORITHM_VERSION.to_owned(),
                same_event: true,
                exact_digest: true,
                score_percent: 100,
                reasons: vec!["事件指纹完全一致".to_owned()],
            }),
        };
        storage
            .insert_locked_question(&second, &classification, &fingerprint, Some(&revision))
            .unwrap();

        let history = storage.recent_questions(20).unwrap();
        assert_eq!(history.len(), 2);
        let first_summary = history
            .iter()
            .find(|summary| summary.question.id == first.id)
            .unwrap();
        assert!(first_summary.has_liuyao);
        assert!(!first_summary.has_yarrow);
        let second_summary = history
            .iter()
            .find(|summary| summary.question.id == second.id)
            .unwrap();
        assert!(second_summary.is_event_revision);

        let first_detail = storage.load_question_detail(first.id).unwrap().unwrap();
        assert_eq!(first_detail.question.id, first.id);
        assert_eq!(first_detail.question.text, first.text);
        assert_eq!(first_detail.later_revision_ids, vec![second.id]);
        let second_detail = storage.load_question_detail(second.id).unwrap().unwrap();
        let revision_detail = second_detail.revision.unwrap();
        assert_eq!(
            revision_detail.change_description,
            "项目交付日期发生实质变化"
        );
        assert_eq!(revision_detail.match_score_percent, Some(100));
        assert_eq!(revision_detail.match_exact_digest, Some(true));
        assert_eq!(
            revision_detail.match_algorithm_version.as_deref(),
            Some(EVENT_MATCH_ALGORITHM_VERSION)
        );

        let audit = storage.audit_history_replay(first.id).unwrap();
        let proof = audit.liuyao.unwrap();
        let expected =
            hexagram_transformation(liuyao_cast.primary_bits, liuyao_cast.changed_bits).unwrap();
        assert_eq!(proof.operation_count, 6);
        assert_eq!(proof.primary_hexagram, expected.primary);
        assert_eq!(proof.changed_hexagram, expected.changed);
        assert!(!proof.primary_hexagram.interpretation_eligible);
        let eight_palace = proof
            .liuyao_eight_palace_audit
            .expect("Liuyao replay includes eight-palace host/response audit");
        assert_eq!(
            eight_palace.hexagram.bits_bottom_up,
            liuyao_cast.primary_bits
        );
        assert!(!eight_palace.na_jia_assigned);
        assert!(!eight_palace.interpretation_eligible);
        assert!(!eight_palace.rule_eligible);
        let najia = proof
            .liuyao_najia_audit
            .expect("Liuyao replay includes received Na Jia audit");
        assert_eq!(
            najia.eight_palace.hexagram.bits_bottom_up,
            liuyao_cast.primary_bits
        );
        assert_eq!(najia.lines_bottom_up.len(), 6);
        assert!(najia.na_jia_stems_assigned);
        assert!(najia.na_jia_branches_assigned);
        assert!(!najia.six_relatives_assigned);
        assert!(!najia.interpretation_eligible);
        assert!(!najia.rule_eligible);
        let relatives = proof
            .liuyao_six_relatives_audit
            .expect("Liuyao replay includes received six-relatives audit");
        assert_eq!(
            relatives.najia.eight_palace.hexagram.bits_bottom_up,
            liuyao_cast.primary_bits
        );
        assert_eq!(relatives.lines_bottom_up.len(), 6);
        assert!(relatives.six_relatives_assigned);
        assert!(!relatives.six_spirits_assigned);
        assert!(!relatives.useful_god_selected);
        assert!(!relatives.strength_assessed);
        assert!(!relatives.interpretation_eligible);
        assert!(!relatives.rule_eligible);
        let spirits = proof
            .liuyao_six_spirits_audit
            .expect("Liuyao replay includes received six-spirits audit when day stem exists");
        assert_eq!(
            spirits
                .six_relatives
                .najia
                .eight_palace
                .hexagram
                .bits_bottom_up,
            liuyao_cast.primary_bits
        );
        assert_eq!(spirits.lines_bottom_up.len(), 6);
        assert_eq!(spirits.day_convention, "beijing_civil_midnight");
        assert!(spirits.six_spirits_assigned);
        assert!(!spirits.hidden_spirits_assigned);
        assert!(!spirits.useful_god_selected);
        assert!(!spirits.strength_assessed);
        assert!(!spirits.interpretation_eligible);
        assert!(!spirits.rule_eligible);
        let hidden = proof
            .liuyao_hidden_spirits_audit
            .expect("Liuyao replay includes received hidden-spirit audit");
        assert_eq!(
            hidden.current.najia.eight_palace.hexagram.bits_bottom_up,
            liuyao_cast.primary_bits
        );
        assert!(hidden.hidden_spirits_assigned);
        assert!(!hidden.flying_hidden_interactions_assessed);
        assert!(!hidden.useful_god_selected);
        assert!(!hidden.strength_assessed);
        assert!(!hidden.interpretation_eligible);
        assert!(!hidden.rule_eligible);
        let calendar_moving = proof
            .liuyao_calendar_moving_audit
            .expect("Liuyao replay includes locked month/day and moving-line audit");
        assert_eq!(
            calendar_moving.primary_hexagram.bits_bottom_up,
            liuyao_cast.primary_bits
        );
        assert_eq!(
            calendar_moving.changed_hexagram.bits_bottom_up,
            liuyao_cast.changed_bits
        );
        assert_eq!(
            calendar_moving.moving_line_mask_bottom_up,
            liuyao_cast.primary_bits ^ liuyao_cast.changed_bits
        );
        assert!(calendar_moving.month_build_identity_assigned);
        assert!(calendar_moving.day_identity_assigned);
        assert!(calendar_moving.moving_transformation_verified);
        assert!(!calendar_moving.month_day_line_interactions_assessed);
        assert!(!calendar_moving.strength_assessed);
        assert!(!calendar_moving.interpretation_eligible);
        assert!(!calendar_moving.rule_eligible);
        let calendar_relations = proof
            .liuyao_calendar_relations_audit
            .expect("Liuyao replay includes xun-void and month/day branch observations");
        assert_eq!(calendar_relations.primary_observations_bottom_up.len(), 6);
        assert_eq!(
            calendar_relations
                .transformed_moving_observations_bottom_up
                .len(),
            liuyao_cast
                .lines
                .iter()
                .filter(|line| line.line_type.is_moving())
                .count()
        );
        assert_eq!(
            calendar_relations.xun_start.stem,
            divination_core::HeavenlyStem::Jia
        );
        assert_eq!(
            calendar_relations.month_build_five_element,
            calendar_relations.month_build_branch.five_element()
        );
        assert_eq!(
            calendar_relations.day_branch_five_element,
            calendar_relations.day_branch.five_element()
        );
        for observation in calendar_relations
            .primary_observations_bottom_up
            .iter()
            .chain(
                calendar_relations
                    .transformed_moving_observations_bottom_up
                    .iter(),
            )
        {
            assert_eq!(
                observation.month_build_five_element_relation,
                divination_core::liuyao_directional_five_element_relation(
                    calendar_relations.month_build_five_element,
                    observation.branch_five_element,
                )
            );
            assert_eq!(
                observation.day_branch_five_element_relation,
                divination_core::liuyao_directional_five_element_relation(
                    calendar_relations.day_branch_five_element,
                    observation.branch_five_element,
                )
            );
        }
        assert!(calendar_relations.raw_relations_identified);
        assert!(!calendar_relations.relation_effects_assessed);
        assert!(!calendar_relations.useful_god_selected);
        assert!(!calendar_relations.strength_assessed);
        assert!(!calendar_relations.interpretation_eligible);
        assert!(!calendar_relations.rule_eligible);
        let branch_pairs = proof
            .liuyao_branch_pair_relations_audit
            .expect("Liuyao replay includes six-combination and six-opposition raw pairs");
        assert_eq!(branch_pairs.primary_visible_pair_observations.len(), 15);
        assert_eq!(
            branch_pairs.moving_transformation_observations.len(),
            liuyao_cast
                .lines
                .iter()
                .filter(|line| line.line_type.is_moving())
                .count()
        );
        assert_eq!(
            branch_pairs.flying_hidden_observations.len(),
            branch_pairs.hidden_spirits.hidden_lines_bottom_up.len()
        );
        assert!(branch_pairs.raw_branch_pairs_compared);
        assert!(!branch_pairs.whole_hexagram_six_combination_classified);
        assert!(!branch_pairs.whole_hexagram_six_opposition_classified);
        assert!(!branch_pairs.relation_effects_assessed);
        assert!(!branch_pairs.interpretation_eligible);
        assert!(!branch_pairs.rule_eligible);
        let progress_retreat = proof
            .liuyao_progress_retreat_audit
            .expect("Liuyao replay includes progress/retreat raw moving-line identities");
        assert_eq!(
            progress_retreat.moving_observations_bottom_up.len(),
            liuyao_cast
                .lines
                .iter()
                .filter(|line| line.line_type.is_moving())
                .count()
        );
        assert_eq!(progress_retreat.explicit_progress_pair_count, 7);
        assert_eq!(progress_retreat.explicit_retreat_pair_count, 7);
        assert!(progress_retreat.raw_identity_assigned);
        assert!(progress_retreat.omitted_earth_cycle_pairs_left_unclassified);
        assert!(!progress_retreat.progression_or_retreat_effect_assessed);
        assert!(!progress_retreat.strength_assessed);
        assert!(!progress_retreat.timing_assessed);
        assert!(!progress_retreat.interpretation_eligible);
        assert!(!progress_retreat.rule_eligible);
        let fanyin_fuyin = proof
            .liuyao_fanyin_fuyin_audit
            .expect("Liuyao replay includes raw Fanyin/Fuyin prerequisites");
        assert_eq!(
            fanyin_fuyin.moving_line_observations_bottom_up.len(),
            liuyao_cast
                .lines
                .iter()
                .filter(|line| line.line_type.is_moving())
                .count()
        );
        assert_eq!(fanyin_fuyin.inner_scope.line_numbers_bottom_up, [1, 2, 3]);
        assert_eq!(fanyin_fuyin.outer_scope.line_numbers_bottom_up, [4, 5, 6]);
        assert!(fanyin_fuyin.raw_prerequisites_identified);
        assert!(!fanyin_fuyin.whole_hexagram_fanyin_classified);
        assert!(!fanyin_fuyin.whole_hexagram_fuyin_classified);
        assert!(!fanyin_fuyin.relation_effects_assessed);
        assert!(!fanyin_fuyin.strength_assessed);
        assert!(!fanyin_fuyin.timing_assessed);
        assert!(!fanyin_fuyin.interpretation_eligible);
        assert!(!fanyin_fuyin.rule_eligible);
        let sanhe = proof
            .liuyao_sanhe_audit
            .expect("Liuyao replay includes raw Sanhe member occurrences");
        assert_eq!(sanhe.groups.len(), 4);
        assert_eq!(sanhe.canonical_group_count, 4);
        assert_eq!(
            sanhe
                .groups
                .iter()
                .flat_map(|group| &group.occurrences)
                .count(),
            8 + liuyao_cast
                .lines
                .iter()
                .filter(|line| line.line_type.is_moving())
                .count()
        );
        assert!(sanhe.raw_occurrence_inventory_built);
        assert!(!sanhe.source_scope_activation_requirements_classified);
        assert!(!sanhe.any_sanhe_formation_classified);
        assert!(!sanhe.combination_effects_assessed);
        assert!(!sanhe.strength_assessed);
        assert!(!sanhe.timing_assessed);
        assert!(!sanhe.interpretation_eligible);
        assert!(!sanhe.rule_eligible);
        assert!(audit.yarrow.is_none());
    }

    #[test]
    fn history_replay_rejects_tampered_database_material() {
        let mut storage = Storage::open_in_memory().unwrap();
        let question = sample_question();
        let classification = KeywordClassifier::default()
            .classify(question.category, &question.text)
            .unwrap();
        let fingerprint = build_event_fingerprint(&question, &classification);
        storage
            .insert_locked_question(&question, &classification, &fingerprint, None)
            .unwrap();

        let mut transcript = LiuyaoTranscript::new(question.id, [0x91; 32]);
        for sequence in 0..6 {
            transcript.push_event(entropy_event(sequence)).unwrap();
        }
        let cast = transcript.finish().unwrap();
        storage
            .save_liuyao_artifact(question.id, &transcript, &cast)
            .unwrap();
        storage
            .connection
            .execute(
                "UPDATE random_artifacts SET seed_hex = 'tampered' WHERE question_id = ?1",
                [question.id.to_string()],
            )
            .unwrap();

        assert!(matches!(
            storage.audit_history_replay(question.id),
            Err(StorageError::ArtifactIntegrity(_))
        ));
    }

    #[test]
    fn storage_rejects_ambiguous_other_even_with_caller_supplied_classification() {
        let mut storage = Storage::open_in_memory().unwrap();
        let mut question = sample_question();
        question.category = QuestionCategory::Other;
        question.text = "健康 APP 上线后怎么办？".to_owned();
        let forged = KeywordClassifier::default()
            .classify(QuestionCategory::Career, &question.text)
            .unwrap();
        let fingerprint = build_event_fingerprint(&question, &forged);
        assert!(matches!(
            storage.insert_locked_question(&question, &forged, &fingerprint, None),
            Err(StorageError::ParameterValidation(_))
        ));
        assert_eq!(storage.question_count().unwrap(), 0);
    }

    #[test]
    fn storage_keeps_a_uniquely_mapped_other_health_type() {
        let mut storage = Storage::open_in_memory().unwrap();
        let mut question = sample_question();
        question.category = QuestionCategory::Other;
        question.text = "健康情况会如何？".to_owned();
        question.target_event = "健康情况变化".to_owned();
        let classification = KeywordClassifier::default()
            .classify_detailed(question.category, &question.text)
            .unwrap();
        let fingerprint = build_event_fingerprint(&question, &classification);
        storage
            .insert_locked_question(&question, &classification, &fingerprint, None)
            .unwrap();
        let detail = storage.load_question_detail(question.id).unwrap().unwrap();
        assert_eq!(
            detail.ancient_question_type,
            AncientQuestionType::HealthTrendReference
        );
    }

    #[test]
    fn storage_rejects_an_embedded_app_alias_even_if_caller_forges_a_project_type() {
        let mut storage = Storage::open_in_memory().unwrap();
        let mut question = sample_question();
        question.category = QuestionCategory::Other;
        question.text = "happy".to_owned();
        let forged = KeywordClassifier::default()
            .classify(QuestionCategory::Career, &question.text)
            .unwrap();
        let fingerprint = build_event_fingerprint(&question, &forged);
        assert!(matches!(
            storage.insert_locked_question(&question, &forged, &fingerprint, None),
            Err(StorageError::ParameterValidation(_))
        ));
        assert_eq!(storage.question_count().unwrap(), 0);
    }

    #[test]
    fn storage_keeps_health_mapping_when_happy_is_not_an_app_alias() {
        let mut storage = Storage::open_in_memory().unwrap();
        let mut question = sample_question();
        question.category = QuestionCategory::Other;
        question.text = "健康 happy".to_owned();
        question.target_event = "健康情况变化".to_owned();
        let classification = KeywordClassifier::default()
            .classify_detailed(question.category, &question.text)
            .unwrap();
        let fingerprint = build_event_fingerprint(&question, &classification);
        storage
            .insert_locked_question(&question, &classification, &fingerprint, None)
            .unwrap();
        let detail = storage.load_question_detail(question.id).unwrap().unwrap();
        assert_eq!(
            detail.ancient_question_type,
            AncientQuestionType::HealthTrendReference
        );
    }

    #[test]
    fn storage_rejects_a_forged_event_fingerprint() {
        let mut storage = Storage::open_in_memory().unwrap();
        let question = sample_question();
        let classification = KeywordClassifier::default()
            .classify(question.category, &question.text)
            .unwrap();
        let mut fingerprint = build_event_fingerprint(&question, &classification);
        fingerprint.digest_hex = "forged".to_owned();
        assert!(matches!(
            storage.insert_locked_question(&question, &classification, &fingerprint, None),
            Err(StorageError::ParameterValidation(_))
        ));
        assert_eq!(storage.question_count().unwrap(), 0);
    }

    #[test]
    fn storage_rejects_a_classification_from_the_wrong_fixed_category() {
        let mut storage = Storage::open_in_memory().unwrap();
        let question = sample_question();
        let forged = KeywordClassifier::default()
            .classify(QuestionCategory::Career, &question.text)
            .unwrap();
        let fingerprint = build_event_fingerprint(&question, &forged);
        assert!(matches!(
            storage.insert_locked_question(&question, &forged, &fingerprint, None),
            Err(StorageError::ParameterValidation(_))
        ));
        assert_eq!(storage.question_count().unwrap(), 0);
    }
}
