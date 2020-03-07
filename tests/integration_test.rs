extern crate ctftool;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI32, Ordering};

use assert_cmd::cargo::cargo_bin;
use assert_cmd::Command;
use ctftool::{ctf, git};
use hyper::server::Server;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response};
use lazy_static::lazy_static;
use log::{info, warn};
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

#[test]
fn test_exe() -> Result<()> {
    ctftool::init_logging();
    let work_dir = WorkDir::new()?;
    let mut command = Command::cargo_bin("ctf")?;
    command
        .args(&["init"])
        .current_dir(work_dir.temp_dir.path())
        .assert()
        .success();
    Ok(())
}

async fn main(work_dir: &WorkDir, args: &[&str]) -> Result<()> {
    let mut args_with_0 = vec!["ctftool"];
    args_with_0.extend(args);
    ctftool::main(args_with_0.iter(), work_dir.temp_dir.path().to_path_buf()).await
}

#[tokio::main]
async fn main_sync(work_dir: &WorkDir, args: &[&str]) -> Result<()> {
    main(work_dir, args).await
}

#[test]
fn test_init() -> Result<()> {
    ctftool::init_logging();
    let work_dir = WorkDir::new()?;
    main_sync(&work_dir, &["init"])?;
    assert!(main_sync(&work_dir, &["init"]).is_err());
    Ok(())
}

#[test]
fn test_remote() -> Result<()> {
    ctftool::init_logging();
    let work_dir = WorkDir::new()?;
    main_sync(&work_dir, &["init"])?;
    main_sync(&work_dir, &["remote", "show"])?;
    let url = "http://localhost.test";
    main_sync(&work_dir, &["remote", "add", "origin", url])?;
    assert!(main_sync(&work_dir, &["remote", "add", "origin", url]).is_err());
    main_sync(&work_dir, &["remote", "show"])?;
    main_sync(&work_dir, &["remote", "rm", "origin"])?;
    assert!(main_sync(&work_dir, &["remote", "rm", "origin"]).is_err());
    Ok(())
}

struct StaticServer {
    port: u16,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
    shutdown_complete_rx: tokio::sync::oneshot::Receiver<()>,
}

lazy_static! {
    static ref REQUEST_COUNTER: AtomicI32 = AtomicI32::new(0);
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
                    let request_id = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
                    info!("#{} {} {}", request_id, req.method(), req.uri());
                    let root = root.clone();
                    async move {
                        let response = match StaticServer::handle_request(&root, req).await {
                            Ok(response) => response,
                            Err(e) => {
                                warn!("{}", e);
                                Response::builder()
                                    .status(500)
                                    .body(Body::from(""))
                                    .unwrap()
                            }
                        };
                        info!("#{} {}", request_id, response.status());
                        Ok::<_, Error>(response)
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
    let url = format!("http://localhost:{}", server.port);
    let work_dir = WorkDir::new()?;
    main(&work_dir, &["init"]).await?;
    main(&work_dir, &["remote", "add", "origin", &url]).await?;
    let mut ctx = ctf::load(work_dir.temp_dir.path().to_path_buf())?;
    let remote = ctf::find_remote_mut(&mut ctx.ctf, "origin")?;
    remote.rewrite_rules.push(ctf::RewriteRule {
        regex: "^https://drive.google.com/".into(),
        rep: format!("{}/drive.google.com/", url),
    });
    git::commit(&ctx, "Add rewrite rules")?;
    main(&work_dir, &["fetch"]).await?;
    server.shutdown().await?;
    Ok(())
}

#[tokio::main]
#[test]
async fn test_fetch_watevr() -> Result<()> {
    ctftool::init_logging();
    test_fetch("watevr").await
}

#[tokio::main]
#[test]
async fn test_fetch_ctfd() -> Result<()> {
    ctftool::init_logging();
    test_fetch("ctfd").await
}
