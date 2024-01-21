use windows::Win32::Media::Audio::AUDIO_VOLUME_NOTIFICATION_DATA;
use windows::Win32::Media::Audio::Endpoints::{IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl};

use crate::set_led_color;

// from https://github.com/Raphiiko/OyasumiVR/blob/4ec9eaaeca4d2541e7bd66734b6c438e4017d019/src-core/src/os/audio/device.rs#L114
#[windows::core::implement(IAudioEndpointVolumeCallback)]
pub struct AudioDeviceVolumeNotificationClient {}

impl AudioDeviceVolumeNotificationClient {
    pub fn new() -> IAudioEndpointVolumeCallback {
        Self {}.into()
    }
}

impl IAudioEndpointVolumeCallback_Impl for AudioDeviceVolumeNotificationClient {
    #[allow(non_snake_case)]
    fn OnNotify(&self, _notify: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> windows::core::Result<()> {
        unsafe {
            let muted = (*_notify).bMuted;
            println!("{:#?}", muted);
            set_led_color(muted);
            Ok(())
        }
    }
}