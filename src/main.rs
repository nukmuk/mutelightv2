mod audio_device_volume_notification_client;
mod device_changed_notification_client;

use audio_device_volume_notification_client::*;

use std::{ptr, thread};
use std::ptr::{null, null_mut};
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{park, sleep};
use std::time::Duration;
use cue_sdk::device::CueDevice;
use cue_sdk::device::DeviceLayout::Keyboard;
use cue_sdk::led::{CueLed, LedColor};
use cue_sdk::led::KeyboardLedId::{KeyMute, KeyScanNextTrack, KeyWinLock};
use rdev::{Event, EventType, grab, Key};

use windows::Win32::{Media::Audio::*, System::Com::*};
use windows::Win32::Foundation::{BOOL, FALSE, HWND, TRUE};
use windows::Win32::Media::Audio::Endpoints::{IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl, IAudioEndpointVolumeEx};
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::{GetMessageA, MSG, WM_INPUT, WM_KEYFIRST};
use log::{debug, error};
use tokio::sync::mpsc;
use crate::device_changed_notification_client::DeviceChangedNotificationClient;

#[tokio::main]
async fn main() {
    unsafe {
        let device_handle = thread::spawn(|| {
            let enumerator = get_audio_enumerator();
            detect_device_changed(enumerator);
            park();
        });

        println!("device changed detected");

        sleep(Duration::from_secs(1));

        println!("sleeped for 1 second");

        // let state_handle = thread::spawn(|| {
        //     let endpoint = register_devicestate_change_notify();
        //     park();
        // });

        let hotkey_handle = thread::spawn(|| {
            register_hotkey(VK_INSERT);
        });

        // wait for all threads to finish
        device_handle.join().unwrap();
        // state_handle.join().unwrap();
        hotkey_handle.join().unwrap();
    }
}

fn detect_device_changed(enumerator: IMMDeviceEnumerator) {
    unsafe {
        let notification_client = DeviceChangedNotificationClient::new();

        enumerator.RegisterEndpointNotificationCallback(&notification_client).unwrap();
    }
}

fn set_color(muted: BOOL) {
    let mute_key = 84;
    let cue = cue_sdk::initialize().unwrap();
    let mut devices = cue.get_all_devices().unwrap();

    let color = match muted {
        TRUE => LedColor { red: 255, green: 0, blue: 0 },
        FALSE => LedColor { red: 0, green: 255, blue: 0 },
        _ => LedColor { red: 255, green: 255, blue: 0 },
    };

    dbg!("changing color to {}", color);

    devices.iter_mut().for_each(|mut d: &mut CueDevice| {
        d.leds.get_mut(mute_key).unwrap().update_color_buffer(color).unwrap();
        cue.flush_led_colors_update_buffer_sync().unwrap();
    });
}

unsafe fn get_audio_enumerator() -> IMMDeviceEnumerator {
    // CoInitializeEx(Some(null_mut()), COINIT_MULTITHREADED).unwrap();
    RoInitialize();
    let enumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).unwrap();
    enumerator
}

unsafe fn get_default_input_endpoint(enumerator: &IMMDeviceEnumerator) -> IAudioEndpointVolumeEx {
    let device = enumerator
        .GetDefaultAudioEndpoint(eCapture, eMultimedia)
        .unwrap();

    device.Activate::<IAudioEndpointVolumeEx>(CLSCTX_ALL, Some(ptr::null())).unwrap()
}

fn register_devicestate_change_notify() -> IAudioEndpointVolumeEx {
    unsafe {
        let enumerator = get_audio_enumerator();
        let endpoint = get_default_input_endpoint(&enumerator);

        // let (on_notify_tx, mut on_notify_rx) = ch/annel(10);

        let notification_client = AudioDeviceVolumeNotificationClient::new();

        endpoint.RegisterControlChangeNotify(&notification_client).unwrap();

        // while let Some(msg) = on_notify_rx.recv().await {
        //     println!("received control change notify: {:?}", msg);
        //     let muted = endpoint.GetMute().unwrap();
        //     match muted {
        //         TRUE => set_color(true),
        //         FALSE => set_color(false),
        //         _ => {}
        //     };
        // }

        endpoint

        // println!("unregistering control change notify");
        // endpoint.UnregisterControlChangeNotify(&notification_client).unwrap();
    }
}

fn register_hotkey(key: VIRTUAL_KEY) {
    unsafe {
        let hotkey = windows::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey(HWND(0), 1, MOD_NOREPEAT, key.0.into());
        println!("hotkey registered: {:?}", hotkey);

        // let enumerator = get_audio_enumerator();

        let mut msg = MSG::default();
        while GetMessageA(&mut msg, HWND(-1), 0, 0) != FALSE {
            println!("{:?}", msg);
            //     toggle_mute(&enumerator);
        };
    };
}

fn toggle_mute(enumerator: &IMMDeviceEnumerator) {
    unsafe {
        let endpoint = get_default_input_endpoint(enumerator);

        let muted = endpoint.GetMute().unwrap();
        match muted {
            TRUE => endpoint.SetMute(FALSE, ptr::null()).unwrap(),
            FALSE => endpoint.SetMute(TRUE, ptr::null()).unwrap(),
            _ => {}
        }
    }
}

/*

maybe old code idk:

    unsafe {
        // CoInitializeEx(Some(ptr::null()), COINIT_MULTITHREADED).unwrap();

        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).unwrap();
        let device = enumerator
            .GetDefaultAudioEndpoint(eCapture, eMultimedia)
            .unwrap();

        let endpoint = device.Activate::<IAudioEndpointVolumeEx>(CLSCTX_ALL, Some(ptr::null())).unwrap();
        let muted = endpoint.GetMute().unwrap();
        match muted {
            TRUE => endpoint.SetMute(FALSE, ptr::null()).unwrap(),
            FALSE => endpoint.SetMute(TRUE, ptr::null()).unwrap(),
            _ => {}
        }

        println!("{:#?}", muted);
        // let manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, ptr::null()).unwrap();
        // let sessions = manager.GetSessionEnumerator().unwrap();
        //
        // for n in 0..sessions.GetCount().unwrap() {
        //     let session_control = sessions.GetSession(n).unwrap();
        // }

        // CoUninitialize();
    }
*/