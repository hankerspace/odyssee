//! First-start runtime bootstrap helpers.

use crate::constants::{IMAGE_MODEL_FILE, IMAGE_MODEL_URL, LLM_MODEL_FILE, LLM_MODEL_URL};
use crate::models::{ModelAssetStatus, ModelPreparationStatus, ModelStatus};
use crate::paths::{ensure_storage, executable_exists, image_sidecar_ready, RuntimePaths};
use serde::Deserialize;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const UNSUPPORTED_FLUX2_IMAGE_MODEL_ERROR: &str = "FLUX.2 Klein requires separate VAE and LLM companion models; \
the current image runner only supports single-file stable-diffusion.cpp models.";
const UNSUPPORTED_FLUX2_IMAGE_MODEL_FILE: &str = "flux-2-klein-base-4b-Q4_0.gguf";

struct ModelAsset {
    label: &'static str,
    file_name: String,
    url: String,
    path: PathBuf,
}

struct ReleaseBinaryAsset {
    label: &'static str,
    file_name: String,
    source_file_names: Vec<String>,
    path: PathBuf,
    release_page_url: &'static str,
    release_api_url: &'static str,
    rule: Result<ReleaseAssetRule, String>,
}

struct ReleaseAssetRule {
    include: &'static [&'static str],
    exclude: &'static [&'static str],
}

#[derive(Deserialize)]
struct GitHubRelease {
    assets: Vec<GitHubReleaseAsset>,
}

#[derive(Clone, Deserialize)]
struct GitHubReleaseAsset {
    name: String,
    browser_download_url: String,
}

/// Builds the model and sidecar readiness payload without starting downloads.
pub(crate) fn get_model_status_for_paths(paths: &RuntimePaths) -> ModelStatus {
    let llm_binary_ready = executable_exists(&paths.llm_binary);
    let image_binary_ready = image_sidecar_ready(paths);
    let llm_model_ready = paths.llm_model.exists();
    let image_model_ready = image_model_ready(&paths.image_model);
    let llm_ready = llm_binary_ready && llm_model_ready;
    let image_ready = image_binary_ready && image_model_ready;

    ModelStatus {
        model_directory: paths.model_dir.display().to_string(),
        cache_directory: paths.cache_dir.display().to_string(),
        llm_binary: paths.llm_binary.display().to_string(),
        image_binary: paths.image_binary.display().to_string(),
        llm_model: paths.llm_model.display().to_string(),
        image_model: paths.image_model.display().to_string(),
        llm_binary_ready,
        image_binary_ready,
        llm_model_ready,
        image_model_ready,
        llm_ready,
        image_ready,
        database_path: paths.database_path.display().to_string(),
        downloads: model_preparation_status(paths),
    }
}

pub(crate) fn prepare_models_for_paths(
    paths: &RuntimePaths,
) -> Result<ModelPreparationStatus, String> {
    ensure_storage(paths)?;
    log::info!(
        "Preparing optional local sidecars and models under '{}'",
        paths.data_dir.display()
    );

    let llm_binary = prepare_release_binary(&llm_binary_asset(paths));
    let image_binary = prepare_release_binary(&image_binary_asset(paths));
    let llm = prepare_asset(&llm_asset(paths));
    let image = prepare_image_asset(&image_asset(paths));

    Ok(ModelPreparationStatus {
        llm_binary,
        image_binary,
        llm,
        image,
    })
}

pub(crate) fn model_preparation_status(paths: &RuntimePaths) -> ModelPreparationStatus {
    log::info!("Checking optional local runtime preparation status");
    ModelPreparationStatus {
        llm_binary: status_for_release_binary(&llm_binary_asset(paths), false, None),
        image_binary: status_for_release_binary(&image_binary_asset(paths), false, None),
        llm: status_for_model_asset(&llm_asset(paths), false, None),
        image: status_for_image_asset(&image_asset(paths), false, None),
    }
}

pub(crate) fn image_model_ready(path: &Path) -> bool {
    path.exists() && unsupported_image_model_error(path).is_none()
}

fn llm_binary_asset(paths: &RuntimePaths) -> ReleaseBinaryAsset {
    ReleaseBinaryAsset {
        label: "llama.cpp sidecar",
        file_name: executable_file_name("llama-cli"),
        source_file_names: executable_file_names(&["llama-cli"]),
        path: paths.llm_binary.clone(),
        release_page_url: "https://github.com/ggml-org/llama.cpp/releases/latest",
        release_api_url: "https://api.github.com/repos/ggml-org/llama.cpp/releases/latest",
        rule: llm_release_rule(),
    }
}

