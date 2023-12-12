use windows::core::PCWSTR;
use windows::Win32::Media::Audio::{AUDIO_VOLUME_NOTIFICATION_DATA, eCapture, eCommunications, eConsole, EDataFlow, eMultimedia, eRender, ERole, IMMNotificationClient, IMMNotificationClient_Impl};
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolumeCallback_Impl;
use windows::Win32::UI::Shell::PropertiesSystem::PROPERTYKEY;

use crate::set_color;

// from https://github.com/Raphiiko/OyasumiVR/blob/4ec9eaaeca4d2541e7bd66734b6c438e4017d019/src-core/src/os/audio/device.rs#L114
#[windows::core::implement(IMMNotificationClient)]
pub struct DeviceChangedNotificationClient {}

impl DeviceChangedNotificationClient {
    pub fn new() -> IMMNotificationClient {
        Self {}.into()
    }
}

impl IMMNotificationClient_Impl for DeviceChangedNotificationClient {
    #[allow(non_snake_case)]
    fn OnDeviceStateChanged(&self, pwstrdeviceid: &PCWSTR, dwnewstate: u32) -> windows::core::Result<()> {
        Ok(())
    }
    #[allow(non_snake_case)]
    fn OnDeviceAdded(&self, pwstrdeviceid: &PCWSTR) -> windows::core::Result<()> {
        Ok(())
    }
    #[allow(non_snake_case)]
    fn OnDeviceRemoved(&self, pwstrdeviceid: &PCWSTR) -> windows::core::Result<()> {
        Ok(())
    }
    #[allow(non_snake_case)]
    fn OnDefaultDeviceChanged(&self, flow: EDataFlow, role: ERole, pwstrdefaultdeviceid: &PCWSTR) -> windows::core::Result<()> {
        if flow != eCapture || role == eCommunications {
            return Ok(());
        }

        println!("OnDefaultDeviceChanged");
        Ok(())
    }
    #[allow(non_snake_case)]
    fn OnPropertyValueChanged(&self, pwstrdeviceid: &PCWSTR, key: &PROPERTYKEY) -> windows::core::Result<()> {
        Ok(())
    }
}