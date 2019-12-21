// vim: set ai et ts=4 sts=4 sw=4:
#![macro_use]
#![allow(dead_code, unused_macros)]

use std::ops::Drop;

pub struct DebugPrinterStatus {
    pub enabled: bool,
    pub indent_level: usize,
}
pub static mut DPRINT_STATUS: DebugPrinterStatus = DebugPrinterStatus {
    enabled: true,
    indent_level: 0,
};

pub struct DebugPrinterScope {
}
impl DebugPrinterScope {
    pub fn new() -> Self {
        unsafe { DPRINT_STATUS.indent_level += 1; }
        DebugPrinterScope { }
    }
}
impl Drop for DebugPrinterScope {
    fn drop(&mut self) {
        unsafe { DPRINT_STATUS.indent_level -= 1; }
    }
}

pub struct DebugPrinterDisable {
    old_status: bool,
}
impl DebugPrinterDisable {
    pub fn new() -> Self {
        unsafe {
            let result = Self { old_status: DPRINT_STATUS.enabled };
            DPRINT_STATUS.enabled = false;
            result
        }
    }
}
impl Drop for DebugPrinterDisable {
    fn drop(&mut self) {
        unsafe { DPRINT_STATUS.enabled = self.old_status; }
    }
}

macro_rules! dprint {
    ($($arg:tt)*) => {{
        let enabled = unsafe { DPRINT_STATUS.enabled };
        if enabled {
            let indent: String = unsafe { "    ".repeat(DPRINT_STATUS.indent_level) };
            let mut formatted: String = format!($($arg)*);
            formatted.insert_str(0, &indent);
            println!("{}", formatted.replace('\n', &("\n".to_owned() + &indent)));
        }
    }}
}

macro_rules! dscope {
    () => { let _dprint_scope = DebugPrinterScope::new(); }
}

macro_rules! ddisable {
    () => { let _dprint_disable = DebugPrinterDisable::new(); }
}
