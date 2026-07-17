use super::*;

use crate::model::model_manager::ModelManager;

fn write_ready_generation(path: &std::path::Path, model: &ModelContainer, body: &[u8], order: u64) {
    fs::create_dir_all(path).expect("create generation");
    fs::write(path.join(SAFETENSORS_MODEL), body).expect("write model");
    fs::write(path.join(GENERATION_MANIFEST), model.identity()).expect("write manifest");
    fs::write(path.join(OPERATION_METADATA), order.to_string()).expect("write ordering metadata");
}

#[test]
fn restart_reconciliation_promotes_only_authenticated_complete_stage() {
    let model = single_file_model(
        unique_model_name("recover-stage"),
        "https://example.invalid/model".to_owned(),
        b"model",
    );
    let root = models_dir().expect("models root");
    fs::create_dir_all(&root).expect("create models root");
    let stale = root.join(format!(".{}.stage-restarted", model.name()));
    let backup = root.join(format!(".{}.backup-restarted", model.name()));
    fs::create_dir(&stale).expect("create staged generation");
    fs::create_dir(&backup).expect("create backup generation");
    fs::write(stale.join(SAFETENSORS_MODEL), b"model").expect("stage model");
    fs::write(stale.join(GENERATION_MANIFEST), model.identity()).expect("stage manifest");
    fs::write(backup.join("prior"), b"old").expect("backup prior model");

    test_runtime()
        .block_on(reconcile_generations(&model, &root))
        .expect("reconcile staged generation");

    assert!(model.clone().ready().expect("recovered readiness"));
    assert!(!stale.exists());
    assert!(!backup.exists());
    cleanup_model(&model);
}

#[test]
fn restart_reconciliation_restores_backup_when_no_stage_or_final_exists() {
    let model = single_file_model(
        unique_model_name("recover-backup"),
        "https://example.invalid/model".to_owned(),
        b"model",
    );
    let root = models_dir().expect("models root");
    fs::create_dir_all(&root).expect("create models root");
    let backup = root.join(format!(".{}.backup-restarted", model.name()));
    write_ready_generation(&backup, &model, b"restored", 10);

    test_runtime()
        .block_on(reconcile_generations(&model, &root))
        .expect("restore backup generation");

    assert!(model.clone().ready().expect("restored readiness"));
    assert_eq!(
        fs::read(model.safetensors_path().expect("model path")).expect("restored model"),
        b"restored"
    );
    cleanup_model(&model);
}

#[test]
fn restart_reconciliation_uses_durable_order_for_multiple_backups() {
    let model = single_file_model(
        unique_model_name("ordered-backup"),
        "https://example.invalid/model".to_owned(),
        b"model",
    );
    let root = models_dir().expect("models root");
    fs::create_dir_all(&root).expect("create models root");
    let lexically_later_old = root.join(format!(".{}.backup-z-old", model.name()));
    let lexically_earlier_new = root.join(format!(".{}.backup-a-new", model.name()));
    write_ready_generation(&lexically_later_old, &model, b"old", 10);
    write_ready_generation(&lexically_earlier_new, &model, b"new", 20);

    test_runtime()
        .block_on(reconcile_generations(&model, &root))
        .expect("restore newest backup generation");

    assert_eq!(
        fs::read(model.safetensors_path().expect("model path")).expect("ordered model"),
        b"new"
    );
    assert!(!lexically_later_old.exists());
    cleanup_model(&model);
}

#[test]
fn restart_reconciliation_skips_newer_mismatched_backup_for_matching_backup() {
    let model = single_file_model(
        unique_model_name("matching-backup"),
        "https://example.invalid/model".to_owned(),
        b"model",
    );
    let root = models_dir().expect("models root");
    fs::create_dir_all(&root).expect("create models root");
    let matching = root.join(format!(".{}.backup-matching", model.name()));
    let mismatched = root.join(format!(".{}.backup-mismatched", model.name()));
    write_ready_generation(&matching, &model, b"matching", 10);
    write_ready_generation(&mismatched, &model, b"mismatched", 20);
    fs::write(
        mismatched.join(GENERATION_MANIFEST),
        "different-specification",
    )
    .expect("replace mismatched manifest");

    test_runtime()
        .block_on(reconcile_generations(&model, &root))
        .expect("restore matching backup");

    assert_eq!(
        fs::read(model.safetensors_path().expect("model path")).expect("restored model"),
        b"matching"
    );
    assert!(!mismatched.exists());
    cleanup_model(&model);
}

