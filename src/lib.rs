// SPDX-FileCopyrightText: Copyright 2025 Arm Limited and/or its affiliates <open-source-office@arm.com>
// SPDX-License-Identifier: MIT OR Apache-2.0

#![no_std]
#![doc = include_str!("../README.md")]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(unsafe_op_in_unsafe_fn)]

use core::fmt::Debug;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use thiserror::Error;

/// Internal error type of the PSCI module
#[derive(Debug, Error)]
pub enum Error {
    #[error("Unrecognised PSCI function ID {0}")]
    UnrecognisedFunctionId(u32),
    #[error("Unrecognised PSCI error code {0}")]
    UnrecognisedErrorCode(i32),
    #[error("Invalid PSCI version {0}")]
    InvalidVersion(u32),
    #[error("Invalid power state value {0}")]
    InvalidPowerState(u32),
    #[error("Invalid 32 bit CPU MPIDR value {0}")]
    InvalidMpidr32(u32),
    #[error("Invalid 64 bit CPU MPIDR value {0}")]
    InvalidMpidr64(u64),
    #[error("Unrecognised SYSTEM_OFF2 type {0}")]
    UnrecognisedSystemOff2Type(u32),
    #[error("Unrecognised SYSTEM_RESET2 type {0}")]
    UnrecognisedSystemReset2Type(u32),
    #[error("Unrecognised PSCI_FEATURES flags {0}")]
    UnrecognisedPsciFeaturesFlags(u32),
    #[error("Unrecognised NODE_HW_STATE {0}")]
    UnrecognisedHwState(u32),
    #[error("Unrecognised PSCI_SET_SUSPEND_MODE mode {0}")]
    UnrecognisedSuspendMode(u32),
    #[error("Invalid lower affinity level {0}")]
    InvalidLowerAffinityLevel(u32),
}

impl From<Error> for ReturnCode {
    fn from(value: Error) -> Self {
        match value {
            Error::UnrecognisedFunctionId(_) => Self::NotSupported,
            _ => Self::InvalidParameters,
        }
    }
}

/// 5.1 Function prototypes
#[derive(Clone, Copy, Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[num_enum(error_type(name = Error, constructor = Error::UnrecognisedFunctionId))]
#[repr(u32)]
pub enum FunctionId {
    PsciVersion = 0x84000000,
    CpuSuspend32 = 0x84000001,
    CpuSuspend64 = 0xc4000001,
    CpuOff = 0x84000002,
    CpuOn32 = 0x84000003,
    CpuOn64 = 0xc4000003,
    AffinityInfo32 = 0x84000004,
    AffinityInfo64 = 0xc4000004,
    Migrate32 = 0x84000005,
    Migrate64 = 0xc4000005,
    MigrateInfoType = 0x84000006,
    MigrateInfoUpCpu32 = 0x84000007,
    MigrateInfoUpCpu64 = 0xc4000007,
    SystemOff = 0x84000008,
    SystemOff232 = 0x84000015,
    SystemOff264 = 0xc4000015,
    SystemReset = 0x84000009,
    SystemReset232 = 0x84000012,
    SystemReset264 = 0xc4000012,
    MemProtect = 0x84000013,
    MemProtectCheckRange32 = 0x84000014,
    MemProtectCheckRange64 = 0xc4000014,
    PsciFeatures = 0x8400000a,
    CpuFreeze = 0x8400000b,
    CpuDefaultSuspend32 = 0x8400000c,
    CpuDefaultSuspend64 = 0xc400000c,
    NodeHwState32 = 0x8400000d,
    NodeHwState64 = 0xc400000d,
    SystemSuspend32 = 0x8400000e,
    SystemSuspend64 = 0xc400000e,
    PsciSetSuspendMode = 0x8400000f,
    PsciStatResidency32 = 0x84000010,
    PsciStatResidency64 = 0xc4000010,
    PsciStatCount32 = 0x84000011,
    PsciStatCount64 = 0xc4000011,
}

/// Table 5 Return error codes
#[derive(Clone, Copy, Debug, Eq, Error, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[num_enum(error_type(name = Error, constructor = Error::UnrecognisedErrorCode))]
#[repr(i32)]
pub enum ReturnCode {
    #[error("Not supported")]
    NotSupported = -1,
    #[error("Invalid parameters")]
    InvalidParameters = -2,
    #[error("Denied")]
    Denied = -3,
    #[error("Already on")]
    AlreadyOn = -4,
    #[error("On pending")]
    OnPending = -5,
    #[error("Internal failure")]
    InternalFailure = -6,
    #[error("Not present")]
    NotPresent = -7,
    #[error("Disabled")]
    Disabled = -8,
    #[error("Invalid address")]
    InvalidAddress = -9,
}

