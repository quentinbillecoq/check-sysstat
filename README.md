Advanced system analysis and monitoring ASAM :

CPU
-----------

### Alerts
- [x] CPUs hot add or remove detection
- [x] CPU global usage with threshold (C/W)

### Summary Informations
- [x] Number of socket (Physical processor)
- [x] Number of CPU(s) (Physical core / logical core (Thread))
- [ ] Temperature (If available)

### Detailed Informations
- [ ] Architecture 
- [ ] CPU op-mode(s)
- [ ] Byte Order
- [x] Number of socket (Physical processor)
- [x] Number of CPU(s) (Physical core / logical core (Thread))
- [ ] Thread(s) per core
- [ ] Core(s) per socket
- [ ] NUMA node(s)
- [ ] NUMA nodes Informations(s)
- [ ] Vendor ID
- [ ] BIOS Vendor ID
- [ ] CPU family
- [ ] Model
- [ ] Model name
- [ ] BIOS Model name
- [ ] Stepping
- [ ] CPU MHz
- [ ] BogoMIPS
- [ ] Hypervisor vendor
- [ ] Virtualization type
- [ ] L1d cache
- [ ] L1i cache
- [ ] L2 cache
- [ ] L3 cache
- [ ] Flags

### Vulnerabilities
- [ ] itlb_multihit
- [ ] l1tf
- [ ] mds
- [ ] meltdown
- [ ] spec_store_bypass
- [ ] spectre_v1
- [ ] spectre_v2
- [ ] srbds
- [ ] tsx_async_abort

### Summary Numa Node Informations
- [ ] Node list with associated CPU 

### Process Info
- [x] Context switch
    - [ ] Total since startup
    - [x] Per seconde via last script run
- [x] Processes created
    - [ ] Total since startup
    - [x] Per seconde via last script run
- [x] Number of processes running
- [x] Number of processes blocked

### CPU Times
- [ ] Summary Informations
    - [ ] Total time since startup
    - [ ] Total time since last script run
- [x] Detail per CPU Times and per CPU (with total of all CPU)
    - Options
        - [x] (Default) Time in percent since last script run
        - [ ] Total time since startup
        - [ ] Total time since last script run
    - List
        - [x] User
        - [x] Nice
        - [x] System
        - [x] IOWAIT
        - [x] Irq
        - [x] Soft
        - [x] Steal
        - [x] Guest
        - [x] Guest nice
        - [x] Idle
        - [ ] Total times

### Softirqs (Software Interrupt)
- [ ] Summary Informations
    - [ ] Total interrupt since startup
    - [ ] Total interrupt since last script run
    - [ ] Total interrupt avg per second since last script run
- [x] Detail per interrupt and per CPU (with total of all CPU)
    - Options
        - [x] (Default) Interrupt avg per second since last script run
        - [ ] Number of interrupt since startup
        - [ ] Number of interrupt since last script run
    - List
        - [x] HI: clock interrupts
        - [x] TIMER: timer interrupts
        - [x] NET_TX: network transmit interrupts
        - [x] NET_RX: network receive interrupts
        - [x] BLOCK: block device interrupts
        - [x] IRQ_POLL: IRQ polling interrupts
        - [x] TASKLET: tasklet interrupts
        - [x] SCHED: scheduling interrupts
        - [x] HRTIMER: high-resolution timer interrupts
        - [x] RCU: RCU interrupts
        - [ ] Total interrupts