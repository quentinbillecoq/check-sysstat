use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::fs::OpenOptions;
use std::io::Write;
use std::io::Read;
use std::time::SystemTime;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use clap::{Command, Arg, ArgAction};
use std::process::exit;
use linked_hash_map::LinkedHashMap;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CPUInfoFile {
    processor: usize,
    vendor_id: String,
    cpu_family: usize,
    model: usize,
    model_name: String,
    stepping: usize,
    microcode: Option<String>, // 2.6.22 and newer
    cpu_mhz: Option<f32>, // 2.2 and newer
    cache_size: Option<String>, // 2.2 and newer
    physical_id: Option<usize>, // 2.6.0 and newer
    siblings: Option<usize>, // 2.6.0 and newer
    core_id: Option<usize>, // 2.6.0 and newer
    cpu_cores: Option<usize>, // 2.6.0 and newer
    apicid: Option<usize>, // 2.6.0 and newer
    initial_apicid: Option<usize>, // 2.6.0 and newer
    fpu: bool,
    fpu_exception: bool,
    cpuid_level: usize,
    wp: bool,
    flags: String,
    bugs: Option<Vec<String>>,
    bogomips: f32,
    clflush_size: Option<usize>, // 2.6.0 and newer
    cache_alignment: Option<usize>, // 2.6.0 and newer
    address_sizes: Option<String>, // 2.6.0 and newer
}
impl Default for CPUInfoFile {
    fn default() -> Self {
        CPUInfoFile {
            processor: 0,
            vendor_id: String::new(),
            cpu_family: 0,
            model: 0,
            model_name: String::new(),
            stepping: 0,
            microcode: None,
            cpu_mhz: None,
            cache_size: None,
            physical_id: None,
            siblings: None,
            core_id: None,
            cpu_cores: None,
            apicid: None,
            initial_apicid: None,
            fpu: false,
            fpu_exception: false,
            cpuid_level: 0,
            wp: false,
            flags: String::new(),
            bugs: None,
            bogomips: 0.0,
            clflush_size: None,
            cache_alignment: None,
            address_sizes: None,
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct CpuStat {
    pub name: String,
    pub stat: CpuTime,
    pub stat_prct: HashMap<String, f64>,
    pub softirqs: CpuSoftIrqs,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SocketInfo {
    pub id: usize,
    pub vendor_id: String,
    pub cpu_family: usize,
    pub model: usize,
    pub model_name: String,
    pub cpu_mhz: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CpuInfo {
    pub id: usize,
    pub physical_id: Option<usize>,
    pub online: String,
    pub all_infos: CPUInfoFile,
    //pub hotplug_state: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CpuGlobalInfo {
    pub socket_nbr_detected: i64,
    pub cpu_nbr_detected: usize,
    pub cpu_nbr_online: usize,
    pub cpu_nbr_offline: usize,
    pub cpu_nbr_physical: usize,
    pub cpu_nbr_logical: usize,
    pub socket: HashMap<usize, SocketInfo>,
    pub cpu: HashMap<usize, CpuInfo>,
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

fn get_cpu_infos(cpu_status: &HashMap<usize, String>) -> (HashMap<usize, CpuInfo>, Vec<usize>, HashMap<usize, SocketInfo>) {
    let cpuinfofile = parse_cpuinfo();
    let mut cpuinfos = HashMap::new();
    let mut socketinfos = HashMap::new();
    let mut list_socket: Vec<usize> = Vec::new();
    for (cpuid, cpustatus) in cpu_status {
        if cpustatus == "online" {
            if !list_socket.contains(&cpuinfofile[cpuid].physical_id.unwrap()) { 
                list_socket.push(cpuinfofile[cpuid].physical_id.unwrap()); 
                socketinfos.insert(*cpuid, SocketInfo{
                    id: cpuinfofile[cpuid].physical_id.unwrap(),
                    vendor_id: cpuinfofile[cpuid].vendor_id.to_string(),
                    cpu_family: cpuinfofile[cpuid].cpu_family,
                    model: cpuinfofile[cpuid].model,
                    model_name: cpuinfofile[cpuid].model_name.to_string(),
                    cpu_mhz: cpuinfofile[cpuid].cpu_mhz,
                });
            }
            cpuinfos.insert(*cpuid, CpuInfo{
                id: *cpuid,
                physical_id: Some(cpuinfofile[cpuid].physical_id.unwrap()),
                online: (&cpustatus).to_string(),
                all_infos: cpuinfofile[cpuid].clone(),
            });
        }else{
            cpuinfos.insert(*cpuid, CpuInfo{
                id: *cpuid,
                physical_id: None,
                online: (&cpustatus).to_string(),
                all_infos: CPUInfoFile::default(),
            });
        }
    }
    (cpuinfos, list_socket, socketinfos)
}

fn get_cpu_global_infos() -> CpuGlobalInfo {
    let cpu_status = get_cpu_status();
    let (cpu, list_socket, socket) = get_cpu_infos(&cpu_status);
    let mut cpu_nbr_physical: usize = 0;
    let mut cpu_nbr_logical: usize = 0;
    let socket_nbr_detected: i64 = list_socket.len() as i64;

    for (_key, val) in &cpu {
        if val.online == "online" {
            cpu_nbr_physical = (val.all_infos.cpu_cores.unwrap() as i64 * socket_nbr_detected) as usize;
            cpu_nbr_logical = (val.all_infos.siblings.unwrap() as i64 * socket_nbr_detected) as usize - cpu_nbr_physical;
            break;
        }
    }

    CpuGlobalInfo{
        socket_nbr_detected,
        cpu_nbr_detected: cpu_status.keys().len() as usize,
        cpu_nbr_online: cpu_status.values().filter(|&x| x == "online").count() as usize,
        cpu_nbr_offline: cpu_status.values().filter(|&x| x == "offline").count() as usize,
        cpu_nbr_physical,
        cpu_nbr_logical,
        socket,
        cpu,
        
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

fn calculate_percentages(cpu_time: &CpuTime) -> HashMap<String, f64> {
    let total_time = cpu_time.user + cpu_time.nice + cpu_time.system + cpu_time.idle +
        cpu_time.iowait.unwrap_or(0) + cpu_time.irq.unwrap_or(0) +
        cpu_time.softirq.unwrap_or(0) + cpu_time.steal.unwrap_or(0) +
        cpu_time.guest.unwrap_or(0) + cpu_time.guest_nice.unwrap_or(0);

    let mut percentages = HashMap::new();
    percentages.insert("user".to_string(), (cpu_time.user as f64 / total_time as f64) * 100.0);
    percentages.insert("nice".to_string(), (cpu_time.nice as f64 / total_time as f64) * 100.0);
    percentages.insert("system".to_string(), (cpu_time.system as f64 / total_time as f64) * 100.0);
    percentages.insert("idle".to_string(), (cpu_time.idle as f64 / total_time as f64) * 100.0);

    if let Some(iowait) = cpu_time.iowait {
        percentages.insert("iowait".to_string(), (iowait as f64 / total_time as f64) * 100.0);
    }

    if let Some(irq) = cpu_time.irq {
        percentages.insert("irq".to_string(), (irq as f64 / total_time as f64) * 100.0);
    }

    if let Some(softirq) = cpu_time.softirq {
        percentages.insert("softirq".to_string(), (softirq as f64 / total_time as f64) * 100.0);
    }

    if let Some(steal) = cpu_time.steal {
        percentages.insert("steal".to_string(), (steal as f64 / total_time as f64) * 100.0);
    }

    if let Some(guest) = cpu_time.guest {
        percentages.insert("guest".to_string(), (guest as f64 / total_time as f64) * 100.0);
    }

    if let Some(guest_nice) = cpu_time.guest_nice {
        percentages.insert("guest_nice".to_string(), (guest_nice as f64 / total_time as f64) * 100.0);
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
                stat_prct: v1_cpu.stat_prct.clone(),
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

pub fn save_stats(file_stats: &str, timestamp: i64, getcpunow: &Vec<CpuStat>, ctxt: usize, processes: usize) {
    let mut file = File::create(file_stats).unwrap();
    writeln!(file, "cputime {}", timestamp).expect("Failed to write save file");
    writeln!(file, "cpujson {}", serde_json::to_string(&getcpunow).unwrap()).expect("Failed to write save file");
    writeln!(file, "ctxt {}", ctxt).expect("Failed to write save file");
    writeln!(file, "processes {}", processes).expect("Failed to write save file");
}

fn get_cpu_stats() -> (Vec<CpuStat>, usize, usize, usize ,usize){
    let contents_cpu_stats = fs::read_to_string("/proc/stat");
    let binding = contents_cpu_stats.expect("Failed to open /proc/stat");
    let lines_cpu_stats = binding.lines();

    let mut cpu_infos: Vec<CpuStat> = Vec::new();

    let all_softirqs = get_softirqs();

    let mut ctxt: usize = 0;
    let mut processes: usize = 0;
    let mut procs_running: usize = 0;
    let mut procs_blocked: usize = 0;

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

            let percentages = calculate_percentages(&stat);

            cpu_infos.push(CpuStat{
                name,
                stat,
                stat_prct: percentages,
                softirqs
            })
        }
        if line.starts_with("ctxt") {
            let data: Vec<&str> = line.split_whitespace().collect();
            ctxt = data[1].parse().unwrap();
        }
        if line.starts_with("processes") {
            let data: Vec<&str> = line.split_whitespace().collect();
            processes = data[1].parse().unwrap();
        }
        if line.starts_with("procs_running") {
            let data: Vec<&str> = line.split_whitespace().collect();
            procs_running = data[1].parse().unwrap();
        }
        if line.starts_with("procs_blocked") {
            let data: Vec<&str> = line.split_whitespace().collect();
            procs_blocked = data[1].parse().unwrap();
        }
    }
    (cpu_infos, ctxt, processes, procs_running, procs_blocked)
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

fn parse_cpuinfo() -> HashMap<usize, CPUInfoFile> {
    let file = File::open("/proc/cpuinfo").unwrap();
    let reader = BufReader::new(file);
    let mut cpus = HashMap::new();
    let mut cpu = CPUInfoFile::default();

    let lines = reader.lines();

    for line in lines {
        let line = match line {
            Ok(line) => line,
            Err(_) => continue,
        };
        let parts: Vec<&str> = line.split(':').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            cpus.insert(cpu.processor, cpu);
            cpu = CPUInfoFile::default();
            continue;
        }
        let key = parts[0];
        let value = parts[1];

        match key {
            "processor" => {
                cpu.processor = value.parse().unwrap_or(0);
            }
            "vendor_id" => {
                cpu.vendor_id = value.to_string();
            }
            "cpu family" => {
                cpu.cpu_family = value.parse().unwrap_or(0);
            }
            "model" => {
                cpu.model = value.parse().unwrap_or(0);
            }
            "model name" => {
                cpu.model_name = value.to_string();
            }
            "stepping" => {
                cpu.stepping = value.parse().unwrap_or(0);
            }
            "microcode" => {
                cpu.microcode = Some(value.to_string());
            }
            "cpu MHz" => {
                let parsed = value.parse::<f32>();
                if parsed.is_ok() {
                    cpu.cpu_mhz = Some(parsed.unwrap())
                }
            }
            "cache size" => {
                cpu.cache_size = Some(value.to_string());
            }
            "physical id" => {
                let parsed = value.parse::<usize>();
                if parsed.is_ok() {
                    cpu.physical_id = Some(parsed.unwrap());
                }
            }
            "siblings" => {
                let parsed = value.parse::<usize>();
                if parsed.is_ok() {
                    cpu.siblings = Some(parsed.unwrap());
                }
            }
            "core id" => {
                let parsed = value.parse::<usize>();
                if parsed.is_ok() {
                    cpu.core_id = Some(parsed.unwrap());
                }
            }
            "cpu cores" => {
                let parsed = value.parse::<usize>();
                if parsed.is_ok() {
                    cpu.cpu_cores = Some(parsed.unwrap());
                }
            }
            "apicid" => {
                let parsed = value.parse::<usize>();
                if parsed.is_ok() {
                    cpu.apicid = Some(parsed.unwrap());
                }
            }
            "initial apicid" => {
                let parsed = value.parse::<usize>();
                if parsed.is_ok() {
                    cpu.initial_apicid = Some(parsed.unwrap());
                }
            }
            "fpu" => {
                cpu.fpu = value == "yes";
            }
            "fpu_exception" => {
                cpu.fpu_exception = value == "yes";
            }
            "cpuid level" => {
                cpu.cpuid_level = value.parse().unwrap_or(0);
            }
            "wp" => {
                cpu.wp = value == "yes";
            }
            "flags" => {
                cpu.flags = value.to_string();
            }
            "bugs" => {
                let cpubug: String = value.to_string();
                cpu.bugs = Some(cpubug.split_whitespace().map(|s| s.to_owned()).collect());
            }
            "bogomips" => {
                let parsed = value.parse::<f32>();
                if parsed.is_ok() {
                    cpu.bogomips = parsed.unwrap();
                }
            }
            "clflush size" => {
                let parsed = value.parse::<usize>();
                if parsed.is_ok() {
                    cpu.clflush_size = Some(parsed.unwrap());
                }
            }
            "cache_alignment" => {
                let parsed = value.parse::<usize>();
                if parsed.is_ok() {
                    cpu.cache_alignment = Some(parsed.unwrap());
                }
            }
            "address sizes" => {
                cpu.address_sizes = Some(value.to_string());
            }
            _ => {}
        }
    }
    cpus
}

fn get_cpu_temperature() -> String {
    match File::open("/sys/class/thermal/thermal_zone0/temp") {
        Ok(mut file) => {
            let mut temperature = String::new();
            file.read_to_string(&mut temperature).unwrap();
            let temperature = temperature.trim().parse::<f64>().unwrap();
            let temperature = temperature / 1000.0;
            format!("{:.2} °C", temperature)
        },
        Err(_) => String::from("N/A"),
    }
}


fn output_horizontal_table() {

}

fn output_vertical_table(cli: bool, separator: bool, table: LinkedHashMap<String,String>) -> String{
    let mut output = String::new();

    // Retrieval of max key and value size
    let mut max_key_length = 0;
    let mut max_value_length = 0;
    for (key, value) in &table {
        let length_key = key.len();
        let length_value = value.len();
        if length_key > max_key_length {
            max_key_length = length_key;
        }
        if length_value > max_value_length {
            max_value_length = length_value;
        }
    }

    if cli {
        output.push_str("\n");
        if separator {
            output.push_str(&"-".repeat(max_key_length+max_value_length+4).to_string());
            output.push_str("\n");
        }
        for (key, val) in table {
            output.push_str(&format!("{0: <key_width$} | {1: <val_width$}",
                key, 
                val,
                key_width=max_key_length, val_width=max_value_length
            ).to_string());
            output.push_str("\n");
        }
        if separator {
            output.push_str(&"-".repeat(max_key_length+max_value_length+4).to_string());
            output.push_str("\n");
        }
    }else {

    }

    output.trim_end_matches('\n').to_string()
}


fn main() {

    // 
    // -- RETURN CODE Count --
    //
    let _rc_ok = 0;
    let _rc_warning = 0;
    let _rc_critical = 0;
    let _rc_unknown = 0;

    // 
    // -- ARGS --
    //
    let args = Command::new("check-sysstat-cpu")
                .author("Quentin BILLECOQ, quentin@billecoq.fr")
                .version("1.0.O")
                .about("Get CPU Stats")
                .arg(Arg::new("all")
                    .short('A')
                    .long("all")
                    .action(ArgAction::SetTrue)
                    .help("Get all CPU infos and stats")
                    .required(false)
                )
                .arg(Arg::new("infos")
                    .short('i')
                    .long("infos")
                    .action(ArgAction::SetTrue)
                    .help("Get CPU infos")
                    .required(false)
                )
                .arg( Arg::new("times")
                    .short('T')
                    .long("times")
                    .action(ArgAction::SetTrue)
                    .help("Get CPU times")
                    .required(false)
                )
                .arg(Arg::new("interrupts")
                    .short('I')
                    .long("interrupts")
                    .action(ArgAction::SetTrue)
                    .help("Get CPU interrupts")
                    .required(false)
                )
                .arg(Arg::new("softirqs")
                    .short('S')
                    .long("softirqs")
                    .action(ArgAction::SetTrue)
                    .help("Get CPU software interrupts")
                    .required(false)
                )
                .arg(Arg::new("warning")
                    .short('W')
                    .long("wargning")
                    .value_name("threshold")
                    .default_value("0")
                    .action(ArgAction::Set)
                    .value_parser(0..101)
                    .help("Threshold for warning alert")
                    .required(false)
                )
                .arg(Arg::new("critical")
                    .short('C')
                    .long("critical")
                    .value_name("threshold")
                    .default_value("0")
                    .action(ArgAction::Set)
                    .value_parser(0..101)
                    .help("Threshold for critical alert")
                    .required(false)
                )
                .arg(Arg::new("cli")
                    .long("cli")
                    .action(ArgAction::SetTrue)
                    .help("pass the output in cli mode")
                    .required(false)
                )
                .arg(Arg::new("statsfile")
                    .short('s')
                    .long("statsfile")
                    .value_name("PATH")
                    .default_value("/tmp/check-sysstat-cpu.stats")
                    .action(ArgAction::Set)
                    .help("File where is stocked last stats for next the run")
                    .required(false)
                )
                .get_matches();
    
    // 
    // -- Default value --
    //
    let args_cpu_all: bool = *args.get_one::<bool>("all").unwrap_or(&false);
    let args_cpu_infos: bool = *args.get_one::<bool>("infos").unwrap_or(&false);
    let args_cpu_times: bool = *args.get_one::<bool>("times").unwrap_or(&false);
    let args_cpu_interrupts: bool = *args.get_one::<bool>("interrupts").unwrap_or(&false);
    let args_cpu_softirqs: bool = *args.get_one::<bool>("softirqs").unwrap_or(&false);
    let warning_threshold: usize = *args.get_one::<i64>("warning").unwrap() as usize;
    let critical_threshold: usize = *args.get_one::<i64>("critical").unwrap() as usize;
    let cli_mode: bool = *args.get_one::<bool>("cli").unwrap_or(&false);
    let file_stats: &str = args.get_one::<String>("statsfile").unwrap();

    let timestamp: i64 = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64;


    // 
    // -- Main --
    //

    let mut first_start = true;
    let (getcpunow, ctxt, processes, procs_running, procs_blocked) = get_cpu_stats();
    let mut cpulastdata: Vec<CpuStat> = Vec::new();
    let mut lastchecktime: i64 = 0;
    let mut rate_ctxt: usize = 0;
    let mut rate_processes: usize = 0;

    if !fs::metadata(file_stats).is_ok() {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(file_stats)
            ;
        file.expect("REASON").write_all("".as_bytes()).expect("Error writing to file");
    }else {
        first_start = false;
        let contents_stats = fs::read_to_string(file_stats);
        let binding_stats = contents_stats.expect("REASON");
        let lines_stats = binding_stats.lines();
        for line in lines_stats {
            if line.starts_with("cputime") {
                let data: Vec<&str> = line.split_whitespace().collect();
                let date: i64 = match data[1].to_string().parse() {
                    Ok(n) => n,
                    Err(_) => panic!("Error parsing date string to u64"),
                };
                lastchecktime = timestamp-date;
                if lastchecktime<1 {
                    lastchecktime = 1;
                }
                println!("Temps depuis dernier check : {}s", lastchecktime)
            }
            if line.starts_with("cpujson") {
                let data: Vec<&str> = line.split_whitespace().collect();
                cpulastdata = serde_json::from_str(data[1]).unwrap();
            }
            if line.starts_with("ctxt") {
                let data: Vec<&str> = line.split_whitespace().collect();
                let value: i64 = match data[1].to_string().parse() {
                    Ok(n) => n,
                    Err(_) => panic!("Error parsing date string to u64"),
                };
                rate_ctxt = ((ctxt as i64-value)/lastchecktime) as usize;
            }
            if line.starts_with("processes") {
                let data: Vec<&str> = line.split_whitespace().collect();
                let value: i64 = match data[1].to_string().parse() {
                    Ok(n) => n,
                    Err(_) => panic!("Error parsing date string to u64"),
                };
                rate_processes = ((processes as i64-value)/lastchecktime) as usize;
            }
        }
    }


    save_stats(file_stats, timestamp, &getcpunow, ctxt, processes);

    let (mut diff_vec, added_removed) = compare_cpu_infos(getcpunow, cpulastdata);
    diff_vec.sort_by(|a, b| {
        match (a.name.as_str(), b.name.as_str()) {
            ("cpu", _) => std::cmp::Ordering::Less,
            (_, "cpu") => std::cmp::Ordering::Greater,
            (a_name, b_name) => {
                let a_num = a_name.split("cpu").nth(1).unwrap().parse::<i32>().unwrap_or(std::i32::MAX);
                let b_num = b_name.split("cpu").nth(1).unwrap().parse::<i32>().unwrap_or(std::i32::MAX);
                a_num.cmp(&b_num)
            }
        }
    });
    
    
    
    


    // ALERTS
    for stats in &diff_vec {
        if stats.name == "cpu" {
            let prct_total_used = round(100.0-stats.stat_prct["idle"],2);
            println!("Total cpu used : {:?}%", prct_total_used);
            if warning_threshold != 0 {
                if prct_total_used >= warning_threshold as f64 && (critical_threshold == 0 || prct_total_used <= critical_threshold as f64) {
                    println!("Warning : CPU usage exceed warning threshold {}% (Threshold : {}%)", prct_total_used, warning_threshold);
                }
            }
            if critical_threshold != 0 {
                if prct_total_used >= critical_threshold as f64 {
                    println!("Critical : CPU usage exceed critical threshold {}% (Threshold : {}%)", prct_total_used, critical_threshold);
                }
            }
        }
    }
    if lastchecktime < 2 {
        println!("Interval between two executions is too short, the information may be erroneous. It is advisable to wait at least 2 seconds before a second execution. Last interval ({}s)", lastchecktime);
    }
    for (cpu, act) in added_removed {
        println!("{} : {}", cpu.to_string(), act.to_string());
    }
    
    
    // CPU INFOS
    if args_cpu_all | args_cpu_infos {
        let mut system_infos_hm: LinkedHashMap<String, String> = LinkedHashMap::new();
        let system_infos = get_system_infos();
        system_infos_hm.insert(
            "Socket(s)".to_string(),
            system_infos.cpu.socket_nbr_detected.to_string(),
        );
        system_infos_hm.insert(
            "CPU(s)".to_string(), 
            format!("{} ({} Core / {} Thread)", system_infos.cpu.cpu_nbr_online, system_infos.cpu.cpu_nbr_physical, system_infos.cpu.cpu_nbr_logical).to_string(),
        );
        system_infos_hm.insert(
            "Temp.".to_string(), 
            get_cpu_temperature().to_string(),
        );

        println!();
        println!("{}", output_vertical_table(cli_mode, false, system_infos_hm));
    }

    // PROCESS INFOS
    if args_cpu_all | args_cpu_infos {
        let mut process_infos_hm: LinkedHashMap<String, String> = LinkedHashMap::new();
        process_infos_hm.insert(
            "Context switch/s".to_string(),
            rate_ctxt.to_string(),
        );
        process_infos_hm.insert(
            "Processes created/s".to_string(), 
            rate_processes.to_string(),
        );
        process_infos_hm.insert(
            "Processes running".to_string(), 
            procs_running.to_string(),
        );
        process_infos_hm.insert(
            "Processes bloacked/s".to_string(), 
            procs_blocked.to_string(),
        );
        
        print!("{}", output_vertical_table(cli_mode, true, process_infos_hm));
    }

    // CPU TIMES
    if args_cpu_all | args_cpu_times {
        println!();
        println!(
            "{0: <7} | {1: <10} | {2: <10} | {3: <10} | {4: <10} | {5: <10} | {6: <10} | {7: <10} | {8: <10} | {9: <10} | {10: <10}",
            "CPU", "%usr", "%nice", "%sys", "%iowait", "%irq", "%soft", "%steal", "%guest", "%gnice", "%idle"
        );
        println!("-------------------------------------------------------------------------------------------------------------------------------------");
        if !first_start {
            for stats in &diff_vec {
                    let name = if stats.name == "cpu" { "all" } else { &stats.name };

                    let percentages = &stats.stat_prct;

                    println!("{0: <7} | {1: <10} | {2: <10} | {3: <10} | {4: <10} | {5: <10?} | {6: <10?} | {7: <10?} | {8: <10?} | {9: <10?} | {10: <10?}", 
                        name,
                        round(percentages["user"], 2),
                        round(percentages["nice"], 2),
                        round(percentages["system"], 2),
                        round(percentages["iowait"], 2),
                        round(percentages["irq"], 2),
                        round(percentages["softirq"], 2),
                        round(percentages["steal"], 2),
                        round(percentages["guest"], 2),
                        round(percentages["guest_nice"], 2),
                        round(percentages["idle"], 2),
                );
            }
        }else {
            // If first run, stats initializing
            println!("First run : initializing...");
        }
    }
    
    // CPU SOFTIRQS
    if args_cpu_all | args_cpu_softirqs {
        println!();
        println!(
            "{0: <7} | {1: <10} | {2: <10} | {3: <10} | {4: <10} | {5: <10} | {6: <10} | {7: <10} | {8: <10} | {9: <10} | {10: <10}",
            "CPU", "HI/s", "TIMER/s", "NET_TX/s", "NET_RX/s", "BLOCK/s", "IRQ_POLL/s", "TASKLET/s", "SCHED/s", "HRTIMER/s", "RCU/s"
        );
        println!("-------------------------------------------------------------------------------------------------------------------------------------");
        if !first_start {
            for stats in &diff_vec {
                let name = if stats.name == "cpu" { "all" } else { &stats.name };
                println!("{0: <7} | {1: <10} | {2: <10} | {3: <10} | {4: <10} | {5: <10} | {6: <10} | {7: <10} | {8: <10} | {9: <10} | {10: <10}", 
                    name,
                    stats.softirqs.hi/lastchecktime,
                    stats.softirqs.timer/lastchecktime,
                    stats.softirqs.net_tx/lastchecktime,
                    stats.softirqs.net_rx/lastchecktime,
                    stats.softirqs.block/lastchecktime,
                    stats.softirqs.irq_poll/lastchecktime,
                    stats.softirqs.tasklet/lastchecktime,
                    stats.softirqs.sched/lastchecktime,
                    stats.softirqs.hrtimer/lastchecktime,
                    stats.softirqs.rcu/lastchecktime,
                );
            }
        }else {
            // If first run, stats initializing
            println!("First run : initializing...");
        }
    }

    // CPU INTERRUPTS
    if args_cpu_all | args_cpu_interrupts {

    }


    // Output


    // Management of return codes
    if _rc_unknown > 0 {
        exit(3);
    }else if _rc_critical > 0 {
        exit(2);
    }else if _rc_warning > 0 {
        exit(1);
    }else {
        exit(0);
    }

}
