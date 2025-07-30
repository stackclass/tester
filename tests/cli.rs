// Copyright (c) The StackClass Authors. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{collections::HashMap, process::ExitCode, sync::Arc};

use tester::{Case, CaseError, Definition, Harness, run};

fn pass_func(_harness: &Harness) -> Result<(), CaseError> {
    Ok(())
}

fn fail_func(_harness: &Harness) -> Result<(), CaseError> {
    Err("fail".to_string().into())
}

fn build_test_cases_json(slugs: &[&str]) -> String {
    let mut test_cases = Vec::new();
    for (index, slug) in slugs.iter().enumerate() {
        test_cases.push(serde_json::json!({
            "slug": slug,
            "log_prefix": format!("test-{}", index + 1),
            "title": format!("Stage #{}: {}", index + 1, slug),
        }));
    }
    serde_json::to_string(&test_cases).unwrap()
}

#[test]
fn test_all_stages_pass() {
    let env = HashMap::from([
        ("STACKCLASS_REPOSITORY_DIR".to_string(), "examples/echo-tester".to_string()),
        ("STACKCLASS_TEST_CASES_JSON".to_string(), build_test_cases_json(&["test-1", "test-2"])),
    ]);

    let definition = Definition {
        executable_name: "your_program.sh".to_string(),
        cases: vec![
            Case::new("test-1", Arc::new(pass_func)),
            Case::new("test-2", Arc::new(pass_func)),
        ],
        ..Default::default()
    };

    let exit_code = run(env, definition);
    assert_eq!(exit_code, ExitCode::SUCCESS);
}

#[test]
fn test_one_stage_fails() {
    let env = HashMap::from([
        ("STACKCLASS_REPOSITORY_DIR".to_string(), "examples/echo-tester".to_string()),
        ("STACKCLASS_TEST_CASES_JSON".to_string(), build_test_cases_json(&["test-1", "test-2"])),
    ]);

    let definition = Definition {
        executable_name: "your_program.sh".to_string(),
        cases: vec![
            Case::new("test-1", Arc::new(pass_func)),
            Case::new("test-2", Arc::new(fail_func)),
        ],
        ..Default::default()
    };

    let exit_code = run(env, definition);
    assert_eq!(exit_code, ExitCode::FAILURE);
}
