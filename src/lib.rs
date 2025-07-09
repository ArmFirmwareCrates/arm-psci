// SPDX-FileCopyrightText: Copyright The arm-psci Contributors.
// SPDX-License-Identifier: MIT OR Apache-2.0

#![no_std]
#![doc = include_str!("../README.md")]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(unsafe_op_in_unsafe_fn)]

//! # Specification
//!
//! This implementation is based on
//! [Arm Power State Coordination Interface](https://developer.arm.com/documentation/den0022/latest/)
//! Platform Design Document Version 1.3 issue F.b. (DEN0022).
//!
//! The type descriptions below refer to sections of this particular version of the specification.

use bitflags::bitflags;
use core::fmt::Debug;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use thiserror::Error;

/// Internal error type of the PSCI module
#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("unrecognised PSCI function ID {0}")]
    UnrecognisedFunctionId(u32),
    #[error("unrecognised PSCI error code {0}")]
    UnrecognisedErrorCode(i32),
    #[error("overflowing PSCI error code {0}")]
    OverflowingErrorCode(i64),
    #[error("invalid PSCI version {0}")]
    InvalidVersion(u32),
    #[error("invalid power state value {0}")]
    InvalidPowerState(u32),
    #[error("invalid 32 bit CPU MPIDR value {0}")]
    InvalidMpidr32(u32),
    #[error("invalid 64 bit CPU MPIDR value {0}")]
    InvalidMpidr64(u64),
    #[error("unrecognised SYSTEM_OFF2 type {0}")]
    UnrecognisedSystemOff2Type(u32),
    #[error("unrecognised SYSTEM_RESET2 type {0}")]
    UnrecognisedSystemReset2Type(u32),
    #[error("unrecognised PSCI_FEATURES flags {0}")]
    UnrecognisedPsciFeaturesFlags(u32),
    #[error("unrecognised NODE_HW_STATE {0}")]
    UnrecognisedHwState(u32),
    #[error("unrecognised PSCI_SET_SUSPEND_MODE mode {0}")]
    UnrecognisedSuspendMode(u32),
    #[error("invalid lower affinity level {0}")]
    InvalidLowerAffinityLevel(u32),
    #[error("ignored non-zero aff3 when converting to u32 {0}")]
    IgnoredNonZeroAff3(u8),
}

impl From<Error> for ErrorCode {
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

/// Composite type for capturing success and error return codes.
/// See Table 5 Return error codes
///
/// Clients can use `ReturnCode` to parse the result register value of a PSCI calls and easily
/// determine if it was a success or an error.
///
/// Error codes are handled by the `ErrorCode` type. Having a separate type for errors helps using
/// `Result<(), ErrorCode>`. If a single type would include both success and error values,
/// then `Err(ErrorCode::Success)` would be incomprehensible.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReturnCode {
    Success,
    Error(ErrorCode),
}

impl TryFrom<i32> for ReturnCode {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self, <Self as TryFrom<i32>>::Error> {
        Ok(match value {
            0 => Self::Success,
            error_code => Self::Error(ErrorCode::try_from(error_code)?),
        })
    }
}

impl From<ReturnCode> for i32 {
    fn from(value: ReturnCode) -> Self {
        match value {
            ReturnCode::Success => 0,
            ReturnCode::Error(error_code) => error_code.into(),
        }
    }
}

/// Error codes
/// See Table 5 Return error codes
#[derive(Clone, Copy, Debug, Eq, Error, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[num_enum(error_type(name = Error, constructor = Error::UnrecognisedErrorCode))]
#[repr(i32)]
pub enum ErrorCode {
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

impl TryFrom<u32> for ErrorCode {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::try_from_primitive(value as i32)
    }
}

impl TryFrom<u64> for ErrorCode {
    type Error = Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        let signed_value =
            i32::try_from(value as i64).map_err(|_| Error::OverflowingErrorCode(value as i64))?;
        Self::try_from_primitive(signed_value)
    }
}

impl From<ErrorCode> for u32 {
    /// Converts `ErrorCode` to be suitable for returning it in a 32-bit register.
    /// See 5.2.2 Return error codes
    fn from(value: ErrorCode) -> Self {
        i32::from(value) as u32
    }
}

impl From<ErrorCode> for u64 {
    /// Converts `ErrorCode` to be suitable for returning it in a 64-bit register, i.e. it does
    /// sign extension to 64 bits.
    /// See 5.2.2 Return error codes
    fn from(value: ErrorCode) -> Self {
        i32::from(value) as u64
    }
}

/// Structure for describing PSCI major and minor version.
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
            Err(Error::InvalidVersion(value))
        } else {
            Ok(Self {
                major: (value >> 16) as u16,
                minor: value as u16,
            })
        }
    }
}

