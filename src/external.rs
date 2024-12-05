use std::marker::{PhantomData, PhantomPinned};

use core_foundation::{
    base::CFTypeRef,
    dictionary::{CFDictionaryRef, CFMutableDictionaryRef},
    string::CFStringRef,
};

pub type CVoidRef = *const std::ffi::c_void;

#[repr(C)]
pub struct IOReportSubscription {
    _data: [u8; 0],
    _phantom: PhantomData<(*mut u8, PhantomPinned)>,
}

pub type IOReportSubscriptionRef = *const IOReportSubscription;

#[link(name = "IOReport", kind = "dylib")]
extern "C" {
    // pub fn IOReportCopyAllChannels(a: u64, b: u64) -> CFMutableDictionaryRef;
    pub fn IOReportCopyChannelsInGroup(
        a: CFStringRef,
        b: CFStringRef,
        c: u64,
        d: u64,
        e: u64,
    ) -> CFDictionaryRef;
    pub fn IOReportMergeChannels(a: CFDictionaryRef, b: CFDictionaryRef, nil: CFTypeRef);
    pub fn IOReportCreateSubscription(
        a: CVoidRef,
        b: CFMutableDictionaryRef,
        c: *mut CFMutableDictionaryRef,
        d: u64,
        b: CFTypeRef,
    ) -> IOReportSubscriptionRef;
    pub fn IOReportCreateSamples(
        a: IOReportSubscriptionRef,
        b: CFMutableDictionaryRef,
        c: CFTypeRef,
    ) -> CFDictionaryRef;
    pub fn IOReportCreateSamplesDelta(
        a: CFDictionaryRef,
        b: CFDictionaryRef,
        c: CFTypeRef,
    ) -> CFDictionaryRef;
    // pub fn IOReportChannelGetGroup(a: CFDictionaryRef) -> CFStringRef;
    // pub fn IOReportChannelGetSubGroup(a: CFDictionaryRef) -> CFStringRef;
    pub fn IOReportChannelGetChannelName(a: CFDictionaryRef) -> CFStringRef;
    // pub fn IOReportChannelGetFormat(a: CFDictionaryRef) -> IOReportFormat;
    // pub fn IOReportChannelGetDriverName(a: CFDictionaryRef) -> CFStringRef;
    pub fn IOReportSimpleGetIntegerValue(a: CFDictionaryRef, b: i32) -> i64;
    pub fn IOReportChannelGetUnitLabel(a: CFDictionaryRef) -> CFStringRef;
    // pub fn IOReportStateGetCount(a: CFDictionaryRef) -> i32;
    // pub fn IOReportStateGetNameForIndex(a: CFDictionaryRef, b: i32) -> CFStringRef;
    // pub fn IOReportStateGetResidency(a: CFDictionaryRef, b: i32) -> i64;
}

// pub type IOReportFormat = u8;