fn image_binary_asset(paths: &RuntimePaths) -> ReleaseBinaryAsset {
    ReleaseBinaryAsset {
        label: "stable-diffusion.cpp sidecar",
        file_name: executable_file_name("sd"),
        source_file_names: executable_file_names(&["sd", "sd-cli"]),
        path: paths.image_binary.clone(),
        release_page_url: "https://github.com/leejet/stable-diffusion.cpp/releases/latest",
        release_api_url: "https://api.github.com/repos/leejet/stable-diffusion.cpp/releases/latest",
        rule: stable_diffusion_release_rule(),
    }
}

fn llm_asset(paths: &RuntimePaths) -> ModelAsset {
    ModelAsset {
        label: "LLM Phi-4 Mini",
        file_name: LLM_MODEL_FILE.to_string(),
        url: LLM_MODEL_URL.to_string(),
        path: paths.llm_model.clone(),
    }
}

fn image_asset(paths: &RuntimePaths) -> ModelAsset {
    ModelAsset {
        label: "Image SD-Turbo",
        file_name: IMAGE_MODEL_FILE.to_string(),
        url: IMAGE_MODEL_URL.to_string(),
        path: paths.image_model.clone(),
    }
}

fn prepare_asset(asset: &ModelAsset) -> ModelAssetStatus {
    if asset.path.exists() {
        log::info!(
            "Model asset already present: label='{}', path='{}'",
            asset.label,
            asset.path.display()
        );
        return status_for_model_asset(asset, false, None);
    }

    log::info!(
        "Model asset missing; attempting explicit download: label='{}'",
        asset.label
    );
    match download_asset(asset) {
        Ok(()) => {
            log::info!(
                "Model asset downloaded: label='{}', path='{}'",
                asset.label,
                asset.path.display()
            );
            status_for_model_asset(asset, true, None)
        }
        Err(error) => {
            log::warn!(
                "Model asset download failed: label='{}', error='{error}'",
                asset.label
            );
            status_for_model_asset(asset, false, Some(error))
        }
    }
}

fn prepare_image_asset(asset: &ModelAsset) -> ModelAssetStatus {
    if unsupported_image_model_error(&asset.path).is_some() {
        log::warn!(
            "Image model is not compatible with the current single-file runner: path='{}'",
            asset.path.display()
        );
        return status_for_image_asset(asset, false, None);
    }

    if asset.path.exists() {
        log::info!(
            "Model asset already present: label='{}', path='{}'",
            asset.label,
            asset.path.display()
        );
        return status_for_image_asset(asset, false, None);
    }

    log::info!(
        "Image model asset missing; attempting explicit download: label='{}'",
        asset.label
    );
    match download_asset(asset) {
        Ok(()) => {
            log::info!(
                "Image model asset downloaded: label='{}', path='{}'",
                asset.label,
                asset.path.display()
            );
            status_for_image_asset(asset, true, None)
        }
        Err(error) => {
            log::warn!(
                "Image model asset download failed: label='{}', error='{error}'",
                asset.label
            );
            status_for_image_asset(asset, false, Some(error))
        }
    }
}

fn prepare_release_binary(asset: &ReleaseBinaryAsset) -> ModelAssetStatus {
    if executable_exists(&asset.path) {
        log::info!(
            "Sidecar already executable: label='{}', path='{}'",
            asset.label,
            asset.path.display()
        );
        return status_for_release_binary(asset, false, None);
    }

    let rule = match &asset.rule {
        Ok(rule) => rule,
        Err(error) => return status_for_release_binary(asset, false, Some(error.clone())),
    };

    log::info!(
        "Sidecar missing; resolving release asset: label='{}'",
        asset.label
    );
    match latest_release_asset(asset.release_api_url, rule)
        .and_then(|release_asset| download_and_install_binary(asset, &release_asset))
    {
        Ok(download_url) => status_for_release_binary_with_url(asset, download_url, true, None),
        Err(error) => {
            log::warn!(
                "Sidecar preparation failed: label='{}', error='{error}'",
                asset.label
            );
            status_for_release_binary(asset, false, Some(error))
        }
    }
}

