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

use crate::{Definition, Tester};
use std::process::ExitCode;

/// Executes the provided test definition and returns an exit code.
pub fn run(definition: Definition) -> ExitCode {
    // Create a new tester instance
    let tester = match Tester::new(definition) {
        Ok(tester) => tester,
        Err(err) => {
            eprintln!("Error creating tester: {err}");
            return ExitCode::FAILURE;
        }
    };

    // Prints the debug context if debugging is enabled.
    tester.print_debug_context();

    // Early exit if validation fails
    if let Err(err) = tester.validate() {
        eprintln!("Validation failed: {err}");
        return ExitCode::FAILURE;
    }

    // Execute test stages. Return failure if any stage fails.
    if !tester.run() {
        return ExitCode::FAILURE;
    }

    // All stages passed successfully.
    ExitCode::SUCCESS
}
