// SPDX-License-Identifier: GPL-2.0

//! Rust module reading RISC-V CSRs, using inline assembly.

use kernel::prelude::*;
use core::arch::asm; // For the `asm!` macro.
use core::hint::black_box; // For preventing optimizations in the benchmark loop.

module! {
    type: MyCSRModule,
    name: "my_csr_module",
    authors: ["Victor Gram Thomsen"],
    description: "Reads RISC-V CSRs using inline assembly",
    license: "GPL",
}

struct MyCSRModule {
}

 // Not 100% necessary, since the macro is only used in an already 
// arch-guarded block, but good to be safe.
#[cfg(target_arch = "riscv64")]
macro_rules! read_csr { 
    ($csr:ident) => {{ 
        let value: u64;
        // SAFETY: reading a CSR is a pure read with no side effects
        unsafe { 
            asm!(
                concat!("csrr {0}, ", stringify!($csr)),
                out(reg) value 
            );
        }
        value
    }};
}

impl kernel::Module for MyCSRModule {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust RISC-V CSR sample (init)\n");
        pr_info!("Am I built-in? {}\n", !cfg!(MODULE));


        #[cfg(target_arch = "riscv64")]
        {
            // Initial simple reads
            let time = read_csr!(time);
            pr_info!("RISC-V time CSR: {}\n", time);
            let cycle = read_csr!(cycle);
            pr_info!("RISC-V cycle CSR: {}\n", cycle);
            let instret = read_csr!(instret);
            pr_info!("RISC-V instret CSR: {}\n", instret);

            // Simple "benchmark" to show the difference in cycle and instret counts before and
            // after a loop.
            //let cycle_start = read_csr!(cycle);
            let t_start = read_csr!(time);
            let instret_start = read_csr!(instret);
            
            let mut sum: u64 = 0;
            for i in 0..10_000_000u64 {
                sum = sum.wrapping_add(black_box(i));
                unsafe { asm!("", options(nostack, preserves_flags)); }
            }
            let t_end = read_csr!(time);
            //let cycle_end = read_csr!(cycle);
            let instret_end = read_csr!(instret);
            
            pr_info!("loop anti-DCE sum: {}\n", black_box(sum)); // Prevent the loop from being optimized away.
            //pr_info!("cycle delta: {}\n", cycle_end - cycle_start);
            pr_info!("time delta: {}\n", t_end - t_start);
            pr_info!("instret delta: {}\n", instret_end - instret_start);
        }

        Ok(MyCSRModule {})
    }
}

impl Drop for MyCSRModule {
    fn drop(&mut self) {
        pr_info!("Rust RISC-V CSR sample (exit)\n");
    }
}