/// Structure for describing PSCI major and minor version
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
}

impl TryFrom<u32> for Version {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        const MBZ_BITS: u32 = 0x8000_0000;

        if value & MBZ_BITS != 0 {
            return Err(Error::InvalidVersion(value));
        }

        Ok(Self {
            major: (value >> 16) as u16,
            minor: value as u16,
        })
    }
}

impl From<Version> for u32 {
    fn from(value: Version) -> Self {
        (value.major as u32) << 16 | value.minor as u32
    }
}

/// 5.4.2 CPU_SUSPEND parameters: power_state / StateType
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PowerStateType {
    StandbyOrRetention,
    PowerOff,
}

/// Table 8 power_state parameter bit fields in Extended StateID format
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct PowerState {
    state_type: PowerStateType,
    state_id: u32,
}

impl TryFrom<u32> for PowerState {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        const MBZ_BITS: u32 = 0xb000_0000;
        if value & MBZ_BITS != 0 {
            return Err(Error::InvalidPowerState(value));
        }

        Ok(Self {
            state_type: if (value & 0x4000_0000) == 0 {
                PowerStateType::StandbyOrRetention
            } else {
                PowerStateType::PowerOff
            },
            state_id: value & 0x0fff_ffff,
        })
    }
}

impl From<PowerState> for u32 {
    fn from(value: PowerState) -> Self {
        assert_eq!(value.state_id & 0xf000_0000, 0x0000_0000);

        (match value.state_type {
            PowerStateType::StandbyOrRetention => 0,
            PowerStateType::PowerOff => 1,
        } << 30)
            | value.state_id
    }
}

/// Entry point descriptor
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum EntryPoint {
    Entry32 {
        entry_point_address: u32,
        context_id: u32,
    },
    Entry64 {
        entry_point_address: u64,
        context_id: u64,
    },
}

/// The type contains the affinity fields of the MPIDR register
/// For AArch32 callers this contains Affinity 0, 1, 2 fields and for AAarch64 callers it has
/// Affinity 0, 1, 2, 3 fields.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Mpidr {
    aff0: u8,
    aff1: u8,
    aff2: u8,
    aff3: Option<u8>,
}

impl TryFrom<u32> for Mpidr {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        const MBZ_BITS: u32 = 0xff00_0000;

        if value & MBZ_BITS != 0 {
            return Err(Error::InvalidMpidr32(value));
        }

        Ok(Self {
            aff0: value as u8,
            aff1: (value >> 8) as u8,
            aff2: (value >> 16) as u8,
            aff3: None,
        })
    }
}

impl TryFrom<u64> for Mpidr {
    type Error = Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        const MBZ_BITS: u64 = 0xffff_ff00_ff00_0000;

        if value & MBZ_BITS != 0 {
            return Err(Error::InvalidMpidr64(value));
        }

        Ok(Self {
            aff0: value as u8,
            aff1: (value >> 8) as u8,
            aff2: (value >> 16) as u8,
            aff3: Some((value >> 32) as u8),
        })
    }
}

impl From<Mpidr> for u32 {
    fn from(value: Mpidr) -> Self {
        assert_eq!(value.aff3, None);
        (value.aff2 as u32) << 16 | (value.aff1 as u32) << 8 | value.aff0 as u32
    }
}

impl From<Mpidr> for u64 {
    fn from(value: Mpidr) -> Self {
        assert!(value.aff3.is_some());

        (value.aff3.unwrap() as u64) << 32
            | (value.aff2 as u64) << 16
            | (value.aff1 as u64) << 8
            | value.aff0 as u64
    }
}

/// 5.1.5 AFFINITY_INFO return value
#[derive(Clone, Copy, Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[repr(u32)]
pub enum AffinityInfo {
    On = 0,
    Off = 1,
    OnPending = 2,
}

/// 5.1.8 MIGRATE_INFO_UP_CPU return value
#[derive(Clone, Copy, Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[repr(u32)]
pub enum MigrateInfoType {
    MigrateCapable = 0,
    NotMigrateCapable = 1,
    MigrationNotRequired = 2,
}

