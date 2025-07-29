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

use tester::{Case, Definition};

fn main() -> ExitCode {
    // Collect existing environment variables
    let mut env: HashMap<String, String> = std::env::vars().collect();

    // Add STACKCLASS_REPOSITORY_DIR pointing to the current directory (./)
    if std::env::var("STACKCLASS_REPOSITORY_DIR").is_err() {
        env.insert("STACKCLASS_REPOSITORY_DIR".to_string(), "examples/echo-tester".to_string());
    }

    // Add STACKCLASS_TEST_CASES_JSON with a simple test case
    if std::env::var("STACKCLASS_TEST_CASES_JSON").is_err() {
        env.insert(
            "STACKCLASS_TEST_CASES_JSON".to_string(),
            r#"[{"slug": "hello-world", "title": "Hello World Test", "log_prefix": "hello"}]"#
                .to_string(),
        );
    }

    // Create a minimal test definition with a Hello World test case
    let definition = Definition {
        executable_name: "your_program.sh".to_string(),
        cases: vec![Case::new("hello-world", Arc::new(|_| Ok(())))],
        ..Default::default()
    };

    // Run the test CLI
    tester::run(env, definition)
}
