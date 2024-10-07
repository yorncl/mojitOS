use bitflags::bitflags;
use core::arch::asm;

extern "C" {
    fn cpuid_supported() -> u32;
}

bitflags! {
    pub struct Flags: u64 {
        // ECX
        const SSE3 = 1 << 0;
        const PCLMUL = 1 << 1;
        const DTES64 = 1 << 2;
        const MONITOR = 1 << 3;
        const DS_CPL = 1 << 4;
        const VMX = 1 << 5;
        const SMX = 1 << 6;
        const EST = 1 << 7;
        const TM2 = 1 << 8;
        const SSSE3 = 1 << 9;
        const CID = 1 << 10;
        const SDBG = 1 << 11;
        const FMA = 1 << 12;
        const CX16 = 1 << 13;
        const XTPR = 1 << 14;
        const PDCM = 1 << 15;
        const PCID = 1 << 17;
        const DCA = 1 << 18;
        const SSE4_1 = 1 << 19;
        const SSE4_2 = 1 << 20;
        const X2APIC = 1 << 21;
        const MOVBE = 1 << 22;
        const POPCNT = 1 << 23;
        const TSC1 = 1 << 24;
        const AES = 1 << 25;
        const XSAVE = 1 << 26;
        const OSXSAVE = 1 << 27;
        const AVX = 1 << 28;
        const F16C = 1 << 29;
        const RDRAND = 1 << 30;
        const HYPERVISOR = 1 << 31;
        // EDX
        const FPU = 1 << 0 << 32;
        const VME = 1 << 1 << 32;
        const DE = 1 << 2 << 32;
        const PSE = 1 << 3 << 32;
        const TSC2 = 1 << 4 << 32;
        const MSR = 1 << 5 << 32;
        const PAE = 1 << 6 << 32;
        const MCE = 1 << 7 << 32;
        const CX8 = 1 << 8 << 32;
        const APIC = 1 << 9 << 32;
        const SEP = 1 << 11 << 32;
        const MTRR = 1 << 12 << 32;
        const PGE = 1 << 13 << 32;
        const MCA = 1 << 14 << 32;
        const CMOV = 1 << 15 << 32;
        const PAT = 1 << 16 << 32;
        const PSE36 = 1 << 17 << 32;
        const PSN = 1 << 18 << 32;
        const CLFLUSH = 1 << 19 << 32;
        const DS = 1 << 21 << 32;
        const ACPI = 1 << 22 << 32;
        const MMX = 1 << 23 << 32;
        const FXSR = 1 << 24 << 32;
        const SSE = 1 << 25 << 32;
        const SSE2 = 1 << 26 << 32;
        const SS = 1 << 27 << 32;
        const HTT = 1 << 28 << 32;
        const TM = 1 << 29 << 32;
        const IA64 = 1 << 30 << 32;
        const PBE = 1 << 31 << 32;
    }
}

struct Cpu {
    vendor_string: [u8; 12],
    features: Flags,
}

impl Cpu {
    const fn new() -> Self {
        Cpu {
            vendor_string: [0; 12],
            features: Flags::empty(),
        }
    }
}

static mut CPU: Cpu = Cpu::new();

pub fn has_feature(feat: Flags) -> bool {
    unsafe { CPU.features.contains(feat) }
}
pub fn vendor() -> &'static str {
    unsafe { core::str::from_utf8(&CPU.vendor_string).unwrap() }
}

pub fn cpuid_fetch() {
    unsafe {
        if cpuid_supported() != 0 {
            // Read the vendor string
            asm!(
                "cpuid",
                "mov [edi], ebx",
                "mov [edi + 4], edx",
                "mov [edi + 8], ecx",
                in("rdi") CPU.vendor_string.as_ptr(),
                inout("eax") 0 => _,
                out("ebx") _,
                out("ecx") _,
                out("edx") _,
            );
            // Read the features
            let ecx: u32;
            let edx: u32;
            asm!(
                "cpuid",
                "mov {0}, edx",
                "mov {1}, ecx",
                out(reg) edx,
                out(reg) ecx,
                inout("eax") 1 => _,
                out("ebx") _,
                out("ecx") _,
                out("edx") _,
            );
            // TODO check for newer CPU to know if ECX has bogus info
            CPU.features = Flags::from_bits(((edx as u64) << 32) | ecx as u64).unwrap();
        } else {
            panic!("CPUID not supported !");
        }
    }
}