#[test]
fn restart_reconciliation_ignores_mismatched_stage_and_restores_backup() {
    let model = single_file_model(
        unique_model_name("mismatched-stage"),
        "https://example.invalid/model".to_owned(),
        b"model",
    );
    let root = models_dir().expect("models root");
    fs::create_dir_all(&root).expect("create models root");
    let stage = root.join(format!(".{}.stage-mismatch", model.name()));
    let backup = root.join(format!(".{}.backup-valid", model.name()));
    write_ready_generation(&stage, &model, b"untrusted", 30);
    fs::write(stage.join(GENERATION_MANIFEST), "different-specification")
        .expect("replace stage manifest");
    write_ready_generation(&backup, &model, b"trusted", 20);

    test_runtime()
        .block_on(reconcile_generations(&model, &root))
        .expect("ignore mismatched stage");

    assert_eq!(
        fs::read(model.safetensors_path().expect("model path")).expect("backup model"),
        b"trusted"
    );
    assert!(!stage.exists());
    cleanup_model(&model);
}

#[test]
fn restart_reconciliation_quarantines_incomplete_final_before_backup_restore() {
    let model = single_file_model(
        unique_model_name("incomplete-final"),
        "https://example.invalid/model".to_owned(),
        b"model",
    );
    let root = models_dir().expect("models root");
    let final_directory = root.join(model.name());
    let backup = root.join(format!(".{}.backup-valid", model.name()));
    fs::create_dir_all(&final_directory).expect("create incomplete final");
    fs::write(final_directory.join(SAFETENSORS_MODEL), b"incomplete")
        .expect("write incomplete final");
    write_ready_generation(&backup, &model, b"trusted", 20);

    test_runtime()
        .block_on(reconcile_generations(&model, &root))
        .expect("restore over incomplete final");

    assert_eq!(
        fs::read(model.safetensors_path().expect("model path")).expect("restored model"),
        b"trusted"
    );
    assert!(
        fs::read_dir(&root)
            .expect("models entries")
            .filter_map(std::result::Result::ok)
            .any(|entry| entry
                .file_name()
                .to_string_lossy()
                .starts_with(&format!(".{}.incomplete-", model.name())))
    );
    cleanup_model(&model);
    for entry in fs::read_dir(&root).expect("models entries") {
        let entry = entry.expect("model entry");
        if entry
            .file_name()
            .to_string_lossy()
            .starts_with(&format!(".{}.incomplete-", model.name()))
        {
            fs::remove_dir_all(entry.path()).expect("cleanup quarantined final");
        }
    }
}

#[test]
fn restart_reconciliation_treats_cleanup_failure_as_repeatable_ready_warning() {
    let model = single_file_model(
        unique_model_name("reconcile-cleanup"),
        "https://example.invalid/model".to_owned(),
        b"model",
    );
    let root = models_dir().expect("models root");
    let final_directory = root.join(model.name());
    let stale = root.join(format!(".{}.stage-stale", model.name()));
    write_ready_generation(&final_directory, &model, b"ready", 20);
    write_ready_generation(&stale, &model, b"stale", 10);

    for _ in 0..2 {
        test_runtime()
            .block_on(reconcile_generations_with(
                &FailingPublishFilesystem::failing_cleanup(),
                &model,
                &root,
            ))
            .expect("cleanup warning does not fail ready generation");
    }
    assert!(model.clone().ready().expect("final remains ready"));
    assert!(stale.exists());
    cleanup_model(&model);
    fs::remove_dir_all(stale).expect("cleanup stale fixture");
}

#[test]
fn cleanup_warning_does_not_redownload_authenticated_final() {
    let body = b"ready model";
    let (url, requests) = serve_responses(vec![response("200 OK", body)], Duration::ZERO);
    let model = single_file_model(unique_model_name("cleanup-no-redownload"), url, body);
    test_runtime()
        .block_on(model.download())
        .expect("initial authenticated download");
    let root = models_dir().expect("models root");
    let stale = root.join(format!(".{}.stage-stale", model.name()));
    write_ready_generation(&stale, &model, b"stale", 1);

    test_runtime()
        .block_on(reconcile_generations_with(
            &FailingPublishFilesystem::failing_cleanup(),
            &model,
            &root,
        ))
        .expect("cleanup warning preserves success");
    test_runtime()
        .block_on(model.download())
        .expect("ready retry performs housekeeping without download");

    assert_eq!(requests.load(Ordering::SeqCst), 1);
    assert!(model.clone().ready().expect("final remains ready"));
    assert!(!stale.exists());
    cleanup_model(&model);
}

