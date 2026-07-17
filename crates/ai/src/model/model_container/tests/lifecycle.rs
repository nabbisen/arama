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
