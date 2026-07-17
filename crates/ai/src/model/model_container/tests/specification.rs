use super::*;

#[test]
fn ready_for_single_file_model_requires_authenticated_manifest() {
    let model = ModelContainer {
        name: unique_model_name("single"),
        source_url: SourceUrl::ModelSafetensors("https://example.invalid/model".to_owned()),
        expected_sha256: "unused",
        config_expected_sha256: None,
        max_model_bytes: 1024,
        max_config_bytes: None,
    };

    assert!(!model.clone().ready().expect("ready check"));

    let safetensors_path = model.safetensors_path().expect("safetensors path");
    fs::create_dir_all(safetensors_path.parent().expect("model dir")).expect("create model dir");
    fs::write(&safetensors_path, b"model").expect("write safetensors");

    assert!(!model.clone().ready().expect("ready check"));
    fs::write(
        safetensors_path
            .parent()
            .expect("model dir")
            .join(GENERATION_MANIFEST),
        "wrong-generation",
    )
    .expect("write mismatched manifest");
    assert!(!model.clone().ready().expect("ready check"));
    fs::write(
        safetensors_path
            .parent()
            .expect("model dir")
            .join(GENERATION_MANIFEST),
        model.identity(),
    )
    .expect("write generation manifest");
    assert!(model.clone().ready().expect("ready check"));
    fs::remove_dir_all(safetensors_path.parent().expect("model dir")).expect("cleanup model dir");
}

#[test]
fn ready_for_config_model_requires_safetensors_and_config() {
    let model = ModelContainer {
        name: unique_model_name("config"),
        source_url: SourceUrl::ModelSafetensorsConfigJson((
            "https://example.invalid/model".to_owned(),
            "https://example.invalid/config".to_owned(),
        )),
        expected_sha256: "unused",
        config_expected_sha256: Some("unused"),
        max_model_bytes: 1024,
        max_config_bytes: Some(1024),
    };

    assert!(!model.clone().ready().expect("ready check"));

    let safetensors_path = model.safetensors_path().expect("safetensors path");
    fs::create_dir_all(safetensors_path.parent().expect("model dir")).expect("create model dir");
    fs::write(&safetensors_path, b"model").expect("write safetensors");

    assert!(!model.clone().ready().expect("ready check"));

    let config_path = model.config_json_path().expect("config path");
    fs::write(&config_path, b"{}").expect("write config");

    assert!(!model.clone().ready().expect("ready check"));
    fs::write(
        safetensors_path
            .parent()
            .expect("model dir")
            .join(GENERATION_MANIFEST),
        model.identity(),
    )
    .expect("write generation manifest");
    assert!(model.clone().ready().expect("ready check"));
    fs::remove_dir_all(safetensors_path.parent().expect("model dir")).expect("cleanup model dir");
}

#[test]
fn authenticated_download_helper_formats_hex_digest() {
    assert_eq!(
        sha256_hex(&Sha256::digest(b"arama")),
        "0d22554a4efcf5eb5aa3bef02fa51ce1a1c8ba77fe45d6d959148250c1211702"
    );
}

#[test]
fn public_constructor_rejects_unsafe_names_and_unpinned_digests() {
    assert!(
        ModelContainer::new(
            "../escape",
            SourceUrl::ModelSafetensors("https://example.invalid/model".to_owned()),
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
            1024,
            None,
        )
        .is_err()
    );
    assert!(
        ModelContainer::new(
            "safe-name",
            SourceUrl::ModelSafetensors("https://example.invalid/model".to_owned()),
            "unused",
            None,
            1024,
            None,
        )
        .is_err()
    );
    assert!(
        ModelContainer::new(
            "safe-name",
            SourceUrl::ModelSafetensors("https://example.invalid/model".to_owned()),
            "0000000000000000000000000000000000000000000000000000000000000000",
            None,
            1024,
            Some(1024),
        )
        .is_err()
    );
}

#[test]
fn registry_rejects_conflicting_immutable_specifications_for_one_name() {
    let name = unique_model_name("conflicting-spec");
    let first = single_file_model(
        name.clone(),
        "https://example.invalid/first".to_owned(),
        b"first",
    );
    let second = single_file_model(
        name.clone(),
        "https://example.invalid/second".to_owned(),
        b"second",
    );

    download_entry(&name, &first.identity()).expect("register first specification");
    assert!(download_entry(&name, &second.identity()).is_err());
}