impl From<Version> for u32 {
    fn from(value: Version) -> Self {
        const MAJOR_MBZ_BITS: u16 = 0x8000;

        assert!((value.major & MAJOR_MBZ_BITS) == 0);

        ((value.major as u32) << 16) | value.minor as u32
    }
}

/// Table 8 power_state parameter bit fields in Extended StateID format.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PowerState {
    StandbyOrRetention(u32),
    PowerDown(u32),
}

impl PowerState {
    const STATE_TYPE_PD_BIT: u32 = 1 << 30;
    const STATE_ID_MASK: u32 = 0x0fff_ffff;
    const MBZ_BITS: u32 = !(Self::STATE_TYPE_PD_BIT | Self::STATE_ID_MASK);
}

impl TryFrom<u32> for PowerState {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value & Self::MBZ_BITS != 0 {
            return Err(Error::InvalidPowerState(value));
        }

        let state_id = value & Self::STATE_ID_MASK;

        Ok(if value & Self::STATE_TYPE_PD_BIT != 0 {
            Self::PowerDown(state_id)
        } else {
            Self::StandbyOrRetention(state_id)
        })
    }
}

impl From<PowerState> for u32 {
    fn from(value: PowerState) -> Self {
        let (state_type_bit, state_id) = match value {
            PowerState::StandbyOrRetention(state_id) => (0, state_id),
            PowerState::PowerDown(state_id) => (PowerState::STATE_TYPE_PD_BIT, state_id),
        };

        assert_eq!(state_id & !PowerState::STATE_ID_MASK, 0x0000_0000);

        state_type_bit | state_id
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

impl EntryPoint {
    /// Returns the entry point address.
    pub fn entry_point_address(&self) -> u64 {
        match self {
            Self::Entry32 {
                entry_point_address,
                ..
            } => (*entry_point_address).into(),
            Self::Entry64 {
                entry_point_address,
                ..
            } => *entry_point_address,
        }
    }

    /// Returns the context ID.
    pub fn context_id(&self) -> u64 {
        match self {
            Self::Entry32 { context_id, .. } => (*context_id).into(),
            Self::Entry64 { context_id, .. } => *context_id,
        }
    }
}

/// The type contains the affinity fields of the MPIDR register.
/// For AArch32 callers this contains Affinity 0, 1, 2 fields and for AAarch64 callers it has
/// Affinity 0, 1, 2, 3 fields.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Mpidr {
    pub aff0: u8,
    pub aff1: u8,
    pub aff2: u8,
    pub aff3: Option<u8>,
}

impl Mpidr {
    const MBZ_BITS_64: u64 = 0xffff_ff00_ff00_0000;

    /// Create Mpidr instance from aff3-0 values
    pub const fn from_aff3210(aff3: u8, aff2: u8, aff1: u8, aff0: u8) -> Self {
        Self {
            aff0,
            aff1,
            aff2,
            aff3: Some(aff3),
        }
    }

    /// Create Mpidr instance from aff2-0 values
    pub const fn from_aff210(aff2: u8, aff1: u8, aff0: u8) -> Self {
        Self {
            aff0,
            aff1,
            aff2,
            aff3: None,
        }
    }

    /// The `MPIDR_EL1` register contains bits other then the aff3-0 fields. However the PSCI
    /// specification request bits\[40:63\] and bits\[24:31\] to be set to zero when forwarding an
    /// MPIDR value as an argument of a PSCI function call. Because of this, the `TryFrom`
    /// implementation returns an error if these bits are set. In other cases the `Mpidr` value is
    /// constructed from the `MPIDR_EL1` register value of the local core. This function does this
    /// by ignoring other bits. Do not use this function for creating `Mpidr` instance from a PSCI
    /// function argument.
    pub fn from_register_value(mpidr_el1: u64) -> Self {
        Self::try_from(mpidr_el1 & !Self::MBZ_BITS_64).unwrap()
    }
}

impl TryFrom<u32> for Mpidr {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        const MBZ_BITS: u32 = 0xff00_0000;

        if value & MBZ_BITS != 0 {
            Err(Error::InvalidMpidr32(value))
        } else {
            Ok(Self {
                aff0: value as u8,
                aff1: (value >> 8) as u8,
                aff2: (value >> 16) as u8,
                aff3: None,
            })
        }
    }
}

impl TryFrom<u64> for Mpidr {
    type Error = Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value & Self::MBZ_BITS_64 != 0 {
            Err(Error::InvalidMpidr64(value))
        } else {
            Ok(Self {
                aff0: value as u8,
                aff1: (value >> 8) as u8,
                aff2: (value >> 16) as u8,
                aff3: Some((value >> 32) as u8),
            })
        }
    }
}

