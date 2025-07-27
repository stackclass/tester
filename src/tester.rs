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

use std::path::PathBuf;

use crate::{Context, Definition, Result, Runner, Step};

/// Manages the execution environment & runner for test cases.
pub struct Tester {
    context: Context,
    definition: Definition,
}

impl Tester {
    /// Creates a Tester based on the Definition provided
    pub fn new(definition: Definition) -> Result<Self> {
        let env = std::env::vars().collect();
        let context = Context::from_env(env, &definition)?;

        Ok(Self { context, definition })
    }

    /// Runs all stages up to the current stage. Returns true if all stages pass.
    pub fn run(&self) -> bool {
        self.get_runner().run(self.context.is_debug, &self.get_executable())
    }

    /// Prints the debug context if debugging is enabled.
    pub fn print_debug_context(&self) {
        if !self.context.is_debug {
            return;
        }

        println!("{:?}", self.context);
    }

    /// Creates a TestRunner with steps mapped from context and definition test cases.
    fn get_runner(&self) -> Runner {
        let steps: Vec<Step> = self
            .context
            .cases
            .iter()
            .filter_map(|context_case| {
                // Find matching test case in definition
                let definition_case = self.definition.find_case(&context_case.slug)?;

                Some(Step {
                    case: definition_case,
                    log_prefix: context_case.log_prefix.clone(),
                    title: context_case.title.clone(),
                })
            })
            .collect();

        Runner::new(steps)
    }

    /// Gets the executable path from context
    fn get_executable(&self) -> PathBuf {
        self.context.executable_path.clone()
    }

    /// Validates that all test cases in the context have matching test cases in the definition.
    /// Returns an error if any test case in the context does not match the definition.
    pub fn validate(&self) -> Result<()> {
        for context_case in &self.context.cases {
            let definition_case =
                self.definition.find_case(&context_case.slug).ok_or_else(|| {
                    format!(
                        "tester context does not have test case with slug {}",
                        context_case.slug
                    )
                })?;

            if definition_case.slug != context_case.slug {
                return Err(format!(
                    "tester context does not have test case with slug {}",
                    context_case.slug
                )
                .into());
            }
        }

        Ok(())
    }
}
