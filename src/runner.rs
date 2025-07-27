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

use std::path::Path;

use crate::Case;

pub struct Runner<'a> {
    _steps: Vec<Step<'a>>,
}

pub struct Step<'a> {
    pub case: &'a Case,
    pub log_prefix: String,
    pub title: String,
}

impl<'a> Runner<'a> {
    pub fn new(steps: Vec<Step<'a>>) -> Self {
        Self { _steps: steps }
    }

    pub fn run(&self, _is_debug: bool, _executable: &Path) -> bool {
        // Implementation would run each step and return success/failure
        unimplemented!()
    }
}
