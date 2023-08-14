use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::read_to_string, path::Path, time::Duration};

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

pub fn is_on_ac() -> bool {
    let power_supply_dir = Path::new("/sys/class/power_supply");
    let power_supplies = power_supply_dir.read_dir().unwrap();

    for entry in power_supplies {
        let entry = entry.unwrap().path();

        // Skip batteries
        if read_to_string(entry.join("type"))
            .map(|t| t != "Mains\n")
            .unwrap_or(true)
        {
            continue;
        }

        if read_to_string(entry.join("online"))
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
    let power_supplies = power_supply_dir.read_dir().unwrap();

    let mut total_capacity = 0;
    let mut full_total = 0.0;
    let mut now_total = 0.0;
    let mut current_total = 0.0;
    for entry in power_supplies {
        let entry = entry.unwrap().path();

        // Skip non-batteries
        if read_to_string(entry.join("type"))
            .map(|t| t != "Battery\n")
            .unwrap_or(true)
        {
            continue;
        }

        let capacity = read_to_string(entry.join("capacity"))
            .unwrap()
            .trim_end_matches('\n')
            .parse::<i32>()
            .unwrap();

        let status = read_to_string(entry.join("status"))
            .unwrap()
            .trim_end_matches('\n')
            .to_owned();

        let full = read_to_string(entry.join("energy_full"))
            .or_else(|_| read_to_string(entry.join("charge_full")))
            .unwrap()
            .trim_end_matches('\n')
            .parse::<f64>()
            .unwrap();

        let now = read_to_string(entry.join("energy_now"))
            .or_else(|_| read_to_string(entry.join("charge_now")))
            .unwrap()
            .trim_end_matches('\n')
            .parse::<f64>()
            .unwrap();

        let current = read_to_string(entry.join("power_now"))
            .or_else(|_| read_to_string(entry.join("current_now")))
            .unwrap()
            .trim_end_matches('\n')
            .parse::<f64>()
            .unwrap();

        total_capacity += capacity;
        full_total += full;
        now_total += now;
        current_total += current;

        let remaining_time = battery_time(on_ac, full, now, current);
        batteries.insert(
            entry.file_name().unwrap().to_string_lossy().to_string(),
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
