use anyhow::{Context, Result, bail};
use flate2::read::ZlibDecoder;
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use zip::ZipArchive;

const CAPSULE_FORMAT: &str = "ufo-model-capsule-v1";
const MANIFEST_NAME: &str = "manifest.json";
const UFO_MAGIC: &[u8; 4] = b"UFO9";
const COPY_CHUNK: usize = 1024 * 1024;

#[derive(Debug, Clone)]
pub struct CapsuleLoad {
    pub model_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub cache_status: CapsuleCacheStatus,
    pub capsule_sha256: String,
    pub capsule_bytes: u64,
    pub model_name: String,
    pub source_bytes: u64,
    pub file_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapsuleCacheStatus {
    Extracted,
    ReusedVerified,
}

impl CapsuleCacheStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Extracted => "extracted",
            Self::ReusedVerified => "reused-verified",
        }
    }
}

#[derive(Debug, Deserialize)]
struct CapsuleManifest {
    format: String,
    mode: String,
    model_name: Option<String>,
    files: Vec<CapsuleFile>,
}

#[derive(Debug, Clone, Deserialize)]
struct CapsuleFile {
    path: String,
    archive_name: String,
    transform: String,
    original_size: u64,
    original_sha256: String,
    stored_payload_sha256: Option<String>,
    ufo_level: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct CacheSentinel {
    capsule_sha256: String,
    format: String,
    file_count: usize,
}

pub fn default_capsule_cache_root(capsule: &Path) -> PathBuf {
    capsule
        .parent()
        .map(|parent| parent.join("engine-capsule-cache"))
        .unwrap_or_else(|| std::env::temp_dir().join("zymatica-engine-capsule-cache"))
}

pub fn load_capsule_to_cache(
    capsule: &Path,
    cache_root: Option<&Path>,
    refresh: bool,
) -> Result<CapsuleLoad> {
    let capsule = capsule
        .canonicalize()
        .with_context(|| format!("resolving capsule {}", capsule.display()))?;
    let capsule_bytes = fs::metadata(&capsule)
        .with_context(|| format!("stat {}", capsule.display()))?
        .len();
    let capsule_sha256 = sha256_file(&capsule)?;
    let cache_root = cache_root
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_capsule_cache_root(&capsule));
    let cache_dir = cache_root.join(&capsule_sha256);

    let mut archive = open_capsule(&capsule)?;
    let manifest = read_manifest(&mut archive)?;
    let source_bytes = manifest.files.iter().map(|entry| entry.original_size).sum();
    let file_count = manifest.files.len();
    let model_name = manifest
        .model_name
        .clone()
        .unwrap_or_else(|| "unknown-model".to_string());

    if refresh && cache_dir.exists() {
        fs::remove_dir_all(&cache_dir)
            .with_context(|| format!("removing capsule cache {}", cache_dir.display()))?;
    }

    if cache_is_verified(&cache_dir, &capsule_sha256, &manifest)? {
        return Ok(CapsuleLoad {
            model_dir: cache_dir.clone(),
            cache_dir,
            cache_status: CapsuleCacheStatus::ReusedVerified,
            capsule_sha256,
            capsule_bytes,
            model_name,
            source_bytes,
            file_count,
        });
    }

    fs::create_dir_all(&cache_dir)
        .with_context(|| format!("creating capsule cache {}", cache_dir.display()))?;
    extract_manifest_files(&capsule, &manifest, &cache_dir)
        .with_context(|| format!("extracting {}", capsule.display()))?;
    write_sentinel(&cache_dir, &capsule_sha256, &manifest)?;

    Ok(CapsuleLoad {
        model_dir: cache_dir.clone(),
        cache_dir,
        cache_status: CapsuleCacheStatus::Extracted,
        capsule_sha256,
        capsule_bytes,
        model_name,
        source_bytes,
        file_count,
    })
}

#[derive(Debug, Clone)]
pub struct InMemoryCapsule {
    pub model_name: String,
    pub capsule_sha256: String,
    pub capsule_bytes: u64,
    pub source_bytes: u64,
    pub config_json: Vec<u8>,
    pub tokenizer_json: Option<Vec<u8>>,
    pub files: HashMap<String, Arc<Vec<u8>>>,
}

#[derive(Debug, Clone)]
pub struct CapsuleVerification {
    pub format: String,
    pub mode: String,
    pub model_name: String,
    pub capsule_sha256: String,
    pub capsule_bytes: u64,
    pub source_bytes: u64,
    pub stored_payload_bytes: u64,
    pub file_count: usize,
    pub zip_entry_count: usize,
    pub raw_file_count: usize,
    pub ufo_file_count: usize,
    pub direct_sha256_count: usize,
    pub stored_sha256_count: usize,
}

#[derive(Debug, Clone, Copy)]
struct VerifiedCapsuleFile {
    original_size: u64,
    stored_size: u64,
    is_ufo: bool,
    has_direct_sha256: bool,
    has_stored_sha256: bool,
}

