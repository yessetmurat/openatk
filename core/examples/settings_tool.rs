//! Development tool: dump, restore and watch the mouse settings.
//!
//! Usage: `settings_tool dump <file> | restore <file> | watch | set-polling <hz>`

use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;
use std::sync::mpsc;
use std::time::Instant;

use hidapi::HidApi;
use openatk_core::device::{DevicePath, HidLink, Inbox, Input, Mouse, MouseError, Session, Timing};
use openatk_core::protocol::address;
use openatk_core::settings::PollingRate;

const USAGE: &str = "usage: settings_tool dump <file> | restore <file> | watch | set-polling <hz>";
const MAX_LINE_BYTES: usize = 10;
const SHORTCUT_SLOTS: u16 = 6;
const SHORTCUT_STRIDE: u16 = 32;
const BUTTON_SLOTS: u16 = 6;
const BUTTON_LEN: u8 = 4;
const BLOCK_LEN: u8 = 10;
const SHORTCUT_LINE_LEN: u8 = 10;
const STAGE_ADDRESSES: [u16; 4] = [0x0c, 0x14, 0x1c, 0x24];
const STAGE_LEN: u8 = 8;
const LIFT_OFF_LEN: u8 = 2;
const SLEEP_BLOCK_LEN: u8 = 6;
const ANGLE_BLOCK_LEN: u8 = 4;

type Tool = Mouse<HidLink, ()>;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["dump", file] => dump(Path::new(file)),
        ["restore", file] => restore(Path::new(file)),
        ["watch"] => watch(),
        ["set-polling", hz] => set_polling(hz),
        _ => {
            eprintln!("{USAGE}");

            return ExitCode::from(2);
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");

            ExitCode::FAILURE
        }
    }
}

fn open_link() -> Result<(HidLink, Inbox<()>, DevicePath), String> {
    let api = HidApi::new().map_err(|error| error.to_string())?;
    let path = HidLink::find_path(&api).ok_or("no mouse or dongle found")?;
    let (sender, receiver) = mpsc::channel();
    let link = HidLink::open(&api, path, sender).map_err(|error| error.to_string())?;

    Ok((link, Inbox::new(receiver), path))
}

fn connect() -> Result<Tool, String> {
    let (link, inbox, path) = open_link()?;
    let mut session = Session::new(link, inbox, Timing::default());
    let model = Mouse::identify(&mut session, path).map_err(|error| error.to_string())?;

    Ok(Mouse::new(session, model, path))
}

fn dump_ranges() -> Vec<(u16, u8)> {
    let mut ranges = vec![
        (address::GENERAL_BLOCK, BLOCK_LEN),
        (address::LIFT_OFF, LIFT_OFF_LEN),
    ];

    ranges.extend(STAGE_ADDRESSES.iter().map(|&start| (start, STAGE_LEN)));
    ranges.extend(
        (0..BUTTON_SLOTS).map(|slot| (address::BUTTONS + slot * u16::from(BUTTON_LEN), BUTTON_LEN)),
    );
    ranges.extend([
        (address::SENSOR_BLOCK, BLOCK_LEN),
        (address::SLEEP_BLOCK, SLEEP_BLOCK_LEN),
        (address::ANGLE_BLOCK, ANGLE_BLOCK_LEN),
    ]);
    for slot in 0..SHORTCUT_SLOTS {
        let start = address::SHORTCUTS + slot * SHORTCUT_STRIDE;

        ranges.push((start, SHORTCUT_LINE_LEN));
        ranges.push((start + u16::from(SHORTCUT_LINE_LEN), SHORTCUT_LINE_LEN));
    }

    ranges
}

fn dump(file: &Path) -> Result<(), String> {
    let mut mouse = connect()?;
    let mut text = String::new();

    for (start, len) in dump_ranges() {
        let data = mouse
            .read_eeprom(start, len)
            .map_err(|error| failed_at(start, &error))?;

        text.push_str(&format_line(start, &data));
        text.push('\n');
    }

    let mut out = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(file)
        .map_err(|error| format!("{}: {error}", file.display()))?;

    out.write_all(text.as_bytes())
        .map_err(|error| error.to_string())
}

fn restore(file: &Path) -> Result<(), String> {
    let text = fs::read_to_string(file).map_err(|error| error.to_string())?;
    let lines = parse_dump(&text)?;
    let mut mouse = connect()?;

    for (start, data) in lines {
        mouse
            .write_eeprom(start, &data)
            .map_err(|error| failed_at(start, &error))?;
    }

    Ok(())
}

fn watch() -> Result<(), String> {
    let (_link, mut inbox, _path) = open_link()?;
    let started = Instant::now();

    while let Some(input) = inbox.next() {
        match input {
            Input::Report(bytes) => {
                println!("{:>9.3}s {}", started.elapsed().as_secs_f64(), hex(&bytes));
            }
            Input::LinkClosed => break,
            _ => {}
        }
    }

    Ok(())
}

fn set_polling(hz: &str) -> Result<(), String> {
    let hz: u32 = hz.parse().map_err(|_| format!("bad rate: {hz}"))?;
    let rate = PollingRate::from_hz(hz).ok_or_else(|| format!("unsupported rate: {hz} Hz"))?;
    let mut mouse = connect()?;

    mouse.set_polling(rate).map_err(|error| error.to_string())?;
    println!("polling set to {hz} Hz");

    Ok(())
}

fn failed_at(address: u16, error: &MouseError) -> String {
    format!("failed at address {address:#06x}: {error}")
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Formats one dump line: `0x0060 01 01 00 53`.
fn format_line(address: u16, data: &[u8]) -> String {
    format!("{address:#06x} {}", hex(data))
}

/// Parses a whole dump. Every line must be well formed and lie inside a writable range,
/// so that `restore` never starts a write it cannot finish.
fn parse_dump(text: &str) -> Result<Vec<(u16, Vec<u8>)>, String> {
    text.lines()
        .enumerate()
        .map(|(index, line)| {
            let (start, data) =
                parse_line(line).map_err(|error| format!("line {}: {error}", index + 1))?;

            if !address::is_writable(start, data.len()) {
                return Err(format!(
                    "line {}: {start:#06x} with {} bytes is not writable",
                    index + 1,
                    data.len()
                ));
            }

            Ok((start, data))
        })
        .collect()
}

/// Parses one dump line into an address and 1 to 10 data bytes.
fn parse_line(line: &str) -> Result<(u16, Vec<u8>), String> {
    let mut words = line.split_whitespace();
    let address = words
        .next()
        .and_then(|word| word.strip_prefix("0x"))
        .and_then(|digits| u16::from_str_radix(digits, 16).ok())
        .ok_or("bad address")?;
    let data = words
        .map(|word| {
            if word.len() == 2 {
                u8::from_str_radix(word, 16).map_err(|_| format!("bad byte: {word}"))
            } else {
                Err(format!("bad byte: {word}"))
            }
        })
        .collect::<Result<Vec<u8>, String>>()?;

    if data.is_empty() || data.len() > MAX_LINE_BYTES {
        return Err(format!(
            "expected 1 to {MAX_LINE_BYTES} bytes, got {}",
            data.len()
        ));
    }

    Ok((address, data))
}

#[cfg(test)]
#[path = "settings_tool/tests.rs"]
mod tests;