#[test]
fn manager_construction_does_not_recreate_final_during_publication_window() {
    let model = single_file_model(
        unique_model_name("manager-publication-window"),
        "https://example.invalid/model".to_owned(),
        b"model",
    );
    let root = models_dir().expect("models root");
    fs::create_dir_all(&root).expect("create models root");
    let final_directory = root.join(model.name());
    let backup = root.join(format!(".{}.backup-window", model.name()));
    let staging = root.join(format!(".{}.stage-window", model.name()));
    write_ready_generation(&final_directory, &model, b"old", 10);
    write_ready_generation(&staging, &model, b"new", 20);
    fs::rename(&final_directory, &backup).expect("enter publication window");

    let _manager = ModelManager::new(model.clone()).expect("construct manager");

    assert!(!final_directory.exists());
    fs::rename(&staging, &final_directory).expect("activate staged generation");
    assert!(model.clone().ready().expect("new final ready"));
    fs::remove_dir_all(backup).expect("cleanup backup");
    cleanup_model(&model);
}

#[test]
fn publication_activation_failure_restores_prior_generation() {
    let root = models_dir()
        .expect("models root")
        .join(unique_model_name("publish-restore"));
    let staging = root.join("staging");
    let final_directory = root.join("final");
    let backup = root.join("backup");
    fs::create_dir_all(&staging).expect("create staging");
    fs::create_dir_all(&final_directory).expect("create final");
    fs::write(staging.join("value"), b"new").expect("write staged value");
    fs::write(final_directory.join("value"), b"old").expect("write prior value");

    let result = publish_generation_with(
        &FailingPublishFilesystem::failing(&[2]),
        &staging,
        &final_directory,
        &backup,
    );

    assert!(result.is_err());
    assert_eq!(
        fs::read(final_directory.join("value")).expect("restored value"),
        b"old"
    );
    assert!(staging.exists());
    assert!(!backup.exists());
    fs::remove_dir_all(root).expect("cleanup publication fixture");
}

#[test]
fn publication_backup_failure_leaves_prior_generation_untouched() {
    let root = models_dir()
        .expect("models root")
        .join(unique_model_name("publish-backup"));
    let staging = root.join("staging");
    let final_directory = root.join("final");
    let backup = root.join("backup");
    fs::create_dir_all(&staging).expect("create staging");
    fs::create_dir_all(&final_directory).expect("create final");
    fs::write(staging.join("value"), b"new").expect("write staged value");
    fs::write(final_directory.join("value"), b"old").expect("write prior value");

    let result = publish_generation_with(
        &FailingPublishFilesystem::failing(&[1]),
        &staging,
        &final_directory,
        &backup,
    );

    assert!(result.is_err());
    assert_eq!(
        fs::read(final_directory.join("value")).expect("prior value"),
        b"old"
    );
    assert!(staging.exists());
    assert!(!backup.exists());
    fs::remove_dir_all(root).expect("cleanup publication fixture");
}

#[test]
fn publication_restore_failure_retains_backup_for_reconciliation() {
    let root = models_dir()
        .expect("models root")
        .join(unique_model_name("publish-retain"));
    let staging = root.join("staging");
    let final_directory = root.join("final");
    let backup = root.join("backup");
    fs::create_dir_all(&staging).expect("create staging");
    fs::create_dir_all(&final_directory).expect("create final");
    fs::write(final_directory.join("value"), b"old").expect("write prior value");

    let result = publish_generation_with(
        &FailingPublishFilesystem::failing(&[2, 3]),
        &staging,
        &final_directory,
        &backup,
    );

    let error = result.expect_err("publication and restoration must fail");
    assert!(
        error
            .to_string()
            .contains("failed to restore prior generation")
    );
    assert!(!final_directory.exists());
    assert_eq!(
        fs::read(backup.join("value")).expect("retained backup"),
        b"old"
    );
    fs::remove_dir_all(root).expect("cleanup publication fixture");
}

#[test]
fn publication_cleanup_failure_keeps_new_generation_and_recoverable_backup() {
    let root = models_dir()
        .expect("models root")
        .join(unique_model_name("publish-cleanup"));
    let staging = root.join("staging");
    let final_directory = root.join("final");
    let backup = root.join("backup");
    fs::create_dir_all(&staging).expect("create staging");
    fs::create_dir_all(&final_directory).expect("create final");
    fs::write(staging.join("value"), b"new").expect("write staged value");
    fs::write(final_directory.join("value"), b"old").expect("write prior value");

    publish_generation_with(
        &FailingPublishFilesystem::failing_cleanup(),
        &staging,
        &final_directory,
        &backup,
    )
    .expect("cleanup failure is recoverable after successful activation");

    assert_eq!(
        fs::read(final_directory.join("value")).expect("new value"),
        b"new"
    );
    assert_eq!(
        fs::read(backup.join("value")).expect("recoverable backup"),
        b"old"
    );
    fs::remove_dir_all(root).expect("cleanup publication fixture");
}
