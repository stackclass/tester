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

use crate::Harness;
use std::{error::Error, fmt, sync::Arc, time::Duration};

/// A generic error type that can represent any error implementing `std::error::Error`.
pub type CaseError = Box<dyn Error + Send + Sync>;

/// A function type representing a test case's logic.
pub type Function = Arc<dyn Fn(&Harness) -> Result<(), CaseError> + Send + Sync>;

/// Represents a test case that will be executed against the user's code.
pub struct Case {
    /// Unique identifier for the test case. Must match the stage's slug.
    pub slug: String,

    /// The function that contains the test logic.
    pub function: Function,

    /// Maximum duration the test case is allowed to run.
    pub timeout: Duration,
}

impl Case {
    /// Creates a new `Case` with the given slug and function.
    pub fn new<S: Into<String>>(slug: S, function: Function) -> Self {
        Self { slug: slug.into(), function, timeout: Duration::from_secs(10) }
    }

    /// Returns the timeout duration.
    /// defaulting to 10 seconds if none is specified.
    pub fn default_timeout(&self) -> Duration {
        if self.timeout == Duration::ZERO { Duration::from_secs(10) } else { self.timeout }
    }
}

impl fmt::Debug for Case {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Case").field("slug", &self.slug).field("timeout", &self.timeout).finish()
    }
}