pub fn verify_capsule(capsule: &Path) -> Result<CapsuleVerification> {
    let capsule = capsule
        .canonicalize()
        .with_context(|| format!("resolving capsule {}", capsule.display()))?;
    let capsule_bytes = fs::metadata(&capsule)
        .with_context(|| format!("stat {}", capsule.display()))?
        .len();
    let capsule_sha256 = sha256_file(&capsule)?;
    let mut archive = open_capsule(&capsule)?;
    let manifest = read_manifest(&mut archive)?;
    let zip_entry_count = archive.len();
    validate_archive_coverage(&mut archive, &manifest)?;
    drop(archive);

    let capsule_path_buf = capsule.to_path_buf();
    let results: Vec<Result<VerifiedCapsuleFile>> = {
        #[cfg(feature = "parallel")]
        {
            manifest
                .files
                .par_iter()
                .map(|entry| verify_capsule_member(&capsule_path_buf, entry))
                .collect()
        }
        #[cfg(not(feature = "parallel"))]
        {
            manifest
                .files
                .iter()
                .map(|entry| verify_capsule_member(&capsule_path_buf, entry))
                .collect()
        }
    };

    let mut source_bytes = 0_u64;
    let mut stored_payload_bytes = 0_u64;
    let mut raw_file_count = 0_usize;
    let mut ufo_file_count = 0_usize;
    let mut direct_sha256_count = 0_usize;
    let mut stored_sha256_count = 0_usize;

    for result in results {
        let verified = result?;
        source_bytes = source_bytes
            .checked_add(verified.original_size)
            .context("overflow in verified capsule source byte count")?;
        stored_payload_bytes = stored_payload_bytes
            .checked_add(verified.stored_size)
            .context("overflow in verified capsule stored payload byte count")?;
        if verified.is_ufo {
            ufo_file_count += 1;
        } else {
            raw_file_count += 1;
        }
        if verified.has_direct_sha256 {
            direct_sha256_count += 1;
        }
        if verified.has_stored_sha256 {
            stored_sha256_count += 1;
        }
    }

    Ok(CapsuleVerification {
        format: manifest.format,
        mode: manifest.mode,
        model_name: manifest
            .model_name
            .unwrap_or_else(|| "unknown-model".to_string()),
        capsule_sha256,
        capsule_bytes,
        source_bytes,
        stored_payload_bytes,
        file_count: manifest.files.len(),
        zip_entry_count,
        raw_file_count,
        ufo_file_count,
        direct_sha256_count,
        stored_sha256_count,
    })
}

pub fn load_capsule_to_memory(capsule: &Path) -> Result<InMemoryCapsule> {
    let capsule = capsule
        .canonicalize()
        .with_context(|| format!("resolving capsule {}", capsule.display()))?;
    let capsule_bytes = fs::metadata(&capsule)?.len();
    let capsule_sha256 = sha256_file(&capsule)?;
    let mut archive = open_capsule(&capsule)?;
    let manifest = read_manifest(&mut archive)?;
    let model_name = manifest
        .model_name
        .clone()
        .unwrap_or_else(|| "unknown-model".to_string());

    let mut source_bytes = 0u64;
    for entry in &manifest.files {
        source_bytes += entry.original_size;
    }

    let capsule_path_buf = capsule.to_path_buf();
    let entries = manifest.files;
    let read_entry = |entry: CapsuleFile| -> Result<(String, Vec<u8>)> {
        let relative_path = safe_relative_path(&entry.path)?;
        let key = relative_path.to_string_lossy().to_string();

        let mut archive = open_capsule(&capsule_path_buf)?;
        let mut member = archive
            .by_name(&entry.archive_name)
            .with_context(|| format!("capsule missing {}", entry.archive_name))?;
        let mut payload = Vec::new();
        member
            .read_to_end(&mut payload)
            .with_context(|| format!("reading {}", entry.archive_name))?;

        if entry.transform.starts_with("ufo") {
            if let Some(expected) = &entry.stored_payload_sha256 {
                let got = sha256_bytes(&payload);
                if got != *expected {
                    bail!(
                        "stored payload hash mismatch for {}: got {} expected {}",
                        entry.path,
                        got,
                        expected
                    );
                }
            }
            let level = ufo_level(&entry)?;
            let raw = decompress_ufo_levels(&payload, level)
                .with_context(|| format!("decoding {} as UFO level {}", entry.path, level))?;
            verify_digest_and_size(raw.len() as u64, &sha256_bytes(&raw), &entry)?;
            Ok((key, raw))
        } else if entry.transform == "raw" {
            verify_digest_and_size(payload.len() as u64, &sha256_bytes(&payload), &entry)?;
            Ok((key, payload))
        } else {
            bail!(
                "unknown capsule transform {} for {}",
                entry.transform,
                entry.path
            );
        }
    };
    let results: Vec<Result<(String, Vec<u8>)>> = {
        #[cfg(feature = "parallel")]
        {
            entries.into_par_iter().map(read_entry).collect()
        }
        #[cfg(not(feature = "parallel"))]
        {
            entries.into_iter().map(read_entry).collect()
        }
    };

    let mut files = HashMap::new();
    for res in results {
        let (key, raw) = res?;
        files.insert(key, Arc::new(raw));
    }

    let config_json = files
        .get("config.json")
        .context("capsule missing config.json")?
        .as_ref()
        .clone();
    let tokenizer_json = files.get("tokenizer.json").map(|arc| arc.as_ref().clone());

    Ok(InMemoryCapsule {
        model_name,
        capsule_sha256,
        capsule_bytes,
        source_bytes,
        config_json,
        tokenizer_json,
        files,
    })
}