fn download_asset(asset: &ModelAsset) -> Result<(), String> {
    if let Some(parent) = asset.path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let part_path = partial_path(&asset.path);
    // Write to a temporary .part file so failed downloads never masquerade as ready models.
    let output = File::create(&part_path).map_err(|error| error.to_string())?;
    drop(output);

    let status = Command::new("curl")
        .args([
            "--location",
            "--fail",
            "--silent",
            "--show-error",
            "--output",
        ])
        .arg(&part_path)
        .arg(&asset.url)
        .stdin(Stdio::null())
        .status()
        .map_err(|error| format!("curl indisponible pour télécharger le modèle: {error}"))?;

    if !status.success() {
        let _ = fs::remove_file(&part_path);
        return Err(format!("curl a échoué avec le statut {status}"));
    }

    fs::rename(&part_path, &asset.path).map_err(|error| error.to_string())?;
    Ok(())
}

fn latest_release_asset(
    api_url: &str,
    rule: &ReleaseAssetRule,
) -> Result<GitHubReleaseAsset, String> {
    let output = Command::new("curl")
        .args([
            "--location",
            "--fail",
            "--silent",
            "--show-error",
            "--header",
            "User-Agent: odyssee-kids-runtime-bootstrap",
        ])
        .arg(api_url)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| {
            format!("curl indisponible pour consulter les releases GitHub: {error}")
        })?;

    if !output.status.success() {
        return Err(format!(
            "GitHub releases a répondu avec le statut {}",
            output.status
        ));
    }

    let release: GitHubRelease =
        serde_json::from_slice(&output.stdout).map_err(|error| error.to_string())?;
    select_release_asset(&release.assets, rule)
        .cloned()
        .ok_or_else(|| "aucun binaire compatible trouvé dans la dernière release".to_string())
}

fn download_and_install_binary(
    asset: &ReleaseBinaryAsset,
    release_asset: &GitHubReleaseAsset,
) -> Result<String, String> {
    if let Some(parent) = asset.path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let archive_path = asset
        .path
        .with_file_name(format!("{}.part", release_asset.name));
    let extraction_dir = asset
        .path
        .with_file_name(format!("{}.extract", asset.file_name));
    let _ = fs::remove_file(&archive_path);
    let _ = fs::remove_dir_all(&extraction_dir);

    let download_asset = ModelAsset {
        label: asset.label,
        file_name: release_asset.name.clone(),
        url: release_asset.browser_download_url.clone(),
        path: archive_path.clone(),
    };
    download_asset_file(&download_asset, "le binaire sidecar")?;
    extract_archive(&archive_path, &release_asset.name, &extraction_dir)?;

    let extracted_binary = find_release_binary_file(&extraction_dir, asset).ok_or_else(|| {
        format!(
            "{} introuvable dans l'archive {} (noms cherchés: {})",
            asset.file_name,
            release_asset.name,
            asset.source_file_names.join(", ")
        )
    })?;
    let runtime_source_dir = extracted_binary.parent().unwrap_or(&extraction_dir);
    let runtime_target_dir = asset.path.parent().unwrap_or(&asset.path);
    copy_directory_contents(runtime_source_dir, runtime_target_dir)?;
    let installed_binary = runtime_target_dir.join(
        extracted_binary
            .file_name()
            .ok_or_else(|| "nom de binaire extrait invalide".to_string())?,
    );
    if installed_binary != asset.path {
        fs::copy(&installed_binary, &asset.path).map_err(|error| error.to_string())?;
    }
    make_executable(&asset.path)?;

    let _ = fs::remove_file(&archive_path);
    let _ = fs::remove_dir_all(&extraction_dir);

    if executable_exists(&asset.path) {
        Ok(release_asset.browser_download_url.clone())
    } else {
        Err(format!(
            "{} a été installé mais n'est pas exécutable",
            asset.file_name
        ))
    }
}

