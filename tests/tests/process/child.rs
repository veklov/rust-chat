use std::io::{self, Read, StdoutLock, Write};
use std::process::{Child, Command, Stdio};
use std::thread::{self, JoinHandle};

use anyhow::{Context, Result};

pub struct BackgroundChild {
    name: &'static str,
    child: Option<Child>,
    stdout: Option<JoinHandle<io::Result<Vec<u8>>>>,
    stderr: Option<JoinHandle<io::Result<Vec<u8>>>>,
}

impl BackgroundChild {
    pub fn spawn(
        name: &'static str,
        cmd: &mut Command,
    ) -> Result<BackgroundChild> {
        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .spawn()
            .with_context(|| format!("Failed to spawn {:?} binary", cmd.get_program()))?;
        let mut stdout = child.stdout.take()
            .unwrap(); // Assigned to Some(_) above
        let mut stderr = child.stderr.take()
            .unwrap();  // Assigned to Some(_) above
        let stdout = Some(thread::spawn(move || Self::read(&mut stdout)));
        let stderr = Some(thread::spawn(move || Self::read(&mut stderr)));
        Ok(BackgroundChild { name, child: Some(child), stdout, stderr })
    }

    pub fn stop(&mut self, print_output: bool) {
        if let Some(mut child) = self.child.take() {
            if let Err(error) = child.kill() {
                println!("Could not kill {}: {}, {:?}", self.name, error, error);
            }

            let status = child.wait()
                .map_err(|error|
                    println!("Could not wait for {}: {}, {:?}", self.name, error, error))
                .map(|s| s.to_string())
                .unwrap_or("None".to_owned());

            if !print_output {
                return;
            }

            let mut stdout = io::stdout().lock();
            Self::print_status(&mut stdout, self.name, status);
            Self::print_stream(&mut stdout, self.name, "stdout", &mut self.stdout);
            Self::print_stream(&mut stdout, self.name, "stderr", &mut self.stderr);
        }
    }

    fn print_status(stdout: &mut StdoutLock, process_name: &str, status: String) {
        writeln!(stdout, "{} status: {}", process_name, status)
            .unwrap(); // println! panics too
    }

    fn print_stream(stdout: &mut StdoutLock, process_name: &str, stream_name: &str, stream: &mut Option<JoinHandle<io::Result<Vec<u8>>>>) {
        let stream_data = Self::get_stream_data(process_name, stream_name, stream);
        if stream_data.len() > 0 {
            let formatted_stream_data = Self::tab(&String::from_utf8_lossy(&stream_data));
            writeln!(stdout, "{} stdout:\n{}", process_name, formatted_stream_data)
                .unwrap(); // println! panics too
        }
    }

    fn get_stream_data(process_name: &str, stream_name: &str, stream: &mut Option<JoinHandle<io::Result<Vec<u8>>>>) -> Vec<u8> {
        stream.take()
            .unwrap() // Set to Some(_) in constructor

            .join()
            .map_err(|error|
                println!("Could not join {} reader thread for {}: {:?}", stream_name, process_name, error))
            .unwrap_or(Ok(vec![]))

            .map_err(|error|
                println!("Could not read {} for {}: {}, {:?}", stream_name, process_name, error, error))
            .unwrap_or(vec![])
    }

    fn read<R: Read>(r: &mut R) -> io::Result<Vec<u8>> {
        let mut dst = Vec::new();
        r.read_to_end(&mut dst)?;
        Ok(dst)
    }

    fn tab(s: &str) -> String {
        let mut result = String::new();
        for line in s.lines() {
            result.push_str("    ");
            result.push_str(line);
            result.push_str("\n");
        }
        return result;
    }
}

impl Drop for BackgroundChild {
    fn drop(&mut self) {
        self.stop(std::thread::panicking());
    }
}