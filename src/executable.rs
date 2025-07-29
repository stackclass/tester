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

use crate::{Result, TesterError};
use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
    process::{Child, Command, ExitStatus, Stdio},
    sync::{Arc, Mutex, mpsc},
    thread,
    time::{Duration, Instant},
};

/// Represents an executable process with configurable execution parameters.
///
/// This struct provides methods to start, manage, and interact with a child process,
/// including reading its output, writing to its stdin, and handling timeouts.
#[derive(Debug)]
pub struct Executable {
    /// Path to the executable file.
    path: PathBuf,

    /// Maximum duration the process is allowed to run before timing out.
    timeout: Duration,

    /// Optional working directory for the process.
    working_dir: Option<PathBuf>,

    /// Handle to the child process, wrapped in an `Arc<Mutex>` for thread safety.
    process: Option<Arc<Mutex<Child>>>,

    /// Captured stdout of the process, if available.
    stdout: Option<Vec<u8>>,

    /// Captured stderr of the process, if available.
    stderr: Option<Vec<u8>>,

    /// Receiver for capturing stdout and stderr asynchronously.
    rx: Option<mpsc::Receiver<(Vec<u8>, bool)>>,
}

/// Creates a shallow clone of the `Executable`.
impl Clone for Executable {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            timeout: self.timeout,
            working_dir: self.working_dir.clone(),
            process: self.process.clone(),
            stdout: self.stdout.clone(),
            stderr: self.stderr.clone(),
            rx: None,
        }
    }
}

/// Ensures the process is killed when the `Executable` is dropped.
impl Drop for Executable {
    fn drop(&mut self) {
        if self.is_running() {
            let _ = self.kill(); // Ignore errors during drop
        }
    }
}

impl Executable {
    /// Creates a new `Executable` instance.
    pub fn new(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            return Err(TesterError::ExecutableNotFound(path));
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&path)?;
            if metadata.permissions().mode() & 0o111 == 0 {
                return Err(TesterError::ProcessExecution("File is not executable".to_string()));
            }
        }

        Ok(Self {
            path,
            timeout: Duration::from_secs(10),
            working_dir: None,
            process: None,
            stdout: None,
            stderr: None,
            rx: None,
        })
    }

    /// Sets a custom timeout for the process.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the working directory for the process.
    pub fn with_working_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }

    /// Checks if the process is currently running.
    pub fn is_running(&self) -> bool {
        if let Some(process) = &self.process {
            let mut process = process.lock().unwrap();
            match process.try_wait() {
                Ok(None) => true,     // Process is still running
                Ok(Some(_)) => false, // Process has exited
                Err(_) => false,      // Assume process is not running if try_wait fails
            }
        } else {
            false
        }
    }

    /// Starts the process with the given arguments.
    pub fn start(&mut self, args: &[&str]) -> Result<()> {
        if self.is_running() {
            return Err(TesterError::ProcessAlreadyRunning);
        }

        let mut cmd = Command::new(&self.path);
        cmd.args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());

        if let Some(dir) = &self.working_dir {
            cmd.current_dir(dir);
        }

        let mut process = cmd.spawn().map_err(|e| TesterError::ProcessExecution(e.to_string()))?;

        // Use a bounded channel to avoid unbounded memory usage
        let (tx, rx) = mpsc::sync_channel(1024);
        let mut stdout = process.stdout.take().ok_or(TesterError::StdoutCaptureFailed)?;
        let mut stderr = process.stderr.take().ok_or(TesterError::StderrCaptureFailed)?;

        // Spawn a thread to capture stdout asynchronously.
        let tx1 = tx.clone();
        thread::spawn(move || {
            let mut stdout_buf = Vec::new();
            match stdout.read_to_end(&mut stdout_buf) {
                Err(_) => {
                    let _ = tx1.send((Vec::new(), true));
                }
                Ok(_) => {
                    let _ = tx1.send((stdout_buf, true));
                }
            }
        });

        // Spawn a thread to capture stderr asynchronously.
        thread::spawn(move || {
            let mut stderr_buf = Vec::new();
            match stderr.read_to_end(&mut stderr_buf) {
                Err(_) => {
                    let _ = tx.send((Vec::new(), false));
                }
                Ok(_) => {
                    let _ = tx.send((stderr_buf, false));
                }
            }
        });

        self.process = Some(Arc::new(Mutex::new(process)));
        self.rx = Some(rx);

        Ok(())
    }

    /// Writes data to the process's stdin.
    pub fn write_stdin(&mut self, input: &[u8]) -> Result<()> {
        if !self.is_running() {
            return Err(TesterError::NoProcessRunning);
        }
        let mut process = self.process.as_mut().unwrap().lock().unwrap();
        let stdin = process.stdin.as_mut().ok_or(TesterError::StdinCaptureFailed)?;
        stdin.write_all(input).map_err(|e| TesterError::ProcessExecution(e.to_string()))?;

        Ok(())
    }

    /// Waits for the process to complete and returns its output.
    pub fn wait(&mut self) -> Result<(Vec<u8>, Vec<u8>, ExitStatus)> {
        let start = Instant::now();
        let process = self.process.as_mut().ok_or(TesterError::NoProcessRunning)?;
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        loop {
            let status = {
                let mut process = process.lock().unwrap();
                process.try_wait()?
            };

            if let Some(status) = status {
                if let Some(rx) = &self.rx {
                    for (buf, is_stdout) in rx.try_iter() {
                        if is_stdout {
                            stdout = buf;
                        } else {
                            stderr = buf;
                        }
                    }
                }
                self.process = None;
                return Ok((stdout, stderr, status));
            }

            if start.elapsed() > self.timeout {
                self.kill()?;
                return Err(TesterError::WaitTimeout(self.timeout));
            }

            thread::sleep(Duration::from_millis(100));
        }
    }

    /// Kills the process.
    pub fn kill(&mut self) -> Result<()> {
        if let Some(process) = &self.process {
            let mut process = process.lock().unwrap();
            process.kill().map_err(|e| TesterError::ProcessKillFailed(e.to_string()))?;
        }
        self.process = None;

        Ok(())
    }

    /// Starts the process, writes stdin, and waits for completion.
    pub fn run_with_stdin(
        &mut self,
        stdin: &[u8],
        args: &[&str],
    ) -> Result<(Vec<u8>, Vec<u8>, ExitStatus)> {
        self.start(args)?;
        self.write_stdin(stdin)?;
        // Close stdin after writing to signal EOF
        if let Some(process) = &self.process {
            let mut process = process.lock().unwrap();
            process.stdin.take();
        }
        self.wait()
    }

    /// Non-blocking check for process status.
    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>> {
        if let Some(process) = &self.process {
            let mut process = process.lock().unwrap();
            Ok(process.try_wait()?)
        } else {
            Ok(None)
        }
    }
}
