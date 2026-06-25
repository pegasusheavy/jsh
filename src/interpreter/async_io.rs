//! Async I/O utilities for external command execution
//!
//! Provides non-blocking I/O for pipeline stages and background jobs.

use crossbeam_channel::{bounded, Receiver, Sender};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStderr, ChildStdout, Stdio};
use std::thread::{self, JoinHandle};

/// Size of the I/O buffer
const IO_BUFFER_SIZE: usize = 8192;

/// Maximum number of output lines to buffer
const MAX_BUFFERED_LINES: usize = 10000;

/// Output from an async command
#[derive(Debug, Clone)]
pub enum CommandOutput {
    /// A line of stdout
    Stdout(String),
    /// A line of stderr
    Stderr(String),
    /// Command completed with exit code
    Done(i32),
    /// An error occurred
    Error(String),
}

/// Handle to an async command execution
pub struct AsyncCommand {
    /// Thread handle for stdout reader
    stdout_thread: Option<JoinHandle<()>>,
    /// Thread handle for stderr reader
    stderr_thread: Option<JoinHandle<()>>,
    /// Thread handle for process waiter
    wait_thread: Option<JoinHandle<i32>>,
    /// Receiver for output
    pub output_rx: Receiver<CommandOutput>,
    /// Child process handle (for killing)
    child: Option<Child>,
}

impl AsyncCommand {
    /// Create a new async command from a child process
    pub fn from_child(mut child: Child) -> Self {
        let (tx, rx) = bounded::<CommandOutput>(MAX_BUFFERED_LINES);

        // Spawn stdout reader thread
        let stdout_thread = if let Some(stdout) = child.stdout.take() {
            let tx_stdout = tx.clone();
            Some(thread::spawn(move || {
                read_lines_to_channel(stdout, tx_stdout, false);
            }))
        } else {
            None
        };

        // Spawn stderr reader thread
        let stderr_thread = if let Some(stderr) = child.stderr.take() {
            let tx_stderr = tx.clone();
            Some(thread::spawn(move || {
                read_lines_to_channel(stderr, tx_stderr, true);
            }))
        } else {
            None
        };

        // Spawn process waiter thread
        let wait_thread = {
            let tx_wait = tx;
            Some(thread::spawn(move || {
                match child.wait() {
                    Ok(status) => {
                        let code = status.code().unwrap_or(-1);
                        let _ = tx_wait.send(CommandOutput::Done(code));
                        code
                    }
                    Err(e) => {
                        let _ = tx_wait.send(CommandOutput::Error(e.to_string()));
                        -1
                    }
                }
            }))
        };

        Self {
            stdout_thread,
            stderr_thread,
            wait_thread,
            output_rx: rx,
            child: None, // Already moved to wait thread
        }
    }

    /// Wait for the command to complete and return exit code
    pub fn wait(mut self) -> i32 {
        // Wait for reader threads
        if let Some(handle) = self.stdout_thread.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.stderr_thread.take() {
            let _ = handle.join();
        }

        // Wait for process and get exit code
        if let Some(handle) = self.wait_thread.take() {
            handle.join().unwrap_or(-1)
        } else {
            -1
        }
    }

    /// Try to receive output without blocking
    pub fn try_recv(&self) -> Option<CommandOutput> {
        self.output_rx.try_recv().ok()
    }

    /// Receive output, blocking until available
    pub fn recv(&self) -> Option<CommandOutput> {
        self.output_rx.recv().ok()
    }

    /// Collect all output into vectors
    pub fn collect_output(mut self) -> (Vec<String>, Vec<String>, i32) {
        let mut stdout_lines = Vec::new();
        let mut stderr_lines = Vec::new();
        let mut exit_code = 0;

        // Drain the channel until every sender (both reader threads and the
        // wait thread) has been dropped. We must NOT stop as soon as `Done`
        // arrives: `child.wait()` can return before a reader thread has flushed
        // its final line, so breaking on `Done` would race and drop output.
        // Because the channel is bounded, we keep receiving concurrently with
        // the reader threads (rather than joining them first, which could
        // deadlock once a reader fills the buffer with no one draining it).
        // The `iter()` ends naturally when all senders are gone, by which point
        // all output has been received.
        for output in self.output_rx.iter() {
            match output {
                CommandOutput::Stdout(line) => stdout_lines.push(line),
                CommandOutput::Stderr(line) => stderr_lines.push(line),
                CommandOutput::Done(code) => exit_code = code,
                CommandOutput::Error(e) => {
                    stderr_lines.push(format!("Error: {}", e));
                    exit_code = -1;
                }
            }
        }

        // All output drained; reap the worker threads so they don't linger.
        if let Some(handle) = self.stdout_thread.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.stderr_thread.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.wait_thread.take() {
            let _ = handle.join();
        }

        (stdout_lines, stderr_lines, exit_code)
    }
}