/// 5.1.10 SYSTEM_OFF2 type field
#[derive(Clone, Copy, Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[num_enum(error_type(name = Error, constructor = Error::UnrecognisedSystemOff2Type))]
#[repr(u32)]
pub enum SystemOff2Type {
    HibernateOff = 0x00000001,
}

/// Additional off/reset parameter
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cookie {
    Cookie32(u32),
    Cookie64(u64),
}

/// 5.1.12 SYSTEM_RESET2 architectural reset type
#[derive(Clone, Copy, Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[num_enum(error_type(name = Error, constructor = Error::UnrecognisedSystemReset2Type))]
#[repr(u32)]
pub enum ArchitecturalResetType {
    SystemWarmReset = 0x00000000,
}

/// 5.1.12 SYSTEM_RESET2 reset type
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResetType {
    Architectural(ArchitecturalResetType),
    VendorSpecific(u32),
}

impl TryFrom<u32> for ResetType {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        const VENDOR_SPECIFIC_BIT: u32 = 0x8000_0000;

        let reset_type = if value & VENDOR_SPECIFIC_BIT == 0 {
            Self::Architectural(value.try_into()?)
        } else {
            Self::VendorSpecific(value)
        };

        Ok(reset_type)
    }
}

impl From<ResetType> for u32 {
    fn from(value: ResetType) -> Self {
        match value {
            ResetType::Architectural(architectural_reset_type) => architectural_reset_type.into(),
            ResetType::VendorSpecific(vendor_specific_type) => vendor_specific_type,
        }
    }
}

/// 5.1.14 MEM_PROTECT_CHECK_RANGE memory range descriptor
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum MemProtectRange {
    Range32 { base: u32, length: u32 },
    Range64 { base: u64, length: u64 },
}

/// 5.1.15 PSCI_FEATURES psci_func_id field
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PsciFeature {
    PsciFunction(FunctionId),
    SmcccVersion,
}

impl PsciFeature {
    const SMCCC_VERSION: u32 = 0x8000_0000;
}

impl TryFrom<u32> for PsciFeature {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value == Self::SMCCC_VERSION {
            Ok(Self::SmcccVersion)
        } else {
            Ok(Self::PsciFunction(value.try_into()?))
        }
    }
}

impl From<PsciFeature> for u32 {
    fn from(value: PsciFeature) -> u32 {
        match value {
            PsciFeature::PsciFunction(function_id) => function_id.into(),
            PsciFeature::SmcccVersion => PsciFeature::SMCCC_VERSION,
        }
    }
}

/// Table 11 Return values if a function is implemented / CPU_SUSPEND
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct FeatureFlagsCpuSuspend {
    extended_power_state: bool,
    os_initiated_mode_supported: bool,
}

impl TryFrom<u32> for FeatureFlagsCpuSuspend {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        const MBZ_BITS: u32 = 0xffff_fffc;

        if value & MBZ_BITS != 0 {
            return Err(Error::UnrecognisedPsciFeaturesFlags(value));
        }

        Ok(Self {
            extended_power_state: value & 0x0000_0002 != 0x0000_0000,
            os_initiated_mode_supported: value & 0x0000_0001 != 0x0000_0000,
        })
    }
}

/// Table 11 Return values if a function is implemented / SYSTEM_OFF2
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct FeatureFlagsSystemOff2 {
    hibernate_off: bool,
}

impl TryFrom<u32> for FeatureFlagsSystemOff2 {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        const MBZ_BITS: u32 = 0xffff_fffe;

        if value & MBZ_BITS != 0 {
            return Err(Error::UnrecognisedPsciFeaturesFlags(value));
        }

        Ok(Self {
            hibernate_off: value & 0x0000_0001 != 0,
        })
    }
}

/// 5.1.18 NODE_HW_STATE return value
#[derive(Clone, Copy, Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[num_enum(error_type(name = Error, constructor = Error::UnrecognisedHwState))]
#[repr(u32)]
pub enum HwState {
    On = 0,
    Off = 1,
    Standby = 2,
}

/// 5.1.20 PSCI_SET_SUSPEND_MODE mode field
#[derive(Clone, Copy, Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[num_enum(error_type(name = Error, constructor = Error::UnrecognisedSuspendMode))]
#[repr(u32)]
pub enum SuspendMode {
    PlatformCoordinated = 0,
    OsInitiated = 1,
}

