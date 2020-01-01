use std::path::{Path, PathBuf};

use assert_cmd::cargo::cargo_bin;
use assert_cmd::Command;
use hyper::server::Server;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response};
use tempdir::TempDir;

use anyhow::{anyhow, Error, Result};

struct WorkDir {
    temp_dir: TempDir,
}

impl WorkDir {
    fn new() -> Result<WorkDir> {
        let temp_dir = TempDir::new("ctf")?;
        Ok(WorkDir { temp_dir })
    }
}

fn command(work_dir: &WorkDir, args: &[&str]) -> Result<Command> {
    let mut command = Command::cargo_bin("ctf")?;
    command.args(args).current_dir(work_dir.temp_dir.path());
    Ok(command)
}

#[test]
fn test_init() -> Result<()> {
    let work_dir = WorkDir::new()?;
    command(&work_dir, &["init"])?.assert().success();
    command(&work_dir, &["init"])?.assert().failure();
    Ok(())
}

#[test]
fn test_remote() -> Result<()> {
    let work_dir = WorkDir::new()?;
    command(&work_dir, &["init"])?.assert().success();
    command(&work_dir, &["remote", "show"])?.assert().success();
    let url = "http://localhost.test";
    command(&work_dir, &["remote", "add", "origin", url])?
        .assert()
        .success();
    command(&work_dir, &["remote", "add", "origin", url])?
        .assert()
        .failure();
    command(&work_dir, &["remote", "show"])?.assert().success();
    command(&work_dir, &["remote", "rm", "origin"])?
        .assert()
        .success();
    command(&work_dir, &["remote", "rm", "origin"])?
        .assert()
        .failure();
    Ok(())
}

struct StaticServer {
    port: u16,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
    shutdown_complete_rx: tokio::sync::oneshot::Receiver<()>,
}

impl StaticServer {
    async fn handle_request(root: &Path, req: Request<Body>) -> Result<Response<Body>> {
        let mut path = root.to_path_buf();
        let uri_path = Path::new(req.uri().path()).strip_prefix("/")?;
        path.push(uri_path);
        if path.is_dir() {
            path.push("index");
        }
        let content = std::fs::read(path)?;
        Ok(Response::new(Body::from(content)))
    }

    fn spawn(root: PathBuf) -> Result<StaticServer> {
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        let (shutdown_complete_tx, shutdown_complete_rx) = tokio::sync::oneshot::channel::<()>();
        let make_svc = make_service_fn(move |_| {
            let root = root.clone();
            async move {
                Ok::<_, Error>(service_fn(move |req| {
                    let root = root.clone();
                    async move {
                        match StaticServer::handle_request(&root, req).await {
                            ok @ Ok(_) => ok,
                            Err(e) => {
                                println!("{}", e);
                                Ok(Response::builder()
                                    .status(500)
                                    .body(Body::from(""))
                                    .unwrap())
                            }
                        }
                    }
                }))
            }
        });
        let server = Server::bind(&([127, 0, 0, 1], 0).into()).serve(make_svc);
        let port = server.local_addr().port();
        let graceful = server.with_graceful_shutdown(async {
            if let Err(e) = shutdown_rx.await {
                println!("{}", e);
            }
        });
        tokio::task::spawn(async {
            if let Err(e) = graceful.await {
                println!("Server error: {}", e);
            }
            if let Err(_) = shutdown_complete_tx.send(()) {
                println!("Could not send shutdown completion notification");
            }
        });
        Ok(StaticServer {
            port,
            shutdown_tx,
            shutdown_complete_rx,
        })
    }

    async fn shutdown(self) -> Result<()> {
        self.shutdown_tx
            .send(())
            .map_err(|_| anyhow!("Could not send shutdown request"))?;
        self.shutdown_complete_rx.await?;
        Ok(())
    }
}

async fn test_fetch(engine: &str) -> Result<()> {
    let mut root = cargo_bin("ctf");
    root.pop();
    root.pop();
    root.pop();
    root.push("tests");
    root.push(engine);
    let server = StaticServer::spawn(root)?;
    tokio::task::spawn_blocking({
        let url = format!("http://localhost:{}", server.port);
        move || -> Result<()> {
            let work_dir = WorkDir::new()?;
            command(&work_dir, &["init"])?.assert().success();
            command(&work_dir, &["remote", "add", "origin", &url])?
                .assert()
                .success();
            command(&work_dir, &["fetch"])?.assert().success();
            Ok(())
        }
    })
    .await??;
    server.shutdown().await?;
    Ok(())
}

#[tokio::main]
#[test]
async fn test_fetch_watevr() -> Result<()> {
    test_fetch("watevr").await
}

#[tokio::main]
#[test]
async fn test_fetch_ctfd() -> Result<()> {
    test_fetch("ctfd").await
}