fn download_asset_file(asset: &ModelAsset, label: &str) -> Result<(), String> {
    let output = File::create(&asset.path).map_err(|error| error.to_string())?;
    drop(output);

    let status = Command::new("curl")
        .args([
            "--location",
            "--fail",
            "--silent",
            "--show-error",
            "--header",
            "User-Agent: odyssee-kids-runtime-bootstrap",
            "--output",
        ])
        .arg(&asset.path)
        .arg(&asset.url)
        .stdin(Stdio::null())
        .status()
        .map_err(|error| format!("curl indisponible pour télécharger {label}: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        let _ = fs::remove_file(&asset.path);
        Err(format!("curl a échoué avec le statut {status}"))
    }
}

fn extract_archive(
    archive_path: &Path,
    archive_name: &str,
    extraction_dir: &Path,
) -> Result<(), String> {
    fs::create_dir_all(extraction_dir).map_err(|error| error.to_string())?;
    let archive_name = archive_name.to_lowercase();
    let status = if archive_name.ends_with(".zip") {
        extract_zip(archive_path, extraction_dir)?
    } else if archive_name.ends_with(".tar.gz") || archive_name.ends_with(".tgz") {
        Command::new("tar")
            .args(["-xzf"])
            .arg(archive_path)
            .args(["-C"])
            .arg(extraction_dir)
            .stdin(Stdio::null())
            .status()
            .map_err(|error| format!("tar indisponible pour extraire le sidecar: {error}"))?
    } else {
        return Err(format!(
            "format d'archive non pris en charge: {archive_name}"
        ));
    };

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "l'extraction du sidecar a échoué avec le statut {status}"
        ))
    }
}

#[cfg(windows)]
fn extract_zip(
    archive_path: &Path,
    extraction_dir: &Path,
) -> Result<std::process::ExitStatus, String> {
    Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Expand-Archive -Force -LiteralPath $args[0] -DestinationPath $args[1]",
        ])
        .arg(archive_path)
        .arg(extraction_dir)
        .stdin(Stdio::null())
        .status()
        .map_err(|error| format!("PowerShell indisponible pour extraire le sidecar: {error}"))
}

#[cfg(not(windows))]
fn extract_zip(
    archive_path: &Path,
    extraction_dir: &Path,
) -> Result<std::process::ExitStatus, String> {
    Command::new("unzip")
        .args(["-q", "-o"])
        .arg(archive_path)
        .args(["-d"])
        .arg(extraction_dir)
        .stdin(Stdio::null())
        .status()
        .map_err(|error| format!("unzip indisponible pour extraire le sidecar: {error}"))
}

fn find_file_named(directory: &Path, file_name: &str) -> Option<PathBuf> {
    for entry in fs::read_dir(directory).ok()?.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_file_named(&path, file_name) {
                return Some(found);
            }
        } else if path
            .file_name()
            .map(|name| name.to_string_lossy() == file_name)
            .unwrap_or(false)
        {
            return Some(path);
        }
    }
    None
}

fn find_release_binary_file(directory: &Path, asset: &ReleaseBinaryAsset) -> Option<PathBuf> {
    asset
        .source_file_names
        .iter()
        .find_map(|file_name| find_file_named(directory, file_name))
}

fn copy_directory_contents(source: &Path, target: &Path) -> Result<(), String> {
    fs::create_dir_all(target).map_err(|error| error.to_string())?;
    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_directory_contents(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn make_executable(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path)
            .map_err(|error| error.to_string())?
            .permissions();
        permissions.set_mode(permissions.mode() | 0o755);
        fs::set_permissions(path, permissions).map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn status_for_model_asset(
    asset: &ModelAsset,
    downloaded: bool,
    error: Option<String>,
) -> ModelAssetStatus {
    status_for_asset(
        asset.label,
        &asset.file_name,
        &asset.path,
        &asset.url,
        asset.path.exists(),
        downloaded,
        error,
    )
}

fn status_for_image_asset(
    asset: &ModelAsset,
    downloaded: bool,
    error: Option<String>,
) -> ModelAssetStatus {
    let compatibility_error = unsupported_image_model_error(&asset.path).map(str::to_string);
    status_for_asset(
        asset.label,
        &asset.file_name,
        &asset.path,
        &asset.url,
        image_model_ready(&asset.path),
        downloaded,
        error.or(compatibility_error),
    )
}

fn unsupported_image_model_error(path: &Path) -> Option<&'static str> {
    if path.exists()
        && path
            .file_name()
            .map(|name| name.to_string_lossy() == UNSUPPORTED_FLUX2_IMAGE_MODEL_FILE)
            .unwrap_or(false)
    {
        Some(UNSUPPORTED_FLUX2_IMAGE_MODEL_ERROR)
    } else {
        None
    }
}

fn status_for_release_binary(
    asset: &ReleaseBinaryAsset,
    downloaded: bool,
    error: Option<String>,
) -> ModelAssetStatus {
    status_for_release_binary_with_url(asset, asset.release_page_url.to_string(), downloaded, error)
}

fn status_for_release_binary_with_url(
    asset: &ReleaseBinaryAsset,
    url: String,
    downloaded: bool,
    error: Option<String>,
) -> ModelAssetStatus {
    status_for_asset(
        asset.label,
        &asset.file_name,
        &asset.path,
        &url,
        executable_exists(&asset.path),
        downloaded,
        error,
    )
}

fn status_for_asset(
    label: &str,
    file_name: &str,
    path: &Path,
    url: &str,
    ready: bool,
    downloaded: bool,
    error: Option<String>,
) -> ModelAssetStatus {
    ModelAssetStatus {
        label: label.to_string(),
        file_name: file_name.to_string(),
        path: path.display().to_string(),
        url: url.to_string(),
        ready,
        downloaded,
        error,
    }
}

fn select_release_asset<'a>(
    assets: &'a [GitHubReleaseAsset],
    rule: &ReleaseAssetRule,
) -> Option<&'a GitHubReleaseAsset> {
    assets
        .iter()
        .filter(|asset| {
            let name = asset.name.to_lowercase();
            rule.include.iter().all(|token| name.contains(token))
                && rule.exclude.iter().all(|token| !name.contains(token))
        })
        .min_by_key(|asset| asset.name.len())
}

