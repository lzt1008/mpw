use std::{
    collections::VecDeque,
    mem::{self},
    ops::{Deref, DerefMut},
    ptr::{self},
    sync::mpsc::Sender,
    thread::{self, JoinHandle},
    time::Duration,
};

use core_foundation::{
    array::CFArray,
    base::{kCFAllocatorDefault, TCFType},
    dictionary::{
        CFDictionary, CFDictionaryCreateMutableCopy, CFDictionaryGetCount, CFDictionaryRef,
        CFMutableDictionary,
    },
    string::CFString,
};
use scopefn::RunIf;

use crate::external::{
    IOReportChannelGetChannelName, IOReportChannelGetUnitLabel, IOReportCopyChannelsInGroup,
    IOReportCreateSamples, IOReportCreateSamplesDelta, IOReportCreateSubscription,
    IOReportMergeChannels, IOReportSimpleGetIntegerValue, IOReportSubscriptionRef,
};

pub fn get_channel<'a>(
    items: impl IntoIterator<Item = (&'a str, Option<&'a str>)>,
) -> CFMutableDictionary {
    let channels = items
        .into_iter()
        .map(|(group, subgroup)| unsafe {
            CFDictionary::wrap_under_create_rule(IOReportCopyChannelsInGroup(
                CFString::new(group).as_concrete_TypeRef(),
                subgroup.map_or(ptr::null(), |sub| CFString::new(sub).as_concrete_TypeRef()),
                0,
                0,
                0,
            ))
        })
        .collect::<Vec<CFDictionary>>();

    let chan: &CFDictionary = &channels[0];
    for channel in channels.iter().skip(1) {
        unsafe {
            IOReportMergeChannels(
                chan.as_concrete_TypeRef(),
                channel.as_concrete_TypeRef(),
                ptr::null(),
            )
        }
    }

    unsafe {
        CFMutableDictionary::wrap_under_create_rule(CFDictionaryCreateMutableCopy(
            kCFAllocatorDefault,
            CFDictionaryGetCount(chan.as_concrete_TypeRef()),
            chan.as_concrete_TypeRef(),
        ))
    }
}

pub fn get_subscription(channel: &CFMutableDictionary) -> IOReportSubscriptionRef {
    unsafe {
        IOReportCreateSubscription(
            ptr::null(),
            channel.as_concrete_TypeRef(),
            &mut mem::zeroed(),
            0,
            ptr::null(),
        )
        .as_ref()
        .expect("Failed to create subscription")
    }
}

#[derive(Debug)]
pub struct IOReportChannel {
    // pub group: String,
    // pub subgroup: Option<String>,
    pub name: String,
    pub unit: String,
    // pub format: IOReportFormat,
    pub value: i64,
}

impl IOReportChannel {
    pub fn new(dict: CFDictionaryRef) -> Self {
        unsafe {
            Self {
                // group: IOReportChannelGetGroup(dict).to_string(),
                // subgroup: IOReportChannelGetSubGroup(dict)
                //     .as_non_null()
                //     .map(|v| v.to_string()),
                name: CFString::wrap_under_get_rule(IOReportChannelGetChannelName(dict))
                    .to_string(),
                unit: CFString::wrap_under_get_rule(IOReportChannelGetUnitLabel(dict)).to_string(),
                // format: IOReportChannelGetFormat(dict),
                value: IOReportSimpleGetIntegerValue(dict, 0),
            }
        }
    }

    pub fn as_power(&self, duration: Duration) -> f64 {
        let energy = match self.unit.as_str() {
            "nJ" => self.value as f64 / 1e6,
            "uJ" => self.value as f64 / 1e3,
            "mJ" => self.value as f64,
            _ => panic!("Unsupported unit {}", self.unit),
        };
        energy / duration.as_secs_f64()
    }
}

pub struct IOReport {
    channel: CFMutableDictionary,
    subscription: IOReportSubscriptionRef,
}

impl IOReport {
    pub fn new<'a>(items: impl IntoIterator<Item = (&'a str, Option<&'a str>)>) -> Self {
        let channel = get_channel(items);
        let subscription = get_subscription(&channel);
        Self {
            channel,
            subscription,
        }
    }

    pub fn create_samples_delta(
        &self,
        duration: Duration,
    ) -> CFDictionary<CFString, CFArray<CFDictionary>> {
        let start: CFDictionary = unsafe {
            CFDictionary::wrap_under_create_rule(IOReportCreateSamples(
                self.subscription,
                self.channel.as_concrete_TypeRef(),
                ptr::null(),
            ))
        };
        thread::sleep(duration);
        let end: CFDictionary = unsafe {
            CFDictionary::wrap_under_create_rule(IOReportCreateSamples(
                self.subscription,
                self.channel.as_concrete_TypeRef(),
                ptr::null(),
            ))
        };

        unsafe {
            CFDictionary::wrap_under_create_rule(IOReportCreateSamplesDelta(
                start.as_concrete_TypeRef(),
                end.as_concrete_TypeRef(),
                ptr::null(),
            ))
        }
    }
}