fn open_capsule(path: &Path) -> Result<ZipArchive<File>> {
    let file = File::open(path).with_context(|| format!("opening capsule {}", path.display()))?;
    ZipArchive::new(file).with_context(|| format!("reading ZIP capsule {}", path.display()))
}

fn read_manifest(archive: &mut ZipArchive<File>) -> Result<CapsuleManifest> {
    let mut manifest_file = archive
        .by_name(MANIFEST_NAME)
        .context("capsule missing manifest.json")?;
    let mut bytes = Vec::new();
    manifest_file
        .read_to_end(&mut bytes)
        .context("reading capsule manifest")?;
    let manifest: CapsuleManifest =
        serde_json::from_slice(&bytes).context("parsing capsule manifest")?;
    match (manifest.format.as_str(), manifest.mode.as_str()) {
        (CAPSULE_FORMAT, "lossless") | ("ufo-v2", "quantized") => {}
        _ => bail!(
            "unsupported capsule format/mode pair: format={} mode={}",
            manifest.format,
            manifest.mode
        ),
    }
    if manifest.files.is_empty() {
        bail!("capsule manifest contains no files");
    }
    validate_manifest_integrity_policy(&manifest)?;
    Ok(manifest)
}

fn validate_manifest_integrity_policy(manifest: &CapsuleManifest) -> Result<()> {
    let mut paths = HashSet::new();
    let mut archive_names = HashSet::new();
    for entry in &manifest.files {
        safe_relative_path(&entry.path)?;
        safe_relative_path(&entry.archive_name)?;
        if !paths.insert(entry.path.clone()) {
            bail!("duplicate capsule manifest path {}", entry.path);
        }
        if !archive_names.insert(entry.archive_name.clone()) {
            bail!(
                "duplicate capsule manifest archive name {}",
                entry.archive_name
            );
        }
        if manifest.format == "ufo-v2" && !is_sha256_hex(&entry.original_sha256) {
            bail!(
                "ufo-v2 quantized capsule entry {} must include a direct SHA-256",
                entry.path
            );
        }
        if !entry.original_sha256.is_empty() && !is_sha256_hex(&entry.original_sha256) {
            bail!("invalid SHA-256 for capsule entry {}", entry.path);
        }
        if let Some(stored) = &entry.stored_payload_sha256
            && !is_sha256_hex(stored)
        {
            bail!(
                "invalid stored payload SHA-256 for capsule entry {}",
                entry.path
            );
        }
    }
    Ok(())
}

fn validate_archive_coverage(
    archive: &mut ZipArchive<File>,
    manifest: &CapsuleManifest,
) -> Result<()> {
    let manifest_entries = manifest
        .files
        .iter()
        .map(|entry| entry.archive_name.as_str())
        .collect::<HashSet<_>>();
    for index in 0..archive.len() {
        let file = archive.by_index(index)?;
        if file.is_dir() || file.name() == MANIFEST_NAME {
            continue;
        }
        if !manifest_entries.contains(file.name()) {
            bail!(
                "capsule archive contains unmanifested member {}",
                file.name()
            );
        }
    }
    Ok(())
}

