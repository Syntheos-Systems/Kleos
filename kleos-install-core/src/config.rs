//! Configuration types and generation for the Kleos installer wizard.
//!
//! The wizard collects a curated subset of settings, maps them onto a real
//! [`kleos_config::Config`] -- the exact struct the server deserializes at
//! startup -- and emits two files:
//!
//! * `engram.toml`: a flat config file whose keys are the real `Config` field
//!   names, produced by serializing the mapped `Config`. Because it is the same
//!   schema the server reads, it is guaranteed to round-trip; the two can no
//!   longer drift. (The previous generator wrote `[server]`/`[embedding]`/...
//!   tables that the flat `Config` silently discarded, so most wizard choices
//!   were dropped.)
//! * `.env`: secrets plus the settings the server reads *only* from the
//!   environment -- the `KLEOS_CONFIG_FILE` pointer that makes the server load
//!   the TOML at all, open-access flags, CORS origins, and remote
//!   embedding/reranker credentials.

use std::path::{Path, PathBuf};

use crate::error::InstallError;

/// Core server configuration values collected during the wizard.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerConfig {
    /// Bind address for the HTTP server (e.g. "127.0.0.1").
    pub host: String,
    /// TCP port the server listens on.
    pub port: u16,
    /// Absolute path to the directory where Kleos stores runtime data.
    pub data_dir: PathBuf,
    /// Filename (relative to data_dir) of the SQLCipher database.
    pub db_path: String,
    /// Optional comma-separated list of allowed CORS origins.
    pub cors_origins: Option<String>,
}

/// Default server configuration used when the wizard does not override a field.
impl Default for ServerConfig {
    /// Return a sensible default server configuration.
    ///
    /// Binds to localhost on port 4200, stores data in `./data`, and uses
    /// `kleos.db` as the database filename.
    fn default() -> Self {
        ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 4200,
            data_dir: PathBuf::from("./data"),
            db_path: "kleos.db".to_string(),
            cors_origins: None,
        }
    }
}

/// Configuration for the embedding provider used by the server.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum EmbeddingConfig {
    /// Use a locally downloaded ONNX model.
    LocalOnnx {
        /// Model identifier / name (e.g. "BAAI/bge-m3").
        model: String,
        /// Output embedding dimension expected from the model.
        dimension: u32,
        /// Whether the installer should automatically download the model files.
        auto_download: bool,
    },
    /// Use a remote HTTP embedding endpoint (OpenAI-compatible).
    Remote {
        /// Base URL of the embedding API endpoint.
        url: String,
        /// API key sent in the `Authorization: Bearer` header.
        api_key: String,
        /// Optional model name passed in the request body.
        model: Option<String>,
        /// Expected output dimension of the remote model.
        dimension: u32,
    },
}

/// Configuration for the reranker component used during retrieval.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RerankerConfig {
    /// Use a locally downloaded ONNX reranker model.
    LocalOnnx,
    /// Use a remote HTTP reranker endpoint.
    Remote {
        /// Base URL of the reranker API endpoint.
        endpoint: String,
        /// API key for the remote reranker.
        api_key: String,
        /// Model name to request from the remote endpoint.
        model: String,
    },
    /// Disable reranking entirely.
    Disabled,
}

/// Security-related secrets and access-control settings.
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// 64-char hex key used to encrypt the SQLCipher database.
    pub encryption_key: String,
    /// 64-char hex pepper mixed into API key hashes before storage.
    pub api_key_pepper: String,
    /// The initial API key generated for the first user, prefixed with "kleos_".
    pub initial_api_key: String,
    /// 64-char hex secret used to sign HMAC tokens.
    pub hmac_secret: String,
    /// When `true`, the server allows unauthenticated read-only access
    /// (development / demo mode). Enabling it writes all three env vars the
    /// server requires together (see [`InstallerConfig::generate_env`]).
    pub open_access: bool,
}

/// Complete installer configuration assembled from all wizard steps.
///
/// This is the canonical in-memory representation from which engram.toml and
/// .env are generated.
#[derive(Debug, Clone)]
pub struct InstallerConfig {
    /// Server bind / storage configuration, present when the server is being installed.
    pub server: Option<ServerConfig>,
    /// Embedding provider selection and parameters.
    pub embedding: Option<EmbeddingConfig>,
    /// Reranker selection and parameters.
    pub reranker: Option<RerankerConfig>,
    /// Security secrets -- always required.
    pub security: SecurityConfig,
}

