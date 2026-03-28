use std::collections::HashMap;

use crate::feed::{config::FieldOverride, Field, FieldDefinition};

/// Applies label overrides to field metadata in deterministic precedence order.
///
/// Precedence: base defaults -> feed-type explicit overrides -> config overrides.
pub fn apply_definition_overrides(
    definitions: Vec<FieldDefinition>,
    explicit_overrides: &HashMap<String, FieldOverride>,
    config_overrides: &HashMap<String, FieldOverride>,
) -> Vec<FieldDefinition> {
    definitions
        .into_iter()
        .map(|mut definition| {
            let effective = merged_override(&definition.name, explicit_overrides, config_overrides);

            if let Some(label) = effective.label {
                definition.label = label;
            }

            definition
        })
        .collect()
}

/// Applies visibility + label overrides to activity field output in deterministic precedence order.
///
/// Precedence: base defaults -> feed-type explicit overrides -> config overrides.
pub fn apply_activity_overrides(
    fields: Vec<Field>,
    explicit_overrides: &HashMap<String, FieldOverride>,
    config_overrides: &HashMap<String, FieldOverride>,
) -> Vec<Field> {
    fields
        .into_iter()
        .filter_map(|mut field| {
            let effective = merged_override(&field.name, explicit_overrides, config_overrides);

            if matches!(effective.visible, Some(false)) {
                return None;
            }

            if let Some(label) = effective.label {
                field.label = label;
            }

            Some(field)
        })
        .collect()
}

fn merged_override(
    field_name: &str,
    explicit_overrides: &HashMap<String, FieldOverride>,
    config_overrides: &HashMap<String, FieldOverride>,
) -> FieldOverride {
    let mut merged = FieldOverride {
        visible: None,
        label: None,
    };

    if let Some(override_cfg) = explicit_overrides.get(field_name) {
        if override_cfg.visible.is_some() {
            merged.visible = override_cfg.visible;
        }

        if override_cfg.label.is_some() {
            merged.label = override_cfg.label.clone();
        }
    }

    if let Some(override_cfg) = config_overrides.get(field_name) {
        if override_cfg.visible.is_some() {
            merged.visible = override_cfg.visible;
        }

        if override_cfg.label.is_some() {
            merged.label = override_cfg.label.clone();
        }
    }

    merged
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::feed::{
        config::FieldOverride, Field, FieldDefinition, FieldType, FieldValue, StatusKind,
    };

    use super::{apply_activity_overrides, apply_definition_overrides};

    #[test]
    fn apply_definition_overrides_uses_config_label_last() {
        let definitions = vec![FieldDefinition {
            name: "review".to_string(),
            label: "Review".to_string(),
            field_type: FieldType::Status,
            description: "Current review state".to_string(),
        }];

        let explicit = HashMap::from([(
            "review".to_string(),
            FieldOverride {
                visible: None,
                label: Some("Explicit".to_string()),
            },
        )]);

        let from_config = HashMap::from([(
            "review".to_string(),
            FieldOverride {
                visible: None,
                label: Some("Config".to_string()),
            },
        )]);

        let overridden = apply_definition_overrides(definitions, &explicit, &from_config);
        assert_eq!(overridden[0].label, "Config");
    }

    #[test]
    fn apply_activity_overrides_hides_field_and_ignores_unknowns() {
        let fields = vec![
            Field {
                name: "review".to_string(),
                label: "Review".to_string(),
                value: FieldValue::Status {
                    value: "approved".to_string(),
                    kind: StatusKind::AttentionPositive,
                },
            },
            Field {
                name: "labels".to_string(),
                label: "Labels".to_string(),
                value: FieldValue::Text {
                    value: "wip".to_string(),
                },
            },
        ];

        let explicit = HashMap::new();

        let from_config = HashMap::from([
            (
                "review".to_string(),
                FieldOverride {
                    visible: Some(false),
                    label: None,
                },
            ),
            (
                "labels".to_string(),
                FieldOverride {
                    visible: None,
                    label: Some("Tags".to_string()),
                },
            ),
            (
                "non-existent".to_string(),
                FieldOverride {
                    visible: Some(false),
                    label: Some("ignored".to_string()),
                },
            ),
        ]);

        let overridden = apply_activity_overrides(fields, &explicit, &from_config);
        assert_eq!(overridden.len(), 1);
        assert_eq!(overridden[0].name, "labels");
        assert_eq!(overridden[0].label, "Tags");
    }

    #[test]
    fn apply_definition_overrides_explicit_only() {
        let definitions = vec![FieldDefinition {
            name: "output".to_string(),
            label: "Output".to_string(),
            field_type: FieldType::Text,
            description: "test".to_string(),
        }];

        let explicit = HashMap::from([(
            "output".to_string(),
            FieldOverride {
                visible: None,
                label: Some("Custom".to_string()),
            },
        )]);

        let from_config = HashMap::new();

        let overridden = apply_definition_overrides(definitions, &explicit, &from_config);
        assert_eq!(overridden[0].label, "Custom");
    }

    #[test]
    fn apply_definition_overrides_no_overrides_preserves_defaults() {
        let definitions = vec![FieldDefinition {
            name: "output".to_string(),
            label: "Output".to_string(),
            field_type: FieldType::Text,
            description: "test".to_string(),
        }];

        let overridden = apply_definition_overrides(definitions, &HashMap::new(), &HashMap::new());
        assert_eq!(overridden[0].label, "Output");
    }

    #[test]
    fn apply_activity_overrides_visible_true_keeps_field() {
        let fields = vec![Field {
            name: "status".to_string(),
            label: "Status".to_string(),
            value: FieldValue::Text {
                value: "ok".to_string(),
            },
        }];

        let from_config = HashMap::from([(
            "status".to_string(),
            FieldOverride {
                visible: Some(true),
                label: None,
            },
        )]);

        let overridden = apply_activity_overrides(fields, &HashMap::new(), &from_config);
        assert_eq!(overridden.len(), 1);
    }

    #[test]
    fn apply_activity_overrides_empty_fields_returns_empty() {
        let overridden = apply_activity_overrides(Vec::new(), &HashMap::new(), &HashMap::new());
        assert!(overridden.is_empty());
    }
}
