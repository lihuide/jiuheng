use crate::{AncientQuestionType, QuestionCategory};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Classification {
    pub ancient_type: AncientQuestionType,
    pub matched_terms: Vec<String>,
    pub normalized_terms: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassificationError {
    NoMatchingType,
    AmbiguousTypes(Vec<AncientQuestionType>),
}

#[derive(Debug, Clone)]
pub struct KeywordClassifier {
    synonyms: BTreeMap<String, String>,
    mappings: Vec<(AncientQuestionType, Vec<String>)>,
}

impl Default for KeywordClassifier {
    fn default() -> Self {
        let synonyms = [
            ("app", "项目"),
            ("应用", "项目"),
            ("上线", "项目"),
            ("发布", "项目"),
            ("下载量", "成败"),
            ("成交", "交易"),
            ("合同", "交易"),
            ("下雨", "雨"),
            ("降雨", "雨"),
            ("考试", "考"),
            ("录取", "考"),
            ("比赛", "胜负"),
            ("竞标", "胜负"),
            ("丢失", "寻物"),
        ]
        .into_iter()
        .map(|(a, b)| (a.to_owned(), b.to_owned()))
        .collect();

        let mappings = vec![
            (
                AncientQuestionType::WeatherRain,
                vec!["雨", "天气", "雪", "风"],
            ),
            (
                AncientQuestionType::ProjectOutcome,
                vec!["项目", "成败", "计划", "方案"],
            ),
            (
                AncientQuestionType::ProfitAndLoss,
                vec!["财", "盈利", "亏损", "收益"],
            ),
            (
                AncientQuestionType::CompetitionOutcome,
                vec!["胜负", "竞赛", "对阵"],
            ),
            (
                AncientQuestionType::ExaminationOutcome,
                vec!["考", "学业", "名次"],
            ),
            (
                AncientQuestionType::TransactionOutcome,
                vec!["交易", "合作", "买卖"],
            ),
            (
                AncientQuestionType::MissingPersonOrObject,
                vec!["寻物", "寻人", "失踪"],
            ),
            (
                AncientQuestionType::TravelSafety,
                vec!["出行", "行程", "远行"],
            ),
            (
                AncientQuestionType::LitigationOutcome,
                vec!["官非", "诉讼", "纠纷"],
            ),
            (
                AncientQuestionType::DwellingFortune,
                vec!["家宅", "住宅", "居所"],
            ),
            (
                AncientQuestionType::RelationshipOutcome,
                vec!["感情", "婚姻", "关系"],
            ),
            (
                AncientQuestionType::HealthTrendReference,
                vec!["健康", "疾病", "身体"],
            ),
            (
                AncientQuestionType::NatalLongTerm,
                vec!["命运", "长期", "一生"],
            ),
        ]
        .into_iter()
        .map(|(kind, terms)| (kind, terms.into_iter().map(str::to_owned).collect()))
        .collect();

        Self { synonyms, mappings }
    }
}

impl KeywordClassifier {
    pub fn classify(&self, category: QuestionCategory, text: &str) -> Option<Classification> {
        self.classify_detailed(category, text).ok()
    }

                                                                             
                                                                          
    pub fn classify_detailed(
        &self,
        category: QuestionCategory,
        text: &str,
    ) -> Result<Classification, ClassificationError> {
        if category != QuestionCategory::Other {
            return Ok(Classification {
                ancient_type: fixed_category_mapping(category)
                    .expect("every non-Other category has a fixed mapping"),
                matched_terms: vec![category.zh_name().to_owned()],
                normalized_terms: tokenize(text),
            });
        }

        let lowered = text.to_lowercase();
        let mut expanded = lowered.clone();
        let mut normalized_terms = tokenize(&lowered);
        for (source, canonical) in &self.synonyms {
            if contains_alias(&lowered, source) {
                expanded.push_str(canonical);
                normalized_terms.insert(canonical.clone());
            }
        }

        let mut matches = Vec::new();
        for (kind, terms) in &self.mappings {
            let hits: Vec<String> = terms
                .iter()
                .filter(|term| expanded.contains(term.as_str()))
                .cloned()
                .collect();
            if hits.is_empty() {
                continue;
            }
            matches.push((kind.clone(), hits));
        }
        match matches.len() {
            0 => Err(ClassificationError::NoMatchingType),
            1 => {
                let (ancient_type, matched_terms) = matches.pop().expect("one mapping");
                Ok(Classification {
                    ancient_type,
                    matched_terms,
                    normalized_terms,
                })
            }
            _ => Err(ClassificationError::AmbiguousTypes(
                matches.into_iter().map(|(kind, _)| kind).collect(),
            )),
        }
    }
}

                                                                           
                                                                            
                                                                           
fn contains_alias(text: &str, alias: &str) -> bool {
    if !alias.chars().all(is_ascii_identifier_character) {
        return text.contains(alias);
    }
    text.match_indices(alias).any(|(start, _)| {
        let end = start + alias.len();
        !text[..start]
            .chars()
            .next_back()
            .is_some_and(is_ascii_identifier_character)
            && !text[end..]
                .chars()
                .next()
                .is_some_and(is_ascii_identifier_character)
    })
}

fn is_ascii_identifier_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_'
}

