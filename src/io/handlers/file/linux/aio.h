#ifndef LINUX_AIO_HEADER_INCLUDED
#define LINUX_AIO_HEADER_INCLUDED

#define _GNU_SOURCE

#include <unistd.h>
#include <sys/syscall.h>
#include <linux/aio_abi.h>
#include <linux/time.h>

int io_setup(unsigned nr, aio_context_t *ctxp);
int io_destroy(aio_context_t ctx);
int io_submit(aio_context_t ctx, long nr, struct iocb **iocbpp);
int io_getevents(aio_context_t ctx, long min_nr, long max_nr,
        struct io_event *events, struct timespec *timeout);

#endif //LINUX_AIO_HEADER_INCLUDED
