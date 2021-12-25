use super::{util, IntifaceConfiguration};
use buttplug::util::device_configuration::load_protocol_config_from_json;
use sentry::SentryFutureExt;
use std::sync::{
  atomic::{AtomicBool, AtomicU32, Ordering},
  Arc,
};
use std::{
  fs::File,
  io::{self, copy},
};
use thiserror::Error;
use tracing::{error, info};

#[cfg(debug_assertions)]
const BUTTPLUG_REPO_OWNER: &str = "qdot";
#[cfg(not(debug_assertions))]
const BUTTPLUG_REPO_OWNER: &str = "buttplugio";
#[cfg(debug_assertions)]
const INTIFACE_REPO_OWNER: &str = "qdot";
#[cfg(not(debug_assertions))]
const INTIFACE_REPO_OWNER: &str = "intiface";
const INTIFACE_DESKTOP_REPO: &str = "intiface-desktop-egui";
const INTIFACE_ENGINE_REPO: &str = "intiface-cli-rs";
const PRERELEASE_TAG: &str = "420.69.666";
const DEVICE_CONFIG_VERSION_URL: &str = "https://buttplug-rs-device-config.buttplug.io/version";
const DEVICE_CONFIG_URL: &str = "https://buttplug-rs-device-config.buttplug.io";

#[derive(Debug, Error)]
pub enum UpdateError {
  #[error("Error establishing connection to update URL {0}: {1}")]
  ConnectionError(String, String),
  #[error("Error retreiving information from Github: {0}")]
  GithubError(String),
  #[error("Invalid data received from version check: {0}")]
  InvalidData(String),
}

#[derive(Default)]
pub struct UpdateManager {
  needs_device_file_update: Arc<AtomicBool>,
  needs_engine_update: Arc<AtomicBool>,
  needs_application_update: Arc<AtomicBool>,
  latest_engine_version: Arc<AtomicU32>,
  latest_device_file_version: Arc<AtomicU32>,
  is_updating: Arc<AtomicBool>,
  has_errors: Arc<AtomicBool>,
}

impl UpdateManager {
  pub fn check_for_updates(&self, config: &IntifaceConfiguration) {
    let is_updating = self.is_updating.clone();
    let engine_version = config.current_engine_version().clone();
    let device_check_fut = UpdateManager::check_for_device_file_update(
      self.needs_device_file_update.clone(),
      self.latest_device_file_version.clone(),
    );
    let engine_check_fut = UpdateManager::check_for_engine_update(
      engine_version,
      self.needs_engine_update.clone(),
      self.latest_engine_version.clone(),
    );
    tokio::spawn(async move {
      is_updating.store(true, Ordering::SeqCst);
      tokio::join!(
        async move { device_check_fut.await }.bind_hub(sentry::Hub::current().clone()),
        async move { engine_check_fut.await }.bind_hub(sentry::Hub::current().clone())
      );
      is_updating.store(false, Ordering::SeqCst);
    });
  }

  pub fn get_updates(&self) {
    let needs_device_file_update = self.needs_device_file_update.clone();
    let needs_engine_update = self.needs_engine_update.clone();
    let is_updating = self.is_updating.clone();
    tokio::spawn(async move {
      is_updating.store(true, Ordering::SeqCst);
      if needs_device_file_update.load(Ordering::SeqCst) {
        UpdateManager::download_device_file_update().await;
      }
      if needs_engine_update.load(Ordering::SeqCst) {
        UpdateManager::download_engine_update().await;
      }
      is_updating.store(false, Ordering::SeqCst);
    });
  }

  pub fn needs_updates(&self) -> bool {
    self.needs_application_update.load(Ordering::SeqCst)
      || self.needs_device_file_update.load(Ordering::SeqCst)
      || self.needs_engine_update.load(Ordering::SeqCst)
  }

  pub fn is_updating(&self) -> bool {
    self.is_updating.load(Ordering::SeqCst)
  }

  async fn check_for_device_file_update(
    needs_device_file_update: Arc<AtomicBool>,
    latest_device_file_version: Arc<AtomicU32>,
  ) {
    if !util::device_config_file_path().exists() {
      info!("No device configuration file found, prompting for update.");
      needs_device_file_update.store(true, Ordering::SeqCst);
      return;
    }

    let device_config_file = String::from_utf8(
      tokio::fs::read(util::device_config_file_path())
        .await
        .unwrap(),
    )
    .unwrap();
    let device_config = match load_protocol_config_from_json(&device_config_file) {
      Ok(cfg) => cfg,
      Err(e) => {
        error!("{:?}", e);
        needs_device_file_update.store(true, Ordering::SeqCst);
        return;
      }
    };

    let version = reqwest::get(DEVICE_CONFIG_VERSION_URL)
      .await
      /*
      .map_err(|e| {
        UpdateError::ConnectionError(DEVICE_CONFIG_VERSION_URL.to_string(), e.to_string())
      })?
      */
      .unwrap()
      .text()
      .await
      .unwrap()
      //.map_err(|e| UpdateError::InvalidData(e.to_string()))?
      .parse::<u32>()
      .unwrap();
    //.map_err(|e| UpdateError::InvalidData(e.to_string()))?;
    info!(
      "Local device file version: {} - Remote device file Version: {}",
      device_config.version, version
    );
    latest_device_file_version.store(version, Ordering::SeqCst);
    needs_device_file_update.store(version > device_config.version, Ordering::SeqCst);
  }

