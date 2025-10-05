#include "trimorph.h"
#include <sys/mount.h>
#include <sched.h>
#include <sys/syscall.h>
#include <time.h>

// Global variables to track the daemon state
static volatile sig_atomic_t daemon_running = 1;
static pthread_mutex_t jail_mutex = PTHREAD_MUTEX_INITIALIZER;

static void signal_handler(int sig) {
    daemon_running = 0;
}

/**
 * Sets up overlay filesystem for a jail
 */
int setup_overlay(const char* jail_name, char* upper_dir, char* work_dir) {
    // Create temporary directories for overlay
    char base_root[MAX_PATH_LEN];
    snprintf(base_root, sizeof(base_root), "%s/%s", TRIMORPH_BASE_DIR, jail_name);
    
    // Create upper and work directories
    snprintf(upper_dir, MAX_PATH_LEN, "%s/trimorph_%s_upper_%d", TRIMORPH_RUNTIME_DIR, jail_name, getpid());
    snprintf(work_dir, MAX_PATH_LEN, "%s/trimorph_%s_work_%d", TRIMORPH_RUNTIME_DIR, jail_name, getpid());
    
    if (mkdir(upper_dir, 0755) == -1 && errno != EEXIST) {
        fprintf(stderr, "Error creating upper dir %s: %s\n", upper_dir, strerror(errno));
        return -1;
    }
    
    if (mkdir(work_dir, 0755) == -1 && errno != EEXIST) {
        fprintf(stderr, "Error creating work dir %s: %s\n", work_dir, strerror(errno));
        return -1;
    }
    
    // Create overlay mount command (we'll use system for now, but could be done with syscall)
    char overlay_mount[MAX_CMD_LEN];
    snprintf(overlay_mount, sizeof(overlay_mount), 
             "mount -t overlay overlay -o lowerdir=%s,upperdir=%s,workdir=%s %s/%s", 
             base_root, upper_dir, work_dir, TRIMORPH_BASE_DIR, jail_name);
    
    return 0;
}

/**
 * Cleans up overlay filesystem for a jail
 */
int cleanup_overlay(const char* upper_dir, const char* work_dir) {
    // Unmount overlay - simplified for now
    if (upper_dir && strlen(upper_dir) > 0) {
        rmdir(upper_dir);
    }
    if (work_dir && strlen(work_dir) > 0) {
        rmdir(work_dir);
    }
    return 0;
}

/**
 * Starts a jail by executing its bootstrap command
 */
int start_jail(const char* jail_name) {
    // Find the jail configuration
    jail_config_t* configs = NULL;
    int count = 0;
    int result = load_all_jails(&configs, &count);
    if (result != 0) {
        return -1;
    }
    
    int found = 0;
    jail_config_t target_config;
    for (int i = 0; i < count; i++) {
        if (strcmp(configs[i].name, jail_name) == 0) {
            target_config = configs[i];
            found = 1;
            break;
        }
    }
    
    free(configs);
    
    if (!found) {
        fprintf(stderr, "Error: Jail '%s' not found\n", jail_name);
        return -1;
    }
    
    // Lock to ensure thread safety
    pthread_mutex_lock(&jail_mutex);
    
    // Check if jail is already running
    if (target_config.status == JAIL_RUNNING) {
        fprintf(stderr, "Jail '%s' is already running\n", jail_name);
        pthread_mutex_unlock(&jail_mutex);
        return -1;
    }
    
    // Fork and execute the bootstrap command
    pid_t pid = fork();
    if (pid == 0) {
        // Child process - execute bootstrap command
        execl("/bin/sh", "sh", "-c", target_config.bootstrap, (char *)NULL);
        fprintf(stderr, "Failed to execute bootstrap command for jail %s\n", jail_name);
        exit(1);
    } else if (pid > 0) {
        // Parent process - wait for bootstrap to complete
        int status;
        waitpid(pid, &status, 0);
        
        if (WIFEXITED(status) && WEXITSTATUS(status) == 0) {
            target_config.status = JAIL_RUNNING;
            target_config.pid = pid;
            result = 0;
        } else {
            target_config.status = JAIL_ERROR;
            result = -1;
        }
    } else {
        // Fork failed
        fprintf(stderr, "Failed to fork for jail %s: %s\n", jail_name, strerror(errno));
        result = -1;
    }
    
    pthread_mutex_unlock(&jail_mutex);
    return result;
}

/**
 * Stops a running jail
 */
int stop_jail(const char* jail_name) {
    // Find the jail configuration
    jail_config_t* configs = NULL;
    int count = 0;
    int result = load_all_jails(&configs, &count);
    if (result != 0) {
        return -1;
    }
    
    int found = 0;
    jail_config_t* target_config = NULL;
    for (int i = 0; i < count; i++) {
        if (strcmp(configs[i].name, jail_name) == 0) {
            target_config = &configs[i];
            found = 1;
            break;
        }
    }
    
    if (!found) {
        fprintf(stderr, "Error: Jail '%s' not found\n", jail_name);
        free(configs);
        return -1;
    }
    
    // Check if jail is running
    if (target_config->status != JAIL_RUNNING) {
        fprintf(stderr, "Jail '%s' is not running\n", jail_name);
        free(configs);
        return -1;
    }
    
    // Kill the jail process
    if (kill(target_config->pid, SIGTERM) == 0) {
        // Wait a bit for graceful shutdown
        sleep(1);
        // Force kill if still running
        kill(target_config->pid, SIGKILL);
        target_config->status = JAIL_STOPPED;
        result = 0;
    } else {
        fprintf(stderr, "Failed to stop jail %s: %s\n", jail_name, strerror(errno));
        result = -1;
    }
    
    free(configs);
    return result;
}