#[derive(Debug, Default)]
pub struct Power {
    pub cpu: f64,
    pub ecpu: f64,
    pub pcpu: f64,

    pub isp: f64,
    pub gpu: f64,
    pub gpu_sram: f64,
    pub ane: f64,
    pub ave: f64,
    pub dram: f64,
    pub pcie: f64,
}

pub struct PowerStore {
    pub capacity: usize,
    pub data: VecDeque<Power>,
}

impl Deref for PowerStore {
    type Target = VecDeque<Power>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for PowerStore {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl PowerStore {
    pub fn new(cap: usize) -> Self {
        Self {
            capacity: cap,
            data: VecDeque::new(),
        }
    }

    pub fn push_back(&mut self, power: Power) {
        if self.data.len() >= self.capacity {
            self.data.pop_front();
        }
        self.data.push_back(power);
    }

    pub fn get_power_data(&self, width: usize, f: impl Fn(&Power) -> u64) -> Vec<u64> {
        self.iter()
            .run_if(self.len() > width, |v| v.skip(self.len() - width))
            .map(f)
            .collect()
    }

    pub fn cpu(&self, width: usize) -> Vec<u64> {
        self.get_power_data(width, |v| v.cpu as u64)
    }

    pub fn e_cpu(&self, width: usize) -> Vec<u64> {
        self.get_power_data(width, |v| v.ecpu as u64)
    }

    pub fn p_cpu(&self, width: usize) -> Vec<u64> {
        self.get_power_data(width, |v| v.pcpu as u64)
    }

    pub fn gpu(&self, width: usize) -> Vec<u64> {
        self.get_power_data(width, |v| v.gpu as u64)
    }

    pub fn gpu_sram(&self, width: usize) -> Vec<u64> {
        self.get_power_data(width, |v| v.gpu_sram as u64)
    }

    pub fn ane(&self, width: usize) -> Vec<u64> {
        self.get_power_data(width, |v| v.ane as u64)
    }

    pub fn ave(&self, width: usize) -> Vec<u64> {
        self.get_power_data(width, |v| v.ave as u64)
    }

    pub fn dram(&self, width: usize) -> Vec<u64> {
        self.get_power_data(width, |v| v.dram as u64)
    }

    pub fn pcie(&self, width: usize) -> Vec<u64> {
        self.get_power_data(width, |v| v.pcie as u64)
    }

    pub fn isp(&self, width: usize) -> Vec<u64> {
        self.get_power_data(width, |v| v.isp as u64)
    }

    pub fn total(&self, width: usize) -> Vec<u64> {
        self.get_power_data(width, |v| {
            (v.cpu + v.gpu + v.ane + v.dram + v.ave + v.isp) as u64
        })
    }
}

pub fn spawn_report_thread(tx: Sender<Power>, interval: Duration) -> JoinHandle<()> {
    thread::spawn(move || {
        let reporter = IOReport::new([("Energy Model", None)]);
        loop {
            let sample = reporter.create_samples_delta(interval);

            let res = tx.send(
                sample
                    .get(CFString::from_static_string("IOReportChannels"))
                    .iter()
                    .map(|v| IOReportChannel::new(v.as_concrete_TypeRef()))
                    .filter(|v| !is_unnecessary_field(v))
                    .map(|v| (v.as_power(interval), v.name))
                    .fold(Power::default(), |mut acc, (power, name)| {
                        match name.as_str() {
                            "CPU Energy" => acc.cpu = power,
                            "GPU Energy" => acc.gpu = power,
                            "ISP0" => acc.isp = power,
                            "AVE0" => acc.ave = power,
                            "DRAM0" => acc.dram = power,
                            "PACC0_CPU" | "PACC1_CPU" => acc.pcpu += power,
                            "EACC_CPU" => acc.ecpu += power,
                            "GPU SRAM0" => acc.gpu_sram += power,
                            _ if name.contains("ANE") => {
                                acc.ane += power;
                            }
                            _ if name.contains("PCIe Port") || name.contains("apciec") => {
                                acc.pcie += power
                            }
                            _ => {}
                        }
                        acc
                    }),
            );

            if let Err(e) = res {
                panic!("{}", e);
            }
        }
    })
}

fn is_unnecessary_field(v: &IOReportChannel) -> bool {
    v.name.starts_with("PCPUDT") || v.name.starts_with("ECPUDT") || v.name.starts_with("PCPU1DT")
}