/// Read lines from a reader and send to channel
fn read_lines_to_channel<R: Read>(reader: R, tx: Sender<CommandOutput>, is_stderr: bool) {
    let buf_reader = BufReader::with_capacity(IO_BUFFER_SIZE, reader);

    for line in buf_reader.lines() {
        match line {
            Ok(l) => {
                let output = if is_stderr {
                    CommandOutput::Stderr(l)
                } else {
                    CommandOutput::Stdout(l)
                };
                if tx.send(output).is_err() {
                    break; // Receiver dropped
                }
            }
            Err(_) => break,
        }
    }
}

/// Pipe buffer for connecting pipeline stages
pub struct PipeBuffer {
    tx: Sender<Vec<u8>>,
    rx: Receiver<Vec<u8>>,
}

impl PipeBuffer {
    /// Create a new pipe buffer
    pub fn new() -> Self {
        let (tx, rx) = bounded(64); // Buffer up to 64 chunks
        Self { tx, rx }
    }

    /// Get the sender (write end)
    pub fn sender(&self) -> Sender<Vec<u8>> {
        self.tx.clone()
    }

    /// Get the receiver (read end)
    pub fn receiver(&self) -> Receiver<Vec<u8>> {
        self.rx.clone()
    }
}

impl Default for PipeBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Writer that sends to a channel
pub struct ChannelWriter {
    tx: Sender<Vec<u8>>,
    buffer: Vec<u8>,
}

impl ChannelWriter {
    pub fn new(tx: Sender<Vec<u8>>) -> Self {
        Self {
            tx,
            buffer: Vec::with_capacity(IO_BUFFER_SIZE),
        }
    }
}

impl Write for ChannelWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);

        // Flush when buffer is full
        if self.buffer.len() >= IO_BUFFER_SIZE {
            self.flush()?;
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if !self.buffer.is_empty() {
            let data = std::mem::take(&mut self.buffer);
            self.tx
                .send(data)
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "channel closed"))?;
        }
        Ok(())
    }
}

impl Drop for ChannelWriter {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

/// Reader that receives from a channel
pub struct ChannelReader {
    rx: Receiver<Vec<u8>>,
    buffer: Vec<u8>,
    pos: usize,
}

impl ChannelReader {
    pub fn new(rx: Receiver<Vec<u8>>) -> Self {
        Self {
            rx,
            buffer: Vec::new(),
            pos: 0,
        }
    }
}

impl Read for ChannelReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // If buffer is exhausted, try to get more data
        if self.pos >= self.buffer.len() {
            match self.rx.recv() {
                Ok(data) => {
                    self.buffer = data;
                    self.pos = 0;
                }
                Err(_) => return Ok(0), // EOF
            }
        }

        // Copy from buffer to output
        let available = self.buffer.len() - self.pos;
        let to_copy = available.min(buf.len());
        buf[..to_copy].copy_from_slice(&self.buffer[self.pos..self.pos + to_copy]);
        self.pos += to_copy;

        Ok(to_copy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn test_pipe_buffer() {
        let pipe = PipeBuffer::new();
        let tx = pipe.sender();
        let rx = pipe.receiver();

        tx.send(vec![1, 2, 3]).unwrap();
        let received = rx.recv().unwrap();
        assert_eq!(received, vec![1, 2, 3]);
    }

    #[test]
    fn test_channel_writer_reader() {
        let pipe = PipeBuffer::new();
        let mut writer = ChannelWriter::new(pipe.sender());
        let mut reader = ChannelReader::new(pipe.receiver());

        // Spawn writer thread
        let write_handle = thread::spawn(move || {
            writer.write_all(b"hello world").unwrap();
            writer.flush().unwrap();
        });

        // Read from reader
        let mut buf = vec![0u8; 20];
        let n = reader.read(&mut buf).unwrap();

        write_handle.join().unwrap();

        assert_eq!(&buf[..n], b"hello world");
    }

    #[test]
    #[cfg(unix)]
    fn test_async_command() {
        let child = Command::new("echo")
            .arg("hello")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let async_cmd = AsyncCommand::from_child(child);
        let (stdout, stderr, code) = async_cmd.collect_output();

        assert_eq!(code, 0);
        assert_eq!(stdout.len(), 1);
        assert_eq!(stdout[0], "hello");
        assert!(stderr.is_empty());
    }
}