/// Reject a config value that would corrupt the line-oriented `.env` format: a
/// newline injects an additional `KEY=VALUE` line.
fn env_value(s: &str) -> Result<&str, InstallError> {
    if s.contains('\n') || s.contains('\r') {
        return Err(InstallError::Config(
            "config value contains a newline; refusing to write .env".to_string(),
        ));
    }
    Ok(s)
}

/// Write a secrets file with owner-only permissions (0600 on Unix). The default
/// umask leaves `std::fs::write` world-readable, which would expose the `.env`
/// holding ENGRAM_DB_KEY, the API-key pepper, and the HMAC secret.
fn write_private_file(path: &Path, contents: &str) -> Result<(), InstallError> {
    write_with_mode(path, contents, 0o600)
}

/// Write a non-secret file with group/world-readable permissions (0644 on Unix).
/// Used for `engram.toml`, which contains no secrets but should have explicit,
/// predictable permissions rather than whatever the process umask yields.
fn write_public_file(path: &Path, contents: &str) -> Result<(), InstallError> {
    write_with_mode(path, contents, 0o644)
}

/// Write `contents` to `path`, forcing the given Unix permission `mode` on both
/// new and pre-existing files. On non-Unix platforms `mode` is ignored.
fn write_with_mode(path: &Path, contents: &str, mode: u32) -> Result<(), InstallError> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(mode)
            .open(path)?;
        // mode() only applies on creation; tighten an already-existing file too.
        f.set_permissions(std::fs::Permissions::from_mode(mode))?;
        f.write_all(contents.as_bytes())?;
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = mode;
        std::fs::write(path, contents)?;
        Ok(())
    }
}

/// Config-file generation: mapping wizard choices onto the real server schema
/// and rendering the `engram.toml` / `.env` pair.
impl InstallerConfig {
    /// Map the wizard-collected values onto a real [`kleos_config::Config`].
    ///
    /// Every field the wizard does not collect keeps its server default, so the
    /// emitted TOML is a complete, valid config. Secrets and env-only settings
    /// (open access, CORS, remote credentials) are deliberately NOT placed here
    /// -- they live in `.env` (see [`Self::generate_env`]).
    fn to_kleos_config(&self) -> kleos_config::Config {
        let mut c = kleos_config::Config::default();

        if let Some(srv) = &self.server {
            c.host = srv.host.clone();
            c.port = srv.port;
            c.data_dir = srv.data_dir.display().to_string();
            c.db_path = srv.db_path.clone();
        }

        match &self.embedding {
            Some(EmbeddingConfig::LocalOnnx {
                model, dimension, ..
            }) => {
                c.embedding_model = model.clone();
                c.embedding_dim = *dimension as usize;
                c.vector_dimensions = *dimension as usize;
            }
            Some(EmbeddingConfig::Remote {
                model, dimension, ..
            }) => {
                // url / api_key go to .env; the dimension must land in Config so
                // the vector store matches the remote model's output size.
                if let Some(m) = model {
                    c.embedding_model = m.clone();
                }
                c.embedding_dim = *dimension as usize;
                c.vector_dimensions = *dimension as usize;
            }
            None => {}
        }

        match &self.reranker {
            // Remote selection is wired entirely through .env (backend=http +
            // endpoint/key/model); the server just needs reranking enabled.
            Some(RerankerConfig::LocalOnnx) | Some(RerankerConfig::Remote { .. }) => {
                c.reranker_enabled = true;
            }
            Some(RerankerConfig::Disabled) => {
                c.reranker_enabled = false;
            }
            None => {}
        }

        c
    }

    /// Generate a complete flat `engram.toml` as a string.
    ///
    /// Serializes the mapped [`kleos_config::Config`] through a [`toml::Value`]
    /// intermediate, which guarantees scalar fields are emitted before nested
    /// tables (a raw `to_string` on the struct can otherwise produce invalid
    /// TOML when a table field precedes later scalars). Returns
    /// `InstallError::Config` if serialization fails.
    pub fn generate_toml(&self) -> Result<String, InstallError> {
        let cfg = self.to_kleos_config();
        let value = toml::Value::try_from(&cfg)
            .map_err(|e| InstallError::Config(format!("serialize config to TOML value: {e}")))?;
        let body = toml::to_string(&value)
            .map_err(|e| InstallError::Config(format!("render engram.toml: {e}")))?;

        let mut out = String::from(
            "# Kleos server configuration (engram.toml).\n\
             # Generated by the installer. Every key maps directly to the server's\n\
             # Config schema -- edit a value and restart the server to change it.\n\
             # Secrets and env-only settings live in the sibling .env file.\n\n",
        );
        out.push_str(&body);
        Ok(out)
    }

