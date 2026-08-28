use std::env;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use evdev::{Device, InputEventKind, Key};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = env::args().collect();
    let mut target_device_path: Option<String> = None;
    let mut es_process_name = "emulationstation".to_string();
    let mut cyberdeck_bin = "cyberdeck-kb".to_string();
    let mut test_mock = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--device" => {
                if i + 1 < args.len() {
                    target_device_path = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--process" => {
                if i + 1 < args.len() {
                    es_process_name = args[i + 1].clone();
                    i += 1;
                }
            }
            "--cyberdeck" => {
                if i + 1 < args.len() {
                    cyberdeck_bin = args[i + 1].clone();
                    i += 1;
                }
            }
            "--test-mock" => {
                test_mock = true;
            }
            "--help" | "-h" => {
                println!("deck-launcher — DarkOS / ArkOS Cyberdeck Mode Switcher");
                println!("Usage: deck-launcher [OPTIONS]");
                println!("  --device <path>      Specify evdev event node (e.g. /dev/input/event4)");
                println!("  --process <name>     Target frontend process name (default: emulationstation)");
                println!("  --cyberdeck <path>   Path to cyberdeck-kb executable (default: cyberdeck-kb)");
                println!("  --test-mock          Run in mock test mode");
                return Ok(());
            }
            _ => {}
        }
        i += 1;
    }

    println!("[deck-launcher] Starting DarkOS Cyberdeck Launcher daemon...");
    println!("[deck-launcher] Target Frontend: {}", es_process_name);

    if test_mock {
        println!("[deck-launcher] Running in TEST MOCK mode.");
        return run_mock_test();
    }

    // Find and open controller
    let mut device = match open_or_find_device(target_device_path.as_deref()) {
        Some(d) => d,
        None => {
            eprintln!("[deck-launcher] Gamepad device not immediately found. Searching...");
            let mut found: Option<Device> = None;
            for _ in 0..10 {
                if let Some(d) = open_or_find_device(target_device_path.as_deref()) {
                    found = Some(d);
                    break;
                }
                thread::sleep(Duration::from_millis(200));
            }
            match found {
                Some(d) => d,
                None => {
                    eprintln!("[deck-launcher] Warning: No gamepad device found at startup.");
                    eprintln!("[deck-launcher] Pass --device /dev/input/eventX if running on RG353M.");
                    return Ok(());
                }
            }
        }
    };

    println!(
        "[deck-launcher] Attached to controller: {}",
        device.name().unwrap_or("Unknown Gamepad")
    );
    println!("[deck-launcher] Active Trigger: Press [F] Button to launch Cyberdeck Mode");

    let running = Arc::new(AtomicBool::new(true));

    while running.load(Ordering::SeqCst) {
        let mut should_reconnect = false;
        match device.fetch_events() {
            Ok(events) => {
                for ev in events {
                    if let InputEventKind::Key(key) = ev.kind() {
                        let is_press = ev.value() == 1;

                        match key {
                            Key::BTN_MODE | Key::KEY_HOMEPAGE | Key::KEY_F => {
                                if is_press {
                                    println!("[deck-launcher] [F] Button pressed! Entering Cyberdeck Mode...");
                                    trigger_cyberdeck_mode(&es_process_name, &cyberdeck_bin);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(25));
            }
            Err(e) => {
                eprintln!("[deck-launcher] Controller event fetch: {}. Retrying in 500ms...", e);
                thread::sleep(Duration::from_millis(500));
                should_reconnect = true;
            }
        }

        if should_reconnect {
            if let Some(d) = open_or_find_device(target_device_path.as_deref()) {
                device = d;
            }
        }
    }

    println!("[deck-launcher] Exiting cleanly.");
    Ok(())
}

fn trigger_cyberdeck_mode(es_proc: &str, cyberdeck_bin: &str) {
    println!("[deck-launcher] Suspending {} (SIGSTOP)...", es_proc);
    let _ = Command::new("killall")
        .arg("-STOP")
        .arg(es_proc)
        .status();

    let _ = Command::new("chvt").arg("1").status();

    println!("[deck-launcher] Launching Cyberdeck Mode ({}) on /dev/tty1...", cyberdeck_bin);

    // Open /dev/tty1 for direct screen and keyboard I/O
    let tty_result = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty1");

    let status = match tty_result {
        Ok(tty_file) => {
            if let (Ok(tty_in), Ok(tty_out)) = (tty_file.try_clone(), tty_file.try_clone()) {
                Command::new(cyberdeck_bin)
                    .env("TERM", "linux")
                    .stdin(Stdio::from(tty_in))
                    .stdout(Stdio::from(tty_out))
                    .stderr(Stdio::from(tty_file))
                    .status()
            } else {
                Command::new(cyberdeck_bin)
                    .env("TERM", "linux")
                    .status()
            }
        }
        Err(e) => {
            eprintln!("[deck-launcher] Could not open /dev/tty1 ({}). Falling back to standard execution.", e);
            Command::new(cyberdeck_bin)
                .env("TERM", "xterm-256color")
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
        }
    };

    match status {
        Ok(s) => println!("[deck-launcher] Cyberdeck session ended with status: {}", s),
        Err(e) => eprintln!("[deck-launcher] Failed to start cyberdeck-kb: {}", e),
    }

    println!("[deck-launcher] Resuming {} (SIGCONT)...", es_proc);
    let _ = Command::new("killall")
        .arg("-CONT")
        .arg(es_proc)
        .status();

    // Debounce to prevent instant re-trigger
    thread::sleep(Duration::from_millis(600));
}

fn open_or_find_device(path: Option<&str>) -> Option<Device> {
    if let Some(p) = path {
        if let Ok(dev) = Device::open(p) {
            return Some(dev);
        }
    }

    let entries = evdev::enumerate();
    for (_, dev) in entries {
        let name = dev.name().unwrap_or("").to_lowercase();
        if name.contains("retrogame")
            || name.contains("joypad")
            || name.contains("gamepad")
            || name.contains("rk_")
            || name.contains("odroid")
        {
            return Some(dev);
        }
    }
    None
}

fn run_mock_test() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("[mock-test] 1. Spawning mock EmulationStation background process...");
    let mut mock_es = Command::new("sh")
        .arg("-c")
        .arg("while true; do sleep 0.1; done")
        .spawn()?;

    let pid = mock_es.id();
    println!("[mock-test] Mock ES started (PID: {}).", pid);

    println!("[mock-test] 2. Testing SIGSTOP signal...");
    let stop_status = Command::new("kill")
        .arg("-STOP")
        .arg(pid.to_string())
        .status()?;
    assert!(stop_status.success(), "Failed to send SIGSTOP");
    println!("[mock-test] Successfully paused mock ES!");

    println!("[mock-test] 3. Testing SIGCONT signal...");
    let cont_status = Command::new("kill")
        .arg("-CONT")
        .arg(pid.to_string())
        .status()?;
    assert!(cont_status.success(), "Failed to send SIGCONT");
    println!("[mock-test] Successfully resumed mock ES!");

    mock_es.kill()?;
    println!("[mock-test] All lifecycle signal checks PASSED successfully.");
    Ok(())
}
