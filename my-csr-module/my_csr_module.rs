// SPDX-License-Identifier: GPL-2.0

//! Rust module reading RISC-V CSRs, using inline assembly.

use kernel::prelude::*;
use core::arch::asm; // For the `asm!` macro.


module! {
    type: MyCSRModule,
    name: "my_csr_module",
    authors: ["Victor Gram Thomsen"],
    description: "Reads RISC-V CSRs using inline assembly",
    license: "GPL",
}

struct MyCSRModule {
}


#[cfg(target_arch = "riscv64")]
fn read_time_csr() -> u64 {
    let value: u64;
    // SAFETY: reading the `time` CSR has no side effects.
    unsafe {
        asm!("csrr {0}, time", out(reg) value);
    }
    value
}


impl kernel::Module for MyCSRModule {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust RISC-V CSR sample (init)\n");
        pr_info!("Am I built-in? {}\n", !cfg!(MODULE));


        #[cfg(target_arch = "riscv64")]
        {
            let time = read_time_csr();
            pr_info!("RISC-V time CSR: {}\n", time);
        }

        Ok(MyCSRModule {})
    }
}

impl Drop for MyCSRModule {
    fn drop(&mut self) {
        pr_info!("Rust RISC-V CSR sample (exit)\n");
    }
}