    /// Generate a `.env` file string with secrets and env-only settings.
    ///
    /// `toml_path` is the absolute path of the generated `engram.toml`; it is
    /// written as `KLEOS_CONFIG_FILE` so the server actually loads the TOML.
    /// (The server resolves its config via `KLEOS_CONFIG_FILE` -> `./engram.toml`
    /// -> XDG; it does not parse a `--config` argument, so without this pointer a
    /// service launched from an arbitrary working directory falls back to
    /// built-in defaults and every TOML value is ignored.)
    pub fn generate_env(&self, toml_path: &Path) -> Result<String, InstallError> {
        let mut out = String::new();
        out.push_str("# Kleos environment secrets -- keep this file private\n");

        // Make the server load the generated TOML regardless of working dir.
        out.push_str(&format!(
            "KLEOS_CONFIG_FILE={}\n",
            env_value(&toml_path.display().to_string())?
        ));

        out.push_str(&format!("ENGRAM_DB_KEY={}\n", self.security.encryption_key));
        out.push_str(&format!(
            "ENGRAM_API_KEY_PEPPER={}\n",
            self.security.api_key_pepper
        ));
        out.push_str(&format!(
            "ENGRAM_HMAC_SECRET={}\n",
            self.security.hmac_secret
        ));

        // Open access (anonymous, read-only) requires all three vars together;
        // the server fails closed if any one is missing, and OPEN_ACCESS=1 with
        // tenant sharding ON is a fatal startup error -- so pin single-tenant.
        if self.security.open_access {
            out.push_str("KLEOS_OPEN_ACCESS=1\n");
            out.push_str("KLEOS_ALLOW_OPEN_ACCESS_IN_RELEASE=yes-i-am-sure\n");
            out.push_str("KLEOS_TENANT_SHARDING=0\n");
        }

        // CORS is read only from ENGRAM_ALLOWED_ORIGINS, never from the TOML.
        if let Some(srv) = &self.server {
            if let Some(origins) = &srv.cors_origins {
                out.push_str(&format!("ENGRAM_ALLOWED_ORIGINS={}\n", env_value(origins)?));
            }
        }

        // Remote embedding endpoint: url + key in env, dimension via the env var
        // the remote provider reads directly.
        if let Some(EmbeddingConfig::Remote {
            url,
            api_key,
            dimension,
            ..
        }) = &self.embedding
        {
            out.push_str(&format!("KLEOS_EMBEDDING_URL={}\n", env_value(url)?));
            out.push_str(&format!(
                "KLEOS_EMBEDDING_API_KEY={}\n",
                env_value(api_key)?
            ));
            out.push_str(&format!("KLEOS_EMBEDDING_DIM={dimension}\n"));
        }

        // Remote reranker endpoint: select the http backend and supply creds.
        if let Some(RerankerConfig::Remote {
            endpoint,
            api_key,
            model,
        }) = &self.reranker
        {
            out.push_str("KLEOS_RERANKER_BACKEND=http\n");
            out.push_str(&format!("KLEOS_RERANKER_URL={}\n", env_value(endpoint)?));
            out.push_str(&format!("KLEOS_RERANKER_API_KEY={}\n", env_value(api_key)?));
            out.push_str(&format!("KLEOS_RERANKER_MODEL={}\n", env_value(model)?));
        }

        Ok(out)
    }

    /// Write `engram.toml` and `.env` into `config_dir`.
    ///
    /// Creates `config_dir` if needed, writes the TOML (0644) first so its
    /// absolute path can be resolved, then writes the private `.env` (0600)
    /// pointing at it. Returns `InstallError::Config` on generation failure or
    /// `InstallError::Io` on a filesystem error.
    pub fn write_config(&self, config_dir: &Path) -> Result<(), InstallError> {
        std::fs::create_dir_all(config_dir)?;

        let toml_path = config_dir.join("engram.toml");
        let toml_content = self.generate_toml()?;
        write_public_file(&toml_path, &toml_content)?;

        // Prefer the canonical absolute path for KLEOS_CONFIG_FILE so it resolves
        // from any working directory; fall back to the joined path if the FS
        // refuses to canonicalize.
        let abs_toml = std::fs::canonicalize(&toml_path).unwrap_or(toml_path);
        let env_content = self.generate_env(&abs_toml)?;
        write_private_file(&config_dir.join(".env"), &env_content)?;

        Ok(())
    }
}

/// Round-trip and emission tests guarding against config silently being dropped.
#[cfg(test)]
mod tests {
    use super::*;

