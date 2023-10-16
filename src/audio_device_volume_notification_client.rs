use log::{debug, error};
use tokio::runtime::Handle;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use windows::Win32::Media::Audio::AUDIO_VOLUME_NOTIFICATION_DATA;
use windows::Win32::Media::Audio::Endpoints::{IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl};

// from https://github.com/Raphiiko/OyasumiVR/blob/4ec9eaaeca4d2541e7bd66734b6c438e4017d019/src-core/src/os/audio/device.rs#L114
#[windows::core::implement(IAudioEndpointVolumeCallback)]
pub struct AudioDeviceVolumeNotificationClient {
    handle: Handle,
    channel: Sender<()>,
}

impl AudioDeviceVolumeNotificationClient {
    pub fn new(channel: Sender<()>) -> IAudioEndpointVolumeCallback {
        let val = Self {
            handle: Handle::current(),
            channel,
        };
        val.into()
    }
}

impl IAudioEndpointVolumeCallback_Impl for AudioDeviceVolumeNotificationClient {
    #[allow(non_snake_case)]
    fn OnNotify(&self, _notify: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> windows::core::Result<()> {
        let tx = self.channel.clone();
        self.handle.spawn(async move {
            if let Err(e) = tx.send(()).await {
                error!(
                    "[Core] onNotify could not send channel notification: {:?}",
                    e
                );
            }
        });
        Ok(())
    }
}