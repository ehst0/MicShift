#![allow(non_snake_case)]

use crate::win32::GUID;
use std::ffi::c_void;
use std::ptr::null_mut;

pub type HRESULT = i32;

const S_FALSE: HRESULT = 1;
const COINIT_APARTMENTTHREADED: u32 = 0x2;
const CLSCTX_ALL: u32 = 0x17;
const STGM_READ: u32 = 0x00000000;
const DEVICE_STATE_ACTIVE: u32 = 0x00000001;
const E_CAPTURE: i32 = 1;
const VT_LPWSTR: u16 = 31;

const CLSID_MMDEVICE_ENUMERATOR: GUID = GUID::new(
    0xbcde0395,
    0xe52f,
    0x467c,
    [0x8e, 0x3d, 0xc4, 0x57, 0x92, 0x91, 0x69, 0x2e],
);

const IID_IMMDEVICE_ENUMERATOR: GUID = GUID::new(
    0xa95664d2,
    0x9614,
    0x4f35,
    [0xa7, 0x46, 0xde, 0x8d, 0xb6, 0x36, 0x17, 0xe6],
);

const CLSID_POLICY_CONFIG_CLIENT: GUID = GUID::new(
    0x870af99c,
    0x171d,
    0x4f9e,
    [0xaf, 0x0d, 0xe6, 0x3d, 0xf4, 0x0c, 0x2b, 0xc9],
);

const IID_IPOLICY_CONFIG: GUID = GUID::new(
    0xf8679f50,
    0x850a,
    0x41cf,
    [0x9c, 0x72, 0x43, 0x0f, 0x29, 0x02, 0x90, 0xc8],
);

const PKEY_DEVICE_FRIENDLY_NAME_FMTID: GUID = GUID::new(
    0xa45c254e,
    0xdf1c,
    0x4efd,
    [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0],
);
const PKEY_DEVICE_FRIENDLY_NAME_PID: u32 = 14;

#[repr(C)]
struct PropertyKey {
    fmtid: GUID,
    pid: u32,
}

const PKEY_DEVICE_FRIENDLY_NAME: PropertyKey = PropertyKey {
    fmtid: PKEY_DEVICE_FRIENDLY_NAME_FMTID,
    pid: PKEY_DEVICE_FRIENDLY_NAME_PID,
};

#[repr(C)]
struct PropVariant {
    vt: u16,
    reserved1: u16,
    reserved2: u16,
    reserved3: u16,
    data: [usize; 2],
}

impl Default for PropVariant {
    fn default() -> Self {
        Self {
            vt: 0,
            reserved1: 0,
            reserved2: 0,
            reserved3: 0,
            data: [0; 2],
        }
    }
}

#[repr(C)]
struct IUnknownVtbl {
    QueryInterface:
        unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> HRESULT,
    AddRef: unsafe extern "system" fn(*mut c_void) -> u32,
    Release: unsafe extern "system" fn(*mut c_void) -> u32,
}

#[repr(C)]
struct IMMDeviceEnumeratorVtbl {
    parent: IUnknownVtbl,
    EnumAudioEndpoints:
        unsafe extern "system" fn(*mut c_void, i32, u32, *mut *mut c_void) -> HRESULT,
    GetDefaultAudioEndpoint: usize,
    GetDevice: usize,
    RegisterEndpointNotificationCallback: usize,
    UnregisterEndpointNotificationCallback: usize,
}

#[repr(C)]
struct IMMDeviceCollectionVtbl {
    parent: IUnknownVtbl,
    GetCount: unsafe extern "system" fn(*mut c_void, *mut u32) -> HRESULT,
    Item: unsafe extern "system" fn(*mut c_void, u32, *mut *mut c_void) -> HRESULT,
}

#[repr(C)]
struct IMMDeviceVtbl {
    parent: IUnknownVtbl,
    Activate: usize,
    OpenPropertyStore: unsafe extern "system" fn(*mut c_void, u32, *mut *mut c_void) -> HRESULT,
    GetId: unsafe extern "system" fn(*mut c_void, *mut *mut u16) -> HRESULT,
    GetState: usize,
}

#[repr(C)]
struct IPropertyStoreVtbl {
    parent: IUnknownVtbl,
    GetCount: usize,
    GetAt: usize,
    GetValue:
        unsafe extern "system" fn(*mut c_void, *const PropertyKey, *mut PropVariant) -> HRESULT,
    SetValue: usize,
    Commit: usize,
}

#[repr(C)]
struct IPolicyConfigVtbl {
    parent: IUnknownVtbl,
    GetMixFormat: usize,
    GetDeviceFormat: usize,
    ResetDeviceFormat: usize,
    SetDeviceFormat: usize,
    GetProcessingPeriod: usize,
    SetProcessingPeriod: usize,
    GetShareMode: usize,
    SetShareMode: usize,
    GetPropertyValue: usize,
    SetPropertyValue: usize,
    SetDefaultEndpoint: unsafe extern "system" fn(*mut c_void, *const u16, i32) -> HRESULT,
    SetEndpointVisibility: usize,
}

#[link(name = "ole32")]
extern "system" {
    fn CoInitializeEx(pvReserved: *mut c_void, dwCoInit: u32) -> HRESULT;
    fn CoUninitialize();
    fn CoCreateInstance(
        rclsid: *const GUID,
        pUnkOuter: *mut c_void,
        dwClsContext: u32,
        riid: *const GUID,
        ppv: *mut *mut c_void,
    ) -> HRESULT;
    fn CoTaskMemFree(pv: *mut c_void);
    fn PropVariantClear(pvar: *mut PropVariant) -> HRESULT;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
}

pub struct ComGuard {
    initialized: bool,
}

