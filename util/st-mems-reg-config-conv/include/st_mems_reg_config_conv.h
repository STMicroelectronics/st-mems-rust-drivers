#ifndef ST_MEMS_REG_CONFIG_CONV_H
#define ST_MEMS_REG_CONFIG_CONV_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

typedef enum FileType {
    FILE_TYPE_JSON = 0,
    FILE_TYPE_UCF  = 1,
} FileType;

/**
 * Generate Rust source from a MEMS configuration file.
 *
 * Parameters:
 *   input_file  - NUL-terminated path to the input configuration file.
 *   output_file - NUL-terminated path to the output Rust file.
 *   array_name  - NUL-terminated Rust array identifier to generate.
 *   sensor_id   - NUL-terminated sensor identifier string.
 *   file_type   - File type (must be one of FILE_TYPE_JSON or FILE_TYPE_UCF).
 *
 * Returns:
 *   0 on success, non-zero on error.
 */
int32_t generate_rs(
    const char *input_file,
    const char *output_file,
    const char *array_name,
    const char *sensor_id,
    FileType file_type
);

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* ST_MEMS_REG_CONFIG_CONV_H */
