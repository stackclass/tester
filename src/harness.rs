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

use std::{
    fmt,
    sync::{Arc, Mutex},
};

use crate::executable::Executable;

/// Alias for a thread-safe collection of teardown functions.
type TeardownFuncs = Arc<Mutex<Vec<Box<dyn FnOnce() + Send>>>>;

#[derive(Clone)]
pub struct Harness {
    /// Executable is the program to be tested.
    executable: Executable,
    /// Teardown functions are run once the test has completed.
    teardown_funcs: TeardownFuncs,
}

impl Harness {
    /// Creates a new `Harness` with the provided executable.
    pub fn new(executable: Executable) -> Self {
        Harness { executable, teardown_funcs: Arc::new(Mutex::new(Vec::new())) }
    }

    /// Registers a teardown function to be executed after the test completes.
    pub fn register_teardown_func<F>(&self, teardown_func: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let mut funcs = self.teardown_funcs.lock().unwrap();
        funcs.push(Box::new(teardown_func));
    }

    /// Runs all registered teardown functions.
    pub fn run_teardown_funcs(&self) {
        let mut funcs = self.teardown_funcs.lock().unwrap();
        while let Some(func) = funcs.pop() {
            func();
        }
    }

    /// Returns a reference to the executable.
    pub fn executable(&self) -> &Executable {
        &self.executable
    }

    /// Creates a new executable instance (clones the existing one).
    pub fn new_executable(&self) -> Executable {
        self.executable.clone()
    }
}

impl fmt::Debug for Harness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Harness").field("executable", &self.executable).finish()
    }
}
