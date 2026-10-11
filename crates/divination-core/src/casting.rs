use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub const LIUYAO_ALGORITHM_VERSION: &str = "jiuheng.liuyao.three_coins.v1";
pub const YARROW_ALGORITHM_VERSION_V1: &str = "jiuheng.yarrow.eighteen_changes.v1";
pub const YARROW_ALGORITHM_VERSION: &str = "jiuheng.yarrow.eighteen_changes.v2";
pub const YARROW_CONVENTION_V1_IMPLICIT_RIGHT: &str =
    "legacy.implicit_right_pile.all_three_changes.v1";
pub const YARROW_CONVENTION_TANG_LEFT: &str =
    "zhouyi_zhengyi.tang.heaven_left.hang_each_change.reconstruction.v1";
pub const YARROW_PROCEDURE_STATUS_ZH: &str = "研究重建，不参与裁决";
pub const YARROW_TOTAL_STALKS: u8 = 50;
pub const YARROW_ACTIVE_STALKS: u8 = 49;
pub const YARROW_HANGING_STALKS: u8 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PointerSample {
    pub elapsed_nanos: u64,
    pub x_micropoints: i32,
    pub y_micropoints: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntropyEvent {
    pub sequence: u8,
    pub wall_clock_unix_nanos: i64,
    pub monotonic_nanos: u64,
    pub press_duration_nanos: u64,
    pub delta_since_previous_nanos: u64,
    pub click_x_micropoints: i32,
    pub click_y_micropoints: i32,
    pub pointer_path: Vec<PointerSample>,
}

impl EntropyEvent {
    fn append_canonical_bytes(&self, output: &mut Vec<u8>) -> Result<(), CastingError> {
        if self.pointer_path.len() > 65_535 {
            return Err(CastingError::EntropyPathTooLong);
        }
        output.push(self.sequence);
        output.extend_from_slice(&self.wall_clock_unix_nanos.to_le_bytes());
        output.extend_from_slice(&self.monotonic_nanos.to_le_bytes());
        output.extend_from_slice(&self.press_duration_nanos.to_le_bytes());
        output.extend_from_slice(&self.delta_since_previous_nanos.to_le_bytes());
        output.extend_from_slice(&self.click_x_micropoints.to_le_bytes());
        output.extend_from_slice(&self.click_y_micropoints.to_le_bytes());
        output.extend_from_slice(&(self.pointer_path.len() as u32).to_le_bytes());
        for sample in &self.pointer_path {
            output.extend_from_slice(&sample.elapsed_nanos.to_le_bytes());
            output.extend_from_slice(&sample.x_micropoints.to_le_bytes());
            output.extend_from_slice(&sample.y_micropoints.to_le_bytes());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoTranscript {
    pub algorithm_version: String,
    pub question_id: Uuid,
    pub session_nonce_hex: String,
    pub events: Vec<EntropyEvent>,
}

impl LiuyaoTranscript {
    pub fn new(question_id: Uuid, session_nonce: [u8; 32]) -> Self {
        Self {
            algorithm_version: LIUYAO_ALGORITHM_VERSION.to_owned(),
            question_id,
            session_nonce_hex: encode_hex(&session_nonce),
            events: Vec::with_capacity(6),
        }
    }

    pub fn with_random_nonce(question_id: Uuid) -> Self {
        Self::new(question_id, rand::random())
    }

    pub fn push_event(&mut self, event: EntropyEvent) -> Result<LineCast, CastingError> {
        if self.algorithm_version != LIUYAO_ALGORITHM_VERSION {
            return Err(CastingError::UnsupportedAlgorithmVersion(
                self.algorithm_version.clone(),
            ));
        }
        if self.events.len() >= 6 {
            return Err(CastingError::LiuyaoAlreadyComplete);
        }
        let expected = self.events.len() as u8;
        if event.sequence != expected {
            return Err(CastingError::InvalidEntropySequence {
                expected,
                actual: event.sequence,
            });
        }
        let nonce = decode_32_byte_hex(&self.session_nonce_hex)?;
        let line = generate_coin_line(self.question_id, &nonce, &event)?;
        self.events.push(event);
        Ok(line)
    }

    pub fn is_complete(&self) -> bool {
        self.events.len() == 6
    }

    pub fn finish(&self) -> Result<LiuyaoCast, CastingError> {
        replay_liuyao(self)
    }
}

pub fn replay_liuyao(transcript: &LiuyaoTranscript) -> Result<LiuyaoCast, CastingError> {
    if transcript.algorithm_version != LIUYAO_ALGORITHM_VERSION {
        return Err(CastingError::UnsupportedAlgorithmVersion(
            transcript.algorithm_version.clone(),
        ));
    }
    if transcript.events.len() != 6 {
        return Err(CastingError::LiuyaoNeedsSixEvents(transcript.events.len()));
    }
    let nonce = decode_32_byte_hex(&transcript.session_nonce_hex)?;
    let mut lines = Vec::with_capacity(6);
    let mut transcript_bytes = Vec::new();
    transcript_bytes.extend_from_slice(LIUYAO_ALGORITHM_VERSION.as_bytes());
    transcript_bytes.extend_from_slice(transcript.question_id.as_bytes());
    transcript_bytes.extend_from_slice(&nonce);

    for (index, event) in transcript.events.iter().enumerate() {
        if event.sequence != index as u8 {
            return Err(CastingError::InvalidEntropySequence {
                expected: index as u8,
                actual: event.sequence,
            });
        }
        event.append_canonical_bytes(&mut transcript_bytes)?;
        lines.push(generate_coin_line(transcript.question_id, &nonce, event)?);
    }

    let transcript_hash = blake3::hash(&transcript_bytes).to_hex().to_string();
    let mut master_hasher = blake3::Hasher::new();
    master_hasher.update(b"jiuheng/liuyao/master-seed/v1\0");
    master_hasher.update(&transcript_bytes);
    let master_seed_hex = master_hasher.finalize().to_hex().to_string();
    let (primary_bits, changed_bits) = structure_bits(&lines);

    Ok(LiuyaoCast {
        algorithm_version: LIUYAO_ALGORITHM_VERSION.to_owned(),
        question_id: transcript.question_id,
        transcript_hash,
        master_seed_hex,
        lines,
        primary_bits,
        changed_bits,
    })
}

pub fn verify_liuyao_replay(
    transcript: &LiuyaoTranscript,
    expected: &LiuyaoCast,
) -> Result<(), CastingError> {
    let replayed = replay_liuyao(transcript)?;
    if replayed == *expected {
        Ok(())
    } else {
        Err(CastingError::ReplayMismatch)
    }
}

fn generate_coin_line(
    question_id: Uuid,
    nonce: &[u8; 32],
    event: &EntropyEvent,
) -> Result<LineCast, CastingError> {
    let mut canonical_event = Vec::new();
    event.append_canonical_bytes(&mut canonical_event)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"jiuheng/liuyao/three-coins/line/v1\0");
    hasher.update(question_id.as_bytes());
    hasher.update(nonce);
    hasher.update(&canonical_event);
    let seed = *hasher.finalize().as_bytes();
    let mut random = ChaCha20Rng::from_seed(seed);
    let mut coins = [CoinFace::Yin; 3];
    for coin in &mut coins {
        *coin = if random.next_u32() & 1 == 1 {
            CoinFace::Yang
        } else {
            CoinFace::Yin
        };
    }
    let sum = coins.iter().map(|coin| coin.numeric_value()).sum::<u8>();
    let line_type = LineType::from_value(sum).expect("three 2-or-3 coins always total 6 through 9");
    Ok(LineCast {
        line_index_bottom_up: event.sequence,
        coins,
        value: sum,
        line_type,
        line_seed_hex: encode_hex(&seed),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoinFace {
    Yin,
    Yang,
}

impl CoinFace {
    pub const fn numeric_value(self) -> u8 {
        match self {
            Self::Yin => 2,
            Self::Yang => 3,
        }
    }

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Yin => "阴面",
            Self::Yang => "阳面",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineType {
    OldYin,
    YoungYang,
    YoungYin,
    OldYang,
}

impl LineType {
    pub const fn from_value(value: u8) -> Option<Self> {
        match value {
            6 => Some(Self::OldYin),
            7 => Some(Self::YoungYang),
            8 => Some(Self::YoungYin),
            9 => Some(Self::OldYang),
            _ => None,
        }
    }

    pub const fn value(self) -> u8 {
        match self {
            Self::OldYin => 6,
            Self::YoungYang => 7,
            Self::YoungYin => 8,
            Self::OldYang => 9,
        }
    }

    pub const fn is_moving(self) -> bool {
        matches!(self, Self::OldYin | Self::OldYang)
    }

    pub const fn primary_is_yang(self) -> bool {
        matches!(self, Self::YoungYang | Self::OldYang)
    }

    pub const fn changed_is_yang(self) -> bool {
        match self {
            Self::OldYin => true,
            Self::OldYang => false,
            _ => self.primary_is_yang(),
        }
    }

    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::OldYin => "老阴",
            Self::YoungYang => "少阳",
            Self::YoungYin => "少阴",
            Self::OldYang => "老阳",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineCast {
    pub line_index_bottom_up: u8,
    pub coins: [CoinFace; 3],
    pub value: u8,
    pub line_type: LineType,
    pub line_seed_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiuyaoCast {
    pub algorithm_version: String,
    pub question_id: Uuid,
    pub transcript_hash: String,
    pub master_seed_hex: String,
                                        
    pub lines: Vec<LineCast>,
                                                 
    pub primary_bits: u8,
                                                                           
    pub changed_bits: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YarrowTranscript {
    pub algorithm_version: String,
    #[serde(default = "legacy_yarrow_convention")]
    pub procedure_convention_id: String,
                                                                         
    pub left_splits: Vec<u8>,
}

impl Default for YarrowTranscript {
    fn default() -> Self {
        Self {
            algorithm_version: YARROW_ALGORITHM_VERSION.to_owned(),
            procedure_convention_id: YARROW_CONVENTION_TANG_LEFT.to_owned(),
            left_splits: Vec::with_capacity(18),
        }
    }
}

fn legacy_yarrow_convention() -> String {
    YARROW_CONVENTION_V1_IMPLICIT_RIGHT.to_owned()
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum YarrowPile {
    Left,
    #[default]
    Right,
}

impl YarrowPile {
    pub const fn zh_name(self) -> &'static str {
        match self {
            Self::Left => "左堆（天）",
            Self::Right => "右堆",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YarrowChange {
    pub line_index_bottom_up: u8,
    pub change_index: u8,
    pub stalks_before: u8,
    pub left_pile: u8,
    pub right_pile: u8,
    pub hanging_stalks: u8,
    #[serde(default)]
    pub hanging_side: YarrowPile,
                                                                                             
    #[serde(default)]
    pub left_after_hanging: u8,
    pub left_remainder: u8,
    pub right_after_hanging: u8,
    pub right_remainder: u8,
    pub removed_stalks: u8,
    pub stalks_after: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YarrowLineCast {
    pub line_index_bottom_up: u8,
    pub changes: Vec<YarrowChange>,
    pub remaining_stalks: u8,
    pub value: u8,
    pub line_type: LineType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YarrowCast {
    pub algorithm_version: String,
    pub transcript_hash: String,
                                        
    pub lines: Vec<YarrowLineCast>,
    pub primary_bits: u8,
    pub changed_bits: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YarrowStepResult {
    pub change: YarrowChange,
    pub completed_line: Option<YarrowLineCast>,
}

#[derive(Debug, Clone)]
pub struct YarrowSession {
    transcript: YarrowTranscript,
    active_stalks: u8,
    current_changes: Vec<YarrowChange>,
    completed_lines: Vec<YarrowLineCast>,
}

impl Default for YarrowSession {
    fn default() -> Self {
        Self {
            transcript: YarrowTranscript::default(),
            active_stalks: YARROW_ACTIVE_STALKS,
            current_changes: Vec::with_capacity(3),
            completed_lines: Vec::with_capacity(6),
        }
    }
}

impl YarrowSession {
    pub fn active_stalks(&self) -> u8 {
        self.active_stalks
    }

    pub fn completed_changes(&self) -> usize {
        self.transcript.left_splits.len()
    }

    pub fn completed_lines(&self) -> &[YarrowLineCast] {
        &self.completed_lines
    }

    pub fn transcript(&self) -> &YarrowTranscript {
        &self.transcript
    }

    pub fn is_complete(&self) -> bool {
        self.completed_lines.len() == 6
    }

    pub fn minimum_left_pile(&self) -> u8 {
        match yarrow_hanging_side(&self.transcript).unwrap_or(YarrowPile::Left) {
            YarrowPile::Left => 2,
            YarrowPile::Right => 1,
        }
    }

    pub fn maximum_left_pile(&self) -> u8 {
        match yarrow_hanging_side(&self.transcript).unwrap_or(YarrowPile::Left) {
            YarrowPile::Left => self.active_stalks.saturating_sub(1),
            YarrowPile::Right => self.active_stalks.saturating_sub(2),
        }
    }

    pub fn apply_split(&mut self, left_pile: u8) -> Result<YarrowStepResult, CastingError> {
        if self.is_complete() {
            return Err(CastingError::YarrowAlreadyComplete);
        }
        let minimum_left = self.minimum_left_pile();
        let maximum_left = self.maximum_left_pile();
        if left_pile < minimum_left || left_pile > maximum_left {
            return Err(CastingError::InvalidYarrowSplit {
                active: self.active_stalks,
                left: left_pile,
            });
        }

        let stalks_before = self.active_stalks;
        let right_pile = stalks_before - left_pile;
        let hanging_stalks = YARROW_HANGING_STALKS;
        let hanging_side = yarrow_hanging_side(&self.transcript)?;
        let (left_after_hanging, right_after_hanging) = match hanging_side {
            YarrowPile::Left => (left_pile - hanging_stalks, right_pile),
            YarrowPile::Right => (left_pile, right_pile - hanging_stalks),
        };
        let left_remainder = remainder_by_four(left_after_hanging);
        let right_remainder = remainder_by_four(right_after_hanging);
        let removed_stalks = hanging_stalks + left_remainder + right_remainder;
        let stalks_after = stalks_before - removed_stalks;
        let expected_removed = if self.current_changes.is_empty() {
            matches!(removed_stalks, 5 | 9)
        } else {
            matches!(removed_stalks, 4 | 8)
        };
        if !expected_removed || !stalks_after.is_multiple_of(4) {
            return Err(CastingError::YarrowInvariantViolation {
                before: stalks_before,
                removed: removed_stalks,
                after: stalks_after,
            });
        }

        let change = YarrowChange {
            line_index_bottom_up: self.completed_lines.len() as u8,
            change_index: self.current_changes.len() as u8,
            stalks_before,
            left_pile,
            right_pile,
            hanging_stalks,
            hanging_side,
            left_after_hanging: if self.transcript.algorithm_version == YARROW_ALGORITHM_VERSION_V1
            {
                0
            } else {
                left_after_hanging
            },
            left_remainder,
            right_after_hanging,
            right_remainder,
            removed_stalks,
            stalks_after,
        };
        self.transcript.left_splits.push(left_pile);
        self.current_changes.push(change.clone());
        self.active_stalks = stalks_after;

        let completed_line = if self.current_changes.len() == 3 {
            let value = self.active_stalks / 4;
            let line_type =
                LineType::from_value(value).ok_or(CastingError::YarrowLineValueOutOfRange {
                    remaining: self.active_stalks,
                    value,
                })?;
            let line = YarrowLineCast {
                line_index_bottom_up: self.completed_lines.len() as u8,
                changes: std::mem::take(&mut self.current_changes),
                remaining_stalks: self.active_stalks,
                value,
                line_type,
            };
            self.completed_lines.push(line.clone());
            self.active_stalks = YARROW_ACTIVE_STALKS;
            Some(line)
        } else {
            None
        };

        Ok(YarrowStepResult {
            change,
            completed_line,
        })
    }

    pub fn finish(&self) -> Result<YarrowCast, CastingError> {
        if !self.is_complete() {
            return Err(CastingError::YarrowNeedsEighteenChanges(
                self.completed_changes(),
            ));
        }
        yarrow_cast_from_parts(&self.transcript, self.completed_lines.clone())
    }
}

pub fn replay_yarrow(transcript: &YarrowTranscript) -> Result<YarrowCast, CastingError> {
    validate_yarrow_transcript_metadata(transcript)?;
    if transcript.left_splits.len() != 18 {
        return Err(CastingError::YarrowNeedsEighteenChanges(
            transcript.left_splits.len(),
        ));
    }
    let mut session = YarrowSession {
        transcript: YarrowTranscript {
            algorithm_version: transcript.algorithm_version.clone(),
            procedure_convention_id: transcript.procedure_convention_id.clone(),
            left_splits: Vec::with_capacity(18),
        },
        active_stalks: YARROW_ACTIVE_STALKS,
        current_changes: Vec::with_capacity(3),
        completed_lines: Vec::with_capacity(6),
    };
    for left in &transcript.left_splits {
        session.apply_split(*left)?;
    }
    session.finish()
}

pub fn verify_yarrow_replay(
    transcript: &YarrowTranscript,
    expected: &YarrowCast,
) -> Result<(), CastingError> {
    let replayed = replay_yarrow(transcript)?;
    if replayed == *expected {
        Ok(())
    } else {
        Err(CastingError::ReplayMismatch)
    }
}

fn yarrow_cast_from_parts(
    transcript: &YarrowTranscript,
    lines: Vec<YarrowLineCast>,
) -> Result<YarrowCast, CastingError> {
    if lines.len() != 6 {
        return Err(CastingError::YarrowNeedsEighteenChanges(
            transcript.left_splits.len(),
        ));
    }
    validate_yarrow_transcript_metadata(transcript)?;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(transcript.algorithm_version.as_bytes());
    if transcript.algorithm_version == YARROW_ALGORITHM_VERSION {
        bytes.push(0);
        bytes.extend_from_slice(transcript.procedure_convention_id.as_bytes());
        bytes.push(0);
    }
    bytes.extend_from_slice(&transcript.left_splits);
    let transcript_hash = blake3::hash(&bytes).to_hex().to_string();
    let line_structures = lines
        .iter()
        .map(|line| LineCast {
            line_index_bottom_up: line.line_index_bottom_up,
            coins: [CoinFace::Yin; 3],
            value: line.value,
            line_type: line.line_type,
            line_seed_hex: String::new(),
        })
        .collect::<Vec<_>>();
    let (primary_bits, changed_bits) = structure_bits(&line_structures);
    Ok(YarrowCast {
        algorithm_version: transcript.algorithm_version.clone(),
        transcript_hash,
        lines,
        primary_bits,
        changed_bits,
    })
}

pub fn is_supported_yarrow_algorithm(version: &str) -> bool {
    matches!(
        version,
        YARROW_ALGORITHM_VERSION_V1 | YARROW_ALGORITHM_VERSION
    )
}

fn validate_yarrow_transcript_metadata(transcript: &YarrowTranscript) -> Result<(), CastingError> {
    let expected_convention = match transcript.algorithm_version.as_str() {
        YARROW_ALGORITHM_VERSION_V1 => YARROW_CONVENTION_V1_IMPLICIT_RIGHT,
        YARROW_ALGORITHM_VERSION => YARROW_CONVENTION_TANG_LEFT,
        _ => {
            return Err(CastingError::UnsupportedAlgorithmVersion(
                transcript.algorithm_version.clone(),
            ));
        }
    };
    if transcript.procedure_convention_id != expected_convention {
        return Err(CastingError::UnsupportedYarrowConvention(
            transcript.procedure_convention_id.clone(),
        ));
    }
    Ok(())
}

fn yarrow_hanging_side(transcript: &YarrowTranscript) -> Result<YarrowPile, CastingError> {
    validate_yarrow_transcript_metadata(transcript)?;
    Ok(match transcript.algorithm_version.as_str() {
        YARROW_ALGORITHM_VERSION_V1 => YarrowPile::Right,
        YARROW_ALGORITHM_VERSION => YarrowPile::Left,
        _ => unreachable!("validated above"),
    })
}

fn remainder_by_four(stalks: u8) -> u8 {
    match stalks % 4 {
        0 => 4,
        remainder => remainder,
    }
}

fn structure_bits<T>(lines: &[T]) -> (u8, u8)
where
    T: AsLineType,
{
    let mut primary = 0_u8;
    let mut changed = 0_u8;
    for (index, line) in lines.iter().enumerate() {
        if line.line_type().primary_is_yang() {
            primary |= 1 << index;
        }
        if line.line_type().changed_is_yang() {
            changed |= 1 << index;
        }
    }
    (primary, changed)
}

trait AsLineType {
    fn line_type(&self) -> LineType;
}

impl AsLineType for LineCast {
    fn line_type(&self) -> LineType {
        self.line_type
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(DIGITS[(byte >> 4) as usize] as char);
        output.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    output
}

fn decode_32_byte_hex(value: &str) -> Result<[u8; 32], CastingError> {
    if value.len() != 64 {
        return Err(CastingError::InvalidNonceHex);
    }
    let mut output = [0_u8; 32];
    for (index, slot) in output.iter_mut().enumerate() {
        let offset = index * 2;
        *slot = u8::from_str_radix(&value[offset..offset + 2], 16)
            .map_err(|_| CastingError::InvalidNonceHex)?;
    }
    Ok(output)
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CastingError {
    #[error("不支持的起法算法版本：{0}")]
    UnsupportedAlgorithmVersion(String),
    #[error("不支持的蓍草程序约定：{0}")]
    UnsupportedYarrowConvention(String),
    #[error("会话随机材料必须是 32 字节十六进制值")]
    InvalidNonceHex,
    #[error("鼠标轨迹采样过长")]
    EntropyPathTooLong,
    #[error("随机材料序号错误：应为 {expected}，实际为 {actual}")]
    InvalidEntropySequence { expected: u8, actual: u8 },
    #[error("六爻三币起法已经完成六次")]
    LiuyaoAlreadyComplete,
    #[error("六爻三币起法需要六次操作，当前为 {0} 次")]
    LiuyaoNeedsSixEvents(usize),
    #[error("蓍草十八变已经完成")]
    YarrowAlreadyComplete,
    #[error("分二数量无效：当前 {active} 策，左堆 {left} 策")]
    InvalidYarrowSplit { active: u8, left: u8 },
    #[error("蓍草变化不变量失败：变前 {before}，移除 {removed}，变后 {after}")]
    YarrowInvariantViolation { before: u8, removed: u8, after: u8 },
    #[error("三变后的策数无法得到合法爻值：剩余 {remaining}，爻值 {value}")]
    YarrowLineValueOutOfRange { remaining: u8, value: u8 },
    #[error("蓍草起法需要十八变，当前为 {0} 变")]
    YarrowNeedsEighteenChanges(usize),
    #[error("重放结果与保存结果不一致")]
    ReplayMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(sequence: u8) -> EntropyEvent {
        EntropyEvent {
            sequence,
            wall_clock_unix_nanos: 1_788_068_725_000_000_000 + sequence as i64 * 1_000_003,
            monotonic_nanos: 9_000_000 + sequence as u64 * 791_921,
            press_duration_nanos: 80_000_000 + sequence as u64 * 113_003,
            delta_since_previous_nanos: 600_000_000 + sequence as u64 * 17_009,
            click_x_micropoints: 120_000 + sequence as i32 * 1_007,
            click_y_micropoints: 240_000 - sequence as i32 * 991,
            pointer_path: vec![
                PointerSample {
                    elapsed_nanos: 1_000,
                    x_micropoints: 119_000,
                    y_micropoints: 239_000,
                },
                PointerSample {
                    elapsed_nanos: 2_000 + sequence as u64,
                    x_micropoints: 120_000,
                    y_micropoints: 240_000,
                },
            ],
        }
    }

    fn complete_liuyao() -> (LiuyaoTranscript, LiuyaoCast) {
        let mut transcript = LiuyaoTranscript::new(Uuid::nil(), [0x5a; 32]);
        for sequence in 0..6 {
            transcript.push_event(event(sequence)).unwrap();
        }
        let cast = transcript.finish().unwrap();
        (transcript, cast)
    }

    #[test]
    fn liuyao_replay_is_byte_for_byte_deterministic() {
        let (transcript, first) = complete_liuyao();
        let second = replay_liuyao(&transcript).unwrap();
        assert_eq!(first, second);
        verify_liuyao_replay(&transcript, &first).unwrap();
        assert_eq!(first.lines.len(), 6);
        assert!(first.lines.iter().all(|line| (6..=9).contains(&line.value)));
    }

    #[test]
    fn liuyao_detects_changed_entropy_material() {
        let (mut transcript, original) = complete_liuyao();
        transcript.events[2].press_duration_nanos += 1;
        assert_eq!(
            verify_liuyao_replay(&transcript, &original),
            Err(CastingError::ReplayMismatch)
        );
    }

    #[test]
    fn liuyao_requires_sequential_six_operations() {
        let mut transcript = LiuyaoTranscript::new(Uuid::nil(), [7; 32]);
        let error = transcript.push_event(event(1)).unwrap_err();
        assert_eq!(
            error,
            CastingError::InvalidEntropySequence {
                expected: 0,
                actual: 1
            }
        );
        assert_eq!(
            transcript.finish().unwrap_err(),
            CastingError::LiuyaoNeedsSixEvents(0)
        );
    }

    fn complete_yarrow() -> (YarrowTranscript, YarrowCast) {
        let mut session = YarrowSession::default();
        while !session.is_complete() {
            let left = session.active_stalks() / 2;
            session.apply_split(left).unwrap();
        }
        let transcript = session.transcript().clone();
        let cast = session.finish().unwrap();
        (transcript, cast)
    }

    #[test]
    fn yarrow_first_change_obeys_five_or_nine_removal() {
        let mut session = YarrowSession::default();
        let result = session.apply_split(24).unwrap();
        assert_eq!(YARROW_TOTAL_STALKS - 1, YARROW_ACTIVE_STALKS);
        assert_eq!(result.change.stalks_before, YARROW_ACTIVE_STALKS);
        assert_eq!(result.change.hanging_stalks, YARROW_HANGING_STALKS);
        assert_eq!(result.change.hanging_side, YarrowPile::Left);
        assert_eq!(result.change.left_after_hanging, 23);
        assert_eq!(result.change.right_after_hanging, 25);
        assert_eq!(result.change.left_remainder, 3);
        assert_eq!(result.change.right_remainder, 1);
        assert_eq!(result.change.removed_stalks, 5);
        assert_eq!(result.change.stalks_after, 44);
    }

    #[test]
    fn yarrow_runs_exactly_eighteen_changes_and_replays() {
        let (transcript, cast) = complete_yarrow();
        assert_eq!(transcript.left_splits.len(), 18);
        assert_eq!(cast.lines.len(), 6);
        assert!(cast.lines.iter().all(|line| (6..=9).contains(&line.value)));
        assert_eq!(replay_yarrow(&transcript).unwrap(), cast);
        verify_yarrow_replay(&transcript, &cast).unwrap();
    }

    #[test]
    fn yarrow_v2_rejects_a_split_that_leaves_no_countable_left_pile_after_hanging() {
        let mut session = YarrowSession::default();
        assert_eq!(
            session.apply_split(1).unwrap_err(),
            CastingError::InvalidYarrowSplit {
                active: YARROW_ACTIVE_STALKS,
                left: 1
            }
        );
    }

    #[test]
    fn yarrow_v1_right_pile_records_remain_replayable() {
        let mut left_splits = Vec::with_capacity(18);
        for _ in 0..6 {
            left_splits.extend_from_slice(&[24, 20, 16]);
        }
        let transcript = YarrowTranscript {
            algorithm_version: YARROW_ALGORITHM_VERSION_V1.to_owned(),
            procedure_convention_id: YARROW_CONVENTION_V1_IMPLICIT_RIGHT.to_owned(),
            left_splits,
        };
        let cast = replay_yarrow(&transcript).unwrap();
        assert_eq!(cast.algorithm_version, YARROW_ALGORITHM_VERSION_V1);
        assert!(cast.lines.iter().all(|line| line.value == 6));
        assert!(cast
            .lines
            .iter()
            .flat_map(|line| &line.changes)
            .all(|change| {
                change.hanging_side == YarrowPile::Right && change.left_after_hanging == 0
            }));
        verify_yarrow_replay(&transcript, &cast).unwrap();
    }

    #[test]
    fn yarrow_v1_transcript_without_convention_field_deserializes_and_replays() {
        let mut left_splits = Vec::with_capacity(18);
        for _ in 0..6 {
            left_splits.extend_from_slice(&[24, 20, 16]);
        }
        let json = serde_json::json!({
            "algorithm_version": YARROW_ALGORITHM_VERSION_V1,
            "left_splits": left_splits,
        });
        let transcript: YarrowTranscript = serde_json::from_value(json).unwrap();
        assert_eq!(
            transcript.procedure_convention_id,
            YARROW_CONVENTION_V1_IMPLICIT_RIGHT
        );
        assert_eq!(
            replay_yarrow(&transcript).unwrap().algorithm_version,
            YARROW_ALGORITHM_VERSION_V1
        );
    }

    #[test]
    fn yarrow_version_two_hash_binds_the_named_procedure_convention() {
        let (mut transcript, _) = complete_yarrow();
        transcript.procedure_convention_id = "different-convention".to_owned();
        assert_eq!(
            replay_yarrow(&transcript).unwrap_err(),
            CastingError::UnsupportedYarrowConvention("different-convention".to_owned())
        );
    }
}