impl TryFrom<Mpidr> for u32 {
    type Error = Error;

    fn try_from(value: Mpidr) -> Result<Self, Self::Error> {
        match value.aff3 {
            // Allow converting Mpidr to u32 if aff3 is not set or set to zero.
            Some(0) | None => {
                Ok(((value.aff2 as u32) << 16) | ((value.aff1 as u32) << 8) | value.aff0 as u32)
            }
            // Aff3 is non zero, we would lose it when converting to u32.
            Some(aff3) => Err(Error::IgnoredNonZeroAff3(aff3)),
        }
    }
}

impl From<Mpidr> for u64 {
    fn from(value: Mpidr) -> Self {
        ((value.aff3.unwrap_or(0) as u64) << 32)
            | ((value.aff2 as u64) << 16)
            | ((value.aff1 as u64) << 8)
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

impl ResetType {
    const VENDOR_SPECIFIC_BIT: u32 = 0x8000_0000;
}

impl TryFrom<u32> for ResetType {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(if value & Self::VENDOR_SPECIFIC_BIT == 0 {
            Self::Architectural(value.try_into()?)
        } else {
            Self::VendorSpecific(value & !Self::VENDOR_SPECIFIC_BIT)
        })
    }
}

impl From<ResetType> for u32 {
    fn from(value: ResetType) -> Self {
        match value {
            ResetType::Architectural(architectural_reset_type) => architectural_reset_type.into(),
            ResetType::VendorSpecific(vendor_specific_type) => {
                vendor_specific_type | ResetType::VENDOR_SPECIFIC_BIT
            }
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
        Ok(if value == Self::SMCCC_VERSION {
            Self::SmcccVersion
        } else {
            Self::PsciFunction(value.try_into()?)
        })
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
#[repr(transparent)]
pub struct FeatureFlagsCpuSuspend(u32);

bitflags! {
    impl FeatureFlagsCpuSuspend : u32 {
        const EXTENDED_POWER_STATE = 0x0000_0002;
        const OS_INITIATED_MODE = 0x0000_0001;
    }
}

impl TryFrom<u32> for FeatureFlagsCpuSuspend {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::from_bits(value).ok_or(Error::UnrecognisedPsciFeaturesFlags(value))
    }
}

impl From<FeatureFlagsCpuSuspend> for u32 {
    fn from(value: FeatureFlagsCpuSuspend) -> Self {
        value.bits()
    }
}

/// Table 11 Return values if a function is implemented / SYSTEM_OFF2
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[repr(transparent)]
pub struct FeatureFlagsSystemOff2(u32);

bitflags! {
    impl FeatureFlagsSystemOff2 : u32 {
        const HIBERNATE_OFF = 0x0000_0001;
    }
}

impl TryFrom<u32> for FeatureFlagsSystemOff2 {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::from_bits(value).ok_or(Error::UnrecognisedPsciFeaturesFlags(value))
    }
}

impl From<FeatureFlagsSystemOff2> for u32 {
    fn from(value: FeatureFlagsSystemOff2) -> Self {
        value.bits()
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

/// Enum for representing PSCI requests and their arguments.
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
    /// Returns the function ID for the call.
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
            } => FunctionId::CpuOn64,
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
                a[1] = target_cpu.into();
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
                a[1] = mpidr.into();
                a[2] = lowest_affinity_level.into();
            }
            Function::Migrate { target_affinity } => {
                a[1] = target_affinity.into();
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
                a[1] = target_cpu.into();
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
                a[1] = target_cpu.into();
                a[2] = u32::from(power_state).into();
            }
            Function::StatCount {
                target_cpu,
                power_state,
            } => {
                a[1] = target_cpu.into();
                a[2] = u32::from(power_state).into();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MPIDR32: Mpidr = Mpidr {
        aff0: 0x11,
        aff1: 0x22,
        aff2: 0x33,
        aff3: None,
    };
    const MPIDR32_VALUE: u32 = 0x00332211;

    const MPIDR64: Mpidr = Mpidr {
        aff0: 0x11,
        aff1: 0x22,
        aff2: 0x33,
        aff3: Some(0x44),
    };

    const MPIDR64_VALUE: u64 = 0x00000044_00332211;

    const POWER_STATE: PowerState = PowerState::PowerDown(0xf12_3456);

    const POWER_STATE_VALUE: u32 = 0x4f12_3456;

    macro_rules! validate_function {
        ( $regs:expr, $function:expr, $function_id:expr ) => {
            assert_eq!(Ok($function), Function::try_from($regs));
            assert_eq!($function_id, ($function).function_id());

            let mut built_regs = [0; 4];
            $function.copy_to_array(&mut built_regs);
            assert_eq!($regs, built_regs);
        };
    }

    #[test]
    fn test_error() {
        assert_eq!(
            ErrorCode::NotSupported,
            ErrorCode::from(Error::UnrecognisedFunctionId(0))
        );

        assert_eq!(
            ErrorCode::InvalidParameters,
            ErrorCode::from(Error::InvalidVersion(0))
        );
    }

    #[test]
    fn test_return_code() {
        assert_eq!(
            Err(Error::UnrecognisedErrorCode(-100)),
            ReturnCode::try_from(-100)
        );

        assert_eq!(Ok(ReturnCode::Success), ReturnCode::try_from(0));
        assert_eq!(
            Ok(ReturnCode::Error(ErrorCode::NotSupported)),
            ReturnCode::try_from(-1)
        );

        assert_eq!(0, i32::from(ReturnCode::Success));
        assert_eq!(-1, i32::from(ReturnCode::Error(ErrorCode::NotSupported)));
    }

    #[test]
    fn test_error_code() {
        assert_eq!(-1, i32::from(ErrorCode::NotSupported));
        assert_eq!(-2, i32::from(ErrorCode::InvalidParameters));
        assert_eq!(-3, i32::from(ErrorCode::Denied));
        assert_eq!(-4, i32::from(ErrorCode::AlreadyOn));
        assert_eq!(-5, i32::from(ErrorCode::OnPending));
        assert_eq!(-6, i32::from(ErrorCode::InternalFailure));
        assert_eq!(-7, i32::from(ErrorCode::NotPresent));
        assert_eq!(-8, i32::from(ErrorCode::Disabled));
        assert_eq!(-9, i32::from(ErrorCode::InvalidAddress));

        assert_eq!(
            Err(Error::UnrecognisedErrorCode(0)),
            ErrorCode::try_from(0u32)
        );

        assert_eq!(
            Ok(ErrorCode::NotSupported),
            ErrorCode::try_from(0xffff_ffffu32)
        );

        assert_eq!(
            Err(Error::UnrecognisedErrorCode(0)),
            ErrorCode::try_from(0u64)
        );

        assert_eq!(
            Err(Error::OverflowingErrorCode(0x7fff_ffff_ffff_ffff)),
            ErrorCode::try_from(0x7fff_ffff_ffff_ffffu64)
        );

        assert_eq!(
            Ok(ErrorCode::NotSupported),
            ErrorCode::try_from(0xffff_ffff_ffff_ffffu64)
        );

        assert_eq!(0xffff_fffe, u32::from(ErrorCode::InvalidParameters));
        assert_eq!(
            0xffff_ffff_ffff_fffe,
            u64::from(ErrorCode::InvalidParameters)
        );
    }

    #[test]
    fn test_version() {
        assert_eq!(
            Err(Error::InvalidVersion(0xffff_ffff)),
            Version::try_from(0xffff_ffff)
        );

        assert_eq!(
            Ok(Version {
                major: 0x7123,
                minor: 0x4567
            }),
            Version::try_from(0x7123_4567)
        );

        assert_eq!(
            0x789a_bcde,
            u32::from(Version {
                major: 0x789a,
                minor: 0xbcde
            })
        );
    }

    #[test]
    fn test_power_state() {
        assert_eq!(
            Err(Error::InvalidPowerState(0x8000_0000)),
            PowerState::try_from(0x8000_0000)
        );

        assert_eq!(
            Err(Error::InvalidPowerState(0x3000_0000)),
            PowerState::try_from(0x3000_0000)
        );

        assert_eq!(
            Ok(PowerState::StandbyOrRetention(0xfff_ffff),),
            PowerState::try_from(0x0fff_ffff)
        );

        assert_eq!(
            Ok(PowerState::PowerDown(0x123_ffff)),
            PowerState::try_from(0x4123_ffff)
        );

        assert_eq!(
            0x0123_4567,
            u32::from(PowerState::StandbyOrRetention(0x123_4567))
        );

        assert_eq!(0x4123_4567, u32::from(PowerState::PowerDown(0x123_4567)));
    }

    #[test]
    fn test_entry_point() {
        let entry = EntryPoint::Entry32 {
            entry_point_address: 0x1234_5678,
            context_id: 0x9abc_def0,
        };

        assert_eq!(0x1234_5678, entry.entry_point_address());
        assert_eq!(0x9abc_def0, entry.context_id());

        let entry = EntryPoint::Entry64 {
            entry_point_address: 0x1234_5678_9abc_def0,
            context_id: 0x9abc_def0_1234_5678,
        };

        assert_eq!(0x1234_5678_9abc_def0, entry.entry_point_address());
        assert_eq!(0x9abc_def0_1234_5678, entry.context_id());
    }

    #[test]
    fn test_mpidr() {
        assert_eq!(
            Err(Error::InvalidMpidr32(0xff00_0000)),
            Mpidr::try_from(0xff00_0000u32)
        );

        assert_eq!(
            Ok(Mpidr::from_aff210(0x11, 0x22, 0x33)),
            Mpidr::try_from(0x0011_2233u32)
        );

        assert_eq!(
            Err(Error::InvalidMpidr64(0xff00_0000_0000_0000)),
            Mpidr::try_from(0xff00_0000_0000_0000u64)
        );

        assert_eq!(
            Ok(Mpidr::from_aff3210(0x44, 0x11, 0x22, 0x33)),
            Mpidr::try_from(0x0000_0044_0011_2233u64)
        );

        assert_eq!(
            Ok(0x0011_2233u32),
            Mpidr::from_aff210(0x11, 0x22, 0x33).try_into()
        );

        assert_eq!(
            0x0000_0044_0011_2233u64,
            Mpidr::from_aff3210(0x44, 0x11, 0x22, 0x33).into()
        );

        assert_eq!(
            Err(Error::IgnoredNonZeroAff3(0x44)),
            u32::try_from(Mpidr::from_aff3210(0x44, 0x11, 0x22, 0x33))
        );

        assert_eq!(
            0x0011_2233u64,
            u64::from(Mpidr::from_aff210(0x11, 0x22, 0x33))
        );

        assert_eq!(
            Mpidr::from_aff3210(0x44, 0x11, 0x22, 0x33),
            Mpidr::from_register_value(0x0000_0044_4111_2233u64)
        );
    }

    #[test]
    fn test_affinity_info_value() {
        assert_eq!(0, u32::from(AffinityInfo::On));
        assert_eq!(1, u32::from(AffinityInfo::Off));
        assert_eq!(2, u32::from(AffinityInfo::OnPending));
    }

    #[test]
    fn test_migration_info_type() {
        assert_eq!(0, u32::from(MigrateInfoType::MigrateCapable));
        assert_eq!(1, u32::from(MigrateInfoType::NotMigrateCapable));
        assert_eq!(2, u32::from(MigrateInfoType::MigrationNotRequired));
    }

    #[test]
    fn test_reset_type() {
        assert_eq!(
            Err(Error::UnrecognisedSystemReset2Type(0x1234_5678)),
            ResetType::try_from(0x1234_5678)
        );

        assert_eq!(
            Ok(ResetType::Architectural(
                ArchitecturalResetType::SystemWarmReset
            )),
            ResetType::try_from(0x0000_0000)
        );

        assert_eq!(
            Ok(ResetType::VendorSpecific(0x0000_0001)),
            ResetType::try_from(0x8000_0001)
        );

        assert_eq!(
            0x0000_0000u32,
            ResetType::Architectural(ArchitecturalResetType::SystemWarmReset).into()
        );
        assert_eq!(
            0x8000_0001u32,
            ResetType::VendorSpecific(0x0000_0001).into()
        );
    }

    #[test]
    fn test_psci_feature() {
        assert_eq!(
            Err(Error::UnrecognisedFunctionId(0x1234_5678)),
            PsciFeature::try_from(0x1234_5678)
        );

        assert_eq!(
            Ok(PsciFeature::SmcccVersion),
            PsciFeature::try_from(0x8000_0000)
        );

        assert_eq!(
            Ok(PsciFeature::PsciFunction(FunctionId::PsciVersion)),
            PsciFeature::try_from(0x8400_0000)
        );

        assert_eq!(0x8000_0000u32, PsciFeature::SmcccVersion.into());
        assert_eq!(
            0x8400_0000u32,
            PsciFeature::PsciFunction(FunctionId::PsciVersion).into()
        );
    }

    #[test]
    fn test_feature_flags_suspend() {
        assert_eq!(
            Err(Error::UnrecognisedPsciFeaturesFlags(0x0000_0004)),
            FeatureFlagsCpuSuspend::try_from(0x0000_0004)
        );

        assert_eq!(
            Ok(FeatureFlagsCpuSuspend::empty()),
            FeatureFlagsCpuSuspend::try_from(0x0000_0000)
        );

        assert_eq!(
            Ok(FeatureFlagsCpuSuspend::OS_INITIATED_MODE),
            FeatureFlagsCpuSuspend::try_from(0x0000_0001)
        );

        assert_eq!(
            Ok(FeatureFlagsCpuSuspend::EXTENDED_POWER_STATE),
            FeatureFlagsCpuSuspend::try_from(0x0000_0002)
        );

        assert_eq!(
            Ok(FeatureFlagsCpuSuspend::OS_INITIATED_MODE
                | FeatureFlagsCpuSuspend::EXTENDED_POWER_STATE),
            FeatureFlagsCpuSuspend::try_from(0x0000_0003)
        );

        assert_eq!(
            0x0000_0003,
            u32::from(
                FeatureFlagsCpuSuspend::OS_INITIATED_MODE
                    | FeatureFlagsCpuSuspend::EXTENDED_POWER_STATE
            )
        );
    }

    #[test]
    fn test_feature_flags_system_off2() {
        assert_eq!(
            Err(Error::UnrecognisedPsciFeaturesFlags(0x0000_0002)),
            FeatureFlagsSystemOff2::try_from(0x0000_0002)
        );

        assert_eq!(
            Ok(FeatureFlagsSystemOff2::empty()),
            FeatureFlagsSystemOff2::try_from(0x0000_0000)
        );

        assert_eq!(
            Ok(FeatureFlagsSystemOff2::HIBERNATE_OFF),
            FeatureFlagsSystemOff2::try_from(0x0000_0001)
        );

        assert_eq!(0x0000_0000u32, FeatureFlagsSystemOff2::empty().into());

        assert_eq!(0x0000_0001u32, FeatureFlagsSystemOff2::HIBERNATE_OFF.into());
    }

    #[test]
    fn test_hw_state() {
        assert_eq!(0, u32::from(HwState::On));
        assert_eq!(1, u32::from(HwState::Off));
        assert_eq!(2, u32::from(HwState::Standby));
    }

    #[test]
    fn test_function_version() {
        validate_function!(
            [0x8400_0000, 0, 0, 0],
            Function::Version,
            FunctionId::PsciVersion
        );
    }

    #[test]
    fn test_cpu_suspend() {
        validate_function!(
            [
                0x8400_0001,
                POWER_STATE_VALUE.into(),
                0xabcdef01,
                0x23456789
            ],
            Function::CpuSuspend {
                state: POWER_STATE,
                entry: EntryPoint::Entry32 {
                    entry_point_address: 0xabcdef01,
                    context_id: 0x23456789
                }
            },
            FunctionId::CpuSuspend32
        );

        validate_function!(
            [
                0xc400_0001,
                POWER_STATE_VALUE.into(),
                0xabcdef01_23456789,
                0x23456789_abcdef01
            ],
            Function::CpuSuspend {
                state: POWER_STATE,
                entry: EntryPoint::Entry64 {
                    entry_point_address: 0xabcdef01_23456789,
                    context_id: 0x23456789_abcdef01
                }
            },
            FunctionId::CpuSuspend64
        );
    }

    #[test]
    fn test_function_cpu_off() {
        validate_function!([0x8400_0002, 0, 0, 0], Function::CpuOff, FunctionId::CpuOff);
    }

    #[test]
    fn test_function_cpu_on() {
        validate_function!(
            [0x8400_0003, MPIDR32_VALUE.into(), 0xabcdef01, 0x23456789],
            Function::CpuOn {
                target_cpu: MPIDR32,
                entry: EntryPoint::Entry32 {
                    entry_point_address: 0xabcdef01,
                    context_id: 0x23456789,
                },
            },
            FunctionId::CpuOn32
        );

        validate_function!(
            [
                0xc400_0003,
                MPIDR64_VALUE,
                0x01234567_89abcdef,
                0x89abcdef_01234567,
            ],
            Function::CpuOn {
                target_cpu: MPIDR64,
                entry: EntryPoint::Entry64 {
                    entry_point_address: 0x01234567_89abcdef,
                    context_id: 0x89abcdef_01234567,
                },
            },
            FunctionId::CpuOn64
        );
    }

    #[test]
    #[should_panic]
    fn test_function_cpu_on_mixed_32_64() {
        let mut regs = [0u64; 4];
        Function::CpuOn {
            target_cpu: MPIDR64,
            entry: EntryPoint::Entry32 {
                entry_point_address: 1,
                context_id: 2,
            },
        }
        .copy_to_array(&mut regs);
    }

    #[test]
    fn test_function_affinity_info() {
        validate_function!(
            [0x8400_0004, MPIDR32_VALUE.into(), 2, 0],
            Function::AffinityInfo {
                mpidr: MPIDR32,
                lowest_affinity_level: 2,
            },
            FunctionId::AffinityInfo32
        );

        validate_function!(
            [0xc400_0004, MPIDR64_VALUE, 2, 0],
            Function::AffinityInfo {
                mpidr: MPIDR64,
                lowest_affinity_level: 2,
            },
            FunctionId::AffinityInfo64
        );

        assert_eq!(
            Err(Error::InvalidLowerAffinityLevel(3)),
            Function::try_from([0x8400_0004, MPIDR32_VALUE.into(), 3, 0])
        );

        assert_eq!(
            Err(Error::InvalidLowerAffinityLevel(4)),
            Function::try_from([0xc400_0004, MPIDR64_VALUE, 4, 0])
        );
    }

    #[test]
    fn test_function_migrate() {
        validate_function!(
            [0x8400_0005, MPIDR32_VALUE.into(), 0, 0],
            Function::Migrate {
                target_affinity: MPIDR32,
            },
            FunctionId::Migrate32
        );

        validate_function!(
            [0xc400_0005, MPIDR64_VALUE, 0, 0],
            Function::Migrate {
                target_affinity: MPIDR64,
            },
            FunctionId::Migrate64
        );
    }

    #[test]
    fn test_function_migrate_info_type() {
        validate_function!(
            [0x8400_0006, 0, 0, 0],
            Function::MigrateInfoType,
            FunctionId::MigrateInfoType
        );
    }

    #[test]
    fn test_function_migrate_info_up_cpu() {
        validate_function!(
            [0x8400_0007, 0, 0, 0],
            Function::MigrateInfoUpCpu { is_32bit: true },
            FunctionId::MigrateInfoUpCpu32
        );

        validate_function!(
            [0xc400_0007, 0, 0, 0],
            Function::MigrateInfoUpCpu { is_32bit: false },
            FunctionId::MigrateInfoUpCpu64
        );
    }

    #[test]
    fn test_function_system_off() {
        validate_function!(
            [0x8400_0008, 0, 0, 0],
            Function::SystemOff,
            FunctionId::SystemOff
        );
    }

    #[test]
    fn test_function_system_off2() {
        validate_function!(
            [0x8400_0015, 0x0000_0001, 0xabcdef01, 0],
            Function::SystemOff2 {
                off_type: SystemOff2Type::HibernateOff,
                cookie: Cookie::Cookie32(0xabcdef01),
            },
            FunctionId::SystemOff232
        );

        validate_function!(
            [0xc400_0015, 0x0000_0001, 0xabcdef01_23456789, 0],
            Function::SystemOff2 {
                off_type: SystemOff2Type::HibernateOff,
                cookie: Cookie::Cookie64(0xabcdef01_23456789),
            },
            FunctionId::SystemOff264
        );
    }

    #[test]
    fn test_function_system_reset() {
        validate_function!(
            [0x8400_0009, 0, 0, 0],
            Function::SystemReset,
            FunctionId::SystemReset
        );
    }

    #[test]
    fn test_function_system_reset2() {
        validate_function!(
            [0x8400_0012, 0, 0xabcdef01, 0],
            Function::SystemReset2 {
                reset_type: ResetType::Architectural(ArchitecturalResetType::SystemWarmReset),
                cookie: Cookie::Cookie32(0xabcdef01),
            },
            FunctionId::SystemReset232
        );

        validate_function!(
            [0xc400_0012, 0, 0xabcdef01_23456789, 0],
            Function::SystemReset2 {
                reset_type: ResetType::Architectural(ArchitecturalResetType::SystemWarmReset),
                cookie: Cookie::Cookie64(0xabcdef01_23456789),
            },
            FunctionId::SystemReset264
        );
    }

    #[test]
    fn test_function_mem_protect() {
        validate_function!(
            [0x8400_0013, 0x0000_0001, 0, 0],
            Function::MemProtect { enabled: true },
            FunctionId::MemProtect
        );

        validate_function!(
            [0x8400_0013, 0x0000_0000, 0, 0],
            Function::MemProtect { enabled: false },
            FunctionId::MemProtect
        );
    }

    #[test]
    fn test_function_mem_protect_check_range() {
        validate_function!(
            [0x8400_0014, 0xabcdef01, 0x23456789, 0],
            Function::MemProtectCheckRange {
                range: MemProtectRange::Range32 {
                    base: 0xabcdef01,
                    length: 0x23456789,
                },
            },
            FunctionId::MemProtectCheckRange32
        );

        validate_function!(
            [0xc400_0014, 0xabcdef01_23456789, 0x23456789_abcdef01, 0],
            Function::MemProtectCheckRange {
                range: MemProtectRange::Range64 {
                    base: 0xabcdef01_23456789,
                    length: 0x23456789_abcdef01,
                },
            },
            FunctionId::MemProtectCheckRange64
        );
    }

    #[test]
    fn test_function_features() {
        validate_function!(
            [0x8400_000a, 0x8000_0000, 0, 0],
            Function::Features {
                psci_func_id: PsciFeature::SmcccVersion,
            },
            FunctionId::PsciFeatures
        );

        validate_function!(
            [0x8400_000a, 0x8400_0001, 0, 0],
            Function::Features {
                psci_func_id: PsciFeature::PsciFunction(FunctionId::CpuSuspend32),
            },
            FunctionId::PsciFeatures
        );
    }

    #[test]
    fn test_function_cpu_freeze() {
        validate_function!(
            [0x8400_000b, 0, 0, 0],
            Function::CpuFreeze,
            FunctionId::CpuFreeze
        );
    }

    #[test]
    fn test_function_cpu_default_suspend() {
        validate_function!(
            [0x8400_000c, 0xabcdef01, 0x23456789, 0],
            Function::CpuDefaultSuspend {
                entry: EntryPoint::Entry32 {
                    entry_point_address: 0xabcdef01,
                    context_id: 0x23456789,
                },
            },
            FunctionId::CpuDefaultSuspend32
        );

        validate_function!(
            [0xc400_000c, 0xabcdef01_23456789, 0x23456789_abcdef01, 0],
            Function::CpuDefaultSuspend {
                entry: EntryPoint::Entry64 {
                    entry_point_address: 0xabcdef01_23456789,
                    context_id: 0x23456789_abcdef01,
                },
            },
            FunctionId::CpuDefaultSuspend64
        );
    }

    #[test]
    fn test_function_node_hw_state() {
        validate_function!(
            [0x8400_000d, MPIDR32_VALUE.into(), 0xabcdef01, 0],
            Function::NodeHwState {
                target_cpu: MPIDR32,
                power_level: 0xabcdef01,
            },
            FunctionId::NodeHwState32
        );

        validate_function!(
            [0xc400_000d, MPIDR64_VALUE, 0xabcdef01, 0],
            Function::NodeHwState {
                target_cpu: MPIDR64,
                power_level: 0xabcdef01,
            },
            FunctionId::NodeHwState64
        );
    }

    #[test]
    fn test_function_system_suspend() {
        validate_function!(
            [0x8400_000e, 0xabcdef01, 0x23456789, 0],
            Function::SystemSuspend {
                entry: EntryPoint::Entry32 {
                    entry_point_address: 0xabcdef01,
                    context_id: 0x23456789,
                },
            },
            FunctionId::SystemSuspend32
        );

        validate_function!(
            [0xc400_000e, 0xabcdef01_23456789, 0x23456789_abcdef01, 0],
            Function::SystemSuspend {
                entry: EntryPoint::Entry64 {
                    entry_point_address: 0xabcdef01_23456789,
                    context_id: 0x23456789_abcdef01,
                },
            },
            FunctionId::SystemSuspend64
        );
    }

    #[test]
    fn test_function_set_suspend_mode() {
        validate_function!(
            [0x8400_000f, 0x0000_0001, 0, 0],
            Function::SetSuspendMode {
                mode: SuspendMode::OsInitiated,
            },
            FunctionId::PsciSetSuspendMode
        );
    }

    #[test]
    fn test_function_stat_residency() {
        validate_function!(
            [
                0x8400_0010,
                MPIDR32_VALUE.into(),
                POWER_STATE_VALUE.into(),
                0,
            ],
            Function::StatResidency {
                target_cpu: MPIDR32,
                power_state: POWER_STATE,
            },
            FunctionId::PsciStatResidency32
        );

        validate_function!(
            [0xc400_0010, MPIDR64_VALUE, POWER_STATE_VALUE.into(), 0],
            Function::StatResidency {
                target_cpu: MPIDR64,
                power_state: POWER_STATE,
            },
            FunctionId::PsciStatResidency64
        );
    }

    #[test]
    fn test_function_stat_count() {
        validate_function!(
            [
                0x8400_0011,
                MPIDR32_VALUE.into(),
                POWER_STATE_VALUE.into(),
                0,
            ],
            Function::StatCount {
                target_cpu: MPIDR32,
                power_state: POWER_STATE,
            },
            FunctionId::PsciStatCount32
        );

        validate_function!(
            [0xc400_0011, MPIDR64_VALUE, POWER_STATE_VALUE.into(), 0],
            Function::StatCount {
                target_cpu: MPIDR64,
                power_state: POWER_STATE,
            },
            FunctionId::PsciStatCount64
        );
    }
}