fn llm_release_rule() -> Result<ReleaseAssetRule, String> {
    if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        Ok(ReleaseAssetRule {
            include: &["bin-macos-arm64"],
            exclude: &["kleidiai"],
        })
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
        Ok(ReleaseAssetRule {
            include: &["bin-macos-x64"],
            exclude: &[],
        })
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "aarch64") {
        Ok(ReleaseAssetRule {
            include: &["bin-ubuntu-arm64"],
            exclude: &["vulkan"],
        })
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        Ok(ReleaseAssetRule {
            include: &["bin-ubuntu-x64"],
            exclude: &["openvino", "rocm", "sycl", "vulkan"],
        })
    } else if cfg!(target_os = "windows") && cfg!(target_arch = "aarch64") {
        Ok(ReleaseAssetRule {
            include: &["bin-win-cpu-arm64"],
            exclude: &[],
        })
    } else if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
        Ok(ReleaseAssetRule {
            include: &["bin-win-cpu-x64"],
            exclude: &[],
        })
    } else {
        Err("plateforme non couverte par les binaires llama.cpp précompilés".to_string())
    }
}

fn stable_diffusion_release_rule() -> Result<ReleaseAssetRule, String> {
    if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        Ok(ReleaseAssetRule {
            include: &["bin-darwin", "arm64"],
            exclude: &[],
        })
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        Ok(ReleaseAssetRule {
            include: &["bin-linux-ubuntu", "x86_64"],
            exclude: &["rocm", "vulkan"],
        })
    } else if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
        Ok(ReleaseAssetRule {
            include: &["bin-win-avx2-x64"],
            exclude: &[],
        })
    } else {
        Err("plateforme non couverte par les binaires stable-diffusion.cpp précompilés".to_string())
    }
}

