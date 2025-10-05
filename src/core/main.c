#include "trimorph.h"

/**
 * Main entry point for trimorph client
 */
int main(int argc, char *argv[]) {
    if (argc < 2) {
        fprintf(stderr, "Usage: %s <command> [args...]\n", argv[0]);
        fprintf(stderr, "Commands: start, stop, status, exec, install, update, list, daemon\n");
        return 1;
    }
    
    // Initialize the system
    if (initialize_system() != 0) {
        fprintf(stderr, "Failed to initialize trimorph system\n");
        return 1;
    }
    
    // Handle commands
    if (strcmp(argv[1], "start") == 0 && argc >= 3) {
        int result = start_jail(argv[2]);
        return result == 0 ? 0 : 1;
    } else if (strcmp(argv[1], "stop") == 0 && argc >= 3) {
        int result = stop_jail(argv[2]);
        return result == 0 ? 0 : 1;
    } else if (strcmp(argv[1], "exec") == 0 && argc >= 4) {
        // Format: trimorph exec <jail_name> <command> [args...]
        char* jail_name = argv[2];
        char* cmd = argv[3];
        
        // Create args array for command
        char** args = malloc((argc - 3) * sizeof(char*));
        for (int i = 0; i < argc - 4; i++) {
            args[i] = argv[i + 4];
        }
        args[argc - 4] = NULL;
        
        int result = execute_in_jail(jail_name, cmd, args);
        free(args);
        return WEXITSTATUS(result);
    } else if (strcmp(argv[1], "list") == 0) {
        jail_config_t* configs = NULL;
        int count = 0;
        int result = load_all_jails(&configs, &count);
        
        if (result == 0) {
            printf("Available jails:\n");
            for (int i = 0; i < count; i++) {
                printf("  %s (%s) - Status: %s\n", 
                       configs[i].name, 
                       configs[i].pkgmgr,
                       configs[i].status == JAIL_RUNNING ? "RUNNING" : 
                       configs[i].status == JAIL_STOPPED ? "STOPPED" : "ERROR");
            }
            free(configs);
        }
        return 0;
    } else if (strcmp(argv[1], "update") == 0) {
        return check_for_updates() == 0 ? 0 : 1;
    } else if (strcmp(argv[1], "auto-update") == 0) {
        return auto_update_packages() == 0 ? 0 : 1;
    } else if (strcmp(argv[1], "install-local") == 0 && argc >= 3) {
        return install_local_package(argv[2]) == 0 ? 0 : 1;
    } else if (strcmp(argv[1], "daemon") == 0) {
        // Run as daemon
        create_daemon();
        return run_daemon();
    } else {
        fprintf(stderr, "Unknown command or insufficient arguments\n");
        fprintf(stderr, "Usage: %s <command> [args...]\n", argv[0]);
        fprintf(stderr, "Commands: start <jail>, stop <jail>, exec <jail> <cmd> [args], list, update, auto-update, install-local <pkg>, daemon\n");
        return 1;
    }
    
    return 0;
}