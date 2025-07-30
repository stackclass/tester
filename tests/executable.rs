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

use std::{fs, io::Write, path::PathBuf, time::Duration};
use tempfile::tempdir;
use tester::{Executable, TesterError};

fn create_test_executable(name: &str, content: &str) -> (PathBuf, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let path = dir.path().join(name);
    let mut file = fs::File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();

    // Flush and sync the file to disk
    file.sync_all().unwrap();

    // Sync the parent directory to ensure metadata is fully written
    let parent_dir = std::fs::File::open(dir.path()).unwrap();
    parent_dir.sync_all().unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        // Sync metadata changes (optional but thorough)
        let _ = std::fs::File::open(&path).unwrap().sync_all();

        // Wait until the file is executable and not busy
        loop {
            let metadata = fs::metadata(&path).unwrap();
            if metadata.permissions().mode() & 0o111 != 0 {
                // Try to open the file in read-only mode to check if it's busy
                if fs::File::open(&path).is_ok() {
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    #[cfg(windows)]
    {
        // On Windows, ensure the file is executable by setting the appropriate attributes
        use std::os::windows::fs::PermissionsExt;
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_readonly(false);
        fs::set_permissions(&path, permissions).unwrap();
    }

    (path, dir)
}

#[cfg(unix)]
#[test]
fn test_start() {
    // Test non-existent executable
    let err = Executable::new(PathBuf::from("/nonexistent")).unwrap_err();
    assert!(matches!(err, TesterError::ExecutableNotFound(_)));

    use std::os::unix::fs::PermissionsExt;
    let (path, _dir) = create_test_executable("not_executable", "");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    let err = Executable::new(path).unwrap_err();
    assert!(matches!(err, TesterError::ProcessExecution(_)));

    // Test valid executable
    let (path, _dir) = create_test_executable("echo.sh", "#!/bin/sh\necho \"$@\"");
    let mut exe = Executable::new(path).unwrap();
    assert!(exe.start(&[]).is_ok());
}

#[cfg(unix)]
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

#[cfg(unix)]
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

#[cfg(unix)]
#[test]
fn test_exit_code() {
    let (path, _dir) = create_test_executable("exit.sh", "#!/bin/sh\nexit $1");
    let mut exe = Executable::new(path).unwrap();

    let (_, _, status) = exe.run(&["0"]).unwrap();
    assert!(status.success());

    let (_, _, status) = exe.run(&["1"]).unwrap();
    assert!(!status.success());
}

#[cfg(windows)]
#[test]
fn test_exit_code() {
    let (path, _dir) = create_test_executable("exit.bat", "@echo off\nexit /b %1");
    let mut exe = Executable::new(path).unwrap();

    let (_, _, status) = exe.run(&["0"]).unwrap();
    assert!(status.success());

    let (_, _, status) = exe.run(&["1"]).unwrap();
    assert!(!status.success());
}

#[cfg(unix)]
#[test]
fn test_double_start() {
    let (path, _dir) = create_test_executable("sleep.sh", "#!/bin/sh\nsleep 1");
    let mut exe = Executable::new(path).unwrap();

    exe.start(&[]).unwrap();
    let err = exe.start(&[]).unwrap_err();
    assert!(matches!(err, TesterError::ProcessAlreadyRunning));
}

#[cfg(unix)]
#[test]
fn test_timeout() {
    let (path, _dir) = create_test_executable("sleep.sh", "#!/bin/sh\nsleep 10");
    let mut exe = Executable::new(path).unwrap().with_timeout(Duration::from_millis(100));

    exe.start(&[]).unwrap();
    let err = exe.wait().unwrap_err();
    assert!(matches!(err, TesterError::WaitTimeout(_)));
}

#[cfg(unix)]
#[test]
fn test_kill_after_timeout() {
    let (path, _dir) = create_test_executable("sleep.sh", "#!/bin/sh\nsleep 10");
    let mut exe = Executable::new(path).unwrap().with_timeout(Duration::from_millis(100));

    exe.start(&[]).unwrap();
    let _ = exe.wait();
    assert!(!exe.is_running());
}
