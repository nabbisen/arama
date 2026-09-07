use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
    time::{SystemTime, UNIX_EPOCH},
};

use sha2::{Digest, Sha256};

use super::{
    CONFIG_JSON, GENERATION_MANIFEST, ModelContainer, ModelDownloadStatus, OPERATION_METADATA,
    PublishFilesystem, SAFETENSORS_MODEL, SourceUrl, acquire_model_lock,
    acquire_model_lock_with_timeout, download_entry, finish_generation, next_operation_sequence,
    publish_generation_with, reconcile_generations, reconcile_generations_with, select_generation,
    sha256_hex, supervise_generation, wait_for_generation,
};

struct FailingPublishFilesystem {
    fail_rename_calls: Vec<usize>,
    fail_remove: bool,
    rename_calls: AtomicUsize,
}

impl FailingPublishFilesystem {
    fn failing(calls: &[usize]) -> Self {
        Self {
            fail_rename_calls: calls.to_vec(),
            fail_remove: false,
            rename_calls: AtomicUsize::new(0),
        }
    }

    fn failing_cleanup() -> Self {
        Self {
            fail_rename_calls: Vec::new(),
            fail_remove: true,
            rename_calls: AtomicUsize::new(0),
        }
    }
}

impl PublishFilesystem for FailingPublishFilesystem {
    fn rename(&self, from: &std::path::Path, to: &std::path::Path) -> std::io::Result<()> {
        let call = self.rename_calls.fetch_add(1, Ordering::SeqCst) + 1;
        if self.fail_rename_calls.contains(&call) {
            return Err(std::io::Error::other(format!(
                "injected rename failure at call {call}"
            )));
        }
        fs::rename(from, to)
    }

    fn remove_dir_all(&self, path: &std::path::Path) -> std::io::Result<()> {
        if self.fail_remove {
            return Err(std::io::Error::other("injected cleanup failure"));
        }
        fs::remove_dir_all(path)
    }
}

fn unique_model_name(suffix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before UNIX epoch")
        .as_nanos();
    format!("test-{suffix}-{}-{nanos}", std::process::id())
}

fn leaked_digest(bytes: &[u8]) -> &'static str {
    Box::leak(sha256_hex(&Sha256::digest(bytes)).into_boxed_str())
}

fn response(status: &str, body: &[u8]) -> Vec<u8> {
    let mut response = format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    response.extend_from_slice(body);
    response
}

fn response_with_declared_length(body: &[u8], declared_length: usize) -> Vec<u8> {
    let mut response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {declared_length}\r\nConnection: close\r\n\r\n"
    )
    .into_bytes();
    response.extend_from_slice(body);
    response
}

fn serve_responses(responses: Vec<Vec<u8>>, first_delay: Duration) -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback fixture");
    let address = listener.local_addr().expect("fixture address");
    let requests = Arc::new(AtomicUsize::new(0));
    let observed = requests.clone();
    thread::spawn(move || {
        for (index, response) in responses.into_iter().enumerate() {
            let (mut stream, _) = listener.accept().expect("accept fixture request");
            observed.fetch_add(1, Ordering::SeqCst);
            let mut request = Vec::new();
            let mut buffer = [0_u8; 512];
            while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                let read = stream.read(&mut buffer).expect("read fixture request");
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..read]);
            }
            if index == 0 {
                thread::sleep(first_delay);
            }
            stream.write_all(&response).expect("write fixture response");
            stream.flush().expect("flush fixture response");
        }
    });
    (format!("http://{address}"), requests)
}

/// Task 036: writes one HTTP response across several separate socket
/// writes with a delay between each, rather than one `write_all` -
/// `serve_responses`' single write lets the kernel/TLS layer coalesce a
/// small body into one `reqwest` chunk, which cannot exercise "more than
/// one progress value for a multi-chunk transfer" reliably. Serves
/// exactly one request, once.
fn serve_chunked_response(header: &str, body_chunks: &[&[u8]], between: Duration) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback fixture");
    let address = listener.local_addr().expect("fixture address");
    let header = header.to_owned();
    let chunks: Vec<Vec<u8>> = body_chunks.iter().map(|chunk| chunk.to_vec()).collect();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept fixture request");
        let mut request = Vec::new();
        let mut buffer = [0_u8; 512];
        while !request.windows(4).any(|window| window == b"\r\n\r\n") {
            let read = stream.read(&mut buffer).expect("read fixture request");
            if read == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..read]);
        }
        stream
            .write_all(header.as_bytes())
            .expect("write fixture header");
        stream.flush().expect("flush fixture header");
        for chunk in &chunks {
            thread::sleep(between);
            stream.write_all(chunk).expect("write fixture chunk");
            stream.flush().expect("flush fixture chunk");
        }
    });
    format!("http://{address}")
}

fn test_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("test runtime")
}

/// Task 046 (audit B6): every test model built through this (or
/// [`single_file_model`]/[`paired_file_model`] below) resolves under its
/// own scratch directory (`ModelContainer::root_override`) instead of the
/// real platform data directory - real installs were accumulating
/// `model/test-*` directories and `.test-*.lock` files indefinitely, and
/// worse, no test was actually isolated from whatever happened to already
/// be on the machine running it. The returned `TempDir` must be kept
/// alive (bound to a variable, not `_`) for as long as the model is used,
/// including across an `.await` inside `test_runtime().block_on(...)`,
/// since dropping it deletes the directory.
fn model_container(
    name: String,
    source_url: SourceUrl,
    expected_sha256: &'static str,
    config_expected_sha256: Option<&'static str>,
    max_model_bytes: u64,
    max_config_bytes: Option<u64>,
) -> (tempfile::TempDir, ModelContainer) {
    let scratch = tempfile::TempDir::new().expect("scratch models directory");
    let model = ModelContainer {
        name,
        source_url,
        expected_sha256,
        config_expected_sha256,
        max_model_bytes,
        max_config_bytes,
        root_override: Some(scratch.path().to_path_buf()),
    };
    (scratch, model)
}

fn single_file_model(
    name: String,
    url: String,
    body: &[u8],
) -> (tempfile::TempDir, ModelContainer) {
    model_container(
        name,
        SourceUrl::ModelSafetensors(url),
        leaked_digest(body),
        None,
        1024,
        None,
    )
}

fn paired_file_model(
    name: String,
    url: String,
    model_body: &[u8],
    config_body: &[u8],
) -> (tempfile::TempDir, ModelContainer) {
    model_container(
        name,
        SourceUrl::ModelSafetensorsConfigJson((url.clone(), url)),
        leaked_digest(model_body),
        Some(leaked_digest(config_body)),
        1024,
        Some(1024),
    )
}

fn cleanup_model(model: &ModelContainer) {
    if let Ok(directory) = model.model_dir() {
        let _ = fs::remove_dir_all(directory);
    }
}

#[path = "tests/lifecycle.rs"]
mod lifecycle;
#[path = "tests/publication.rs"]
mod publication;
#[path = "tests/specification.rs"]
mod specification;
