use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{read_to_string, File},
    io::{self, Read},
    path::Path,
    time::Duration,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BatteryInfo {
    pub capacity: i32,
    pub remaining_time: Duration,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Batteries {
    pub info: HashMap<String, BatteryInfo>,
    pub total_capacity: i32,
    pub remaining_time: Duration,
}

fn read_to_string_buf<P: AsRef<Path>>(path: P, buffer: &mut String) -> io::Result<&mut String> {
    let mut file = File::open(path)?;
    buffer.clear();
    file.read_to_string(buffer)?;
    Ok(buffer)
}

pub fn is_on_ac() -> bool {
    let power_supply_dir = Path::new("/sys/class/power_supply");
    let power_supplies = power_supply_dir
        .read_dir()
        .expect("Failed to read /sys/class/power_supply/");

    let mut buffer = String::with_capacity(16);
    for entry in power_supplies {
        let entry = entry.expect("Failed to get power supply entry").path();

        // Skip batteries
        if read_to_string_buf(entry.join("type"), &mut buffer)
            .map(|t| t != "Mains\n")
            .unwrap_or(true)
        {
            continue;
        }

        if read_to_string_buf(entry.join("online"), &mut buffer)
            .map(|value| value == "1\n")
            .unwrap_or(false)
        {
            return true;
        }
    }

    false
}

fn battery_time(on_ac: bool, full: f64, now: f64, current: f64) -> Duration {
    if current != 0.0 {
        if on_ac {
            // Charge time
            Duration::from_secs_f64((full - now).abs() * 3600.0 / current)
        } else {
            // Discharge time
            Duration::from_secs_f64(now * 3600.0 / current)
        }
    } else {
        Duration::new(0, 0)
    }
}

// https://github.com/elkowar/eww/blob/dc3129aee2806823bdad87785f7ef80651d5245c/crates/eww/src/config/system_stats.rs#L118
// https://github.com/valpackett/systemstat/blob/cbd9c1638b792d1819479f0c2baa5840f65af727/src/platform/linux.rs#L584
pub fn get_batteries() -> Batteries {
    let mut batteries = HashMap::new();

    let on_ac = is_on_ac();

    let power_supply_dir = Path::new("/sys/class/power_supply");
    let power_supplies = power_supply_dir
        .read_dir()
        .expect("Failed to read /sys/class/power_supply/");

    let mut total_capacity = 0;
    let mut full_total = 0.0;
    let mut now_total = 0.0;
    let mut current_total = 0.0;
    let mut buffer = String::with_capacity(16);
    for entry in power_supplies {
        let entry = entry.expect("Failed to get power supply entry").path();

        // Skip non-batteries
        if read_to_string_buf(entry.join("type"), &mut buffer)
            .map(|t| t != "Battery\n")
            .unwrap_or(true)
        {
            continue;
        }

        let capacity = read_to_string_buf(entry.join("capacity"), &mut buffer)
            .expect("Failed to read battery capacity")
            .trim_end_matches('\n')
            .parse::<i32>()
            .expect("Failed to parse battery capacity");

        let status = {
            let mut str = read_to_string_buf(entry.join("status"), &mut buffer)
                .expect("Failed to read battery status")
                .to_owned();
            str.truncate(str.trim_end_matches('\n').len());
            str
        };

        let full = read_to_string(entry.join("energy_full"))
            .or_else(|_| read_to_string(entry.join("charge_full")))
            .expect("Failed to read battery full charge")
            .trim_end_matches('\n')
            .parse::<f64>()
            .expect("Failed to parse battery full charge");

        let now = read_to_string(entry.join("energy_now"))
            .or_else(|_| read_to_string(entry.join("charge_now")))
            .expect("Failed to read battery current charge")
            .trim_end_matches('\n')
            .parse::<f64>()
            .expect("Failed to parse battery current charge");

        let current = read_to_string(entry.join("power_now"))
            .or_else(|_| read_to_string(entry.join("current_now")))
            .expect("Failed to read battery current")
            .trim_end_matches('\n')
            .parse::<f64>()
            .expect("Failed to parse battery current");

        total_capacity += capacity;
        full_total += full;
        now_total += now;
        current_total += current;

        let remaining_time = battery_time(on_ac, full, now, current);
        batteries.insert(
            entry
                .file_name()
                .expect("Failed to get battery name")
                .to_string_lossy()
                .to_string(),
            BatteryInfo {
                capacity,
                remaining_time,
                status,
            },
        );
    }

    Batteries {
        info: batteries,
        total_capacity,
        remaining_time: battery_time(on_ac, full_total, now_total, current_total),
    }
}