/**
 * Executes a command in a specified jail
 */
int execute_in_jail(const char* jail_name, const char* command, char** args) {
    // Find the jail configuration
    jail_config_t* configs = NULL;
    int count = 0;
    int result = load_all_jails(&configs, &count);
    if (result != 0) {
        return -1;
    }
    
    int found = 0;
    jail_config_t target_config;
    for (int i = 0; i < count; i++) {
        if (strcmp(configs[i].name, jail_name) == 0) {
            target_config = configs[i];
            found = 1;
            break;
        }
    }
    
    free(configs);
    
    if (!found) {
        fprintf(stderr, "Error: Jail '%s' not found\n", jail_name);
        return -1;
    }
    
    // Check if systemd-nspawn is available, otherwise use Rust jail runner
    int use_systemd = 0;
    if (access("/usr/bin/systemd-nspawn", X_OK) == 0) {
        use_systemd = 1;
    }
    
    if (use_systemd) {
        // Setup overlay filesystem
        char upper_dir[MAX_PATH_LEN];
        char work_dir[MAX_PATH_LEN];
        result = setup_overlay(jail_name, upper_dir, work_dir);
        if (result != 0) {
            fprintf(stderr, "Error setting up overlay for jail %s\n", jail_name);
            return -1;
        }
        
        // Build the command to execute in the jail
        char full_cmd[MAX_CMD_LEN];
        snprintf(full_cmd, sizeof(full_cmd), 
                 "/usr/bin/systemd-nspawn --quiet --directory=%s --overlay=%s:%s:%s %s %s %s",
                 target_config.root, 
                 target_config.root, upper_dir, work_dir,
                 target_config.pkgmgr, 
                 target_config.pkgmgr_args,
                 command);
        
        // Add the user command and arguments
        if (args) {
            for (int i = 0; args[i]; i++) {
                strncat(full_cmd, " ", sizeof(full_cmd) - strlen(full_cmd) - 1);
                strncat(full_cmd, args[i], sizeof(full_cmd) - strlen(full_cmd) - 1);
            }
        }
        
        printf("Executing command: %s\n", full_cmd);
        
        // Execute the command in the jail
        result = system(full_cmd);
        
        // Cleanup overlay filesystem  
        cleanup_overlay(upper_dir, work_dir);
        
        return WEXITSTATUS(result);
    } else {
        // Use the existing Rust jail runner
        char cmd[MAX_CMD_LEN];
        
        // Build command to call Rust jail runner
        snprintf(cmd, sizeof(cmd), "/usr/local/bin/jail-runner %s %s", jail_name, command);
        
        // Add arguments
        if (args) {
            for (int i = 0; args[i]; i++) {
                strncat(cmd, " ", sizeof(cmd) - strlen(cmd) - 1);
                strncat(cmd, args[i], sizeof(cmd) - strlen(cmd) - 1);
            }
        }
        
        printf("Executing via Rust jail runner: %s\n", cmd);
        
        result = system(cmd);
        return WEXITSTATUS(result);
    }
}

/**
 * Creates a daemon process
 */
int create_daemon() {
    pid_t pid, sid;
    
    // Fork off the parent process
    pid = fork();
    if (pid < 0) {
        exit(EXIT_FAILURE);
    }
    
    // If we got a good PID, then we can exit the parent process
    if (pid > 0) {
        exit(EXIT_SUCCESS);
    }
    
    // Change the file mode mask
    umask(0);
    
    // Create a new SID for the child process
    sid = setsid();
    if (sid < 0) {
        exit(EXIT_FAILURE);
    }
    
    // Change the current working directory
    if ((chdir("/")) < 0) {
        exit(EXIT_FAILURE);
    }
    
    // Close out the standard file descriptors
    close(STDIN_FILENO);
    close(STDOUT_FILENO);
    close(STDERR_FILENO);
    
    // Write PID to file
    FILE* pidfile = fopen(TRIMORPH_PID_FILE, "w");
    if (pidfile) {
        fprintf(pidfile, "%d\n", getpid());
        fclose(pidfile);
    }
    
    return 0;
}

/**
 * Initializes the trimorph system
 */
int initialize_system() {
    // Create necessary directories
    struct stat st = {0};
    
    if (stat(TRIMORPH_RUNTIME_DIR, &st) == -1) {
        mkdir(TRIMORPH_RUNTIME_DIR, 0755);
    }
    
    if (stat(TRIMORPH_CACHE_DIR, &st) == -1) {
        mkdir(TRIMORPH_CACHE_DIR, 0755);
    }
    
    if (stat(TRIMORPH_LOG_DIR, &st) == -1) {
        mkdir(TRIMORPH_LOG_DIR, 0755);
    }
    
    if (stat(TRIMORPH_BASE_DIR, &st) == -1) {
        mkdir(TRIMORPH_BASE_DIR, 0755);
    }
    
    return 0;
}

/**
 * Cleans up the trimorph system
 */
int cleanup_system() {
    // Remove PID file
    unlink(TRIMORPH_PID_FILE);
    return 0;
}