fn verify_capsule_member(capsule: &Path, entry: &CapsuleFile) -> Result<VerifiedCapsuleFile> {
    let mut archive = open_capsule(capsule)?;
    let mut member = archive
        .by_name(&entry.archive_name)
        .with_context(|| format!("capsule missing {}", entry.archive_name))?;
    let stored_size = member.size();
    let mut payload = Vec::new();
    member
        .read_to_end(&mut payload)
        .with_context(|| format!("reading {}", entry.archive_name))?;
    if stored_size != payload.len() as u64 {
        bail!(
            "stored size mismatch for {}: zip header {} read {}",
            entry.archive_name,
            stored_size,
            payload.len()
        );
    }

    if entry.transform.starts_with("ufo") {
        if let Some(expected) = &entry.stored_payload_sha256 {
            let got = sha256_bytes(&payload);
            if got != *expected {
                bail!(
                    "stored payload hash mismatch for {}: got {} expected {}",
                    entry.path,
                    got,
                    expected
                );
            }
        }
        let level = ufo_level(entry)?;
        let raw = decompress_ufo_levels(&payload, level)
            .with_context(|| format!("decoding {} as UFO level {}", entry.path, level))?;
        verify_digest_and_size(raw.len() as u64, &sha256_bytes(&raw), entry)?;
        Ok(VerifiedCapsuleFile {
            original_size: entry.original_size,
            stored_size,
            is_ufo: true,
            has_direct_sha256: !entry.original_sha256.is_empty(),
            has_stored_sha256: entry.stored_payload_sha256.is_some(),
        })
    } else if entry.transform == "raw" {
        verify_digest_and_size(payload.len() as u64, &sha256_bytes(&payload), entry)?;
        Ok(VerifiedCapsuleFile {
            original_size: entry.original_size,
            stored_size,
            is_ufo: false,
            has_direct_sha256: !entry.original_sha256.is_empty(),
            has_stored_sha256: entry.stored_payload_sha256.is_some(),
        })
    } else {
        bail!(
            "unknown capsule transform {} for {}",
            entry.transform,
            entry.path
        );
    }
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn cache_is_verified(
    cache_dir: &Path,
    capsule_sha256: &str,
    manifest: &CapsuleManifest,
) -> Result<bool> {
    if !cache_dir.is_dir() {
        return Ok(false);
    }
    let sentinel_path = cache_dir.join(".zymatica_capsule_cache.json");
    if !sentinel_path.is_file() {
        return Ok(false);
    }
    let sentinel: CacheSentinel = serde_json::from_slice(
        &fs::read(&sentinel_path)
            .with_context(|| format!("reading {}", sentinel_path.display()))?,
    )
    .with_context(|| format!("parsing {}", sentinel_path.display()))?;
    if sentinel.format != manifest.format
        || sentinel.capsule_sha256 != capsule_sha256
        || sentinel.file_count != manifest.files.len()
    {
        return Ok(false);
    }

    for entry in &manifest.files {
        let path = cache_dir.join(safe_relative_path(&entry.path)?);
        if !path.is_file() {
            return Ok(false);
        }
        let meta = fs::metadata(&path).with_context(|| format!("stat {}", path.display()))?;
        if meta.len() != entry.original_size {
            return Ok(false);
        }
        if !entry.original_sha256.is_empty() && sha256_file(&path)? != entry.original_sha256 {
            return Ok(false);
        }
    }
    Ok(true)
}

fn extract_manifest_files(
    capsule_path: &Path,
    manifest: &CapsuleManifest,
    cache_dir: &Path,
) -> Result<()> {
    let capsule_path_buf = capsule_path.to_path_buf();
    let extract_entry = |entry: &CapsuleFile| -> Result<()> {
        if entry.transform.starts_with("ufo") || entry.transform == "raw" {
            let mut archive = open_capsule(&capsule_path_buf)?;
            let mut member = archive
                .by_name(&entry.archive_name)
                .with_context(|| format!("capsule missing {}", entry.archive_name))?;
            let mut payload = Vec::new();
            member
                .read_to_end(&mut payload)
                .with_context(|| format!("reading {}", entry.archive_name))?;

            let out_path = cache_dir.join(safe_relative_path(&entry.path)?);
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("creating {}", parent.display()))?;
            }

            if entry.transform.starts_with("ufo") {
                if let Some(expected) = &entry.stored_payload_sha256 {
                    let got = sha256_bytes(&payload);
                    if got != *expected {
                        bail!(
                            "stored payload hash mismatch for {}: got {} expected {}",
                            entry.path,
                            got,
                            expected
                        );
                    }
                }
                let level = ufo_level(entry)?;
                let raw = decompress_ufo_levels(&payload, level)
                    .with_context(|| format!("decoding {} as UFO level {}", entry.path, level))?;
                verify_and_write_bytes(&out_path, &raw, entry)?;
            } else if entry.transform == "raw" {
                verify_and_write_bytes(&out_path, &payload, entry)?;
            }
            Ok(())
        } else {
            bail!(
                "unknown capsule transform {} for {}",
                entry.transform,
                entry.path
            );
        }
    };
    #[cfg(feature = "parallel")]
    {
        manifest.files.par_iter().try_for_each(extract_entry)?;
    }
    #[cfg(not(feature = "parallel"))]
    {
        manifest.files.iter().try_for_each(extract_entry)?;
    }
    Ok(())
}

fn verify_and_write_bytes(out_path: &Path, bytes: &[u8], entry: &CapsuleFile) -> Result<()> {
    let digest = sha256_bytes(bytes);
    verify_digest_and_size(bytes.len() as u64, &digest, entry)?;
    fs::write(out_path, bytes).with_context(|| format!("writing {}", out_path.display()))?;
    Ok(())
}

fn verify_digest_and_size(size: u64, digest: &str, entry: &CapsuleFile) -> Result<()> {
    if size != entry.original_size {
        bail!(
            "size mismatch for {}: got {} expected {}",
            entry.path,
            size,
            entry.original_size
        );
    }
    if !entry.original_sha256.is_empty() && digest != entry.original_sha256 {
        bail!(
            "sha256 mismatch for {}: got {} expected {}",
            entry.path,
            digest,
            entry.original_sha256
        );
    }
    Ok(())
}

fn write_sentinel(
    cache_dir: &Path,
    capsule_sha256: &str,
    manifest: &CapsuleManifest,
) -> Result<()> {
    let sentinel = serde_json::json!({
        "format": manifest.format,
        "capsule_sha256": capsule_sha256,
        "file_count": manifest.files.len(),
        "model_name": manifest.model_name,
    });
    let path = cache_dir.join(".zymatica_capsule_cache.json");
    fs::write(&path, serde_json::to_vec_pretty(&sentinel)?)
        .with_context(|| format!("writing {}", path.display()))
}

fn safe_relative_path(value: &str) -> Result<PathBuf> {
    if has_windows_path_syntax(value) {
        bail!("unsafe capsule path: {value}");
    }
    let path = Path::new(value);
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                bail!("unsafe capsule path: {value}");
            }
        }
    }
    if out.as_os_str().is_empty() {
        bail!("empty capsule path");
    }
    Ok(out)
}

fn has_windows_path_syntax(value: &str) -> bool {
    let bytes = value.as_bytes();
    value.contains('\\') || (bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':')
}

