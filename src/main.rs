use anyhow::{anyhow, Context, Result};
use clap::Parser;
// Use text-specific clipboard functions
use clipboard_win::{get_clipboard_string, set_clipboard_string};
use rdev::{listen, simulate, Event, EventType, Key};
use std::thread;
use std::time::Duration;

// --- Import the key enum module ---
mod easy_rdev_key;
use easy_rdev_key::PTTKey;

// --- CLI Arguments ---
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "Listens for a hotkey, copies selected text, removes newlines, and pastes the result."
)]
struct Args {
    #[arg(
        short,
        long,
        value_enum,
        help = "Key to trigger the remove-newlines-and-paste action."
    )]
    trigger_key: PTTKey,
}

// --- Core Logic ---
fn remove_newlines_and_paste() -> Result<()> {
    println!("Trigger key pressed. Simulating Copy (Ctrl+C)...");

    // 1. Simulate Ctrl+C
    send_ctrl_c().context("Failed to simulate Ctrl+C")?;

    // 2. Wait for clipboard to update
    //    This delay is crucial! The OS needs time to process the copy command.
    thread::sleep(Duration::from_millis(150)); // Adjust if needed

    println!("Getting text from clipboard...");

    // 3. Get text from clipboard
    let original_text = get_clipboard_string()
        .map_err(|e| anyhow!("Clipboard error getting string: {}", e)) // Map clipboard-win error
        .context("Failed to get text from clipboard. Was text copied?")?;

    if original_text.is_empty() {
        println!("Clipboard text is empty. Skipping.");
        return Ok(());
    }

    // 4. Remove newlines
    //    Replace both Windows (\r\n) and Unix (\n) newlines.
    //    Replacing \r and \n individually covers both cases.
    let modified_text = original_text.replace('\r', "").replace('\n', " "); // Replace newline with a space
    println!(
        "Removed newlines. Result (first 100): {:.100}...",
        modified_text
    );

    // 5. Set modified text to clipboard
    set_clipboard_string(&modified_text)
        .map_err(|e| anyhow!("Clipboard error setting string: {}", e)) // Map clipboard-win error
        .context("Failed to set modified text to clipboard")?;

    // 6. Wait for clipboard to update again
    thread::sleep(Duration::from_millis(150)); // Delay before pasting

    println!("Pasting modified text (Ctrl+V)...");

    // 7. Simulate Ctrl+V
    send_ctrl_v().context("Failed to simulate Ctrl+V")?;

    println!("Paste simulated.");
    Ok(())
}

// --- Simulation Helpers ---

// Helper function to simulate Ctrl+C
fn send_ctrl_c() -> Result<(), rdev::SimulateError> {
    let delay = Duration::from_millis(30);
    simulate(&EventType::KeyPress(Key::ControlLeft))?;
    thread::sleep(delay);
    simulate(&EventType::KeyPress(Key::KeyC))?;
    thread::sleep(delay);
    simulate(&EventType::KeyRelease(Key::KeyC))?;
    thread::sleep(delay);
    simulate(&EventType::KeyRelease(Key::ControlLeft))?;
    Ok(())
}

// Helper function to simulate Ctrl+V (same as before)
fn send_ctrl_v() -> Result<(), rdev::SimulateError> {
    let delay = Duration::from_millis(30);
    simulate(&EventType::KeyPress(Key::ControlLeft))?;
    thread::sleep(delay);
    simulate(&EventType::KeyPress(Key::KeyV))?;
    thread::sleep(delay);
    simulate(&EventType::KeyRelease(Key::KeyV))?;
    thread::sleep(delay);
    simulate(&EventType::KeyRelease(Key::ControlLeft))?;
    Ok(())
}

// --- Main Function ---
fn main() -> Result<()> {
    let args = Args::parse();

    let target_key: rdev::Key = args.trigger_key.into();

    println!("Remove Newlines & Paste Listener Started.");
    println!("Trigger Key: {:?}", args.trigger_key);
    println!("---");
    println!("Select text and press '{:?}' to copy it, remove newlines (replacing with spaces), and paste it back.", args.trigger_key);
    println!("NOTE: This program likely requires administrator privileges to capture global key presses and simulate input.");
    println!("Ctrl+C in this window to exit.");
    println!("---");

    let callback = move |event: Event| {
        match event.event_type {
            EventType::KeyPress(key) if key == target_key => {
                // Call the core logic
                if let Err(e) = remove_newlines_and_paste() {
                    eprintln!("ERROR: {:?}", e);
                    // Maybe add a small visual/audio cue for error? (Optional)
                }
            }
            _ => (), // Ignore other events
        }
    };

    // Blocks the thread until an error occurs
    if let Err(error) = listen(callback) {
        eprintln!(
            "FATAL ERROR setting up global keyboard listener: {:?}",
            error
        );
        eprintln!("This might be a permissions issue. Try running the program as administrator.");
        return Err(anyhow!("Keyboard listener error: {:?}", error));
    }

    Ok(())
}
