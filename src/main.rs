use std::ptr;
use std::thread::sleep;
use std::time::Duration;
use rdev::{Event, EventType, grab, Key};

use windows::Win32::{Media::Audio::*, System::Com::*};
use windows::Win32::Foundation::{FALSE, HWND, TRUE};
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolumeEx;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::{GetMessageA, MSG, WM_INPUT, WM_KEYFIRST};

fn main() {
    unsafe {
        let hotkey = windows::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey(HWND(0), 1, MOD_NOREPEAT, VK_F13.0.into());
        println!("hotkey: {:?}", hotkey);
        println!("hwnd: {:?}", HWND::default());

        let mut msg = MSG::default();

        while GetMessageA(&mut msg, HWND(0), 0, 0) != FALSE {
            println!("{:?}", msg);
        };
    };

    /*    let callback = |mut event: Event| -> Option<Event> {
            if let EventType::KeyPress(Key::CapsLock) = event.event_type {
                println!("Consuming and cancelling CapsLock");
                // None  // CapsLock is now effectively disabled
                Some(event)
            } else { Some(event) }
        };

        rdev::grab(callback).unwrap();

        fn test(event: Event) {
            println!("{:?}", event);
        }

        // This will block.
        if let Err(error) = grab(callback) {
            println!("Error: {:?}", error)
        }*/

    unsafe {
        CoInitializeEx(Some(ptr::null()), COINIT_MULTITHREADED).unwrap();

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
}