fn ufo_level(entry: &CapsuleFile) -> Result<usize> {
    let level = if let Some(level) = entry.ufo_level {
        level
    } else if entry.transform == "ufo9" {
        9
    } else if let Some(rest) = entry.transform.strip_prefix("ufo") {
        rest.parse()
            .with_context(|| format!("parsing transform {}", entry.transform))?
    } else {
        bail!("entry is not a UFO transform: {}", entry.transform);
    };
    if !(1..=9).contains(&level) {
        bail!("UFO level out of range for {}: {}", entry.path, level);
    }
    Ok(level)
}

fn decompress_ufo_levels(data: &[u8], level: usize) -> Result<Vec<u8>> {
    let mut current = data.to_vec();
    for stage in (1..=level).rev() {
        current = decompress_ufo_stage(stage, &current)?;
    }
    Ok(current)
}

fn decompress_ufo_stage(level: usize, data: &[u8]) -> Result<Vec<u8>> {
    match level {
        1 => level1_tokenization_decode(data),
        2 => level2_prefix_suffix_decode(data),
        3 => level3_delta_varint_decode(data),
        4 => level4_dct_decode(data),
        5 => level5_dictionary_decode(data),
        6 => level6_zlib_decode(data),
        7 => level7_geometric_decode(data),
        8 => level8_dna_decode(data),
        9 => level9_cosmic_hash_decode(data),
        _ => bail!("unsupported UFO level {level}"),
    }
}

fn unframe(stage_id: usize, data: &[u8]) -> Result<&[u8]> {
    if data.len() < 6 || &data[..4] != UFO_MAGIC {
        bail!("missing UFO9 frame magic");
    }
    if data[4] as usize != stage_id {
        bail!("expected stage {}, got {}", stage_id, data[4]);
    }
    let (payload_len, offset) = decode_varint(data, 5)?;
    let end = offset
        .checked_add(payload_len as usize)
        .context("UFO frame length overflow")?;
    if end != data.len() {
        bail!("stage frame length mismatch");
    }
    Ok(&data[offset..end])
}

fn decode_varint(data: &[u8], mut offset: usize) -> Result<(u64, usize)> {
    let mut shift = 0u32;
    let mut value = 0u64;
    loop {
        if offset >= data.len() {
            bail!("truncated varint");
        }
        let byte = data[offset];
        offset += 1;
        value |= u64::from(byte & 0x7f) << shift;
        if byte < 0x80 {
            return Ok((value, offset));
        }
        shift += 7;
        if shift > 63 {
            bail!("varint is too large");
        }
    }
}

fn unzigzag(value: u64) -> i64 {
    ((value >> 1) as i64) ^ (-((value & 1) as i64))
}

fn level1_tokenization_decode(data: &[u8]) -> Result<Vec<u8>> {
    let payload = unframe(1, data)?;
    if payload.is_empty() {
        bail!("tokenizer payload is empty");
    }
    let mode = payload[0];
    let (original_len, mut offset) = decode_varint(payload, 1)?;
    if mode == 0 {
        let raw = &payload[offset..];
        if raw.len() as u64 != original_len {
            bail!("token raw payload length mismatch");
        }
        return Ok(raw.to_vec());
    }
    if mode != 1 {
        bail!("unknown tokenizer mode {mode}");
    }
    let (vocab_len, next) = decode_varint(payload, offset)?;
    offset = next;
    let mut vocab = Vec::with_capacity(vocab_len as usize);
    for _ in 0..vocab_len {
        let (size, next) = decode_varint(payload, offset)?;
        offset = next;
        let end = offset + size as usize;
        if end > payload.len() {
            bail!("truncated tokenizer vocabulary");
        }
        let token = std::str::from_utf8(&payload[offset..end])
            .context("invalid UTF-8 tokenizer token")?
            .to_string();
        offset = end;
        vocab.push(token);
    }
    let (token_count, next) = decode_varint(payload, offset)?;
    offset = next;
    let mut out = String::new();
    for _ in 0..token_count {
        let (idx, next) = decode_varint(payload, offset)?;
        offset = next;
        let token = vocab
            .get(idx as usize)
            .with_context(|| format!("token id {idx} out of vocabulary"))?;
        out.push_str(token);
    }
    let bytes = out.into_bytes();
    if bytes.len() as u64 != original_len {
        bail!("tokenized output length mismatch");
    }
    Ok(bytes)
}

fn level2_prefix_suffix_decode(data: &[u8]) -> Result<Vec<u8>> {
    let payload = unframe(2, data)?;
    let (original_len, mut offset) = decode_varint(payload, 0)?;
    let (chunk_size, next) = decode_varint(payload, offset)?;
    offset = next;
    let chunk_size = chunk_size as usize;
    let mut previous: Vec<u8> = Vec::new();
    let mut out = Vec::new();
    while offset < payload.len() {
        let (prefix, next) = decode_varint(payload, offset)?;
        offset = next;
        let (suffix_len, next) = decode_varint(payload, offset)?;
        offset = next;
        let prefix = prefix as usize;
        let suffix_len = suffix_len as usize;
        if prefix > previous.len() {
            bail!("prefix exceeds previous chunk length");
        }
        let end = offset + suffix_len;
        if end > payload.len() {
            bail!("truncated prefix-suffix payload");
        }
        let mut chunk = previous[..prefix].to_vec();
        chunk.extend_from_slice(&payload[offset..end]);
        offset = end;
        if chunk.len() > chunk_size {
            bail!("prefix-suffix chunk exceeds declared size");
        }
        out.extend_from_slice(&chunk);
        previous = chunk;
    }
    if out.len() as u64 != original_len {
        bail!("prefix-suffix output length mismatch");
    }
    Ok(out)
}

