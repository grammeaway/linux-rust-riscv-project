/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _RVCPU_UAPI_H
#define _RVCPU_UAPI_H

#include <linux/ioctl.h>
#include <linux/types.h>

struct rvcpu_snapshot {
    __u64 time;
    __u64 cycle;
    __u64 instret;
};

#define RVCPU_IOC_MAGIC '|'
#define RVCPU_IOC_SNAPSHOT _IOR(RVCPU_IOC_MAGIC, 0x80, struct rvcpu_snapshot)

#endif /* _RVCPU_UAPI_H */
