// Adapted from the Rust-for-Linux rust_minimal.rs sample, from the Linux kernel's samples/rust
// directory. All rights reserved to the original authors. Used here for demonstration purposes
// only.
// SPDX-License-Identifier: GPL-2.0

//! Rust minimal sample.

use kernel::bindings;
use kernel::cpu::{nr_cpu_ids, CpuId};
use kernel::prelude::*;
use kernel::time::{Delta, Instant, Monotonic};

module! {
    type: MyModule,
    name: "my_module",
    authors: ["Rust for Linux Contributors"],
    description: "Rust minimal sample (my module)",
    license: "GPL",
    params: {
        test_parameter: i64 {
            default: 1,
            description: "This parameter has a default of 1",
        },
        test_parameter_two: i32 {
            default: 2,
            description: "This parameter has a default of 2",
        },
    },
}

struct MyModule {
    numbers: KVec<i32>,
}

impl kernel::Module for MyModule {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust minimal sample (init)\n");
        pr_info!("Am I built-in? {}\n", !cfg!(MODULE));
        pr_info!("A new pr_info! macro with {} and {}\n", 42, "hello");
        pr_info!(
            "test_parameter: {}\n",
            *module_parameters::test_parameter.value()
        );
        pr_info!(
            "test_parameter_two: {}\n",
            *module_parameters::test_parameter_two.value()
        );
        let start = Instant::<Monotonic>::now();
        // do some work, or just call now() again
        let later = Instant::<Monotonic>::now();
        let delta: Delta = later - start;
        pr_info!("measured delta: {} ns\n", delta.as_nanos());
        pr_info!("nr_cpu_ids: {}\n", nr_cpu_ids());
        let cpu_id = CpuId::current();
        pr_info!("Current CPU ID: {}\n", cpu_id.as_u32());

        // SAFETY: `jiffies` is a valid global provided by the kernel.
        let j = unsafe { bindings::jiffies };
        pr_info!("jiffies = {}\n", j);

        let mut numbers = KVec::new();
        numbers.push(42, GFP_KERNEL)?;
        numbers.push(120, GFP_KERNEL)?;
        numbers.push(256, GFP_KERNEL)?;

        Ok(MyModule { numbers })
    }
}

impl Drop for MyModule {
    fn drop(&mut self) {
        pr_info!("My numbers are {:?}\n", self.numbers);
        pr_info!("Rust minimal sample (exit)\n");
    }
}

