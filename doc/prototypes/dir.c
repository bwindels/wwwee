#define _GNU_SOURCE

#include <dirent.h>
#include <fcntl.h>
#include <stddef.h>
#include <unistd.h>
#include <stdlib.h>
#include <stdio.h>
#include <sys/stat.h>
#include <sys/syscall.h>

struct linux_dirent64 {
  ino64_t        d_ino;    /* 64-bit inode number */
  off64_t        d_off;    /* 64-bit offset to next structure */
  unsigned short d_reclen; /* Size of this dirent */
  unsigned char  d_type;   /* File type */
  char           d_name[]; /* Filename (null-terminated) */
};


int getdents64(unsigned int fd, struct linux_dirent64 *dirp, unsigned int count) {
    return syscall(__NR_getdents64, fd, dirp, count);
}

int getdents(unsigned int fd, struct linux_dirent64 *dirp, unsigned int count) {
    return syscall(__NR_getdents64, fd, dirp, count);
}

#define BUFFER_SIZE 512

int main(int argc, char *argv[]) {
  int fd;
  char buf[BUFFER_SIZE];

  fd = open(argc > 1 ? argv[1] : ".", O_RDONLY | O_DIRECTORY);
  if (fd == -1) {
    perror("could not open file");
    exit(EXIT_FAILURE);
  }

  while(1) {
    int nread = getdents64(fd, (struct linux_dirent64*)&buf, BUFFER_SIZE);
    if (nread == -1) {
      perror("could not get directory entries");
      exit(EXIT_FAILURE);
    }
    if(nread == 0) {
      break;
    }
    int pos = 0;
    while (pos < nread) {
      struct linux_dirent64* d = (struct linux_dirent64*)(buf + pos);
      unsigned long name_len = d->d_reclen - 2 - offsetof(struct linux_dirent64, d_name);
      printf("%lu - 2 - %lu = %lu\n", d->d_reclen, offsetof(struct linux_dirent64, d_name), name_len);
      write(1, d->d_name, name_len);
      char d_type = *(buf + pos + d->d_reclen - 1);
      if (d_type == DT_DIR) {
        printf("\n");
      }
      else {
        struct stat file_info;
        fstatat(fd, d->d_name, &file_info, AT_SYMLINK_NOFOLLOW);
        printf(" (%lu bytes)\n", file_info.st_size);
      }
      pos += d->d_reclen;
    }

  }

}
