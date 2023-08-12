use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;

use clap::Parser;
use futures::future::{join_all, FutureExt};
use futures::TryFutureExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use sha2::{Digest, Sha256};
use tokio::fs::{create_dir_all, rename, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use anyhow::{anyhow, bail, Result};

use crate::ctf;
use crate::git;

#[derive(Parser)]
pub struct Checkout {
    /// Challenges or binaries to checkout
    specs: Vec<String>,
}

async fn hexdigest_1<H>(path: &Path, progress_bar: &ProgressBar) -> Result<String>
where
    H: Digest,
{
    let mut hash = H::new();
    let mut buf = [0_u8; 8192];
    let mut file = File::open(path).await?;
    let meta = file.metadata().await?;
    progress_bar.set_length(meta.len());
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break Ok(hex::encode(hash.finalize()));
        }
        hash.update(&buf[0..n]);
        progress_bar.inc(n as u64);
    }
}

fn mk_progress_bar() -> Result<ProgressBar> {
    Ok(ProgressBar::with_style(
        ProgressBar::hidden(),
        ProgressStyle::default_bar().template("{wide_msg} {bytes}/{total_bytes}")?,
    ))
}

async fn hexdigest(path: &Path, algorithm: &str, progress: &MultiProgress) -> Result<String> {
    let progress_bar = mk_progress_bar()?;
    progress_bar.set_message(format!("{} {}", algorithm.to_uppercase(), path.display()));
    let progress_bar = progress.add(progress_bar);
    let result = match algorithm {
        "sha256" => hexdigest_1::<Sha256>(path, &progress_bar).await,
        _ => Err(anyhow!("Unsupported checksum algorithm: {}", algorithm)),
    };
    if let Err(e) = &result {
        progress_bar.finish_with_message(format!(
            "ERROR {} {}: {}",
            algorithm.to_uppercase(),
            path.display(),
            e,
        ));
    } else {
        progress_bar.finish_and_clear();
    }
    result
}

async fn download_1(path: &Path, url: &str, progress_bar: &ProgressBar) -> Result<()> {
    let mut response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    if let Some(content_length) = response.content_length() {
        progress_bar.set_length(content_length);
    }
    if let Some(parent) = path.parent() {
        create_dir_all(parent).await?;
    }
    let mut file = File::create(path).await?;
    loop {
        let chunk = response.chunk().await?;
        if let Some(chunk) = chunk {
            file.write_all(&chunk).await?;
            progress_bar.inc(chunk.len() as u64);
        } else {
            break Ok(());
        }
    }
}

async fn download(path: &Path, url: &str, progress: &MultiProgress) -> Result<()> {
    let progress_bar = mk_progress_bar()?;
    progress_bar.set_message(format!("DOWNLOAD {}", url));
    let progress_bar = progress.add(progress_bar);
    let result = download_1(path, url, &progress_bar).await;
    if let Err(e) = &result {
        progress_bar.finish_with_message(format!("ERROR DOWNLOAD {}: {}", url, e));
    } else {
        progress_bar.finish_and_clear();
    }
    result
}

async fn checkout_alternative(
    alternative: &ctf::BinaryAlternative,
    path: PathBuf,
    progress: &MultiProgress,
) -> Result<Option<ctf::Checksum>> {
    if let Some(checksum) = &alternative.checksum {
        match hexdigest(&path, &checksum.algorithm, progress).await {
            Ok(value) if value == checksum.value => return Ok(None),
            Ok(_) => {}
            Err(_) => {}
        }
    }
    let tmp_path = path.clone();
    match &alternative.url {
        Some(url) => download(&tmp_path, url, progress).await?,
        None => return Ok(None),
    }
    let result = if let Some(checksum) = &alternative.checksum {
        let value = hexdigest(&path, &checksum.algorithm, progress).await?;
        if value != checksum.value {
            bail!("Checksum error");
        }
        None
    } else {
        let algorithm = "sha256".to_string();
        let value = hexdigest(&path, &algorithm, progress).await?;
        Some(ctf::Checksum { algorithm, value })
    };
    rename(tmp_path, path).await?;
    Ok(result)
}

struct Key {
    challenge: String,
    binary: String,
    alternative: String,
}

type CheckoutFuture<'a> = Pin<Box<dyn Future<Output = Result<Option<ctf::Checksum>>> + Send + 'a>>;

fn checkout_challenge<'a>(
    context: &'a ctf::Context,
    challenge: &'a ctf::Challenge,
    progress: &'a MultiProgress,
) -> Vec<(Key, CheckoutFuture<'a>)> {
    let mut checkouts = Vec::new();
    for binary in &challenge.binaries {
        for alternative in &binary.alternatives {
            let mut path = context.root.clone();
            path.push(&challenge.name);
            let binary_name = format!("{}.{}", binary.name, alternative.name);
            path.push(binary_name);
            checkouts.push((
                Key {
                    challenge: challenge.name.clone(),
                    binary: binary.name.clone(),
                    alternative: alternative.name.clone(),
                },
                checkout_alternative(alternative, path.clone(), progress)
                    .map_err(move |e| e.context(format!("Could not checkout {}", path.display())))
                    .boxed(),
            ));
        }
    }
    checkouts
}

pub async fn run(checkout: Checkout, current_dir: PathBuf) -> Result<()> {
    let progress = Arc::new(MultiProgress::new());
    let mut context = ctf::load(current_dir)?;
    let mut checkouts = Vec::new();
    match context.path.as_slice() {
        [] => {
            if checkout.specs.is_empty() {
                for challenge in &context.ctf.challenges {
                    checkouts.extend(checkout_challenge(&context, challenge, &progress));
                }
            } else {
                for spec in checkout.specs {
                    checkouts.extend(checkout_challenge(
                        &context,
                        ctf::find_challenge(&context.ctf, &spec)?,
                        &progress,
                    ));
                }
            }
        }
        [challenge_name] => {
            if !checkout.specs.is_empty() {
                bail!("ctf checkout does not take arguments when called from challenge directory");
            }
            checkouts.extend(checkout_challenge(
                &context,
                ctf::find_challenge(&context.ctf, challenge_name)?,
                &progress,
            ));
        }
        _ => bail!("ctf checkout must be called from top or challenge directory"),
    }
    let (keys, futures): (Vec<Key>, Vec<CheckoutFuture>) = checkouts.into_iter().unzip();
    let results = join_all(futures).await;
    let mut result: Result<()> = Ok(());
    for (key, single_result) in keys.into_iter().zip(results.into_iter()) {
        match single_result {
            Ok(Some(checksum)) => {
                let challenge = ctf::find_challenge_mut(&mut context.ctf, &key.challenge)?;
                let binary = ctf::find_binary_mut(&mut challenge.binaries, &key.binary)?;
                let alternative = ctf::find_alternative_mut(binary, &key.alternative)?;
                alternative.checksum = Some(checksum);
            }
            Ok(None) => {}
            Err(e) => result = Err(e),
        }
    }
    git::commit(&context, "Checkout")?;
    result
}
