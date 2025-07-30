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

use std::{fs, path::PathBuf, time::Duration};
use tempfile::tempdir;
use tester::{Executable, TesterError};

fn create_test_executable(name: &str, content: &str) -> (PathBuf, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let path = dir.path().join(name);
    fs::write(&path, content).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    fs::metadata(&path).unwrap();

    (path, dir)
}

#[test]
fn test_start() {
    // Test non-existent executable
    let err = Executable::new(PathBuf::from("/nonexistent")).unwrap_err();
    assert!(matches!(err, TesterError::ExecutableNotFound(_)));

    // Test non-executable file (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let (path, _dir) = create_test_executable("not_executable", "");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        let err = Executable::new(path).unwrap_err();
        assert!(matches!(err, TesterError::ProcessExecution(_)));
    }

    // Test valid executable
    let (path, _dir) = create_test_executable("echo.sh", "#!/bin/sh\necho \"$@\"");
    let mut exe = Executable::new(path).unwrap();
    assert!(exe.start(&[]).is_ok());
}

#[test]
fn test_start_and_kill() {
    let (path, _dir) = create_test_executable("sleep.sh", "#!/bin/sh\nsleep 10");
    let mut exe = Executable::new(path).unwrap();

    // Start and kill
    exe.start(&[]).unwrap();
    assert!(exe.is_running());
    exe.kill().unwrap();
    assert!(!exe.is_running());
}

#[test]
fn test_output_capture() {
    // Test stdout capture
    let (path, _dir) = create_test_executable("stdout.sh", "#!/bin/sh\necho \"$@\"");
    let mut exe = Executable::new(path).unwrap();
    let (stdout, stderr, _) = exe.run(&["test"]).unwrap();
    assert_eq!(stdout, b"test\n");
    assert!(stderr.is_empty());

    // Test stderr capture
    let (path, _dir) = create_test_executable("stderr.sh", "#!/bin/sh\necho \"$@\" >&2");
    let mut exe = Executable::new(path).unwrap();
    let (stdout, stderr, _) = exe.run(&["test"]).unwrap();
    assert!(stdout.is_empty());
    assert_eq!(stderr, b"test\n");
}

#[test]
fn test_exit_code() {
    let (path, _dir) = create_test_executable("exit.sh", "#!/bin/sh\nexit $1");
    let mut exe = Executable::new(path).unwrap();

    let (_, _, status) = exe.run(&["0"]).unwrap();
    assert!(status.success());

    let (_, _, status) = exe.run(&["1"]).unwrap();
    assert!(!status.success());
}

#[test]
fn test_double_start() {
    let (path, _dir) = create_test_executable("sleep.sh", "#!/bin/sh\nsleep 1");
    let mut exe = Executable::new(path).unwrap();

    exe.start(&[]).unwrap();
    let err = exe.start(&[]).unwrap_err();
    assert!(matches!(err, TesterError::ProcessAlreadyRunning));
}

#[test]
fn test_timeout() {
    let (path, _dir) = create_test_executable("sleep.sh", "#!/bin/sh\nsleep 10");
    let mut exe = Executable::new(path).unwrap().with_timeout(Duration::from_millis(100));

    exe.start(&[]).unwrap();
    let err = exe.wait().unwrap_err();
    assert!(matches!(err, TesterError::WaitTimeout(_)));
}

#[test]
fn test_kill_after_timeout() {
    let (path, _dir) = create_test_executable("sleep.sh", "#!/bin/sh\nsleep 10");
    let mut exe = Executable::new(path).unwrap().with_timeout(Duration::from_millis(100));

    exe.start(&[]).unwrap();
    let _ = exe.wait();
    assert!(!exe.is_running());
}
