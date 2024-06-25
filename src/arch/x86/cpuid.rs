use core::arch::asm;
use crate::klog;

extern "C" {
    fn cpuid_supported() -> u32;
}

struct CpuInfo {
    vendor_string: [u8; 12],
    features_edx: u32,
    features_ecx: u32
}

static mut CPUID : CpuInfo = CpuInfo {
    vendor_string: [0; 12],
    features_edx: 0,
    features_ecx: 0
};

// TODO refactor using bitflags ?
#[allow(dead_code)]
mod flags {
        pub const CPUID_FEAT_ECX_SSE3: u32 = 1 << 0;
        pub const CPUID_FEAT_ECX_PCLMUL: u32 = 1 << 1;
        pub const CPUID_FEAT_ECX_DTES64: u32 = 1 << 2;
        pub const CPUID_FEAT_ECX_MONITOR: u32 = 1 << 3;
        pub const CPUID_FEAT_ECX_DS_CPL: u32 = 1 << 4;
        pub const CPUID_FEAT_ECX_VMX: u32 = 1 << 5;
        pub const CPUID_FEAT_ECX_SMX: u32 = 1 << 6;
        pub const CPUID_FEAT_ECX_EST: u32 = 1 << 7;
        pub const CPUID_FEAT_ECX_TM2: u32 = 1 << 8;
        pub const CPUID_FEAT_ECX_SSSE3: u32 = 1 << 9;
        pub const CPUID_FEAT_ECX_CID: u32 = 1 << 10;
        pub const CPUID_FEAT_ECX_SDBG: u32 = 1 << 11;
        pub const CPUID_FEAT_ECX_FMA: u32 = 1 << 12;
        pub const CPUID_FEAT_ECX_CX16: u32 = 1 << 13;
        pub const CPUID_FEAT_ECX_XTPR: u32 = 1 << 14;
        pub const CPUID_FEAT_ECX_PDCM: u32 = 1 << 15;
        pub const CPUID_FEAT_ECX_PCID: u32 = 1 << 17;
        pub const CPUID_FEAT_ECX_DCA: u32 = 1 << 18;
        pub const CPUID_FEAT_ECX_SSE4_1: u32 = 1 << 19;
        pub const CPUID_FEAT_ECX_SSE4_2: u32 = 1 << 20;
        pub const CPUID_FEAT_ECX_X2APIC: u32 = 1 << 21;
        pub const CPUID_FEAT_ECX_MOVBE: u32 = 1 << 22;
        pub const CPUID_FEAT_ECX_POPCNT: u32 = 1 << 23;
        pub const CPUID_FEAT_ECX_TSC: u32 = 1 << 24;
        pub const CPUID_FEAT_ECX_AES: u32 = 1 << 25;
        pub const CPUID_FEAT_ECX_XSAVE: u32 = 1 << 26;
        pub const CPUID_FEAT_ECX_OSXSAVE: u32 = 1 << 27;
        pub const CPUID_FEAT_ECX_AVX: u32 = 1 << 28;
        pub const CPUID_FEAT_ECX_F16C: u32 = 1 << 29;
        pub const CPUID_FEAT_ECX_RDRAND: u32 = 1 << 30;
        pub const CPUID_FEAT_ECX_HYPERVISOR: u32 = 1 << 31;
        pub const CPUID_FEAT_EDX_FPU: u32 = 1 << 0;
        pub const CPUID_FEAT_EDX_VME: u32 = 1 << 1;
        pub const CPUID_FEAT_EDX_DE: u32 = 1 << 2;
        pub const CPUID_FEAT_EDX_PSE: u32 = 1 << 3;
        pub const CPUID_FEAT_EDX_TSC: u32 = 1 << 4;
        pub const CPUID_FEAT_EDX_MSR: u32 = 1 << 5;
        pub const CPUID_FEAT_EDX_PAE: u32 = 1 << 6;
        pub const CPUID_FEAT_EDX_MCE: u32 = 1 << 7;
        pub const CPUID_FEAT_EDX_CX8: u32 = 1 << 8;
        pub const CPUID_FEAT_EDX_APIC: u32 = 1 << 9;
        pub const CPUID_FEAT_EDX_SEP: u32 = 1 << 11;
        pub const CPUID_FEAT_EDX_MTRR: u32 = 1 << 12;
        pub const CPUID_FEAT_EDX_PGE: u32 = 1 << 13;
        pub const CPUID_FEAT_EDX_MCA: u32 = 1 << 14;
        pub const CPUID_FEAT_EDX_CMOV: u32 = 1 << 15;
        pub const CPUID_FEAT_EDX_PAT: u32 = 1 << 16;
        pub const CPUID_FEAT_EDX_PSE36: u32 = 1 << 17;
        pub const CPUID_FEAT_EDX_PSN: u32 = 1 << 18;
        pub const CPUID_FEAT_EDX_CLFLUSH: u32 = 1 << 19;
        pub const CPUID_FEAT_EDX_DS: u32 = 1 << 21;
        pub const CPUID_FEAT_EDX_ACPI: u32 = 1 << 22;
        pub const CPUID_FEAT_EDX_MMX: u32 = 1 << 23;
        pub const CPUID_FEAT_EDX_FXSR: u32 = 1 << 24;
        pub const CPUID_FEAT_EDX_SSE: u32 = 1 << 25;
        pub const CPUID_FEAT_EDX_SSE2: u32 = 1 << 26;
        pub const CPUID_FEAT_EDX_SS: u32 = 1 << 27;
        pub const CPUID_FEAT_EDX_HTT: u32 = 1 << 28;
        pub const CPUID_FEAT_EDX_TM: u32 = 1 << 29;
        pub const CPUID_FEAT_EDX_IA64: u32 = 1 << 30;
        pub const CPUID_FEAT_EDX_PBE: u32 = 1 << 31;
}

fn read_vendor_string(){
    unsafe {
            asm!(
                "cpuid",
                "mov [edi], ebx",
                "mov [edi + 4], edx",
                "mov [edi + 8], ecx",
                in("rdi") CPUID.vendor_string.as_ptr(),
                inout("eax") 0 => _,
                out("ebx") _,
                out("ecx") _,
                out("edx") _,
            );
    }
}

#[no_mangle]
pub fn read_cpu_features() {
    unsafe {
            asm!(
                "cpuid",
                "mov {0}, edx",
                "mov {1}, ecx",
                out(reg) CPUID.features_edx,
                out(reg) CPUID.features_ecx,
                inout("eax") 1 => _,
                out("ebx") _,
                out("ecx") _,
                out("edx") _,
            );
        // TODO check for newer CPU to know if ECX has bogus info
    }
}

pub fn check_local_apic() -> bool {
    unsafe {
        CPUID.features_edx & flags::CPUID_FEAT_EDX_APIC != 0
    }
}

pub fn check_rdmsr() -> bool {
    unsafe {
        CPUID.features_edx & flags::CPUID_FEAT_EDX_MSR != 0
    }
}

pub fn vendor() -> &'static str {
    unsafe {
        core::str::from_utf8(&CPUID.vendor_string).unwrap()
    }
}

pub fn init() {
    unsafe {
        if cpuid_supported() != 0 {
            read_vendor_string();
            read_cpu_features();
        }
        else {
            panic!("CPUID not supported !");
        }
    }
}
