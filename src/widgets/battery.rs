use gtk::{
    glib::{self, clone, MainContext},
    traits::{BoxExt, WidgetExt},
};
use std::{
    cell::RefCell, collections::HashMap, fs::read_to_string, path::Path, rc::Rc, time::Duration,
};
use tokio::time::sleep;

#[derive(Debug, Clone)]
struct BatteryInfo {
    capacity: i32,
    remaining_time: Duration,
    status: String,
}

#[derive(Debug, Clone)]
struct Batteries {
    info: HashMap<String, BatteryInfo>,
    total_capacity: i32,
    remaining_time: Duration,
}

fn is_on_ac() -> bool {
    let power_supply_dir = Path::new("/sys/class/power_supply");
    let power_supplies = power_supply_dir.read_dir().unwrap();

    for entry in power_supplies {
        let entry = entry.unwrap().path();

        // Skip batteries
        if read_to_string(entry.join("type"))
            .map(|t| t != "Mains")
            .unwrap_or(true)
        {
            continue;
        }

        if read_to_string(entry.join("online"))
            .map(|value| value == "1")
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
fn get_batteries() -> Batteries {
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

        let status = read_to_string(entry.join("status")).unwrap();

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

pub fn build_battery() -> gtk::Box {
    let label = gtk::Label::new(None);

    let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    container.set_css_classes(&["widget", "battery"]);
    container.append(&label);

    let main_context = MainContext::default();
    main_context.spawn_local(async move {
        loop {
            let batteries = get_batteries();
            label.set_text(&format!("{}%", batteries.total_capacity));

            sleep(Duration::from_secs(2)).await;
        }
    });

    container
}
