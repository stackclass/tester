use tester::{Context, Definition, TesterError};

use std::collections::HashMap;

#[test]
fn test_requires_app_dir() {
    let env = HashMap::from([(
        "STACKCLASS_TEST_CASES_JSON".to_string(),
        r#"[{ "slug": "test", "log_prefix": "test", "title": "Test" }]"#.to_string(),
    )]);

    let result = Context::from_env(env, &Definition::default());
    assert!(result.is_err(), "Expected an error due to missing STACKCLASS_REPOSITORY_DIR");

    if let Err(err) = result {
        assert!(
            matches!(err, TesterError::MissingEnvVar(_)),
            "Expected MissingEnvVar error, got {err:?}",
        );
    }
}

#[test]
fn test_valid_test_cases() {
    let env = HashMap::from([
        ("STACKCLASS_REPOSITORY_DIR".to_string(), "/tmp".to_string()),
        (
            "STACKCLASS_TEST_CASES_JSON".to_string(),
            r#"[{ "slug": "test", "log_prefix": "test", "title": "Test" }]"#.to_string(),
        ),
    ]);

    let result = Context::from_env(env, &Definition::default());
    assert!(result.is_ok(), "Expected successful context creation");
}

#[test]
fn test_empty_test_cases() {
    let env = HashMap::from([
        ("STACKCLASS_REPOSITORY_DIR".to_string(), "/tmp".to_string()),
        ("STACKCLASS_TEST_CASES_JSON".to_string(), "[]".to_string()),
    ]);

    let result = Context::from_env(env, &Definition::default());
    assert!(matches!(result, Err(TesterError::InvalidTestCase(_))));
}

#[test]
fn test_invalid_test_case_fields() {
    let test_cases = [
        (r#"[{ "slug": "", "log_prefix": "test", "title": "Test" }]"#, "empty slug"),
        (r#"[{ "slug": "test", "log_prefix": "", "title": "Test" }]"#, "empty log_prefix"),
        (r#"[{ "slug": "test", "log_prefix": "test", "title": "" }]"#, "empty title"),
    ];

    for (json, description) in test_cases {
        let env = HashMap::from([
            ("STACKCLASS_REPOSITORY_DIR".to_string(), "/tmp".to_string()),
            ("STACKCLASS_TEST_CASES_JSON".to_string(), json.to_string()),
        ]);

        let result = Context::from_env(env, &Definition::default());
        assert!(
            matches!(result, Err(TesterError::InvalidTestCase(_))),
            "Expected InvalidTestCase for case: {description}",
        );
    }
}

#[test]
fn test_executable_not_found() {
    let env = HashMap::from([
        ("STACKCLASS_REPOSITORY_DIR".to_string(), "/nonexistent".to_string()),
        (
            "STACKCLASS_TEST_CASES_JSON".to_string(),
            r#"[{ "slug": "test", "log_prefix": "test", "title": "Test" }]"#.to_string(),
        ),
    ]);

    let result = Context::from_env(env, &Definition::default());
    assert!(matches!(result, Err(TesterError::ExecutableNotFound(_))));
}

#[test]
fn test_timeout_parsing() {
    let env = HashMap::from([
        ("STACKCLASS_REPOSITORY_DIR".to_string(), "/tmp".to_string()),
        (
            "STACKCLASS_TEST_CASES_JSON".to_string(),
            r#"[{ "slug": "test", "log_prefix": "test", "title": "Test" }]"#.to_string(),
        ),
        ("STACKCLASS_TIMEOUT_SECONDS".to_string(), "30".to_string()),
    ]);

    let context = Context::from_env(env, &Definition::default()).unwrap();
    assert_eq!(context.timeout, std::time::Duration::from_secs(30));
}

#[test]
fn test_debug_flag() {
    let env = HashMap::from([
        ("STACKCLASS_REPOSITORY_DIR".to_string(), "/tmp".to_string()),
        (
            "STACKCLASS_TEST_CASES_JSON".to_string(),
            r#"[{ "slug": "test", "log_prefix": "test", "title": "Test" }]"#.to_string(),
        ),
        ("STACKCLASS_DEBUG".to_string(), "true".to_string()),
    ]);

    let context = Context::from_env(env, &Definition::default()).unwrap();
    assert!(context.is_debug);
}

#[test]
fn test_skip_anti_cheat_flag() {
    let env = HashMap::from([
        ("STACKCLASS_REPOSITORY_DIR".to_string(), "/tmp".to_string()),
        (
            "STACKCLASS_TEST_CASES_JSON".to_string(),
            r#"[{ "slug": "test", "log_prefix": "test", "title": "Test" }]"#.to_string(),
        ),
        ("STACKCLASS_SKIP_ANTI_CHEAT".to_string(), "true".to_string()),
    ]);

    let context = Context::from_env(env, &Definition::default()).unwrap();
    assert!(context.should_skip_anti_cheat);
}