fn executable_file_name(name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

fn executable_file_names(names: &[&str]) -> Vec<String> {
    names
        .iter()
        .map(|name| executable_file_name(name))
        .collect()
}

fn partial_path(path: &Path) -> PathBuf {
    let mut file_name = path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "model.gguf".to_string());
    file_name.push_str(".part");
    path.with_file_name(file_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_path_keeps_original_extension_visible() {
        let path = PathBuf::from("models/phi-4-mini-instruct-q4_k_m.gguf");
        assert_eq!(
            partial_path(&path),
            PathBuf::from("models/phi-4-mini-instruct-q4_k_m.gguf.part")
        );
    }

    #[test]
    fn missing_asset_status_is_not_ready_without_network() {
        let path = PathBuf::from("target/odyssee-missing-model-status-test.gguf");
        let asset = ModelAsset {
            label: "Test model",
            file_name: "test.gguf".to_string(),
            url: "https://example.invalid/test.gguf".to_string(),
            path,
        };
        let status = status_for_model_asset(&asset, false, None);

        assert!(!status.ready);
        assert!(!status.downloaded);
        assert!(status.error.is_none());
    }

    #[test]
    fn finds_stable_diffusion_cli_alias_when_sd_is_absent() {
        let directory = PathBuf::from("target/odyssee-sd-alias-test");
        let binary_dir = directory.join("bin");
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&binary_dir).expect("test directory should be created");
        fs::write(binary_dir.join(executable_file_name("sd-cli")), b"mock")
            .expect("mock binary should be written");
        let asset = ReleaseBinaryAsset {
            label: "stable-diffusion.cpp sidecar",
            file_name: executable_file_name("sd"),
            source_file_names: executable_file_names(&["sd", "sd-cli"]),
            path: directory.join("runtime").join(executable_file_name("sd")),
            release_page_url: "https://example.invalid/release",
            release_api_url: "https://example.invalid/api",
            rule: Ok(ReleaseAssetRule {
                include: &[],
                exclude: &[],
            }),
        };

        let found = find_release_binary_file(&directory, &asset).expect("sd-cli should match");

        assert_eq!(found, binary_dir.join(executable_file_name("sd-cli")));
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn selects_generic_llama_macos_arm64_asset_over_kleidiai_variant() {
        let assets = vec![
            GitHubReleaseAsset {
                name: "llama-b9090-bin-macos-arm64-kleidiai.tar.gz".to_string(),
                browser_download_url: "https://example.invalid/kleidiai".to_string(),
            },
            GitHubReleaseAsset {
                name: "llama-b9090-bin-macos-arm64.tar.gz".to_string(),
                browser_download_url: "https://example.invalid/generic".to_string(),
            },
        ];
        let rule = ReleaseAssetRule {
            include: &["bin-macos-arm64"],
            exclude: &["kleidiai"],
        };

        let selected = select_release_asset(&assets, &rule).expect("asset should match");

        assert_eq!(
            selected.browser_download_url,
            "https://example.invalid/generic"
        );
    }

    #[test]
    fn selects_cpu_llama_windows_asset_without_cuda_runtime() {
        let assets = vec![
            GitHubReleaseAsset {
                name: "llama-b9090-bin-win-cuda-12.4-x64.zip".to_string(),
                browser_download_url: "https://example.invalid/cuda".to_string(),
            },
            GitHubReleaseAsset {
                name: "llama-b9090-bin-win-cpu-x64.zip".to_string(),
                browser_download_url: "https://example.invalid/cpu".to_string(),
            },
        ];
        let rule = ReleaseAssetRule {
            include: &["bin-win-cpu-x64"],
            exclude: &[],
        };

        let selected = select_release_asset(&assets, &rule).expect("asset should match");

        assert_eq!(selected.browser_download_url, "https://example.invalid/cpu");
    }

    #[test]
    fn status_distinguishes_ready_model_from_missing_binary() {
        let paths = RuntimePaths {
            data_dir: PathBuf::from("target/odyssee-status-test"),
            model_dir: PathBuf::from("target/odyssee-status-test/models"),
            cache_dir: PathBuf::from("target/odyssee-status-test/cache"),
            image_cache_dir: PathBuf::from("target/odyssee-status-test/cache/images"),
            database_path: PathBuf::from("target/odyssee-status-test/odyssee.sqlite"),
            llm_binary: PathBuf::from("target/odyssee-status-test/bin/llama-cli"),
            image_binary: PathBuf::from("target/odyssee-status-test/bin/sd"),
            llm_model: PathBuf::from("Cargo.toml"),
            image_model: PathBuf::from("Cargo.toml"),
        };

        let status = get_model_status_for_paths(&paths);

        assert!(status.llm_model_ready);
        assert!(status.image_model_ready);
        assert!(!status.llm_binary_ready);
        assert!(!status.image_binary_ready);
        assert!(!status.llm_ready);
        assert!(!status.image_ready);
    }

    #[test]
    fn image_status_rejects_flux2_model_without_companion_assets() {
        let paths = RuntimePaths {
            data_dir: PathBuf::from("target/odyssee-unsupported-image-model-test"),
            model_dir: PathBuf::from("target/odyssee-unsupported-image-model-test/models"),
            cache_dir: PathBuf::from("target/odyssee-unsupported-image-model-test/cache"),
            image_cache_dir: PathBuf::from("target/odyssee-unsupported-image-model-test/cache/images"),
            database_path: PathBuf::from(
                "target/odyssee-unsupported-image-model-test/odyssee.sqlite",
            ),
            llm_binary: PathBuf::from("target/odyssee-unsupported-image-model-test/bin/llama-cli"),
            image_binary: PathBuf::from("target/odyssee-unsupported-image-model-test/bin/sd"),
            llm_model: PathBuf::from("target/odyssee-unsupported-image-model-test/models/llm.gguf"),
            image_model: PathBuf::from(
                "target/odyssee-unsupported-image-model-test/models/flux-2-klein-base-4b-Q4_0.gguf",
            ),
        };
        let _ = fs::remove_dir_all(&paths.data_dir);
        fs::create_dir_all(&paths.model_dir).expect("model directory should be created");
        fs::write(&paths.image_model, b"GGUF").expect("mock image model should be written");

        let status = get_model_status_for_paths(&paths);

        assert!(!status.image_model_ready);
        assert!(!status.image_ready);
        assert!(status.downloads.image.error.is_some());
        let _ = fs::remove_dir_all(&paths.data_dir);
    }

    #[test]
    fn image_status_accepts_default_single_file_model() {
        let paths = RuntimePaths {
            data_dir: PathBuf::from("target/odyssee-supported-image-model-test"),
            model_dir: PathBuf::from("target/odyssee-supported-image-model-test/models"),
            cache_dir: PathBuf::from("target/odyssee-supported-image-model-test/cache"),
            image_cache_dir: PathBuf::from("target/odyssee-supported-image-model-test/cache/images"),
            database_path: PathBuf::from(
                "target/odyssee-supported-image-model-test/odyssee.sqlite",
            ),
            llm_binary: PathBuf::from("target/odyssee-supported-image-model-test/bin/llama-cli"),
            image_binary: PathBuf::from("target/odyssee-supported-image-model-test/bin/sd"),
            llm_model: PathBuf::from("target/odyssee-supported-image-model-test/models/llm.gguf"),
            image_model: PathBuf::from(format!(
                "target/odyssee-supported-image-model-test/models/{IMAGE_MODEL_FILE}"
            )),
        };
        let _ = fs::remove_dir_all(&paths.data_dir);
        fs::create_dir_all(&paths.model_dir).expect("model directory should be created");
        fs::write(&paths.image_model, b"mock").expect("mock image model should be written");

        let status = get_model_status_for_paths(&paths);

        assert!(status.image_model_ready);
        assert!(status.downloads.image.error.is_none());
        let _ = fs::remove_dir_all(&paths.data_dir);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn image_status_requires_stable_diffusion_dylib_on_macos() {
        let paths = RuntimePaths {
            data_dir: PathBuf::from("target/odyssee-image-dylib-status-test"),
            model_dir: PathBuf::from("target/odyssee-image-dylib-status-test/models"),
            cache_dir: PathBuf::from("target/odyssee-image-dylib-status-test/cache"),
            image_cache_dir: PathBuf::from("target/odyssee-image-dylib-status-test/cache/images"),
            database_path: PathBuf::from("target/odyssee-image-dylib-status-test/odyssee.sqlite"),
            llm_binary: PathBuf::from("target/odyssee-image-dylib-status-test/bin/llama-cli"),
            image_binary: PathBuf::from("target/odyssee-image-dylib-status-test/bin/sd"),
            llm_model: PathBuf::from("target/odyssee-image-dylib-status-test/models/llm.gguf"),
            image_model: PathBuf::from("target/odyssee-image-dylib-status-test/models/image.gguf"),
        };
        let _ = fs::remove_dir_all(&paths.data_dir);
        fs::create_dir_all(paths.image_binary.parent().expect("binary parent should exist"))
            .expect("binary directory should be created");
        fs::create_dir_all(&paths.model_dir).expect("model directory should be created");
        fs::write(&paths.image_binary, b"mock").expect("mock image binary should be written");
        fs::write(&paths.image_model, b"mock").expect("mock image model should be written");
        make_executable(&paths.image_binary).expect("mock image binary should be executable");

        let status = get_model_status_for_paths(&paths);

        assert!(!status.image_binary_ready);
        assert!(status.image_model_ready);
        assert!(!status.image_ready);
        let _ = fs::remove_dir_all(&paths.data_dir);
    }
}
