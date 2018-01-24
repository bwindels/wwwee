//compile with: cc read_aio.c -o read_aio -lcrypto
#define _GNU_SOURCE
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/syscall.h>
#include <sys/eventfd.h>
#include <sys/epoll.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <linux/aio_abi.h>
#include <openssl/sha.h>

#define BUFFER_SIZE 4096 * 1000
#define BUFFER_ALIGNMENT 512

int io_setup(unsigned nr, aio_context_t *ctxp) {
	return syscall(__NR_io_setup, nr, ctxp);
}

int io_destroy(aio_context_t ctx) {
	return syscall(__NR_io_destroy, ctx);
}

int io_submit(aio_context_t ctx, long nr, struct iocb **iocbpp) {
	return syscall(__NR_io_submit, ctx, nr, iocbpp);
}

int io_getevents(aio_context_t ctx, long min_nr, long max_nr,
		struct io_event *events, struct timespec *timeout) {
	return syscall(__NR_io_getevents, ctx, min_nr, max_nr, events, timeout);
}

struct aio_file {
	struct iocb cb;
	int event_fd;
	int fd;
	int read_blocks;
	void* buffer;
	struct epoll_event event;
	aio_context_t ctx;
};

static char read_buffer[BUFFER_SIZE + BUFFER_ALIGNMENT];

void * align_pointer(void* start, size_t alignment) {
  return start + (alignment - ((size_t)start % alignment));
}

int file_setup(struct aio_file* file, char* filename) {
	file->read_blocks = 0;
	file->event_fd = eventfd(0, EFD_NONBLOCK);
	if (file->event_fd == -1) {
		perror("eventfd");
		return -1;
	}
	file->fd = open(filename, O_RDONLY | O_DIRECT | O_NOATIME | O_NONBLOCK);
	if (file->fd == -1) {
		perror("open");
		return -1;
	}
	memset(&file->event, 0, sizeof(file->event));
	file->ctx = 0;
	if(io_setup(1, &file->ctx) == -1) {
		perror("io_setup");
		return -1;
	}

	file->buffer = align_pointer(&read_buffer, BUFFER_ALIGNMENT);

	memset(&file->cb, 0, sizeof(file->cb));
	struct iocb* cb = &file->cb;
	cb->aio_fildes = file->fd;
	cb->aio_flags = IOCB_FLAG_RESFD;
	cb->aio_resfd = file->event_fd;
	cb->aio_lio_opcode = IOCB_CMD_PREAD;
	cb->aio_buf = (uint64_t) file->buffer;
	cb->aio_nbytes = BUFFER_SIZE;

}

int file_register_read(struct aio_file* file, int epoll_fd) {
	file->event.events = EPOLLIN | EPOLLET;
	if (epoll_ctl(epoll_fd, EPOLL_CTL_ADD, file->event_fd, &file->event) == -1) {
		perror("epoll_ctl");
		return -1;
	}
	return 0;
}

int file_read_queue(struct aio_file* file) {
	struct iocb* cb = &file->cb;
  printf("   submitting iocb with ptr %lx\n", cb);

	cb->aio_offset = file->read_blocks * BUFFER_SIZE;
	//printf("submitting read request at %x for %d bytes ... ", cb->aio_offset, cb->aio_nbytes);
	fflush(0);
	if (io_submit(file->ctx, 1, &cb ) == -1) {
		perror("io_submit");
		return -1;
	}
	return 0;
}

int file_read_complete(struct aio_file* file, char** buffer_ptr, size_t* buffer_size_ptr) {
	struct io_event evt_array[2] = {0};
	int count = io_getevents(file->ctx, 1, 1, &evt_array[0], NULL);
	if (count != 1) {
		fprintf(stderr, "io_getevents %d instead of 1\n", count);
		return 1;
	}
	struct io_event evt = evt_array[0];
	if (evt.res < 0) {
		fprintf(stderr, "read: %s\n", strerror(-evt.res));
		return -1;
	}
	//printf("read %d\n", evt.res);

	//if (evt.res != ((struct iocb*)evt.obj)->aio_nbytes) {
	//	printf("didn't read requested size, EOF?\n");
	//	return -1;
	//}
  printf("    finishing iocb with ptr %lx\n", evt.obj);
	fflush(0);
	file->read_blocks += 1;
	*buffer_size_ptr = evt.res;
	*buffer_ptr = file->buffer;
}


int main(int argc, char *argv[]) {
	if (argc < 2) {
		perror("missing file argument");
		return -1;
	}
	char* filename = argv[1];

	struct aio_file file;
	if (file_setup(&file, filename) == -1) {
		return -1;
	}


	int epoll_fd = epoll_create(1);
	if (epoll_fd == -1) {
		perror("epoll_create");
		return -1;
	}
	
	if (file_register_read(&file, epoll_fd) == -1) {
		return -1;
	}


	unsigned char digest[SHA512_DIGEST_LENGTH];
    SHA512_CTX sha_ctx;
    SHA512_Init(&sha_ctx);

	if (file_read_queue(&file) == -1) {
		return -1;
	}
	struct epoll_event event;
	int count;
	while(count = epoll_wait(epoll_fd, &event, 1, 1000) || 1) {
		//if(count == -1) {
		//	perror("epoll_wait");
		//}
		char* buffer;
		size_t size;
		if(file_read_complete(&file, &buffer, &size) == -1) {
			return -1;
		}
		SHA512_Update(&sha_ctx, buffer, size);
		if (size != BUFFER_SIZE) {
			break;
		}
		file_read_queue(&file);
	}

	SHA512_Final(digest, &sha_ctx);
	char hex_digest[SHA512_DIGEST_LENGTH*2+1];
    for (int i = 0; i < SHA512_DIGEST_LENGTH; i++) {
        sprintf(&hex_digest[i*2], "%02x", (unsigned int)digest[i]);
    }

    printf("SHA512 digest: %s\n", hex_digest);


    return 0;
}
