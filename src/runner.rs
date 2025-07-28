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

use std::{sync::mpsc, thread};
use tracing::{Level, error, info, span};

use crate::{Case, Executable, Harness};

/// Represents a test runner that executes a sequence of test steps.
pub struct Runner<'a> {
    steps: Vec<Step<'a>>,
    _is_quiet: bool, // Used for anti-cheat tests, where only critical logs are emitted.
}

/// Represents a single step in a test runner.
pub struct Step<'a> {
    /// The test case to be executed.
    pub case: &'a Case,
    /// A prefix used for logging purposes (e.g., `"stage-1"`).
    pub log_prefix: &'a str,
    /// A human-readable title for the test step (e.g., `"Stage #1: Bind to a port"`).
    pub title: &'a str,
}

impl<'a> Runner<'a> {
    /// Creates a new `Runner` with the given steps.
    pub fn new(steps: Vec<Step<'a>>) -> Self {
        Self { steps, _is_quiet: false }
    }

    /// Creates a new `Runner` with quiet mode enabled.
    pub fn new_quiet(steps: Vec<Step<'a>>) -> Self {
        Self { steps, _is_quiet: true }
    }

    /// Executes all test steps in sequence.
    pub fn run(&self, is_debug: bool, executable: &Executable) -> bool {
        for (index, step) in self.steps.iter().enumerate() {
            if index != 0 {
                println!();
            }

            let span =
                span!(Level::INFO, "test_step", log_prefix = step.log_prefix, title = step.title);
            let _enter = span.enter();

            info!("Running tests for {}", step.title);

            let harness = Harness::new(executable.clone());
            let (tx, rx) = mpsc::channel();

            let case_function = step.case.function.clone();
            let harness_clone = harness.clone();

            thread::spawn(move || {
                let result = case_function(&harness_clone);
                tx.send(result).unwrap();
            });

            let timeout = step.case.default_timeout();

            let result = match rx.recv_timeout(timeout) {
                Ok(Ok(())) => {
                    info!("Test passed.");
                    true
                }
                Ok(Err(err)) => {
                    self.report_test_error(&err, is_debug);
                    false
                }
                Err(_) => {
                    let err = format!("timed out, test exceeded {} seconds", timeout.as_secs());
                    self.report_test_error(&err, is_debug);
                    false
                }
            };

            harness.run_teardown_funcs();

            if !result {
                return false;
            }
        }

        true
    }

    /// Reports a test error with appropriate logging.
    fn report_test_error(&self, err: &impl std::fmt::Display, is_debug: bool) {
        error!("{}", err);

        if is_debug {
            error!("Test failed");
        } else {
            error!(
                "Test failed (try setting 'debug: true' in your codecrafters.yml to see more details)"
            );
        }
    }
}
