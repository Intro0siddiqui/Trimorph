#include "trimorph.h"
#include <sys/socket.h>
#include <sys/un.h>
#include <getopt.h>

// Global daemon state
static int daemon_sock = -1;
static volatile sig_atomic_t running = 1;

/**
 * Handles daemon signals
 */
static void signal_handler(int sig) {
    switch(sig) {
        case SIGINT:
        case SIGTERM:
            printf("Received signal to stop daemon\n");
            running = 0;
            break;
        case SIGHUP:
            // Reload configuration
            printf("Received signal to reload configuration\n");
            break;
    }
}

/**
 * Creates and binds a Unix domain socket for IPC
 */
int create_ipc_socket() {
    struct sockaddr_un addr;
    int sock;
    
    // Create socket
    sock = socket(AF_UNIX, SOCK_STREAM, 0);
    if (sock == -1) {
        perror("socket");
        return -1;
    }
    
    // Set up address structure
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    snprintf(addr.sun_path, sizeof(addr.sun_path), "%s/trimorph.sock", TRIMORPH_RUNTIME_DIR);
    
    // Remove any existing socket
    unlink(addr.sun_path);
    
    // Bind the socket
    if (bind(sock, (struct sockaddr*)&addr, sizeof(addr)) == -1) {
        perror("bind");
        close(sock);
        return -1;
    }
    
    // Listen for connections
    if (listen(sock, 5) == -1) {
        perror("listen");
        close(sock);
        return -1;
    }
    
    return sock;
}

/**
 * Processes a client command
 */
int process_command(int client_fd, char* command) {
    // Parse the command
    char* token = strtok(command, " ");
    if (!token) return -1;
    
    if (strcmp(token, "EXECUTE") == 0) {
        // Format: EXECUTE <jail_name> <command> [args...]
        char* jail_name = strtok(NULL, " ");
        char* cmd = strtok(NULL, "\n");
        
        if (!jail_name || !cmd) {
            write(client_fd, "ERROR: Invalid command format\n", 31);
            return -1;
        }
        
        // Execute command in jail
        int result = execute_in_jail(jail_name, cmd, NULL); // args would need more parsing
        
        if (result == 0) {
            write(client_fd, "SUCCESS: Command executed\n", 26);
        } else {
            write(client_fd, "ERROR: Command failed\n", 23);
        }
    } else if (strcmp(token, "START") == 0) {
        // Format: START <jail_name>
        char* jail_name = strtok(NULL, " ");
        
        if (!jail_name) {
            write(client_fd, "ERROR: Invalid command format\n", 31);
            return -1;
        }
        
        int result = start_jail(jail_name);
        
        if (result == 0) {
            write(client_fd, "SUCCESS: Jail started\n", 23);
        } else {
            write(client_fd, "ERROR: Failed to start jail\n", 27);
        }
    } else if (strcmp(token, "STOP") == 0) {
        // Format: STOP <jail_name>
        char* jail_name = strtok(NULL, " ");
        
        if (!jail_name) {
            write(client_fd, "ERROR: Invalid command format\n", 31);
            return -1;
        }
        
        int result = stop_jail(jail_name);
        
        if (result == 0) {
            write(client_fd, "SUCCESS: Jail stopped\n", 23);
        } else {
            write(client_fd, "ERROR: Failed to stop jail\n", 26);
        }
    } else if (strcmp(token, "STATUS") == 0) {
        // Format: STATUS [jail_name]
        char* jail_name = strtok(NULL, " ");
        
        // This would return status of all jails or specific jail
        write(client_fd, "STATUS: Implementation pending\n", 33);
    } else {
        write(client_fd, "ERROR: Unknown command\n", 24);
    }
    
    return 0;
}

/**
 * Main daemon loop
 */
int run_daemon() {
    struct sockaddr_un client_addr;
    socklen_t client_len = sizeof(client_addr);
    char buffer[MAX_CMD_LEN];
    
    // Set up signal handlers
    signal(SIGINT, signal_handler);
    signal(SIGTERM, signal_handler);
    signal(SIGHUP, signal_handler);
    
    // Create IPC socket
    daemon_sock = create_ipc_socket();
    if (daemon_sock == -1) {
        fprintf(stderr, "Failed to create IPC socket\n");
        return -1;
    }
    
    printf("Trimorph daemon started, listening on %s/trimorph.sock\n", TRIMORPH_RUNTIME_DIR);
    
    // Main loop
    while (running) {
        // Accept client connection
        int client_fd = accept(daemon_sock, (struct sockaddr*)&client_addr, &client_len);
        if (client_fd == -1) {
            if (running) {
                perror("accept");
            }
            continue;
        }
        
        // Read command from client
        memset(buffer, 0, sizeof(buffer));
        int bytes_read = read(client_fd, buffer, sizeof(buffer) - 1);
        if (bytes_read > 0) {
            buffer[bytes_read] = '\0';
            process_command(client_fd, buffer);
        }
        
        close(client_fd);
    }
    
    // Cleanup
    if (daemon_sock != -1) {
        close(daemon_sock);
        char sock_path[MAX_PATH_LEN];
        snprintf(sock_path, sizeof(sock_path), "%s/trimorph.sock", TRIMORPH_RUNTIME_DIR);
        unlink(sock_path);
    }
    
    return 0;
}