#include <sys/types.h>
#include <sys/stat.h>
#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#define O_PATH 2097152

int main() {
    int dir_fd = open("foo", O_DIRECTORY | O_PATH);
    if (dir_fd == -1) {
        perror("bad directory");
        exit(1);
    }
    int file_fd = openat(dir_fd, "", O_RDONLY);
    if (file_fd == -1) {
        perror("bad file");
        exit(1);
    }
}
