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
    acquire_model_lock_with_timeout, download_entry, finish_generation, models_dir,
    next_operation_sequence, publish_generation_with, reconcile_generations,
    reconcile_generations_with, select_generation, sha256_hex, supervise_generation,
    wait_for_generation,
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

fn test_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("test runtime")
}

fn single_file_model(name: String, url: String, body: &[u8]) -> ModelContainer {
    ModelContainer {
        name,
        source_url: SourceUrl::ModelSafetensors(url),
        expected_sha256: leaked_digest(body),
        config_expected_sha256: None,
        max_model_bytes: 1024,
        max_config_bytes: None,
    }
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
