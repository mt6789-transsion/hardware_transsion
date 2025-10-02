#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <fcntl.h>
#include <unistd.h>

#define HBM_PATH "/sys/kernel/tran_display/lcm_hbm_state"

int write_hbm_state(const char *value) {
    int fd = open(HBM_PATH, O_WRONLY);
    if (fd < 0) {
        fprintf(stderr, "Error: cannot open %s: %s\n", HBM_PATH, strerror(errno));
        return -1;
    }

    ssize_t len = write(fd, value, strlen(value));
    if (len < 0) {
        fprintf(stderr, "Error: failed to write '%s' to %s: %s\n",
                value, HBM_PATH, strerror(errno));
        close(fd);
        return -1;
    }

    close(fd);
    printf("Success: wrote '%s' to %s\n", value, HBM_PATH);
    return 0;
}

int main(int argc, char **argv) {
    if (argc != 2) {
        fprintf(stderr, "Usage: %s <0|1>\n", argv[0]);
        return 1;
    }

    if (strcmp(argv[1], "0") != 0 && strcmp(argv[1], "1") != 0) {
        fprintf(stderr, "Error: invalid value '%s'. Use 0 (disable) or 1 (enable).\n", argv[1]);
        return 1;
    }

    return write_hbm_state(argv[1]) == 0 ? 0 : 2;
}
