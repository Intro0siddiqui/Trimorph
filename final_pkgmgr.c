/*
 * Optimized Package Management Core - Final Implementation
 * Implements the underlying package management logic in pure C
 * Provides efficient, portable package management across different systems
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/wait.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <errno.h>
#include <libgen.h>

// Define maximum path length
#define MAX_PATH 1024

// Package format definition structure
typedef struct {
    const char* ext;
    const char* install_cmd;
    const char* verify_cmd;
    int (*install_func)(const char* file);
} pkg_format_t;

// Forward declarations for install functions
int install_deb(const char* file);
int install_arch(const char* file);
int install_rpm(const char* file);
int install_apk(const char* file);
int install_gentoo(const char* file);

// Supported package formats with their handlers
static pkg_format_t pkg_formats[] = {
    {".deb", "dpkg -i '%s'", "dpkg --version", install_deb},
    {".pkg.tar.zst", "pacman -U --noconfirm '%s'", "pacman --version", install_arch},
    {".pkg.tar.xz", "pacman -U --noconfirm '%s'", "pacman --version", install_arch},
    {".pkg.tar.gz", "pacman -U --noconfirm '%s'", "pacman --version", install_arch},
    {".rpm", "rpm -i '%s'", "rpm --version", install_rpm},
    {".apk", "apk add '%s'", "apk --version", install_apk},
    {".tbz", "emerge '%s'", "emerge --version", install_gentoo},
    {NULL, NULL, NULL, NULL}  // Sentinel
};

// Command availability checker
int is_cmd_available(const char* cmd) {
    char check[256];
    snprintf(check, sizeof(check), "command -v %s >/dev/null 2>&1", cmd);
    return system(check) == 0;
}

// Install .deb package
int install_deb(const char* file) {
    if (!is_cmd_available("dpkg") && !is_cmd_available("apt")) {
        fprintf(stderr, "Error: Neither dpkg nor apt is available\n");
        return -1;
    }
    
    char cmd[MAX_PATH];
    if (is_cmd_available("apt")) {
        snprintf(cmd, sizeof(cmd), "apt install -y '%s'", file);
    } else {
        snprintf(cmd, sizeof(cmd), "dpkg -i '%s'", file);
    }
    
    printf("Executing: %s\n", cmd);
    return system(cmd);
}

// Install Arch Linux package
int install_arch(const char* file) {
    if (!is_cmd_available("pacman")) {
        fprintf(stderr, "Error: pacman is not available\n");
        return -1;
    }
    
    char cmd[MAX_PATH];
    snprintf(cmd, sizeof(cmd), "pacman -U --noconfirm '%s'", file);
    printf("Executing: %s\n", cmd);
    return system(cmd);
}

// Install RPM package
int install_rpm(const char* file) {
    if (!is_cmd_available("rpm")) {
        fprintf(stderr, "Error: rpm is not available\n");
        return -1;
    }
    
    char cmd[MAX_PATH];
    if (is_cmd_available("dnf")) {
        snprintf(cmd, sizeof(cmd), "dnf install -y '%s'", file);
    } else if (is_cmd_available("yum")) {
        snprintf(cmd, sizeof(cmd), "yum install -y '%s'", file);
    } else {
        snprintf(cmd, sizeof(cmd), "rpm -i '%s'", file);
    }
    
    printf("Executing: %s\n", cmd);
    return system(cmd);
}

// Install Alpine package
int install_apk(const char* file) {
    if (!is_cmd_available("apk")) {
        fprintf(stderr, "Error: apk is not available\n");
        return -1;
    }
    
    char cmd[MAX_PATH];
    snprintf(cmd, sizeof(cmd), "apk add '%s'", file);
    printf("Executing: %s\n", cmd);
    return system(cmd);
}

// Install Gentoo package (not typical binary packages)
int install_gentoo(const char* file) {
    fprintf(stderr, "Note: Gentoo typically uses source-based packages (ebuilds)\n");
    fprintf(stderr, "Installing binary package: %s\n", file);
    
    // In practice, Gentoo binary packages are handled differently
    // This is just a placeholder for demonstration
    char cmd[MAX_PATH];
    snprintf(cmd, sizeof(cmd), "emerge --usepkg '%s'", file);
    printf("Executing: %s\n", cmd);
    return system(cmd);
}

// Execute a package manager command directly
int run_pkg_manager(const char* pm_name, int argc, char *argv[]) {
    if (!is_cmd_available(pm_name)) {
        fprintf(stderr, "Error: Package manager '%s' is not available\n", pm_name);
        return -1;
    }
    
    // Build the command string
    char cmd[MAX_PATH] = {0};
    strncat(cmd, pm_name, sizeof(cmd) - 1);
    
    // Add all arguments
    for (int i = 0; i < argc; i++) {
        strncat(cmd, " ", sizeof(cmd) - 1 - strlen(cmd));
        strncat(cmd, argv[i], sizeof(cmd) - 1 - strlen(cmd));
    }
    
    printf("Executing: %s\n", cmd);
    return system(cmd);
}

// Install a local package file based on extension
int install_local_package(const char* pkg_file) {
    struct stat st;
    if (stat(pkg_file, &st) != 0) {
        fprintf(stderr, "Error: Package file does not exist: %s\n", pkg_file);
        return -1;
    }
    
    // Determine the file extension
    const char* ext = strrchr(pkg_file, '.');
    if (!ext) {
        fprintf(stderr, "Error: Cannot determine package format for: %s\n", pkg_file);
        return -1;
    }
    
    // Check for compound extensions like .pkg.tar.zst
    const char* compound_ext = NULL;
    if (strlen(ext) > 4) {  // At least something like ".tar.xz"
        char full_ext[16];
        strncpy(full_ext, ext, sizeof(full_ext) - 1);
        full_ext[sizeof(full_ext) - 1] = '\0';
        
        // Look for the package format
        for (int i = 0; pkg_formats[i].ext != NULL; i++) {
            if (strcmp(pkg_formats[i].ext, full_ext) == 0 && pkg_formats[i].install_func) {
                return pkg_formats[i].install_func(pkg_file);
            }
        }
    }
    
    // Look for the simple extension
    for (int i = 0; pkg_formats[i].ext != NULL; i++) {
        if (strcmp(pkg_formats[i].ext, ext) == 0 && pkg_formats[i].install_func) {
            return pkg_formats[i].install_func(pkg_file);
        }
    }
    
    fprintf(stderr, "Error: Unsupported package format: %s\n", ext);
    return -1;
}

// Main function - handles command line arguments
int main(int argc, char *argv[]) {
    if (argc < 2) {
        printf("Usage:\n");
        printf("  %s install <package-file>        - Install a local package\n", argv[0]);
        printf("  %s run <pkgmgr> [args...]        - Execute package manager command\n", argv[0]);
        printf("  %s supported-formats            - List supported package formats\n", argv[0]);
        printf("  %s check <pkgmgr>               - Check if package manager exists\n", argv[0]);
        printf("\nExamples:\n");
        printf("  %s install package.deb\n", argv[0]);
        printf("  %s run apt update\n", argv[0]);
        printf("  %s run pacman -Syu\n", argv[0]);
        printf("  %s check emerge\n", argv[0]);
        return 1;
    }
    
    // Process command
    if (strcmp(argv[1], "install") == 0) {
        if (argc != 3) {
            fprintf(stderr, "Usage: %s install <package-file>\n", argv[0]);
            return 1;
        }
        return install_local_package(argv[2]);
    }
    else if (strcmp(argv[1], "run") == 0) {
        if (argc < 4) {
            fprintf(stderr, "Usage: %s run <pkgmgr> [args...]\n", argv[0]);
            return 1;
        }
        
        // Calculate remaining arguments
        int remaining_args = argc - 3;
        char** pm_args = malloc(remaining_args * sizeof(char*));
        if (!pm_args) {
            fprintf(stderr, "Error: Memory allocation failed\n");
            return -1;
        }
        
        for (int i = 0; i < remaining_args; i++) {
            pm_args[i] = argv[i + 3];
        }
        
        int result = run_pkg_manager(argv[2], remaining_args, pm_args);
        free(pm_args);
        return result;
    }
    else if (strcmp(argv[1], "supported-formats") == 0) {
        printf("Supported package formats:\n");
        for (int i = 0; pkg_formats[i].ext != NULL; i++) {
            printf("  %s\n", pkg_formats[i].ext);
        }
        return 0;
    }
    else if (strcmp(argv[1], "check") == 0) {
        if (argc != 3) {
            fprintf(stderr, "Usage: %s check <pkgmgr>\n", argv[0]);
            return 1;
        }
        
        if (is_cmd_available(argv[2])) {
            printf("%s is available\n", argv[2]);
            return 0;
        } else {
            printf("%s is not available\n", argv[2]);
            return 1;
        }
    }
    else {
        fprintf(stderr, "Error: Unknown command '%s'\n", argv[1]);
        return 1;
    }
    
    return 0;
}