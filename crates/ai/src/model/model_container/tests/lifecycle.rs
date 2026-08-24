use super::*;

use crate::model::model_manager::ModelManager;

#[test]
fn concurrent_callers_join_one_authenticated_publication() {
    let body = b"authenticated model";
    let (url, requests) =
        serve_responses(vec![response("200 OK", body)], Duration::from_millis(75));
    let model = single_file_model(unique_model_name("joined"), url, body);
    let first = model.clone();
    let second = model.clone();

    test_runtime().block_on(async move {
        let first = tokio::task::spawn(async move { first.download().await });
        let second = tokio::task::spawn(async move { second.download().await });
        first.await.expect("first join").expect("first download");
        second.await.expect("second join").expect("second download");
    });

    assert_eq!(requests.load(Ordering::SeqCst), 1);
    assert!(model.clone().ready().expect("published readiness"));
    assert_eq!(
        fs::read(model.safetensors_path().expect("model path")).expect("published model"),
        body
    );
    cleanup_model(&model);
}

#[test]
fn cancelling_starting_caller_does_not_strand_joiner_or_retry() {
    let body = b"cancellation-safe model";
    let (url, requests) =
        serve_responses(vec![response("200 OK", body)], Duration::from_millis(100));
    let model = single_file_model(unique_model_name("cancel-safe"), url, body);
    let starting = model.clone();
    let joining = model.clone();

    test_runtime().block_on(async {
        let starting = tokio::task::spawn(async move { starting.download().await });
        while requests.load(Ordering::SeqCst) == 0 {
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        let joining = tokio::task::spawn(async move { joining.download().await });
        tokio::task::yield_now().await;
        starting.abort();

        joining
            .await
            .expect("joining caller task")
            .expect("registry-owned worker completes");
    });

    assert_eq!(requests.load(Ordering::SeqCst), 1);
    assert_eq!(model.download_status(), super::ModelDownloadStatus::Ready);
    test_runtime()
        .block_on(model.download())
        .expect("subsequent call observes ready generation");
    cleanup_model(&model);
}

#[test]
fn worker_panic_wakes_joiner_cleans_stage_and_allows_retry() {
    let body = b"panic recovery model";
    let (url, requests) = serve_responses(vec![response("200 OK", body)], Duration::ZERO);
    let model = single_file_model(unique_model_name("panic-retry"), url, body);
    let entry = download_entry(model.name(), &model.identity()).expect("registry entry");
    let (generation, owner) = select_generation(&entry);
    assert!(owner);
    let result = generation.result.subscribe();
    let staging = models_dir()
        .expect("models root")
        .join(model.operation_name("stage", generation.id));
    fs::create_dir_all(&staging).expect("create abandoned stage");
    fs::write(staging.join("partial"), b"partial").expect("write partial stage");

    test_runtime().block_on(async {
        supervise_generation(model.clone(), entry, generation, async {
            panic!("injected generation panic")
        });
        let error = wait_for_generation(result)
            .await
            .expect_err("panic becomes retained failure");
        assert!(error.to_string().contains("terminated before completion"));
    });

    assert!(!staging.exists());
    assert_eq!(model.download_status(), ModelDownloadStatus::Failed);
    test_runtime()
        .block_on(model.download())
        .expect("generation retries after panic");
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    assert_eq!(model.download_status(), ModelDownloadStatus::Ready);
    cleanup_model(&model);
}

#[test]
#[allow(deprecated)]
fn model_manager_ensure_joins_shared_manifested_generation() {
    let body = b"compatibility model";
    let (url, requests) =
        serve_responses(vec![response("200 OK", body)], Duration::from_millis(75));
    let model = single_file_model(unique_model_name("manager-join"), url, body);
    let manager = ModelManager::new(model.clone()).expect("create compatibility manager");
    let direct = model.clone();

    test_runtime().block_on(async move {
        let compatibility = tokio::task::spawn(async move { manager.ensure().await });
        let shared = tokio::task::spawn(async move { direct.download().await });
        compatibility
            .await
            .expect("compatibility task")
            .expect("compatibility result");
        shared.await.expect("shared task").expect("shared result");
    });

    assert_eq!(requests.load(Ordering::SeqCst), 1);
    assert!(model.clone().ready().expect("manifest-backed readiness"));
    cleanup_model(&model);
}

#[test]
fn delayed_joiner_keeps_exact_result_across_many_later_generations() {
    let model = single_file_model(
        unique_model_name("delayed-result"),
        "https://example.invalid/model".to_owned(),
        b"model",
    );
    let entry = download_entry(model.name(), &model.identity()).expect("registry entry");
    let (first, first_owner) = select_generation(&entry);
    assert!(first_owner);
    let delayed = first.result.subscribe();
    let first_error = anyhow::anyhow!("first retained failure");
    finish_generation(&entry, &first, &Err(first_error));

    for generation in 0..16 {
        let (later, owner) = select_generation(&entry);
        assert!(owner);
        let error = anyhow::anyhow!("later failure {generation}");
        finish_generation(&entry, &later, &Err(error));
    }

    let result = test_runtime().block_on(wait_for_generation(delayed));
    assert_eq!(
        result
            .expect_err("first generation remains failed")
            .to_string(),
        "first retained failure"
    );
}

#[test]
fn digest_mismatch_removes_only_operation_owned_staging() {
    let (url, _) = serve_responses(vec![response("200 OK", b"wrong bytes")], Duration::ZERO);
    let model = single_file_model(unique_model_name("digest-failure"), url, b"expected");

    let result = test_runtime().block_on(model.download());

    assert!(result.is_err());
    assert!(!model.model_dir().expect("model dir").exists());
    let root = models_dir().expect("models root");
    assert!(
        !fs::read_dir(root)
            .expect("models entries")
            .filter_map(std::result::Result::ok)
            .any(|entry| {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                name.starts_with(&format!(".{}.stage-", model.name()))
                    || name.starts_with(&format!(".{}.backup-", model.name()))
            })
    );
}

#[test]
fn config_failure_preserves_preexisting_generation_files() {
    let model_body = b"new model";
    let (url, requests) = serve_responses(
        vec![
            response("200 OK", model_body),
            response("500 Internal Server Error", b"failed"),
        ],
        Duration::ZERO,
    );
    let name = unique_model_name("config-failure");
    let model = ModelContainer {
        name,
        source_url: SourceUrl::ModelSafetensorsConfigJson((url.clone(), url)),
        expected_sha256: leaked_digest(model_body),
        config_expected_sha256: Some(leaked_digest(b"{}")),
        max_model_bytes: 1024,
        max_config_bytes: Some(1024),
    };
    let final_directory = model.model_dir().expect("final directory");
    fs::create_dir_all(&final_directory).expect("old directory");
    fs::write(final_directory.join(SAFETENSORS_MODEL), b"old model").expect("old model");

    let result = test_runtime().block_on(model.download());

    assert!(result.is_err());
    assert_eq!(requests.load(Ordering::SeqCst), 2);
    assert_eq!(
        fs::read(final_directory.join(SAFETENSORS_MODEL)).expect("preserved old model"),
        b"old model"
    );
    assert!(!final_directory.join(CONFIG_JSON).exists());
    cleanup_model(&model);
}

#[test]
fn declared_oversize_is_rejected_before_publication() {
    let (url, _) = serve_responses(vec![response("200 OK", b"five")], Duration::ZERO);
    let mut model = single_file_model(unique_model_name("oversize"), url, b"five");
    model.max_model_bytes = 3;

    let result = test_runtime().block_on(model.download());

    assert!(result.is_err());
    assert!(!model.model_dir().expect("model dir").exists());
}

#[test]
fn interrupted_body_is_not_published() {
    let (url, _) = serve_responses(
        vec![response_with_declared_length(b"short", 20)],
        Duration::ZERO,
    );
    let model = single_file_model(unique_model_name("interrupted"), url, b"short");

    let result = test_runtime().block_on(model.download());

    assert!(result.is_err());
    assert!(!model.model_dir().expect("model dir").exists());
}

#[test]
fn model_and_config_publish_as_one_complete_generation() {
    let model_body = b"paired model";
    let config_body = b"{\"hidden_size\":768}";
    let (url, requests) = serve_responses(
        vec![
            response("200 OK", model_body),
            response("200 OK", config_body),
        ],
        Duration::ZERO,
    );
    let model = ModelContainer {
        name: unique_model_name("paired-success"),
        source_url: SourceUrl::ModelSafetensorsConfigJson((url.clone(), url)),
        expected_sha256: leaked_digest(model_body),
        config_expected_sha256: Some(leaked_digest(config_body)),
        max_model_bytes: 1024,
        max_config_bytes: Some(1024),
    };

    test_runtime()
        .block_on(model.download())
        .expect("paired generation download");

    assert_eq!(requests.load(Ordering::SeqCst), 2);
    assert!(model.clone().ready().expect("paired readiness"));
    assert_eq!(
        fs::read(model.safetensors_path().expect("model path")).expect("model bytes"),
        model_body
    );
    assert_eq!(
        fs::read(model.config_json_path().expect("config path")).expect("config bytes"),
        config_body
    );
    cleanup_model(&model);
}

/// Task 036: the progress path must emit more than one value for a
/// multi-chunk transfer - the defect this task fixes is that
/// `DownloadProgress::Downloading` was never constructed at all, so
/// "it changed more than once" is the actual claim under test, not
/// merely "it reached the end."
#[test]
fn download_with_progress_emits_more_than_one_value_for_a_multi_chunk_transfer() {
    let chunks: [&[u8]; 4] = [b"one-", b"two-", b"three", b"-four"];
    let body_len: usize = chunks.iter().map(|c| c.len()).sum();
    let header =
        format!("HTTP/1.1 200 OK\r\nContent-Length: {body_len}\r\nConnection: close\r\n\r\n");
    let url = serve_chunked_response(&header, &chunks, Duration::from_millis(20));
    let full_body: Vec<u8> = chunks.concat();
    let model = single_file_model(unique_model_name("progress-multi-chunk"), url, &full_body);

    let (observed, result) = test_runtime().block_on(async {
        let (mut progress, download) = model.download_with_progress().expect("progress handle");
        let worker = tokio::task::spawn(download);

        let mut observed = vec![*progress.borrow()];
        while progress.changed().await.is_ok() {
            observed.push(*progress.borrow());
            if observed.last().unwrap().downloaded as usize >= body_len {
                break;
            }
        }
        (observed, worker.await.expect("download task"))
    });

    result.expect("chunked download succeeds");

    let distinct_byte_counts: std::collections::BTreeSet<u64> =
        observed.iter().map(|p| p.downloaded).collect();
    assert!(
        distinct_byte_counts.len() > 1,
        "expected more than one distinct progress value, got {observed:?}"
    );
    assert_eq!(
        observed.last().expect("at least one value").downloaded,
        body_len as u64,
        "the final observed value must be the real total, not a partial one"
    );
    // Real bytes, monotonically non-decreasing - never a value invented
    // to look like animation (Task 036 §4).
    for pair in observed.windows(2) {
        assert!(pair[0].downloaded <= pair[1].downloaded, "{observed:?}");
    }
    // The very first read can legitimately predate the worker even
    // starting (the channel's own initial default, `(0, None)`, set
    // before anything has run) - the real claim is that once a file's
    // response headers have arrived, its declared length is known and
    // stays known for the rest of the transfer.
    assert!(
        observed
            .iter()
            .skip_while(|p| p.total.is_none())
            .all(|p| p.total == Some(body_len as u64)),
        "content-length was declared, so once known it must stay the real total throughout: {observed:?}"
    );
    cleanup_model(&model);
}

/// Task 036 §3: a response with no `Content-Length` at all must not
/// produce a fake percentage - `total` stays `None` for every
/// observation, and the caller is left to show real bytes downloaded
/// instead of dividing by an assumed bound.
#[test]
fn download_with_progress_reports_no_total_when_content_length_is_absent() {
    let body = b"no declared length at all, read until connection close";
    let url = serve_chunked_response(
        "HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n",
        &[body],
        Duration::ZERO,
    );
    let model = single_file_model(unique_model_name("progress-no-length"), url, body);

    let (observed, result) = test_runtime().block_on(async {
        let (mut progress, download) = model.download_with_progress().expect("progress handle");
        let worker = tokio::task::spawn(download);

        let mut observed = vec![*progress.borrow()];
        while progress.changed().await.is_ok() {
            observed.push(*progress.borrow());
        }
        (observed, worker.await.expect("download task"))
    });

    result.expect("length-less download succeeds");

    assert!(
        observed.iter().all(|p| p.total.is_none()),
        "no file in this generation ever reported a length, so total must stay None throughout: {observed:?}"
    );
    assert_eq!(
        observed.last().expect("at least one value").downloaded,
        body.len() as u64,
        "bytes-so-far must still be the real count even with no known total"
    );
    cleanup_model(&model);
}

/// Task 036 §3: a joiner subscribing to an already-partway-through
/// generation must see the current progress immediately, never 0 -
/// `watch::Receiver` semantics (`subscribe()` replays the latest value)
/// is what makes this true by construction, verified here rather than
/// assumed.
#[test]
fn joiner_observes_current_progress_not_zero() {
    let chunks: [&[u8]; 3] = [b"aaaa", b"bbbb", b"cccc"];
    let body_len: usize = chunks.iter().map(|c| c.len()).sum();
    let header =
        format!("HTTP/1.1 200 OK\r\nContent-Length: {body_len}\r\nConnection: close\r\n\r\n");
    let url = serve_chunked_response(&header, &chunks, Duration::from_millis(40));
    let full_body: Vec<u8> = chunks.concat();
    let model = single_file_model(unique_model_name("progress-joiner"), url, &full_body);
    let joiner_model = model.clone();

    let (joined_at, result) = test_runtime().block_on(async {
        let (mut progress, download) = model.download_with_progress().expect("starting handle");
        let worker = tokio::task::spawn(download);

        // Let at least the first chunk land before the joiner arrives.
        while progress.borrow().downloaded == 0 {
            progress.changed().await.expect("progress channel open");
        }

        let (joiner_progress, _joiner_future) = joiner_model
            .download_with_progress()
            .expect("joining handle");
        let joined_at = *joiner_progress.borrow();

        (joined_at, worker.await.expect("download task"))
    });

    result.expect("joined download succeeds");
    assert!(
        joined_at.downloaded > 0,
        "a joiner arriving mid-transfer must not observe 0 bytes: {joined_at:?}"
    );
    cleanup_model(&model);
}

/// Task 036 §3: a generation is more than one file for
/// `ModelSafetensorsConfigJson` sources. `downloaded` must never go
/// backwards across the model-file-to-config-file transition - a
/// percentage that resets to 0% mid-download is explicitly worse than
/// none. The chosen aggregation (§3, recorded in `GenerationProgress`'s
/// own doc comment) grows the known `total` once the config's own
/// response headers arrive rather than assuming it upfront, so `total`
/// is allowed to *increase* once during the transition - checked here
/// too, so the growth is asserted rather than merely not-decreasing.
#[test]
fn download_with_progress_does_not_reset_across_model_and_config_files() {
    let model_body = b"the model file, larger";
    let config_body = b"{}";
    let (url, _requests) = serve_responses(
        vec![
            response("200 OK", model_body),
            response("200 OK", config_body),
        ],
        Duration::ZERO,
    );
    let model = ModelContainer {
        name: unique_model_name("progress-two-files"),
        source_url: SourceUrl::ModelSafetensorsConfigJson((url.clone(), url)),
        expected_sha256: leaked_digest(model_body),
        config_expected_sha256: Some(leaked_digest(config_body)),
        max_model_bytes: 1024,
        max_config_bytes: Some(1024),
    };

    let (observed, result) = test_runtime().block_on(async {
        let (mut progress, download) = model.download_with_progress().expect("progress handle");
        let worker = tokio::task::spawn(download);

        let mut observed = vec![*progress.borrow()];
        let full_total = (model_body.len() + config_body.len()) as u64;
        while progress.changed().await.is_ok() {
            observed.push(*progress.borrow());
            if observed.last().unwrap().downloaded >= full_total {
                break;
            }
        }
        (observed, worker.await.expect("download task"))
    });

    result.expect("two-file download succeeds");

    for pair in observed.windows(2) {
        assert!(
            pair[0].downloaded <= pair[1].downloaded,
            "downloaded must never go backwards across files: {observed:?}"
        );
    }
    assert_eq!(
        observed.last().expect("at least one value").downloaded,
        (model_body.len() + config_body.len()) as u64,
        "final count must be the sum of both files, not just the model file"
    );
    let totals: Vec<Option<u64>> = observed.iter().map(|p| p.total).collect();
    assert!(
        totals.contains(&Some(model_body.len() as u64)),
        "total must reflect the model-only figure while only its headers are known: {totals:?}"
    );
    assert!(
        totals.contains(&Some((model_body.len() + config_body.len()) as u64)),
        "total must grow to include the config file once its own headers arrive: {totals:?}"
    );
    cleanup_model(&model);
}

#[test]
fn failed_generation_can_be_retried_without_stale_state() {
    let body = b"retry model";
    let (url, requests) = serve_responses(
        vec![
            response("500 Internal Server Error", b"failed"),
            response("200 OK", body),
        ],
        Duration::ZERO,
    );
    let model = single_file_model(unique_model_name("retry"), url, body);

    assert!(test_runtime().block_on(model.download()).is_err());
    assert_eq!(model.download_status(), super::ModelDownloadStatus::Failed);
    test_runtime()
        .block_on(model.download())
        .expect("retry succeeds");

    assert_eq!(requests.load(Ordering::SeqCst), 2);
    assert_eq!(model.download_status(), super::ModelDownloadStatus::Ready);
    cleanup_model(&model);
}