/// Enum for representing PSCI requests and their arguments
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Function {
    Version,
    CpuSuspend {
        state: PowerState,
        entry: EntryPoint,
    },
    CpuOff,
    CpuOn {
        target_cpu: Mpidr,
        entry: EntryPoint,
    },
    AffinityInfo {
        mpidr: Mpidr,
        lowest_affinity_level: u32,
    },
    Migrate {
        target_affinity: Mpidr,
    },
    MigrateInfoType,
    MigrateInfoUpCpu {
        is_32bit: bool,
    },
    SystemOff,
    SystemOff2 {
        off_type: SystemOff2Type,
        cookie: Cookie,
    },
    SystemReset,
    SystemReset2 {
        reset_type: ResetType,
        cookie: Cookie,
    },
    MemProtect {
        enabled: bool,
    },
    MemProtectCheckRange {
        range: MemProtectRange,
    },
    Features {
        psci_func_id: PsciFeature,
    },
    CpuFreeze,
    CpuDefaultSuspend {
        entry: EntryPoint,
    },
    NodeHwState {
        target_cpu: Mpidr,
        power_level: u32,
    },
    SystemSuspend {
        entry: EntryPoint,
    },
    SetSuspendMode {
        mode: SuspendMode,
    },
    StatResidency {
        target_cpu: Mpidr,
        power_state: PowerState,
    },
    StatCount {
        target_cpu: Mpidr,
        power_state: PowerState,
    },
}

impl TryFrom<[u64; 4]> for Function {
    type Error = Error;

