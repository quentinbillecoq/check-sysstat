use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::fs::OpenOptions;
use std::io::Write;
use std::time::SystemTime;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use clap::{Command, Arg, ArgAction};
use std::process::exit;
use num_cpus;

#[derive(Serialize, Deserialize, Debug)]
pub struct CpuSoftIrqs {
    pub hi: i64,
    pub timer: i64,
    pub net_tx: i64,
    pub net_rx: i64,
    pub block: i64,
    pub irq_poll: i64,
    pub tasklet: i64,
    pub sched: i64,
    pub hrtimer: i64,
    pub rcu: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CpuTime {
    pub user: i64,
    pub nice: i64,
    pub system: i64,
    pub idle: i64,
    pub iowait: Option<i64>,
    pub irq: Option<i64>,
    pub softirq: Option<i64>,
    pub steal: Option<i64>,
    pub guest: Option<i64>,
    pub guest_nice: Option<i64>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CpuStat {
    pub name: String,
    pub stat: CpuTime,
    pub softirqs: CpuSoftIrqs,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CpuInfo {
    pub id: i64,
    //pub physical_package_id: i64,
    pub online: bool,
    //pub hotplug_state: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CpuGlobalInfo {
    //pub socket_nbr_detected: i64,
    pub cpu_nbr_detected: usize,
    pub cpu_nbr_online: usize,
    pub cpu_nbr_offline: usize,
    //pub cpu: Vec<CpuInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemInfo {
    pub cpu: CpuGlobalInfo,
}

fn parse_range_string(range_string: &str) -> Vec<usize> {
    let mut result = Vec::new();
    for range in range_string.split(',') {
        if let [start, end] = range.split('-').map(|s| s.parse::<usize>().unwrap()).collect::<Vec<usize>>()[..] {
            for i in start..=end {
                result.push(i);
            }
        } else {
            result.push(range.parse::<usize>().unwrap());
        }
    }
    result
}

fn get_cpu_status() -> HashMap<usize, String> {
    let possible_file = File::open("/sys/devices/system/cpu/possible")
        .expect("Failed to open file /sys/devices/system/cpu/possible");
    let possible_reader = BufReader::new(possible_file);
    let possible_contents = possible_reader.lines().next().unwrap().unwrap();
    let cpus = possible_contents.trim().split("-").collect::<Vec<&str>>();
    let start = cpus[0].parse::<usize>().unwrap();
    let end = cpus[1].parse::<usize>().unwrap();
    let online_file = File::open("/sys/devices/system/cpu/online")
        .expect("Failed to open file /sys/devices/system/cpu/online");
    let online_reader = BufReader::new(online_file);
    let online_contents = online_reader.lines().next().unwrap().unwrap();
    let online_cpus: Vec<usize> = parse_range_string(&online_contents);

    let mut cpu_status = HashMap::new();
    for i in start..=end {
        if online_cpus.contains(&i) {
            cpu_status.insert(i, String::from("online"));
        } else {
            cpu_status.insert(i, String::from("offline"));
        }
    }
    cpu_status
}

fn get_cpu_global_infos() -> CpuGlobalInfo {
    let cpu_status = get_cpu_status();

    CpuGlobalInfo{
        cpu_nbr_detected: cpu_status.keys().len() as usize,
        cpu_nbr_online: cpu_status.values().filter(|&x| x == "online").count() as usize,
        cpu_nbr_offline: cpu_status.values().filter(|&x| x == "offline").count() as usize,
    }
}

fn get_system_infos() -> SystemInfo{
    let cpu: CpuGlobalInfo = get_cpu_global_infos();
    SystemInfo{
        cpu,
    }
}





fn round(x: f64, decimals: u32) -> f64 {
    let y = 10i64.pow(decimals) as f64;
    (x * y).round() / y
}

fn calculate_percentages(cpu_time: &CpuTime) -> HashMap<&str, f64> {
    let total_time = cpu_time.user + cpu_time.nice + cpu_time.system + cpu_time.idle +
        cpu_time.iowait.unwrap_or(0) + cpu_time.irq.unwrap_or(0) +
        cpu_time.softirq.unwrap_or(0) + cpu_time.steal.unwrap_or(0) +
        cpu_time.guest.unwrap_or(0) + cpu_time.guest_nice.unwrap_or(0);

    let mut percentages = HashMap::new();
    percentages.insert("user", (cpu_time.user as f64 / total_time as f64) * 100.0);
    percentages.insert("nice", (cpu_time.nice as f64 / total_time as f64) * 100.0);
    percentages.insert("system", (cpu_time.system as f64 / total_time as f64) * 100.0);
    percentages.insert("idle", (cpu_time.idle as f64 / total_time as f64) * 100.0);

    if let Some(iowait) = cpu_time.iowait {
        percentages.insert("iowait", (iowait as f64 / total_time as f64) * 100.0);
    }

    if let Some(irq) = cpu_time.irq {
        percentages.insert("irq", (irq as f64 / total_time as f64) * 100.0);
    }

    if let Some(softirq) = cpu_time.softirq {
        percentages.insert("softirq", (softirq as f64 / total_time as f64) * 100.0);
    }

    if let Some(steal) = cpu_time.steal {
        percentages.insert("steal", (steal as f64 / total_time as f64) * 100.0);
    }

    if let Some(guest) = cpu_time.guest {
        percentages.insert("guest", (guest as f64 / total_time as f64) * 100.0);
    }

    if let Some(guest_nice) = cpu_time.guest_nice {
        percentages.insert("guest_nice", (guest_nice as f64 / total_time as f64) * 100.0);
    }

    percentages
}

pub fn compare_cpu_infos(v1: Vec<CpuStat>, v2: Vec<CpuStat>) -> (Vec<CpuStat>, HashMap<std::string::String, &'static str>) {
    let mut diff_vec = Vec::new();
    let mut added_removed = HashMap::new();

    let v1_map: HashMap<&str, &CpuStat> = v1.iter().map(|x| (x.name.as_str(), x)).collect();
    let v2_map: HashMap<&str, &CpuStat> = v2.iter().map(|x| (x.name.as_str(), x)).collect();

    for (name, v2_cpu) in v2_map.iter() {
        if let Some(v1_cpu) = v1_map.get(name) {

            //diff cpu time
            let diff_user = v1_cpu.stat.user - v2_cpu.stat.user;
            let diff_nice = v1_cpu.stat.nice - v2_cpu.stat.nice;
            let diff_system = v1_cpu.stat.system - v2_cpu.stat.system;
            let diff_idle = v1_cpu.stat.idle - v2_cpu.stat.idle;
            let diff_iowait = match (v1_cpu.stat.iowait, v2_cpu.stat.iowait) {
                (Some(i1), Some(i2)) => Some(i1 - i2),
                _ => None,
            };
            let diff_irq = match (v1_cpu.stat.irq, v2_cpu.stat.irq) {
                (Some(i1), Some(i2)) => Some(i1 - i2),
                _ => None,
            };
            let diff_softirq = match (v1_cpu.stat.softirq, v2_cpu.stat.softirq) {
                (Some(i1), Some(i2)) => Some(i1 - i2),
                _ => None,
            };
            let diff_steal = match (v1_cpu.stat.steal, v2_cpu.stat.steal) {
                (Some(i1), Some(i2)) => Some(i1 - i2),
                _ => None,
            };
            let diff_guest = match (v1_cpu.stat.guest, v2_cpu.stat.guest) {
                (Some(i1), Some(i2)) => Some(i1 - i2),
                _ => None,
            };
            let diff_guest_nice = match (v1_cpu.stat.guest_nice, v2_cpu.stat.guest_nice) {
                (Some(i1), Some(i2)) => Some(i1 - i2),
                _ => None,
            };

            //diff cpu softirq
            //let diff_hi = v1_cpu[v1_cpu.name.clone()].hi - v2_cpu[v1_cpu.name.clone()].hi;


            let diff_softirqs: CpuSoftIrqs = CpuSoftIrqs{
                hi: v1_cpu.softirqs.hi-v2_cpu.softirqs.hi,
                timer: v1_cpu.softirqs.timer-v2_cpu.softirqs.timer,
                net_tx: v1_cpu.softirqs.net_tx-v2_cpu.softirqs.net_tx,
                net_rx: v1_cpu.softirqs.net_rx-v2_cpu.softirqs.net_rx,
                block: v1_cpu.softirqs.block-v2_cpu.softirqs.block,
                irq_poll: v1_cpu.softirqs.irq_poll-v2_cpu.softirqs.irq_poll,
                tasklet: v1_cpu.softirqs.tasklet-v2_cpu.softirqs.tasklet,
                sched: v1_cpu.softirqs.sched-v2_cpu.softirqs.sched,
                hrtimer: v1_cpu.softirqs.hrtimer-v2_cpu.softirqs.hrtimer,
                rcu: v1_cpu.softirqs.rcu-v2_cpu.softirqs.rcu,
            };

            let diff_cputime: CpuTime = CpuTime{
                user: diff_user,
                nice: diff_nice,
                system: diff_system,
                idle: diff_idle,
                iowait: diff_iowait,
                irq: diff_irq,
                softirq: diff_softirq,
                steal: diff_steal,
                guest: diff_guest,
                guest_nice: diff_guest_nice,
            };

            diff_vec.push(CpuStat {
                name: v1_cpu.name.clone(),
                stat: diff_cputime,
                softirqs: diff_softirqs,
            });
        } else {
            added_removed.insert(format!("{}", name), "removed");

        }
    }

    for (name, _v1_cpu) in v1_map.iter() {
        if !v2_map.contains_key(name) {
            added_removed.insert(format!("{}", name), "added");
        }
    }

    (diff_vec, added_removed)
}

pub fn save_stats(file_stats: &str, timestamp: u64, getcpunow: &Vec<CpuStat>) {
    let mut file = File::create(file_stats).unwrap();
    writeln!(file, "cputime {}", timestamp);
    writeln!(file, "cpujson {}", serde_json::to_string(&getcpunow).unwrap()).unwrap();
}

fn get_cpu_stats() -> Vec<CpuStat>{
    let contents_cpu_stats = fs::read_to_string("/proc/stat");
    let binding = contents_cpu_stats.expect("Failed to open /proc/stat");
    let lines_cpu_stats = binding.lines();

    let mut cpu_infos: Vec<CpuStat> = Vec::new();

    let all_softirqs = get_softirqs();

    for line in lines_cpu_stats {
        if line.starts_with("cpu") {
            let data: Vec<&str> = line.split_whitespace().collect();
            let data_value: &[&str] = &data[1..];
            let data_i64: Vec<i64> = data_value.iter().filter_map(|&s| s.parse::<i64>().ok()).collect();

            let name = data[0].to_string();
            let user: i64 = data_i64[0];
            let nice: i64 = data_i64[1];
            let system: i64 = data_i64[2];
            let idle: i64 = data_i64[3];
            let iowait: Option<i64> = Some(data_i64[4]);
            let irq: Option<i64> = Some(data_i64[5]);
            let softirq: Option<i64> = Some(data_i64[6]);
            let steal: Option<i64> = Some(data_i64[7]);
            let guest: Option<i64> = Some(data_i64[8]);
            let guest_nice: Option<i64> = Some(data_i64[9]);

            let softirqs: CpuSoftIrqs = CpuSoftIrqs{
                hi: all_softirqs[&name]["HI:"].into(),
                timer: all_softirqs[&name]["TIMER:"].into(),
                net_tx: all_softirqs[&name]["NET_TX:"].into(),
                net_rx: all_softirqs[&name]["NET_RX:"].into(),
                block: all_softirqs[&name]["BLOCK:"].into(),
                irq_poll: all_softirqs[&name]["IRQ_POLL:"].into(),
                tasklet: all_softirqs[&name]["TASKLET:"].into(),
                sched: all_softirqs[&name]["SCHED:"].into(),
                hrtimer: all_softirqs[&name]["HRTIMER:"].into(),
                rcu: all_softirqs[&name]["RCU:"].into(),
            };

            let stat: CpuTime  = CpuTime{
                user,
                nice,
                system,
                idle,
                iowait,
                irq,
                softirq,
                steal,
                guest,
                guest_nice
            };

            cpu_infos.push(CpuStat{
                name,
                stat,
                softirqs
            })
        }
    }
    cpu_infos
}

fn get_softirqs() -> HashMap<String, HashMap<String, u32>> {
    let file = File::open("/proc/softirqs").unwrap();
    let reader = BufReader::new(file);

    let mut softirqs: HashMap<String, HashMap<String, u32>> = HashMap::new();

    let mut lines = reader.lines();
    let binding = lines.next().unwrap().unwrap();
    let cpu_names: Vec<&str> = binding.split_whitespace().collect();
    for cpu_name in &cpu_names {
        softirqs.insert(cpu_name.to_lowercase(), HashMap::new());
    }
    for line in lines {
        let line = line.unwrap();
        let fields: Vec<&str> = line.split_whitespace().collect();
        let softirq_name = fields[0];
        for (index, value) in fields[1..].iter().enumerate() {
            let cpu_name = cpu_names[index].to_lowercase();
            let value: u32 = value.parse().unwrap();
            let entry = softirqs.get_mut(&cpu_name).unwrap();
            entry.insert(softirq_name.to_string(), value);
        }
    }

    let mut softirqs_total: HashMap<String, u32> = HashMap::new();
    for (_index, value) in &mut softirqs {
        for (index2, value2) in value {
            if softirqs_total.contains_key(index2) {
                *softirqs_total.get_mut(index2).unwrap() += *value2;
            } else {
                softirqs_total.insert(index2.to_string(), *value2);
            }
        }
    }

    softirqs.insert("cpu".to_string(), softirqs_total);

    softirqs
}


fn main() {
    println!("{}", serde_json::to_string_pretty(&get_system_infos()).unwrap())
}

// fn main() {

//     // 
//     // -- RETURN CODE --
//     // Example : exit(RC_UNKNOWN);
//     let RC_OK = 0;
//     let RC_WARNING = 1;
//     let RC_CRITICAL = 2;
//     let RC_UNKNOWN = 3;

//     // 
//     // -- ARGS --
//     //
//     let args = Command::new("check-sysstat-cpu")
//                 .author("Quentin BILLECOQ, quentin@billecoq.fr")
//                 .version("1.0.O")
//                 .about("Get CPU Stats")
//                 .arg(Arg::new("all")
//                     .short('A')
//                     .long("all")
//                     .action(ArgAction::SetTrue)
//                     .help("Get all CPU infos and stats")
//                     .required(false)
//                 )
//                 .arg(Arg::new("infos")
//                     .short('i')
//                     .long("infos")
//                     .action(ArgAction::SetTrue)
//                     .help("Get CPU infos")
//                     .required(false)
//                 )
//                 .arg( Arg::new("times")
//                     .short('T')
//                     .long("times")
//                     .action(ArgAction::SetTrue)
//                     .help("Get CPU times")
//                     .required(false)
//                 )
//                 .arg(Arg::new("interrupts")
//                     .short('I')
//                     .long("interrupts")
//                     .action(ArgAction::SetTrue)
//                     .help("Get CPU interrupts")
//                     .required(false)
//                 )
//                 .arg(Arg::new("softirqs")
//                     .short('S')
//                     .long("softirqs")
//                     .action(ArgAction::SetTrue)
//                     .help("Get CPU software interrupts")
//                     .required(false)
//                 )
//                 .arg(Arg::new("statsfile")
//                     .short('s')
//                     .long("statsfile")
//                     .value_name("PATH")
//                     .default_value("/tmp/check-sysstat-cpu.stats")
//                     .action(ArgAction::Set)
//                     .help("File where is stocked last stats for next the run")
//                     .required(false)
//                 )
//                 .get_matches();
    
//     // 
//     // -- Default value --
//     //
//     let args_cpu_all: bool = *args.get_one::<bool>("all").unwrap_or(&false);
//     let args_cpu_infos: bool = *args.get_one::<bool>("infos").unwrap_or(&false);
//     let args_cpu_times: bool = *args.get_one::<bool>("times").unwrap_or(&false);
//     let args_cpu_interrupts: bool = *args.get_one::<bool>("interrupts").unwrap_or(&false);
//     let args_cpu_softirqs: bool = *args.get_one::<bool>("softirqs").unwrap_or(&false);
//     let file_stats: &str = args.get_one::<String>("statsfile").unwrap();

//     let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();


//     // 
//     // -- Main --
//     //

//     let mut first_start = true;
//     let getcpunow = get_cpu_stats();
//     let mut cpulastdata: Vec<CpuStat> = Vec::new();
//     let mut lastchecktime: u64 = 0;

//     if !fs::metadata(file_stats).is_ok() {
//         let file = OpenOptions::new()
//             .write(true)
//             .create_new(true)
//             .open(file_stats)
//             ;
//         file.expect("REASON").write_all("".as_bytes());
//     }else {
//         first_start = false;
//         let contents_stats = fs::read_to_string(file_stats);
//         let binding_stats = contents_stats.expect("REASON");
//         let lines_stats = binding_stats.lines();
//         for line in lines_stats {
//             if line.starts_with("cputime") {
//                 let data: Vec<&str> = line.split_whitespace().collect();
//                 let date: u64 = match data[1].to_string().parse() {
//                     Ok(n) => n,
//                     Err(_) => panic!("Error parsing date string to u64"),
//                 };
//                 lastchecktime = timestamp-date;
//                 println!("Temps depuis dernier check : {}s", lastchecktime)
//             }
//             if line.starts_with("cpujson") {
//                 let data: Vec<&str> = line.split_whitespace().collect();
//                 cpulastdata = serde_json::from_str(data[1]).unwrap();
//             }
//         }
//     }


//     save_stats(file_stats, timestamp, &getcpunow);

//     let (mut diff_vec, added_removed) = compare_cpu_infos(getcpunow, cpulastdata);
//     diff_vec.sort_by(|a, b| a.name.cmp(&b.name));

//     // ALERTS 
//     if lastchecktime < 2 {
//         println!("Interval between two executions is too short, the information may be erroneous. It is advisable to wait at least 2 seconds before a second execution. Last interval ({}s)", lastchecktime);
//     }
//     for (cpu, act) in added_removed {
//         println!("{} : {}", cpu.to_string(), act.to_string());
//     }
    
//     // CPU INFOS
//     if args_cpu_all | args_cpu_infos {
//         println!();
//         println!("{0: <10} | {1: <10}",
//             "CPU", num_cpus::get(),
//         );
//         println!("{0: <10} | {1: <10}",
//             "Core", num_cpus::get()*num_cpus::get_physical(),
//         );
//     }

//     // CPU TIMES
//     if args_cpu_all | args_cpu_times {
//         println!();
//         println!(
//             "{0: <7} | {1: <10} | {2: <10} | {3: <10} | {4: <10} | {5: <10} | {6: <10} | {7: <10} | {8: <10} | {9: <10} | {10: <10}",
//             "CPU", "%usr", "%nice", "%sys", "%iowait", "%irq", "%soft", "%steal", "%guest", "%gnice", "%idle"
//         );
//         println!("-------------------------------------------------------------------------------------------------------------------------------------");
//         if !first_start {
//             for stats in &diff_vec {
//                     let name = if stats.name == "cpu" { "all" } else { &stats.name };

//                     let percentages = calculate_percentages(&stats.stat);
//                     //println!("{:?}", percentages);

//                     println!("{0: <7} | {1: <10} | {2: <10} | {3: <10} | {4: <10} | {5: <10?} | {6: <10?} | {7: <10?} | {8: <10?} | {9: <10?} | {10: <10?}", 
//                         name,
//                         round(percentages["user"], 2),
//                         round(percentages["nice"], 2),
//                         round(percentages["system"], 2),
//                         round(percentages["iowait"], 2),
//                         round(percentages["irq"], 2),
//                         round(percentages["softirq"], 2),
//                         round(percentages["steal"], 2),
//                         round(percentages["guest"], 2),
//                         round(percentages["guest_nice"], 2),
//                         round(percentages["idle"], 2),
//                 );
//             }
//         }else {
//             // If first run, stats initializing
//             println!("First run : initializing...");
//         }
//     }
    
//     // CPU SOFTIRQS
//     if args_cpu_all | args_cpu_softirqs {
//         println!();
//         println!(
//             "{0: <7} | {1: <10} | {2: <10}",
//             "CPU", "HI/s", "TIMER/s",
//         );
//         println!("------------------------------------------------------");
//         if !first_start {
//             for stats in &diff_vec {
//                 let name = if stats.name == "cpu" { "all" } else { &stats.name };
//                 println!("{0: <7} | {1: <10} | {2: <10}", 
//                     name,
//                     stats.softirqs.hi,
//                     stats.softirqs.timer,
//                 );
//             }
//         }else {
//             // If first run, stats initializing
//             println!("First run : initializing...");
//         }
//     }

//     println!("{:?}", get_cpu_status())

// }