fn fixed_category_mapping(category: QuestionCategory) -> Option<AncientQuestionType> {
    Some(match category {
        QuestionCategory::Relationship => AncientQuestionType::RelationshipOutcome,
        QuestionCategory::Career => AncientQuestionType::ProjectOutcome,
        QuestionCategory::Wealth => AncientQuestionType::ProfitAndLoss,
        QuestionCategory::Weather => AncientQuestionType::WeatherRain,
        QuestionCategory::Travel => AncientQuestionType::TravelSafety,
        QuestionCategory::Search => AncientQuestionType::MissingPersonOrObject,
        QuestionCategory::Competition => AncientQuestionType::CompetitionOutcome,
        QuestionCategory::Study => AncientQuestionType::ExaminationOutcome,
        QuestionCategory::Cooperation => AncientQuestionType::TransactionOutcome,
        QuestionCategory::Dispute => AncientQuestionType::LitigationOutcome,
        QuestionCategory::Home => AncientQuestionType::DwellingFortune,
        QuestionCategory::LongTermFate => AncientQuestionType::NatalLongTerm,
        QuestionCategory::HealthReference => AncientQuestionType::HealthTrendReference,
        QuestionCategory::Other => return None,
    })
}

pub fn tokenize(text: &str) -> BTreeSet<String> {
    text.to_lowercase()
        .split(|ch: char| ch.is_whitespace() || "，。！？、；：,.!?;:()（）[]【】".contains(ch))
        .filter(|token| !token.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_modern_app_question_without_ai() {
        let result = KeywordClassifier::default()
            .classify(QuestionCategory::Other, "APP 上线30天，下载量能否破一万？")
            .expect("should map");
        assert_eq!(result.ancient_type, AncientQuestionType::ProjectOutcome);
    }

    #[test]
    fn refuses_unknown_other_question() {
        assert!(KeywordClassifier::default()
            .classify(QuestionCategory::Other, "一些无法识别的字符")
            .is_none());
    }

    #[test]
    fn mixed_other_topics_fail_closed_instead_of_using_mapping_order() {
        let classifier = KeywordClassifier::default();
        let error = classifier
            .classify_detailed(QuestionCategory::Other, "健康 APP 上线后怎么办？")
            .unwrap_err();
        let ClassificationError::AmbiguousTypes(types) = error else {
            panic!("mixed topics must be ambiguous");
        };
        assert!(types.contains(&AncientQuestionType::HealthTrendReference));
        assert!(types.contains(&AncientQuestionType::ProjectOutcome));
        assert!(classifier
            .classify(QuestionCategory::Other, "健康 APP 上线后怎么办？")
            .is_none());
        assert!(matches!(
            classifier.classify_detailed(
                QuestionCategory::Other,
                "项目计划能否成功，健康情况又会如何？"
            ),
            Err(ClassificationError::AmbiguousTypes(_))
        ));
    }

    #[test]
    fn health_only_other_question_maps_to_health_reference() {
        let classification = KeywordClassifier::default()
            .classify_detailed(QuestionCategory::Other, "健康情况会如何？")
            .unwrap();
        assert_eq!(
            classification.ancient_type,
            AncientQuestionType::HealthTrendReference
        );
    }

    #[test]
    fn ascii_app_alias_does_not_match_inside_other_words_or_identifiers() {
        let classifier = KeywordClassifier::default();
        for text in [
            "happy",
            "happiness",
            "pineapple",
            "apple",
            "application",
            "myapp",
            "app2",
            "app_name",
        ] {
            assert!(
                matches!(
                    classifier.classify_detailed(QuestionCategory::Other, text),
                    Err(ClassificationError::NoMatchingType)
                ),
                "embedded app must not auto-classify: {text}"
            );
        }
    }

    #[test]
    fn ascii_app_alias_keeps_case_punctuation_and_chinese_boundaries() {
        let classifier = KeywordClassifier::default();
        for text in [
            "APP",
            "App",
            "这个APP上线能否成功",
            "APP上线",
            "我的app怎么样",
            "(app)",
            "APP-store",
            "一个 app",
        ] {
            assert_eq!(
                classifier
                    .classify_detailed(QuestionCategory::Other, text)
                    .unwrap()
                    .ancient_type,
                AncientQuestionType::ProjectOutcome,
                "valid app boundary: {text}"
            );
        }
    }

    #[test]
    fn embedded_app_word_does_not_create_a_false_health_conflict() {
        let classifier = KeywordClassifier::default();
        let result = classifier
            .classify_detailed(QuestionCategory::Other, "健康 happy")
            .unwrap();
        assert_eq!(
            result.ancient_type,
            AncientQuestionType::HealthTrendReference
        );
        assert!(!result.normalized_terms.contains("项目"));
        assert!(matches!(
            classifier.classify_detailed(QuestionCategory::Other, "健康 APP"),
            Err(ClassificationError::AmbiguousTypes(_))
        ));
    }

    #[test]
    fn chinese_aliases_and_fixed_categories_keep_their_existing_mapping() {
        let classifier = KeywordClassifier::default();
        assert_eq!(
            classifier
                .classify_detailed(QuestionCategory::Other, "应用下载量能否达标")
                .unwrap()
                .ancient_type,
            AncientQuestionType::ProjectOutcome
        );
        assert_eq!(
            classifier
                .classify_detailed(QuestionCategory::Career, "happy")
                .unwrap()
                .ancient_type,
            AncientQuestionType::ProjectOutcome
        );
    }
}