    fn try_from(regs: [u64; 4]) -> Result<Self, Error> {
        let fid = FunctionId::try_from(regs[0] as u32)?;

        let msg = match fid {
            FunctionId::PsciVersion => Self::Version,
            FunctionId::CpuSuspend32 => Self::CpuSuspend {
                state: PowerState::try_from(regs[1] as u32)?,
                entry: EntryPoint::Entry32 {
                    entry_point_address: regs[2] as u32,
                    context_id: regs[3] as u32,
                },
            },
            FunctionId::CpuSuspend64 => Self::CpuSuspend {
                state: PowerState::try_from(regs[1] as u32)?,
                entry: EntryPoint::Entry64 {
                    entry_point_address: regs[2],
                    context_id: regs[3],
                },
            },
            FunctionId::CpuOff => Self::CpuOff,
            FunctionId::CpuOn32 => Self::CpuOn {
                target_cpu: (regs[1] as u32).try_into()?,
                entry: EntryPoint::Entry32 {
                    entry_point_address: regs[2] as u32,
                    context_id: regs[3] as u32,
                },
            },
            FunctionId::CpuOn64 => Self::CpuOn {
                target_cpu: regs[1].try_into()?,
                entry: EntryPoint::Entry64 {
                    entry_point_address: regs[2],
                    context_id: regs[3],
                },
            },
            FunctionId::AffinityInfo32 => {
                let lowest_affinity_level = regs[2] as u32;
                if lowest_affinity_level > 2 {
                    return Err(Error::InvalidLowerAffinityLevel(lowest_affinity_level));
                }
                Self::AffinityInfo {
                    mpidr: (regs[1] as u32).try_into()?,
                    lowest_affinity_level,
                }
            }
            FunctionId::AffinityInfo64 => {
                let lowest_affinity_level = regs[2] as u32;
                if lowest_affinity_level > 3 {
                    return Err(Error::InvalidLowerAffinityLevel(lowest_affinity_level));
                }
                Self::AffinityInfo {
                    mpidr: regs[1].try_into()?,
                    lowest_affinity_level,
                }
            }
            FunctionId::Migrate32 => Self::Migrate {
                target_affinity: (regs[1] as u32).try_into()?,
            },
            FunctionId::Migrate64 => Self::Migrate {
                target_affinity: regs[1].try_into()?,
            },
            FunctionId::MigrateInfoType => Self::MigrateInfoType,
            FunctionId::MigrateInfoUpCpu32 => Self::MigrateInfoUpCpu { is_32bit: true },
            FunctionId::MigrateInfoUpCpu64 => Self::MigrateInfoUpCpu { is_32bit: false },
            FunctionId::SystemOff => Self::SystemOff,
            FunctionId::SystemOff232 => Self::SystemOff2 {
                off_type: SystemOff2Type::try_from_primitive(regs[1] as u32)?,
                cookie: Cookie::Cookie32(regs[2] as u32),
            },
            FunctionId::SystemOff264 => Self::SystemOff2 {
                off_type: SystemOff2Type::try_from_primitive(regs[1] as u32)?,
                cookie: Cookie::Cookie64(regs[2]),
            },
            FunctionId::SystemReset => Self::SystemReset,
            FunctionId::SystemReset232 => Self::SystemReset2 {
                reset_type: (regs[1] as u32).try_into()?,
                cookie: Cookie::Cookie32(regs[2] as u32),
            },
            FunctionId::SystemReset264 => Self::SystemReset2 {
                reset_type: (regs[1] as u32).try_into()?,
                cookie: Cookie::Cookie64(regs[2]),
            },
            FunctionId::MemProtect => Self::MemProtect {
                enabled: regs[1] != 0,
            },
            FunctionId::MemProtectCheckRange32 => Self::MemProtectCheckRange {
                range: MemProtectRange::Range32 {
                    base: regs[1] as u32,
                    length: regs[2] as u32,
                },
            },
            FunctionId::MemProtectCheckRange64 => Self::MemProtectCheckRange {
                range: MemProtectRange::Range64 {
                    base: regs[1],
                    length: regs[2],
                },
            },
            FunctionId::PsciFeatures => Self::Features {
                psci_func_id: (regs[1] as u32).try_into()?,
            },
            FunctionId::CpuFreeze => Self::CpuFreeze,
            FunctionId::CpuDefaultSuspend32 => Self::CpuDefaultSuspend {
                entry: EntryPoint::Entry32 {
                    entry_point_address: regs[1] as u32,
                    context_id: regs[2] as u32,
                },
            },
            FunctionId::CpuDefaultSuspend64 => Self::CpuDefaultSuspend {
                entry: EntryPoint::Entry64 {
                    entry_point_address: regs[1],
                    context_id: regs[2],
                },
            },
            FunctionId::NodeHwState32 => Self::NodeHwState {
                target_cpu: (regs[1] as u32).try_into()?,
                power_level: regs[2] as u32,
            },
            FunctionId::NodeHwState64 => Self::NodeHwState {
                target_cpu: regs[1].try_into()?,
                power_level: regs[2] as u32,
            },
            FunctionId::SystemSuspend32 => Self::SystemSuspend {
                entry: EntryPoint::Entry32 {
                    entry_point_address: regs[1] as u32,
                    context_id: regs[2] as u32,
                },
            },
            FunctionId::SystemSuspend64 => Self::SystemSuspend {
                entry: EntryPoint::Entry64 {
                    entry_point_address: regs[1],
                    context_id: regs[2],
                },
            },
            FunctionId::PsciSetSuspendMode => Self::SetSuspendMode {
                mode: SuspendMode::try_from_primitive(regs[1] as u32)?,
            },
            FunctionId::PsciStatResidency32 => Self::StatResidency {
                target_cpu: (regs[1] as u32).try_into()?,
                power_state: PowerState::try_from(regs[2] as u32)?,
            },
            FunctionId::PsciStatResidency64 => Self::StatResidency {
                target_cpu: regs[1].try_into()?,
                power_state: PowerState::try_from(regs[2] as u32)?,
            },
            FunctionId::PsciStatCount32 => Self::StatCount {
                target_cpu: (regs[1] as u32).try_into()?,
                power_state: PowerState::try_from(regs[2] as u32)?,
            },
            FunctionId::PsciStatCount64 => Self::StatCount {
                target_cpu: regs[1].try_into()?,
                power_state: PowerState::try_from(regs[2] as u32)?,
            },
        };

        Ok(msg)
    }
}