  async fn check_for_application_update(
    &self,
    config: &IntifaceConfiguration,
  ) -> Result<bool, UpdateError> {
    let release = octocrab::instance()
      .repos(INTIFACE_REPO_OWNER, INTIFACE_DESKTOP_REPO)
      .releases()
      .get_latest()
      .await
      .map_err(|e| UpdateError::GithubError(e.to_string()))?;
    // TODO: Implement version update check
    //Ok(release.tag_name != (*self.config.read().await).current_application_tag)
    unimplemented!("Need to implement version update check.")
  }

  async fn check_for_engine_update(
    engine_version: u32,
    needs_engine_update: Arc<AtomicBool>,
    latest_engine_version: Arc<AtomicU32>,
  ) -> Result<(), UpdateError> {
    let release = octocrab::instance()
      .repos(INTIFACE_REPO_OWNER, INTIFACE_ENGINE_REPO)
      .releases()
      .get_latest()
      .await
      .map_err(|e| UpdateError::GithubError(e.to_string()))?;
    let current_version_number = release.tag_name[1..].parse::<u32>().unwrap();
    info!(
      "Local engine version: {} - Remote Engine Version: {}",
      engine_version, current_version_number
    );
    latest_engine_version.store(current_version_number, Ordering::SeqCst);
    needs_engine_update.store(current_version_number != engine_version, Ordering::SeqCst);
    Ok(())
  }

  pub async fn download_device_file_update() {
    let response = reqwest::get(DEVICE_CONFIG_URL).await.unwrap();

    let mut dest = { File::create(super::util::device_config_file_path()).unwrap() };
    let content = response.text().await.unwrap();
    copy(&mut content.as_bytes(), &mut dest).unwrap();
  }

  pub async fn download_application_update(&self, config: &IntifaceConfiguration) {
  }

  async fn download_engine_update() {
    #[cfg(target_os = "windows")]
    let platform = "win-x64";
    #[cfg(target_os = "linux")]
    let platform = "linux-x64";
    #[cfg(target_os = "darwin")]
    let platform = "macos-x64";

    let release_name = format!("intiface-cli-rs-{}-Release.zip", platform);
    let release = octocrab::instance()
      .repos(INTIFACE_REPO_OWNER, INTIFACE_ENGINE_REPO)
      .releases()
      .get_latest()
      .await
      .map_err(|e| UpdateError::GithubError(e.to_string()))
      .unwrap();

    for asset in release.assets {
      if asset.name.starts_with(&release_name) {
        info!(
          "Found release asset {} for version {}",
          asset.name, release.tag_name
        );
        info!("Getting {}", asset.browser_download_url);
        let file_bytes = reqwest::get(asset.browser_download_url)
          .await
          .unwrap()
          .bytes()
          .await
          .unwrap();
        let reader = std::io::Cursor::new(&file_bytes);
        let mut files = zip::ZipArchive::new(reader).unwrap();
        for file_idx in 0..files.len() {
          let mut file = files.by_index(file_idx).unwrap();
          if file.enclosed_name().is_none() {
            continue;
          };
          let extension = file.enclosed_name().unwrap().extension();
          if extension.is_some() && extension.unwrap() == "md" {
            info!(
              "Skipping extracting file {} from zip...",
              file.enclosed_name().unwrap().display()
            );
            continue;
          }
          info!(
            "Extracting file {} from zip...",
            file.enclosed_name().unwrap().display()
          );
          let final_out_path = util::engine_file_path();
          let mut outfile = File::create(&final_out_path).unwrap();
          io::copy(&mut file, &mut outfile).unwrap();
        }

        break;
      }
    }
  }

  pub fn update_config(&self, config: &mut IntifaceConfiguration) {
    *config.current_device_file_version_mut() =
      self.latest_device_file_version.load(Ordering::SeqCst);
    *config.current_engine_version_mut() = self.latest_engine_version.load(Ordering::SeqCst);
    super::save_config_file(&serde_json::to_string(&config).unwrap()).unwrap();
  }

  pub async fn install_application(&self, config: &IntifaceConfiguration) {
  }
}
#[cfg(test)]
mod test {
  use super::super::IntifaceConfiguration;
  use super::*;

  /*
  #[test]
  fn test_device_file_update() {
    // Create the runtime
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Execute the future, blocking the current thread until completion
    rt.block_on(async move {
      let config = IntifaceConfiguration::default();
      let manager = UpdateManager::default();
      // Should always return true.
      assert!(manager.check_for_device_file_update().await.unwrap());
    })
  }
  */

  #[test]
  fn test_engine_update() {
    // Create the runtime
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Execute the future, blocking the current thread until completion
    rt.block_on(async move {
      let config = IntifaceConfiguration::default();
      let manager = UpdateManager::default();
      // Should always return true.
      //assert!(manager.check_for_engine_update(&config).await.unwrap());
    })
  }
}