impl ComGuard {
    pub fn new() -> Result<Self, String> {
        let hr = unsafe { CoInitializeEx(null_mut(), COINIT_APARTMENTTHREADED) };
        if failed(hr) {
            return Err(format_hresult("CoInitializeEx", hr));
        }
        Ok(Self { initialized: true })
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.initialized {
            unsafe { CoUninitialize() };
        }
    }
}

fn failed(hr: HRESULT) -> bool {
    hr < 0
}

fn format_hresult(operation: &str, hr: HRESULT) -> String {
    format!("{operation} failed (HRESULT 0x{:08X})", hr as u32)
}

unsafe fn vtable<T>(interface: *mut c_void) -> &'static T {
    &**(interface as *mut *const T)
}

unsafe fn release(interface: *mut c_void) {
    if interface.is_null() {
        return;
    }
    let vt = vtable::<IUnknownVtbl>(interface);
    (vt.Release)(interface);
}

unsafe fn wide_ptr_to_string(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0usize;
    while *ptr.add(len) != 0 {
        len += 1;
    }
    String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len))
}

fn to_wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

pub fn enumerate_active_capture_devices() -> Result<Vec<AudioDevice>, String> {
    unsafe {
        let mut enumerator: *mut c_void = null_mut();
        let hr = CoCreateInstance(
            &CLSID_MMDEVICE_ENUMERATOR,
            null_mut(),
            CLSCTX_ALL,
            &IID_IMMDEVICE_ENUMERATOR,
            &mut enumerator,
        );
        if failed(hr) {
            return Err(format_hresult("CoCreateInstance(MMDeviceEnumerator)", hr));
        }

        let result = enumerate_with(enumerator);
        release(enumerator);
        result
    }
}

unsafe fn enumerate_with(enumerator: *mut c_void) -> Result<Vec<AudioDevice>, String> {
    let enum_vt = vtable::<IMMDeviceEnumeratorVtbl>(enumerator);
    let mut collection: *mut c_void = null_mut();
    let hr =
        (enum_vt.EnumAudioEndpoints)(enumerator, E_CAPTURE, DEVICE_STATE_ACTIVE, &mut collection);
    if failed(hr) {
        return Err(format_hresult("EnumAudioEndpoints", hr));
    }

    let result = enumerate_collection(collection);
    release(collection);
    result
}

unsafe fn enumerate_collection(collection: *mut c_void) -> Result<Vec<AudioDevice>, String> {
    let collection_vt = vtable::<IMMDeviceCollectionVtbl>(collection);
    let mut count = 0u32;
    let hr = (collection_vt.GetCount)(collection, &mut count);
    if failed(hr) {
        return Err(format_hresult("IMMDeviceCollection::GetCount", hr));
    }

    let mut devices = Vec::with_capacity(count as usize);

    for index in 0..count {
        let mut device: *mut c_void = null_mut();
        let hr = (collection_vt.Item)(collection, index, &mut device);
        if failed(hr) || device.is_null() {
            continue;
        }

        if let Ok(item) = read_device(device) {
            devices.push(item);
        }
        release(device);
    }

    Ok(devices)
}

unsafe fn read_device(device: *mut c_void) -> Result<AudioDevice, String> {
    let device_vt = vtable::<IMMDeviceVtbl>(device);

    let mut id_ptr: *mut u16 = null_mut();
    let hr = (device_vt.GetId)(device, &mut id_ptr);
    if failed(hr) {
        return Err(format_hresult("IMMDevice::GetId", hr));
    }
    let id = wide_ptr_to_string(id_ptr);
    CoTaskMemFree(id_ptr as *mut c_void);

    let mut store: *mut c_void = null_mut();
    let hr = (device_vt.OpenPropertyStore)(device, STGM_READ, &mut store);
    if failed(hr) {
        return Err(format_hresult("IMMDevice::OpenPropertyStore", hr));
    }

    let store_vt = vtable::<IPropertyStoreVtbl>(store);
    let mut value = PropVariant::default();
    let hr = (store_vt.GetValue)(store, &PKEY_DEVICE_FRIENDLY_NAME, &mut value);
    let name = if failed(hr) {
        id.clone()
    } else if value.vt == VT_LPWSTR && value.data[0] != 0 {
        wide_ptr_to_string(value.data[0] as *const u16)
    } else {
        id.clone()
    };

    let _ = PropVariantClear(&mut value);
    release(store);

    Ok(AudioDevice { id, name })
}

pub fn set_default_capture_device(device_id: &str) -> Result<(), String> {
    unsafe {
        let mut policy: *mut c_void = null_mut();
        let hr = CoCreateInstance(
            &CLSID_POLICY_CONFIG_CLIENT,
            null_mut(),
            CLSCTX_ALL,
            &IID_IPOLICY_CONFIG,
            &mut policy,
        );
        if failed(hr) {
            return Err(format_hresult("CoCreateInstance(PolicyConfig)", hr));
        }

        let vt = vtable::<IPolicyConfigVtbl>(policy);
        let wide = to_wide(device_id);

        // Set Console, Multimedia, and Communications roles so apps using
        // "Default" follow the same microphone consistently.
        for role in [0i32, 1i32, 2i32] {
            let hr = (vt.SetDefaultEndpoint)(policy, wide.as_ptr(), role);
            if failed(hr) {
                release(policy);
                return Err(format_hresult("IPolicyConfig::SetDefaultEndpoint", hr));
            }
        }

        release(policy);
        Ok(())
    }
}

pub fn find_active_device_by_id(device_id: &str) -> Result<Option<AudioDevice>, String> {
    let devices = enumerate_active_capture_devices()?;
    Ok(devices.into_iter().find(|d| d.id == device_id))
}

#[allow(dead_code)]
fn _s_false_is_success() -> bool {
    !failed(S_FALSE)
}
