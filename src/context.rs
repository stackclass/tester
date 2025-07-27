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

use crate::{Definition, Result, TesterError};
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf, time::Duration};

/// Holds all configuration and runtime context for the tester, including
/// environment variables, test cases, and execution settings.
#[derive(Debug)]
pub struct Context {
    /// Path to the executable being tested.
    pub executable_path: PathBuf,

    /// Whether the tester is running in debug mode (controlled by `STACKCLASS_DEBUG`).
    pub is_debug: bool,

    /// List of test cases to execute, parsed from `STACKCLASS_TEST_CASES_JSON`.
    pub cases: Vec<ContextCase>,

    /// Timeout duration for test execution (default: 15 seconds).
    pub timeout: Duration,

    /// Environment variables passed to the tester.
    pub env: HashMap<String, String>,

    /// Whether to skip anti-cheat test cases (controlled by `STACKCLASS_SKIP_ANTI_CHEAT`).
    pub should_skip_anti_cheat: bool,
}

/// Represents a single test case defined in the `STACKCLASS_TEST_CASES_JSON` environment variable.
#[derive(Debug, Deserialize)]
pub struct ContextCase {
    /// Unique identifier for the test case (e.g., "bind-to-port").
    pub slug: String,

    /// Human-readable title of the test case (e.g., "Stage #1: Bind to a port").
    pub title: String,

    /// Prefix for logs emitted during this test case (e.g., "stage-1").
    pub log_prefix: String,
}

impl Context {
    pub fn from_env(env: HashMap<String, String>, definition: &Definition) -> Result<Self> {
        let submission_dir = env
            .get("STACKCLASS_REPOSITORY_DIR")
            .ok_or(TesterError::MissingEnvVar("STACKCLASS_REPOSITORY_DIR".into()))?;

        let test_cases_json = env
            .get("STACKCLASS_TEST_CASES_JSON")
            .ok_or(TesterError::MissingEnvVar("STACKCLASS_TEST_CASES_JSON".into()))?;

        let cases: Vec<ContextCase> = serde_json::from_str(test_cases_json)
            .map_err(|e| TesterError::JsonParse(e.to_string()))?;

        let executable_path = Self::find_executable(submission_dir, definition)?;
        let is_debug = env.get("STACKCLASS_DEBUG").is_some_and(|v| v == "true");
        let timeout = env
            .get("STACKCLASS_TIMEOUT_SECONDS")
            .and_then(|v| v.parse().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(15));

        let should_skip_anti_cheat =
            env.get("STACKCLASS_SKIP_ANTI_CHEAT").is_some_and(|v| v == "true");

        Ok(Self { executable_path, is_debug, cases, timeout, env, should_skip_anti_cheat })
    }

    /// Locates the executable in the submission directory based on the `Definition`.
    fn find_executable(_dir: &str, _definition: &Definition) -> Result<PathBuf> {
        // Implement logic here, e.g., searching for a binary or script.
        // Return `TesterError::ExecutableNotFound` if not found.
        Ok(PathBuf::new())
    }
}
