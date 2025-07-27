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

use crate::case::Case;

/// Represents a test definition, containing metadata and test cases.
#[derive(Debug)]
pub struct Definition {
    /// The name of the executable associated with the test.
    pub executable_name: String,

    /// The legacy name of the executable, if applicable.
    pub legacy_executable_name: Option<String>,

    /// A collection of test cases to be executed.
    pub cases: Vec<Case>,

    /// A collection of anti-cheat test cases for additional validation.
    pub anti_cheat_cases: Vec<Case>,
}

impl Definition {
    /// Finds a test case by its slug.
    pub fn find_case(&self, slug: &str) -> Option<&Case> {
        self.cases.iter().find(|case| case.slug == slug)
    }
}
