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

use std::{io, path::PathBuf, time::Duration};
use thiserror::Error;

pub type Result<T, E = TesterError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum TesterError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Executable not found at {0}")]
    ExecutableNotFound(PathBuf),

    #[error("Test case timed out after {0:?}")]
    Timeout(Duration),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON parse error: {0}")]
    JsonParse(String),

    #[error("Assertion failed: {0}")]
    AssertionFailed(String),

    #[error("Invalid test case: {0}")]
    InvalidTestCase(String),

    #[error("Process execution failed: {0}")]
    ProcessExecution(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Internal tester error: {0}")]
    InternalError(String),

    #[error("{0}")]
    Custom(String),

    #[error("Process is already running")]
    ProcessAlreadyRunning,

    #[error("No process is currently running")]
    NoProcessRunning,

    #[error("Failed to capture stdin")]
    StdinCaptureFailed,

    #[error("Failed to capture stdout")]
    StdoutCaptureFailed,

    #[error("Failed to capture stderr")]
    StderrCaptureFailed,

    #[error("Failed to wait for process: {0}")]
    ProcessWaitFailed(String),

    #[error("Failed to kill process: {0}")]
    ProcessKillFailed(String),

    #[error("Process wait timed out after {0:?}")]
    WaitTimeout(Duration),
}

impl TesterError {
    /// Creates a new assertion failed error with the given message.
    pub fn assertion<S: Into<String>>(msg: S) -> Self {
        Self::AssertionFailed(msg.into())
    }

    /// Wraps an I/O error with additional context information.
    pub fn io_with_context(err: io::Error, context: &str) -> Self {
        Self::Io(io::Error::new(err.kind(), format!("{context}: {err}")))
    }

    /// Checks if the error is a timeout error.
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout(_) | Self::WaitTimeout(_))
    }

    /// Determines whether the error is recoverable.
    /// Returns false for InternalError and InvalidTestCase variants.
    pub fn is_recoverable(&self) -> bool {
        !matches!(self, Self::InternalError(_) | Self::InvalidTestCase(_))
    }
}

/// Converts a boxed error into a custom TesterError.
impl From<Box<dyn std::error::Error + Send + Sync>> for TesterError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::Custom(err.to_string())
    }
}

/// Converts a String into a custom TesterError.
impl From<String> for TesterError {
    fn from(s: String) -> Self {
        Self::Custom(s)
    }
}

/// Converts a string slice into a custom TesterError.
impl From<&str> for TesterError {
    fn from(s: &str) -> Self {
        Self::Custom(s.to_string())
    }
}

/// Converts a serde_json error into a JSON parse error.
impl From<serde_json::Error> for TesterError {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonParse(err.to_string())
    }
}