fn level3_delta_varint_decode(data: &[u8]) -> Result<Vec<u8>> {
    let payload = unframe(3, data)?;
    let (original_len, mut offset) = decode_varint(payload, 0)?;
    if original_len == 0 {
        return Ok(Vec::new());
    }
    if offset >= payload.len() {
        bail!("delta stream missing first byte");
    }
    let mut previous = payload[offset];
    offset += 1;
    let mut out = vec![previous];
    while out.len() < original_len as usize {
        let (encoded, next) = decode_varint(payload, offset)?;
        offset = next;
        previous = ((i64::from(previous) + unzigzag(encoded)) & 0xff) as u8;
        out.push(previous);
    }
    if offset != payload.len() {
        bail!("delta stream has trailing bytes");
    }
    Ok(out)
}

fn py_round(value: f64) -> i64 {
    let floor = value.floor();
    let diff = value - floor;
    if diff < 0.5 {
        floor as i64
    } else if diff > 0.5 {
        floor as i64 + 1
    } else {
        let floor_i = floor as i64;
        if floor_i % 2 == 0 {
            floor_i
        } else {
            floor_i + 1
        }
    }
}

fn idct(coeffs: &[i16]) -> Vec<u8> {
    let n = coeffs.len();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let mut total = 0.0f64;
        for (k, coeff) in coeffs.iter().enumerate() {
            let alpha = if k == 0 {
                (1.0 / n as f64).sqrt()
            } else {
                (2.0 / n as f64).sqrt()
            };
            total += alpha
                * f64::from(*coeff)
                * (std::f64::consts::PI * (i as f64 + 0.5) * k as f64 / n as f64).cos();
        }
        let rounded = py_round(total + 128.0).clamp(0, 255) as u8;
        out.push(rounded);
    }
    out
}

fn level4_dct_decode(data: &[u8]) -> Result<Vec<u8>> {
    let payload = unframe(4, data)?;
    let (original_len, mut offset) = decode_varint(payload, 0)?;
    let (block_size, next) = decode_varint(payload, offset)?;
    offset = next;
    let block_size = block_size as usize;
    let mut out = Vec::new();
    while out.len() < original_len as usize {
        if offset + 2 * block_size > payload.len() {
            bail!("truncated DCT coefficient block");
        }
        let mut coeffs = Vec::with_capacity(block_size);
        for _ in 0..block_size {
            coeffs.push(i16::from_be_bytes([payload[offset], payload[offset + 1]]));
            offset += 2;
        }
        let reconstructed = idct(&coeffs);
        for predicted in reconstructed {
            let (residual, next) = decode_varint(payload, offset)?;
            offset = next;
            out.push(((i64::from(predicted) + unzigzag(residual)) & 0xff) as u8);
        }
    }
    out.truncate(original_len as usize);
    Ok(out)
}

fn level5_dictionary_decode(data: &[u8]) -> Result<Vec<u8>> {
    let payload = unframe(5, data)?;
    let (original_len, mut offset) = decode_varint(payload, 0)?;
    let (chunk_size, next) = decode_varint(payload, offset)?;
    offset = next;
    let chunk_size = chunk_size as usize;
    let (dict_len, next) = decode_varint(payload, offset)?;
    offset = next;
    let mut dictionary = Vec::with_capacity(dict_len as usize);
    for _ in 0..dict_len {
        let end = offset + chunk_size;
        if end > payload.len() {
            bail!("truncated dictionary");
        }
        dictionary.push(payload[offset..end].to_vec());
        offset = end;
    }
    let mut out = Vec::new();
    while offset < payload.len() {
        let tag = payload[offset];
        offset += 1;
        match tag {
            1 => {
                if offset >= payload.len() {
                    bail!("truncated dictionary reference");
                }
                let idx = payload[offset] as usize;
                offset += 1;
                let chunk = dictionary
                    .get(idx)
                    .with_context(|| format!("dictionary index {idx} out of range"))?;
                out.extend_from_slice(chunk);
            }
            0 => {
                let (size, next) = decode_varint(payload, offset)?;
                offset = next;
                let end = offset + size as usize;
                if end > payload.len() {
                    bail!("truncated dictionary literal");
                }
                out.extend_from_slice(&payload[offset..end]);
                offset = end;
            }
            _ => bail!("unknown dictionary token"),
        }
    }
    if out.len() as u64 != original_len {
        bail!("dictionary output length mismatch");
    }
    Ok(out)
}

fn level6_zlib_decode(data: &[u8]) -> Result<Vec<u8>> {
    let payload = unframe(6, data)?;
    let (original_len, offset) = decode_varint(payload, 0)?;
    let mut decoder = ZlibDecoder::new(&payload[offset..]);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out)?;
    if out.len() as u64 != original_len {
        bail!("zlib output length mismatch");
    }
    Ok(out)
}

