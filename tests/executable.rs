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
use tester::{Executable, TesterError};

#[cfg(unix)]
#[test]
fn test_start() {
    // Test non-existent executable
    let err = Executable::new(PathBuf::from("/nonexistent")).unwrap_err();
    assert!(matches!(err, TesterError::ExecutableNotFound(_)));

    // Test valid executable
    let path = PathBuf::from("tests/bin/echo.sh");
    let mut exe = Executable::new(path).unwrap();
    assert!(exe.start(&[]).is_ok());
}

#[cfg(unix)]
#[test]
fn test_start_and_kill() {
    let path = PathBuf::from("tests/bin/sleep.sh");
    let mut exe = Executable::new(path).unwrap();

    // Start and kill
    exe.start(&[]).unwrap();
    assert!(exe.is_running());
    exe.kill().unwrap();
    assert!(!exe.is_running());
}

#[cfg(unix)]
#[test]
fn test_output_capture() {
    // Test stdout capture
    let path = PathBuf::from("tests/bin/stdout.sh");
    let mut exe = Executable::new(path).unwrap();
    let (stdout, stderr, _) = exe.run(&["test"]).unwrap();
    assert_eq!(stdout, b"test\n");
    assert!(stderr.is_empty());

    // Test stderr capture
    let path = PathBuf::from("tests/bin/stderr.sh");
    let mut exe = Executable::new(path).unwrap();
    let (stdout, stderr, _) = exe.run(&["test"]).unwrap();
    assert!(stdout.is_empty());
    assert_eq!(stderr, b"test\n");
}

#[cfg(unix)]
#[test]
fn test_exit_code() {
    let path = PathBuf::from("tests/bin/exit.sh");
    let mut exe = Executable::new(path).unwrap();

    let (_, _, status) = exe.run(&["0"]).unwrap();
    assert!(status.success());

    let (_, _, status) = exe.run(&["1"]).unwrap();
    assert!(!status.success());
}

#[cfg(windows)]
#[test]
fn test_exit_code() {
    let path = PathBuf::from("tests/bin/exit.bat");
    let mut exe = Executable::new(path).unwrap();

    let (_, _, status) = exe.run(&["0"]).unwrap();
    assert!(status.success());

    let (_, _, status) = exe.run(&["1"]).unwrap();
    assert!(!status.success());
}

#[cfg(unix)]
#[test]
fn test_double_start() {
    let path = PathBuf::from("tests/bin/sleep.sh");
    let mut exe = Executable::new(path).unwrap();

    exe.start(&[]).unwrap();
    let err = exe.start(&[]).unwrap_err();
    assert!(matches!(err, TesterError::ProcessAlreadyRunning));
}

#[cfg(unix)]
#[test]
fn test_timeout() {
    use std::time::Duration;
    let path = PathBuf::from("tests/bin/sleep.sh");
    let mut exe = Executable::new(path).unwrap().with_timeout(Duration::from_millis(100));

    exe.start(&[]).unwrap();
    let err = exe.wait().unwrap_err();
    assert!(matches!(err, TesterError::WaitTimeout(_)));
}

#[cfg(unix)]
#[test]
fn test_kill_after_timeout() {
    use std::time::Duration;
    let path = PathBuf::from("tests/bin/sleep.sh");
    let mut exe = Executable::new(path).unwrap().with_timeout(Duration::from_millis(100));

    exe.start(&[]).unwrap();
    let _ = exe.wait();
    assert!(!exe.is_running());
}