impl Function {
    /// Returns the function ID for the call, if it has one.
    pub fn function_id(&self) -> FunctionId {
        match self {
            Function::Version => FunctionId::PsciVersion,
            Function::CpuSuspend {
                entry: EntryPoint::Entry32 { .. },
                ..
            } => FunctionId::CpuSuspend32,
            Function::CpuSuspend {
                entry: EntryPoint::Entry64 { .. },
                ..
            } => FunctionId::CpuSuspend64,
            Function::CpuOff => FunctionId::CpuOff,
            Function::CpuOn {
                target_cpu: Mpidr { aff3: None, .. },
                entry: EntryPoint::Entry32 { .. },
            } => FunctionId::CpuOn32,
            Function::CpuOn {
                target_cpu: Mpidr { aff3: Some(_), .. },
                entry: EntryPoint::Entry64 { .. },
            } => FunctionId::CpuOn32,
            Function::CpuOn { .. } => panic!("Mixed 32 bit and 64 bit CpuOn arguments"),
            Function::AffinityInfo {
                mpidr: Mpidr { aff3: None, .. },
                ..
            } => FunctionId::AffinityInfo32,
            Function::AffinityInfo {
                mpidr: Mpidr { aff3: Some(_), .. },
                ..
            } => FunctionId::AffinityInfo64,
            Function::Migrate {
                target_affinity: Mpidr { aff3: None, .. },
            } => FunctionId::Migrate32,
            Function::Migrate {
                target_affinity: Mpidr { aff3: Some(_), .. },
            } => FunctionId::Migrate64,
            Function::MigrateInfoType => FunctionId::MigrateInfoType,
            Function::MigrateInfoUpCpu { is_32bit: true } => FunctionId::MigrateInfoUpCpu32,
            Function::MigrateInfoUpCpu { is_32bit: false } => FunctionId::MigrateInfoUpCpu64,
            Function::SystemOff => FunctionId::SystemOff,
            Function::SystemOff2 {
                cookie: Cookie::Cookie32(_),
                ..
            } => FunctionId::SystemOff232,
            Function::SystemOff2 {
                cookie: Cookie::Cookie64(_),
                ..
            } => FunctionId::SystemOff264,
            Function::SystemReset => FunctionId::SystemReset,
            Function::SystemReset2 {
                cookie: Cookie::Cookie32(_),
                ..
            } => FunctionId::SystemReset232,
            Function::SystemReset2 {
                cookie: Cookie::Cookie64(_),
                ..
            } => FunctionId::SystemReset264,
            Function::MemProtect { .. } => FunctionId::MemProtect,
            Function::MemProtectCheckRange {
                range: MemProtectRange::Range32 { .. },
            } => FunctionId::MemProtectCheckRange32,
            Function::MemProtectCheckRange {
                range: MemProtectRange::Range64 { .. },
            } => FunctionId::MemProtectCheckRange64,
            Function::Features { .. } => FunctionId::PsciFeatures,
            Function::CpuFreeze => FunctionId::CpuFreeze,
            Function::CpuDefaultSuspend {
                entry: EntryPoint::Entry32 { .. },
            } => FunctionId::CpuDefaultSuspend32,
            Function::CpuDefaultSuspend {
                entry: EntryPoint::Entry64 { .. },
            } => FunctionId::CpuDefaultSuspend64,
            Function::NodeHwState {
                target_cpu: Mpidr { aff3: None, .. },
                ..
            } => FunctionId::NodeHwState32,
            Function::NodeHwState {
                target_cpu: Mpidr { aff3: Some(_), .. },
                ..
            } => FunctionId::NodeHwState64,
            Function::SystemSuspend {
                entry: EntryPoint::Entry32 { .. },
            } => FunctionId::SystemSuspend32,
            Function::SystemSuspend {
                entry: EntryPoint::Entry64 { .. },
            } => FunctionId::SystemSuspend64,
            Function::SetSuspendMode { .. } => FunctionId::PsciSetSuspendMode,
            Function::StatResidency {
                target_cpu: Mpidr { aff3: None, .. },
                ..
            } => FunctionId::PsciStatResidency32,
            Function::StatResidency {
                target_cpu: Mpidr { aff3: Some(_), .. },
                ..
            } => FunctionId::PsciStatResidency64,
            Function::StatCount {
                target_cpu: Mpidr { aff3: None, .. },
                ..
            } => FunctionId::PsciStatCount32,
            Function::StatCount {
                target_cpu: Mpidr { aff3: Some(_), .. },
                ..
            } => FunctionId::PsciStatCount64,
        }
    }

