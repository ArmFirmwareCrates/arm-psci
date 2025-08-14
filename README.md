# Arm Power State Coordination Interface (PSCI) library

This crate aims to offer functions and user-friendly types for parsing and constructing arguments
of [Arm Power State Coordination Interface](https://developer.arm.com/documentation/den0022/latest/)
(PSCI) calls. This functionality can be beneficial for both firmware and OS components.

However, doing the actual `SMC`/`HVC`/`ERET` calls, or implementing power management logic is beyond
the scope of this crate.

## Implemented features

* Handling all PSCI 1.3 mandatory and optional functions
* Handling both 32-bit and 64-bit call formats
* Dedicated types for common PSCI call arguments
* Unit tests

## Limitations

* The implementation does not handle pre-1.0 format suspend power state (see 5.4.2.1 Original format)

## License

The project is MIT and Apache-2.0 dual licensed, see `LICENSE-APACHE` and `LICENSE-MIT`.

## Maintainers

arm-psci is a trustedfirmware.org maintained project. All contributions are ultimately merged by
the maintainers listed below.

* Bálint Dobszay <balint.dobszay@arm.com>
  [balint-dobszay-arm](https://github.com/balint-dobszay-arm)
* Imre Kis <imre.kis@arm.com>
  [imre-kis-arm](https://github.com/imre-kis-arm)
* Sandrine Afsa <sandrine.afsa@arm.com>
  [sandrine-bailleux-arm](https://github.com/sandrine-bailleux-arm)

## Contributing

Please follow the directions of the [Trusted Firmware Processes](https://trusted-firmware-docs.readthedocs.io/en/latest/generic_processes/index.html).

Contributions are handled through [review.trustedfirmware.org](https://review.trustedfirmware.org/q/project:arm-firmware-crates/arm-psci).

## Arm trademark notice

Arm is a registered trademark of Arm Limited (or its subsidiaries or affiliates).

This project uses some of the Arm product, service or technology trademarks, as listed in the
[Trademark List][1], in accordance with the [Arm Trademark Use Guidelines][2].

Subsequent uses of these trademarks throughout this repository do not need to be prefixed with the
Arm word trademark.

[1]: https://www.arm.com/company/policies/trademarks/arm-trademark-list
[2]: https://www.arm.com/company/policies/trademarks/guidelines-trademarks

--------------

*Copyright The arm-psci Contributors.*
