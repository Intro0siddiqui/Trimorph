#ifndef TRIMORPH_H
#define TRIMORPH_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <dirent.h>
#include <pthread.h>
#include <sys/wait.h>
#include <errno.h>
#include <signal.h>
#include <limits.h>

// Configuration for Trimorph
#define TRIMORPH_CONFIG_DIR "/etc/trimorph"
#define TRIMORPH_JAILS_DIR "/etc/trimorph/jails.d"
#define TRIMORPH_BASE_DIR "/usr/local/trimorph/base"
#define TRIMORPH_RUNTIME_DIR "/var/lib/trimorph"
#define TRIMORPH_CACHE_DIR "/var/cache/trimorph/packages"
#define TRIMORPH_LOG_DIR "/var/log/trimorph"
#define TRIMORPH_PID_FILE "/var/run/trimorphd.pid"

// Maximum lengths
#define MAX_JAIL_NAME 256
#define MAX_PATH_LEN 1024
#define MAX_CMD_LEN 4096
#define MAX_CONFIG_LINE 1024

// Jail status
typedef enum {
    JAIL_STOPPED,
    JAIL_RUNNING,
    JAIL_ERROR
} jail_status_t;

// Jail configuration structure
typedef struct {
    char name[MAX_JAIL_NAME];
    char root[MAX_PATH_LEN];
    char bootstrap[MAX_CMD_LEN];
    char pkgmgr[MAX_PATH_LEN];
    char pkgmgr_args[MAX_CMD_LEN];
    char mounts[MAX_CMD_LEN];  // comma-separated mount points
    char env[MAX_CMD_LEN];     // comma-separated environment variables
    jail_status_t status;
    pid_t pid;  // PID if running as daemon
} jail_config_t;

// Function declarations
int parse_config_file(const char* config_path, jail_config_t* config);
int load_all_jails(jail_config_t** configs, int* count);
int start_jail(const char* jail_name);
int stop_jail(const char* jail_name);
int execute_in_jail(const char* jail_name, const char* command, char** args);
int setup_overlay(const char* jail_name, char* upper_dir, char* work_dir);
int cleanup_overlay(const char* upper_dir, const char* work_dir);
int create_daemon();
int initialize_system();
int cleanup_system();
int run_daemon();

// Installer functions
int install_to_host_from_jail(const char* jail_name, const char* package_name);
int install_local_package(const char* package_file);
int update_installed_packages();
int setup_auto_update();
int check_for_updates();
int auto_update_packages();

#endif