    pub fn copy_to_array(&self, a: &mut [u64; 4]) {
        a.fill(0);
        a[0] = u32::from(self.function_id()).into();

        match *self {
            Function::Version
            | Function::CpuOff
            | Function::MigrateInfoType
            | Function::MigrateInfoUpCpu { .. }
            | Function::SystemOff
            | Function::SystemReset
            | Function::CpuFreeze => {}
            Function::CpuSuspend { state, entry } => {
                a[1] = u32::from(state).into();
                (a[2], a[3]) = match entry {
                    EntryPoint::Entry32 {
                        entry_point_address,
                        context_id,
                    } => (entry_point_address.into(), context_id.into()),
                    EntryPoint::Entry64 {
                        entry_point_address,
                        context_id,
                    } => (entry_point_address, context_id),
                }
            }
            Function::CpuOn { target_cpu, entry } => {
                a[1] = match target_cpu {
                    Mpidr { aff3: None, .. } => u32::from(target_cpu).into(),
                    Mpidr { aff3: Some(_), .. } => u64::from(target_cpu),
                };
                (a[2], a[3]) = match entry {
                    EntryPoint::Entry32 {
                        entry_point_address,
                        context_id,
                    } => (entry_point_address.into(), context_id.into()),
                    EntryPoint::Entry64 {
                        entry_point_address,
                        context_id,
                    } => (entry_point_address, context_id),
                }
            }
            Function::AffinityInfo {
                mpidr,
                lowest_affinity_level,
            } => {
                a[1] = match mpidr {
                    Mpidr { aff3: None, .. } => u32::from(mpidr).into(),
                    Mpidr { aff3: Some(_), .. } => u64::from(mpidr),
                };
                a[2] = lowest_affinity_level.into();
            }
            Function::Migrate { target_affinity } => {
                a[1] = match target_affinity {
                    Mpidr { aff3: None, .. } => u32::from(target_affinity).into(),
                    Mpidr { aff3: Some(_), .. } => u64::from(target_affinity),
                };
            }
            Function::SystemOff2 {
                off_type: hibernate_type,
                cookie,
            } => {
                a[1] = u32::from(hibernate_type).into();
                a[2] = match cookie {
                    Cookie::Cookie32(value) => value.into(),
                    Cookie::Cookie64(value) => value,
                };
            }
            Function::SystemReset2 { reset_type, cookie } => {
                a[1] = u32::from(reset_type).into();
                a[2] = match cookie {
                    Cookie::Cookie32(value) => value.into(),
                    Cookie::Cookie64(value) => value,
                };
            }
            Function::MemProtect { enabled } => {
                a[1] = if enabled { 0x0000_0001 } else { 0x0000_0000 };
            }
            Function::MemProtectCheckRange { range } => {
                (a[1], a[2]) = match range {
                    MemProtectRange::Range32 { base, length } => (base.into(), length.into()),
                    MemProtectRange::Range64 { base, length } => (base, length),
                }
            }
            Function::Features { psci_func_id } => {
                a[1] = u32::from(psci_func_id).into();
            }
            Function::CpuDefaultSuspend { entry } => {
                (a[1], a[2]) = match entry {
                    EntryPoint::Entry32 {
                        entry_point_address,
                        context_id,
                    } => (entry_point_address.into(), context_id.into()),
                    EntryPoint::Entry64 {
                        entry_point_address,
                        context_id,
                    } => (entry_point_address, context_id),
                }
            }
            Function::NodeHwState {
                target_cpu,
                power_level,
            } => {
                a[1] = match target_cpu {
                    Mpidr { aff3: None, .. } => u32::from(target_cpu).into(),
                    Mpidr { aff3: Some(_), .. } => u64::from(target_cpu),
                };
                a[2] = power_level.into();
            }
            Function::SystemSuspend { entry } => {
                (a[1], a[2]) = match entry {
                    EntryPoint::Entry32 {
                        entry_point_address,
                        context_id,
                    } => (entry_point_address.into(), context_id.into()),
                    EntryPoint::Entry64 {
                        entry_point_address,
                        context_id,
                    } => (entry_point_address, context_id),
                }
            }
            Function::SetSuspendMode { mode } => {
                a[1] = u32::from(mode).into();
            }
            Function::StatResidency {
                target_cpu,
                power_state,
            } => {
                a[1] = match target_cpu {
                    Mpidr { aff3: None, .. } => u32::from(target_cpu).into(),
                    Mpidr { aff3: Some(_), .. } => u64::from(target_cpu),
                };
                a[2] = u32::from(power_state).into();
            }
            Function::StatCount {
                target_cpu,
                power_state,
            } => {
                a[1] = match target_cpu {
                    Mpidr { aff3: None, .. } => u32::from(target_cpu).into(),
                    Mpidr { aff3: Some(_), .. } => u64::from(target_cpu),
                };
                a[2] = u32::from(power_state).into();
            }
        }
    }
}
