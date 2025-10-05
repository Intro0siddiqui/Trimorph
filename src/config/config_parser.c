#include "trimorph.h"
#include <ctype.h>

/**
 * Parses a single configuration file into a jail_config_t structure
 */
int parse_config_file(const char* config_path, jail_config_t* config) {
    FILE* file = fopen(config_path, "r");
    if (!file) {
        fprintf(stderr, "Error: Cannot open config file %s: %s\n", config_path, strerror(errno));
        return -1;
    }

    char line[MAX_CONFIG_LINE];
    while (fgets(line, sizeof(line), file)) {
        // Remove comments and trim whitespace
        char* comment = strchr(line, '#');
        if (comment) *comment = '\0';
        
        // Find the = character
        char* separator = strchr(line, '=');
        if (!separator) continue;
        
        // Split into key and value
        *separator = '\0';
        char* key = line;
        char* value = separator + 1;
        
        // Trim whitespace from key and value
        while (isspace(*key)) key++;
        char* key_end = key + strlen(key) - 1;
        while (key_end > key && isspace(*key_end)) *key_end = '\0';
        
        while (isspace(*value)) value++;
        char* value_end = value + strlen(value) - 1;
        while (value_end > value && isspace(*value_end)) *value_end = '\0';
        
        // Map keys to config fields
        if (strcmp(key, "name") == 0) {
            strncpy(config->name, value, MAX_JAIL_NAME - 1);
            config->name[MAX_JAIL_NAME - 1] = '\0';
        } else if (strcmp(key, "root") == 0) {
            strncpy(config->root, value, MAX_PATH_LEN - 1);
            config->root[MAX_PATH_LEN - 1] = '\0';
        } else if (strcmp(key, "bootstrap") == 0) {
            strncpy(config->bootstrap, value, MAX_CMD_LEN - 1);
            config->bootstrap[MAX_CMD_LEN - 1] = '\0';
        } else if (strcmp(key, "pkgmgr") == 0) {
            strncpy(config->pkgmgr, value, MAX_PATH_LEN - 1);
            config->pkgmgr[MAX_PATH_LEN - 1] = '\0';
        } else if (strcmp(key, "pkgmgr_args") == 0) {
            strncpy(config->pkgmgr_args, value, MAX_CMD_LEN - 1);
            config->pkgmgr_args[MAX_CMD_LEN - 1] = '\0';
        } else if (strcmp(key, "mounts") == 0) {
            strncpy(config->mounts, value, MAX_CMD_LEN - 1);
            config->mounts[MAX_CMD_LEN - 1] = '\0';
        } else if (strcmp(key, "env") == 0) {
            strncpy(config->env, value, MAX_CMD_LEN - 1);
            config->env[MAX_CMD_LEN - 1] = '\0';
        }
    }
    
    fclose(file);
    
    // Validate required fields
    if (strlen(config->name) == 0) {
        fprintf(stderr, "Error: Missing name in config %s\n", config_path);
        return -1;
    }
    
    if (strlen(config->root) == 0) {
        fprintf(stderr, "Error: Missing root in config %s\n", config_path);
        return -1;
    }
    
    if (strlen(config->pkgmgr) == 0) {
        fprintf(stderr, "Error: Missing pkgmgr in config %s\n", config_path);
        return -1;
    }
    
    return 0;
}

/**
 * Loads all jail configurations from the jails.d directory
 */
int load_all_jails(jail_config_t** configs, int* count) {
    DIR* dir = opendir(TRIMORPH_JAILS_DIR);
    if (!dir) {
        fprintf(stderr, "Error: Cannot open jails directory %s: %s\n", TRIMORPH_JAILS_DIR, strerror(errno));
        return -1;
    }
    
    struct dirent* entry;
    int capacity = 10; // Initial capacity
    *configs = malloc(capacity * sizeof(jail_config_t));
    if (!*configs) {
        closedir(dir);
        return -1;
    }
    
    *count = 0;
    while ((entry = readdir(dir)) != NULL) {
        // Process .conf files
        if (strstr(entry->d_name, ".conf")) {
            char config_path[MAX_PATH_LEN];
            snprintf(config_path, sizeof(config_path), "%s/%s", TRIMORPH_JAILS_DIR, entry->d_name);
            
            // Expand the array if needed
            if (*count >= capacity) {
                capacity *= 2;
                jail_config_t* temp = realloc(*configs, capacity * sizeof(jail_config_t));
                if (!temp) {
                    free(*configs);
                    closedir(dir);
                    return -1;
                }
                *configs = temp;
            }
            
            // Initialize the config structure
            memset(&(*configs)[*count], 0, sizeof(jail_config_t));
            
            // Parse the config file
            if (parse_config_file(config_path, &(*configs)[*count]) == 0) {
                (*configs)[*count].status = JAIL_STOPPED;
                (*count)++;
            } else {
                fprintf(stderr, "Warning: Failed to parse config file %s, skipping\n", config_path);
            }
        }
    }
    
    closedir(dir);
    
    // Shrink the array to the actual size
    if (*count > 0) {
        jail_config_t* temp = realloc(*configs, *count * sizeof(jail_config_t));
        if (temp) {
            *configs = temp;
        }
    } else {
        free(*configs);
        *configs = NULL;
    }
    
    return 0;
}