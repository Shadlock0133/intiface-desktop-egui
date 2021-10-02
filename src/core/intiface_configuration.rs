use getset::{Getters, Setters, MutGetters, CopyGetters};
use serde::{Deserialize, Serialize};

#[derive(Setters, MutGetters, Getters, CopyGetters, Serialize, Deserialize, PartialEq, Clone)]
#[getset(get_mut = "pub", set = "pub")]
pub struct IntifaceConfiguration {
  #[getset(get = "pub")]
  #[serde(rename="serverName", default)]
  server_name: String,
  #[getset(get_copy = "pub")]
  #[serde(rename="serverMaxPingTime", default)]
  server_max_ping_time: u32,
  #[getset(get_copy = "pub")]
  #[serde(rename="useWebsocketServerInsecure", default)]
  use_websocket_server_insecure: bool,
  #[getset(get_copy = "pub")]
  #[serde(rename="websocketServerAllInterfaces", default)]
  websocket_server_all_interfaces: bool,
  #[getset(get_copy = "pub")]
  #[serde(rename="websocketServerInsecurePort", default)]
  websocket_server_insecure_port: u16,
  #[getset(get = "pub")]
  #[serde(rename="serverLogLevel", default)]
  server_log_level: String,
  #[getset(get_copy = "pub")]
  #[serde(rename="usePrereleaseEngine", default)]
  use_prerelease_engine: bool,
  #[getset(get = "pub")]
  #[serde(rename="currentEngineVersion", default)]
  current_engine_version: String,
  #[getset(get_copy = "pub")]
  #[serde(rename="currentDeviceFileVersion", default)]
  current_device_file_version: u32,
  #[getset(get_copy = "pub")]
  #[serde(rename="checkForUpdatesOnStart", default)]
  check_for_updates_on_start: bool,
  #[getset(get_copy = "pub")]
  #[serde(rename="hasRunSetup", default)]
  has_run_setup: bool,
  #[getset(get_copy = "pub")]
  #[serde(skip)]
  device_file_update_available: bool,
  #[getset(get_copy = "pub")]
  #[serde(skip)]
  engine_update_available: bool,
  #[getset(get_copy = "pub")]
  #[serde(skip)]
  application_update_available: bool,
  #[getset(get_copy = "pub")]
  #[serde(skip)]
  has_usable_engine_executable: bool,
  #[getset(get_copy = "pub")]
  #[serde(rename="startServerOnStartup", default)]
  start_server_on_startup: bool,
  #[getset(get_copy = "pub")]
  #[serde(rename="withBluetoothLE", default)]
  with_bluetooth_le: bool,
  #[getset(get_copy = "pub")]
  #[serde(rename="withSerialPort", default)]
  with_serial_port: bool,
  #[getset(get_copy = "pub")]
  #[serde(rename="withHID", default)]
  with_hid: bool,
  #[getset(get_copy = "pub")]
  #[serde(rename="withLovenseHIDDongle", default)]
  with_lovense_hid_dongle: bool,
  #[getset(get_copy = "pub")]
  #[serde(rename="withLovenseSerialDongle")]
  with_lovense_serial_dongle: bool,
  #[getset(get_copy = "pub")]
  #[serde(rename="withLovenseConnectService")]
  with_lovense_connect_service: bool,
  #[getset(get_copy = "pub")]
  #[serde(rename="withXInput")]
  with_xinput: bool,
}

impl Default for IntifaceConfiguration {
  fn default() -> Self {
    Self {
      server_name: "Intiface Desktop Server".to_owned(),
      server_max_ping_time: 0,
      use_websocket_server_insecure: true,
      websocket_server_all_interfaces: false,
      websocket_server_insecure_port: 12345,
      server_log_level: "info".to_owned(),
      use_prerelease_engine: false,
      current_engine_version: "0".to_owned(),
      current_device_file_version: 0,
      check_for_updates_on_start: true,
      has_run_setup: false,
      device_file_update_available: false,
      engine_update_available: false,
      application_update_available: false,
      has_usable_engine_executable: false,
      start_server_on_startup: false,
      with_bluetooth_le: true,
      with_serial_port: true,
      with_hid: true,
      with_lovense_hid_dongle: true,
      with_lovense_serial_dongle: true,
      with_lovense_connect_service: false,
      with_xinput: true,
    }
  }
}

impl IntifaceConfiguration {
  pub fn load_from_string(json_config: &str) -> Result<IntifaceConfiguration, serde_json::Error> {
    serde_json::from_str::<IntifaceConfiguration>(json_config)
  }
}