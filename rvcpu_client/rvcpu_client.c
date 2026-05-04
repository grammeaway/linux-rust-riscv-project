/* SPDX-License-Identifier: GPL-2.0 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/ioctl.h>

#include "rvcpu_uapi.h"

static void print_snapshot(const char *label, const struct rvcpu_snapshot *s)
{
    printf("%s: time=%llu cycle=%llu instret=%llu\n",
           label,
           (unsigned long long)s->time,
           (unsigned long long)s->cycle,
           (unsigned long long)s->instret);
}

int main(void)
{
    int fd = open("/dev/rvcpu", O_RDONLY);
    if (fd < 0) {
        perror("open /dev/rvcpu");
        return 1;
    }

    /* Path 1: read() returns the snapshot taken at open() time. */
    struct rvcpu_snapshot snap;
    ssize_t n = read(fd, &snap, sizeof(snap));
    if (n != (ssize_t)sizeof(snap)) {
        fprintf(stderr, "short read: got %zd, expected %zu\n", n, sizeof(snap));
        close(fd);
        return 1;
    }
    print_snapshot("open-time read()", &snap);

    /* Path 2: ioctl() takes a fresh snapshot each call. */
    for (int i = 0; i < 5; i++) {
        struct rvcpu_snapshot fresh;
        if (ioctl(fd, RVCPU_IOC_SNAPSHOT, &fresh) < 0) {
            perror("ioctl RVCPU_IOC_SNAPSHOT");
            close(fd);
            return 1;
        }
        char label[32];
        snprintf(label, sizeof(label), "ioctl #%d", i);
        print_snapshot(label, &fresh);

        /* tiny busy delay so successive snapshots differ */
        for (volatile int j = 0; j < 1000000; j++);
    }

    close(fd);
    return 0;
}
