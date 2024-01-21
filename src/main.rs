use std::ptr::null;
use std::thread::park;

use cue_sdk::led::{KeyboardLedId, LedColor, LedId};
use cue_sdk::led::CueLed;
use windows::Win32::{Media::Audio::*, System::Com::*};
use windows::Win32::Foundation::{BOOL, FALSE, HWND, TRUE};
use windows::Win32::Media::Audio::Endpoints::{IAudioEndpointVolumeCallback, IAudioEndpointVolumeEx};
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_HOTKEY};

use audio_device_volume_notification_client::*;

mod audio_device_volume_notification_client;

#[derive(Debug)]
struct DeviceEnumerator(IMMDeviceEnumerator);

fn main() {
    let enumerator: IMMDeviceEnumerator = get_audio_enumerator();
    // getting multiple enumerators always results in same pointer
    let endpoint = get_default_input_endpoint(&enumerator);

    unsafe { set_led_color(endpoint.GetMute().unwrap()); }

    println!("registering to {:?}", endpoint);

    register_hotkey(VK_F13, &enumerator);
    println!("hotkey_handle finished");

    park();
}

fn set_led_color(muted: BOOL) {
    let cue = cue_sdk::initialize().expect("cue sdk should be running");
    let mut devices = cue.get_all_devices().expect("device should be connected");
    let mute_led: Option<&mut CueLed> = devices.first_mut().and_then(|device| {
        device.leds.iter_mut().find(|key| key.id == LedId::Keyboard(KeyboardLedId::KeyMute))
    });

    let color = match muted {
        TRUE => LedColor { red: 255, green: 0, blue: 0 },
        FALSE => LedColor { red: 0, green: 255, blue: 0 },
        _ => LedColor { red: 255, green: 255, blue: 0 },
    };

    dbg!("changing color to {}", color);

    mute_led.expect("keyboard should contain mute led").update_color_buffer(color).unwrap();
    cue.flush_led_colors_update_buffer_sync().unwrap();
}

fn get_audio_enumerator() -> IMMDeviceEnumerator {
    unsafe {
        println!("CoInitialize");
        // CoInitializeEx(Some(null_mut()), COINIT_MULTITHREADED).unwrap();
        CoInitialize(Some(null())).unwrap();
        // RoInitialize(RO_INIT_TYPE(1)).unwrap();
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).unwrap();
        // dbg!(&enumerator);
        enumerator
    }
}

fn get_default_input_endpoint(enumerator: &IMMDeviceEnumerator) -> IAudioEndpointVolumeEx {
    unsafe {
        let device = enumerator
            .GetDefaultAudioEndpoint(eCapture, eMultimedia)
            .unwrap();

        device.Activate::<IAudioEndpointVolumeEx>(CLSCTX_ALL, Some(null())).unwrap()
    }
}

fn register_devicestate_change_notify(endpoint: &IAudioEndpointVolumeEx) -> IAudioEndpointVolumeCallback {
    unsafe {
        let notification_client = AudioDeviceVolumeNotificationClient::new();

        // if endpoint gets dropped it will unregister the notification client
        endpoint.RegisterControlChangeNotify(&notification_client).unwrap();
        notification_client
    }
}

fn register_hotkey(key: VIRTUAL_KEY, enumerator: &IMMDeviceEnumerator) {
    unsafe {
        let hotkey = RegisterHotKey(HWND(0), 1, MOD_NOREPEAT, key.0.into());
        println!("hotkey registered: {:?}", hotkey);

        let endpoint = get_default_input_endpoint(enumerator);

        println!("state_handle started");
        let _volume_callback = register_devicestate_change_notify(&endpoint);
        println!("state_handle finished");

        let mut msg = MSG::default();
        loop {
            GetMessageW(&mut msg, None, 0, 0);

            match msg.message {
                WM_HOTKEY => {
                    match endpoint.GetMute().unwrap() {
                        FALSE => endpoint.SetMute(TRUE, null()).unwrap(),
                        TRUE => endpoint.SetMute(FALSE, null()).unwrap(),
                        _ => {}
                    }
                }
                _ => println!("{:?}", msg)
            }
        };
    }
}