fn level7_geometric_decode(data: &[u8]) -> Result<Vec<u8>> {
    let payload = unframe(7, data)?;
    let (original_len, mut offset) = decode_varint(payload, 0)?;
    if original_len == 0 {
        return Ok(Vec::new());
    }
    if offset + 3 > payload.len() {
        bail!("truncated geometric centroids");
    }
    let centroids = &payload[offset..offset + 3];
    offset += 3;
    let mut out = Vec::with_capacity(original_len as usize);
    while out.len() < original_len as usize {
        let (residual, next) = decode_varint(payload, offset)?;
        offset = next;
        let centroid = centroids[out.len() % 3];
        out.push(((i64::from(centroid) + unzigzag(residual)) & 0xff) as u8);
    }
    Ok(out)
}

fn level8_dna_decode(data: &[u8]) -> Result<Vec<u8>> {
    let payload = unframe(8, data)?;
    let (original_len, mut offset) = decode_varint(payload, 0)?;
    let (motif_count, next) = decode_varint(payload, offset)?;
    offset = next;
    let mut motifs = Vec::with_capacity(motif_count as usize);
    for _ in 0..motif_count {
        let (size, next) = decode_varint(payload, offset)?;
        offset = next;
        let end = offset + size as usize;
        if end > payload.len() {
            bail!("truncated DNA motif");
        }
        motifs.push(payload[offset..end].to_vec());
        offset = end;
    }
    let mut out = Vec::new();
    while out.len() < original_len as usize {
        if offset >= payload.len() {
            bail!("truncated DNA body");
        }
        let tag = payload[offset];
        offset += 1;
        match tag {
            1 => {
                if offset >= payload.len() {
                    bail!("truncated DNA reference");
                }
                let idx = payload[offset] as usize;
                offset += 1;
                let motif = motifs
                    .get(idx)
                    .with_context(|| format!("DNA motif index {idx} out of range"))?;
                out.extend_from_slice(motif);
            }
            0 => {
                if offset >= payload.len() {
                    bail!("truncated DNA literal");
                }
                out.push(payload[offset]);
                offset += 1;
            }
            _ => bail!("unknown DNA token"),
        }
    }
    out.truncate(original_len as usize);
    Ok(out)
}

fn level9_cosmic_hash_decode(data: &[u8]) -> Result<Vec<u8>> {
    let payload = unframe(9, data)?;
    let (original_len, mut offset) = decode_varint(payload, 0)?;
    if offset + 32 > payload.len() {
        bail!("truncated cosmic hash digest");
    }
    let expected = &payload[offset..offset + 32];
    offset += 32;
    let body = &payload[offset..];
    if body.len() as u64 != original_len {
        bail!("cosmic hash body length mismatch");
    }
    let digest = Sha256::digest(body);
    if &digest[..] != expected {
        bail!("cosmic hash verification failed");
    }
    Ok(body.to_vec())
}

fn sha256_file(path: &Path) -> Result<String> {
    let file = File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let mut reader = std::io::BufReader::new(file);
    let mut digest = Sha256::new();
    let mut buffer = vec![0u8; COPY_CHUNK];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(to_hex(&digest.finalize()))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    to_hex(&Sha256::digest(bytes))
}

fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn encode_varint(mut value: u64) -> Vec<u8> {
        let mut out = Vec::new();
        while value >= 0x80 {
            out.push((value as u8 & 0x7f) | 0x80);
            value >>= 7;
        }
        out.push(value as u8);
        out
    }

    fn frame(stage: u8, payload: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(UFO_MAGIC);
        out.push(stage);
        out.extend_from_slice(&encode_varint(payload.len() as u64));
        out.extend_from_slice(payload);
        out
    }

    #[test]
    fn cosmic_hash_stage_rejects_tampering() {
        let body = b"capsule";
        let mut payload = encode_varint(body.len() as u64);
        payload.extend_from_slice(&Sha256::digest(body));
        payload.extend_from_slice(body);
        let mut encoded = frame(9, &payload);
        assert_eq!(level9_cosmic_hash_decode(&encoded).unwrap(), body);
        let last = encoded.len() - 1;
        encoded[last] ^= 1;
        assert!(level9_cosmic_hash_decode(&encoded).is_err());
    }

    #[test]
    fn safe_relative_path_rejects_escape() {
        assert!(safe_relative_path("config.json").is_ok());
        assert!(safe_relative_path("nested/config.json").is_ok());
        assert!(safe_relative_path("../config.json").is_err());
        assert!(safe_relative_path(r"C:\config.json").is_err());
        assert!(safe_relative_path(r"..\config.json").is_err());
    }

    fn add_capsule_member(
        zip: &mut zip::ZipWriter<File>,
        archive_name: &str,
        bytes: &[u8],
    ) -> Result<serde_json::Value> {
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file(archive_name, options)?;
        zip.write_all(bytes)?;
        Ok(serde_json::json!({
            "path": archive_name,
            "archive_name": archive_name,
            "transform": "raw",
            "original_size": bytes.len(),
            "original_sha256": sha256_bytes(bytes),
            "stored_payload_sha256": null,
            "ufo_level": null
        }))
    }

    #[test]
    fn in_memory_capsule_loads_without_cache_materialization() {
        let temp = tempfile::tempdir().unwrap();
        let capsule_path = temp.path().join("tiny.ufomodel.zip");
        let cache_root = default_capsule_cache_root(&capsule_path);
        assert!(!cache_root.exists());

        let config = br#"{"model_type":"gemma4","vocab_size":8}"#;
        let index = br#"{"weight_map":{"model.embed_tokens.weight":"model.safetensors"}}"#;
        let weights = b"not-a-real-safetensors-file-for-loader-boundary-test";

        let file = File::create(&capsule_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let files = vec![
            add_capsule_member(&mut zip, "config.json", config).unwrap(),
            add_capsule_member(&mut zip, "model.safetensors.index.json", index).unwrap(),
            add_capsule_member(&mut zip, "model.safetensors", weights).unwrap(),
        ];
        let manifest = serde_json::json!({
            "format": CAPSULE_FORMAT,
            "mode": "lossless",
            "model_name": "tiny-in-memory-fixture",
            "files": files
        });
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file(MANIFEST_NAME, options).unwrap();
        zip.write_all(manifest.to_string().as_bytes()).unwrap();
        zip.finish().unwrap();

        let loaded = load_capsule_to_memory(&capsule_path).unwrap();
        assert_eq!(loaded.model_name, "tiny-in-memory-fixture");
        assert_eq!(loaded.files.len(), 3);
        assert_eq!(loaded.config_json, config);
        assert_eq!(&loaded.files["model.safetensors"][..], weights.as_slice());
        assert!(!cache_root.exists());
    }

    #[test]
    fn verify_capsule_reports_strict_member_accounting() {
        let temp = tempfile::tempdir().unwrap();
        let capsule_path = temp.path().join("tiny-verified.ufomodel.zip");
        let config = br#"{"model_type":"gemma4","vocab_size":8}"#;
        let weights = b"strict-verifier-fixture";

        let file = File::create(&capsule_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let files = vec![
            add_capsule_member(&mut zip, "config.json", config).unwrap(),
            add_capsule_member(&mut zip, "model.safetensors", weights).unwrap(),
        ];
        let manifest = serde_json::json!({
            "format": CAPSULE_FORMAT,
            "mode": "lossless",
            "model_name": "tiny-verified",
            "files": files
        });
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file(MANIFEST_NAME, options).unwrap();
        zip.write_all(manifest.to_string().as_bytes()).unwrap();
        zip.finish().unwrap();

        let verified = verify_capsule(&capsule_path).unwrap();
        assert_eq!(verified.format, CAPSULE_FORMAT);
        assert_eq!(verified.mode, "lossless");
        assert_eq!(verified.model_name, "tiny-verified");
        assert_eq!(verified.file_count, 2);
        assert_eq!(verified.zip_entry_count, 3);
        assert_eq!(verified.raw_file_count, 2);
        assert_eq!(verified.ufo_file_count, 0);
        assert_eq!(verified.direct_sha256_count, 2);
        assert_eq!(verified.source_bytes, (config.len() + weights.len()) as u64);
        assert_eq!(verified.stored_payload_bytes, verified.source_bytes);
    }

    #[test]
    fn verify_capsule_rejects_unmanifested_archive_members() {
        let temp = tempfile::tempdir().unwrap();
        let capsule_path = temp.path().join("extra-member.ufomodel.zip");
        let config = br#"{"model_type":"gemma4","vocab_size":8}"#;

        let file = File::create(&capsule_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let files = vec![add_capsule_member(&mut zip, "config.json", config).unwrap()];
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("unexpected.bin", options).unwrap();
        zip.write_all(b"not in manifest").unwrap();
        let manifest = serde_json::json!({
            "format": CAPSULE_FORMAT,
            "mode": "lossless",
            "model_name": "extra-member",
            "files": files
        });
        zip.start_file(MANIFEST_NAME, options).unwrap();
        zip.write_all(manifest.to_string().as_bytes()).unwrap();
        zip.finish().unwrap();

        let err = verify_capsule(&capsule_path).unwrap_err();
        assert!(err.to_string().contains("unmanifested member"), "{err:#}");
    }

    #[test]
    fn v2_quantized_capsule_requires_direct_hashes() {
        let temp = tempfile::tempdir().unwrap();
        let capsule_path = temp.path().join("bad-v2.ufomodel.zip");
        let config = br#"{"model_type":"gemma4","vocab_size":8}"#;

        let file = File::create(&capsule_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("config.json", options).unwrap();
        zip.write_all(config).unwrap();
        let manifest = serde_json::json!({
            "format": "ufo-v2",
            "mode": "quantized",
            "model_name": "bad-v2",
            "files": [{
                "path": "config.json",
                "archive_name": "config.json",
                "transform": "raw",
                "original_size": config.len(),
                "original_sha256": ""
            }]
        });
        zip.start_file(MANIFEST_NAME, options).unwrap();
        zip.write_all(manifest.to_string().as_bytes()).unwrap();
        zip.finish().unwrap();

        let err = load_capsule_to_memory(&capsule_path).unwrap_err();
        assert!(
            err.to_string().contains("must include a direct SHA-256"),
            "{err:#}"
        );
    }
}