    /// Build an InstallerConfig with distinctive, non-default values so a
    /// round-trip can prove each one survives.
    fn sample_config() -> InstallerConfig {
        InstallerConfig {
            server: Some(ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 4242,
                data_dir: PathBuf::from("/var/lib/kleos"),
                db_path: "custom.db".to_string(),
                cors_origins: Some("https://app.example.com".to_string()),
            }),
            embedding: Some(EmbeddingConfig::LocalOnnx {
                model: "intfloat/e5-large".to_string(),
                dimension: 512,
                auto_download: true,
            }),
            reranker: Some(RerankerConfig::Disabled),
            security: SecurityConfig {
                encryption_key: "deadbeef".to_string(),
                api_key_pepper: "cafef00d".to_string(),
                initial_api_key: "kleos_test".to_string(),
                hmac_secret: "feedface".to_string(),
                open_access: false,
            },
        }
    }

    // The generated engram.toml MUST deserialize into the real server Config
    // with every wizard value intact -- the regression guard against the table
    // vs flat schema mismatch that silently dropped config.
    #[test]
    fn generated_toml_roundtrips_into_server_config() {
        let installer = sample_config();
        let toml = installer.generate_toml().expect("generate toml");
        let cfg: kleos_config::Config = toml::from_str(&toml).expect("parse as server Config");

        assert_eq!(cfg.host, "0.0.0.0");
        assert_eq!(cfg.port, 4242);
        assert_eq!(cfg.data_dir, "/var/lib/kleos");
        assert_eq!(cfg.db_path, "custom.db");
        assert_eq!(cfg.embedding_model, "intfloat/e5-large");
        assert_eq!(cfg.embedding_dim, 512);
        assert_eq!(cfg.vector_dimensions, 512);
        // "Disabled" in the wizard must actually disable the reranker.
        assert!(!cfg.reranker_enabled);
    }

    // CORS belongs in .env under the key the server reads, never in the TOML.
    #[test]
    fn cors_goes_to_env_not_toml() {
        let installer = sample_config();
        let toml = installer.generate_toml().expect("toml");
        assert!(!toml.contains("app.example.com"));

        let env = installer
            .generate_env(Path::new("/etc/kleos/engram.toml"))
            .expect("env");
        assert!(env.contains("ENGRAM_ALLOWED_ORIGINS=https://app.example.com"));
        assert!(env.contains("KLEOS_CONFIG_FILE=/etc/kleos/engram.toml"));
    }

    // Enabling open access must emit all three required env vars together.
    #[test]
    fn open_access_emits_all_required_vars() {
        let mut installer = sample_config();
        installer.security.open_access = true;
        let env = installer
            .generate_env(Path::new("/etc/kleos/engram.toml"))
            .expect("env");
        assert!(env.contains("KLEOS_OPEN_ACCESS=1"));
        assert!(env.contains("KLEOS_ALLOW_OPEN_ACCESS_IN_RELEASE=yes-i-am-sure"));
        assert!(env.contains("KLEOS_TENANT_SHARDING=0"));
    }

    // Secrets must never leak into the world-readable TOML.
    #[test]
    fn secrets_stay_out_of_toml() {
        let installer = sample_config();
        let toml = installer.generate_toml().expect("toml");
        assert!(!toml.contains("deadbeef"));
        assert!(!toml.contains("cafef00d"));
        assert!(!toml.contains("feedface"));
    }

    // A newline in an env value would inject an extra KEY=VALUE line.
    #[test]
    fn env_value_rejects_newlines() {
        assert!(env_value("ok-value").is_ok());
        assert!(env_value("inject\nKEY=val").is_err());
        assert!(env_value("inject\rKEY=val").is_err());
    }

    // Remote embedding routes url/key/dim to .env and keeps the dimension in the
    // TOML so the vector store matches the model.
    #[test]
    fn remote_embedding_propagates_dimension() {
        let mut installer = sample_config();
        installer.embedding = Some(EmbeddingConfig::Remote {
            url: "https://emb.example.com".to_string(),
            api_key: "sk-123".to_string(),
            model: Some("text-embedding-3-large".to_string()),
            dimension: 3072,
        });
        let toml = installer.generate_toml().expect("toml");
        let cfg: kleos_config::Config = toml::from_str(&toml).expect("parse");
        assert_eq!(cfg.embedding_dim, 3072);
        assert_eq!(cfg.vector_dimensions, 3072);

        let env = installer
            .generate_env(Path::new("/x/engram.toml"))
            .expect("env");
        assert!(env.contains("KLEOS_EMBEDDING_URL=https://emb.example.com"));
        assert!(env.contains("KLEOS_EMBEDDING_API_KEY=sk-123"));
        assert!(env.contains("KLEOS_EMBEDDING_DIM=3072"));
    }
}
