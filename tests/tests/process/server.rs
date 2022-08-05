use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use url::Url;

use crate::process::child::BackgroundChild;

pub struct ServerProcess {
    url: Url,
    _process: BackgroundChild,
}

impl ServerProcess {
    pub fn new(name: &'static str, cmd_builder: impl FnOnce(u16) -> Command) -> Result<ServerProcess> {
        // Allow tests to run in parallel (in theory) by finding any open port
        // available for our driver. We can't bind the port for the driver, but
        // hopefully the OS gives this invocation unique ports across processes
        let server_addr = TcpListener::bind("127.0.0.1:0")?.local_addr()?;

        // Spawn the driver binary, collecting its stdout/stderr in separate
        // threads. We'll print this output later.
        let mut cmd = cmd_builder(server_addr.port());
        let mut process = BackgroundChild::spawn(name, &mut cmd)?;

        // Wait for the driver to come online and bind its port before we try to
        // connect to it.
        let start = Instant::now();
        let max = Duration::new(5, 0);
        let mut bound = false;
        while start.elapsed() < max {
            if TcpStream::connect(&server_addr).is_ok() {
                bound = true;
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
        if !bound {
            process.stop(true);
            bail!("server failed to bind port during startup")
        }

        let url = Url::parse(&format!("http://{}", server_addr))?;

        Ok(ServerProcess { url, _process: process })
    }

    pub fn get_url(&self) -> &Url {
        &self.url
    }
}
