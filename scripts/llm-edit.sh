#!/bin/bash
# Heavily based on a script by @inventorblack, and
# shared on [ClaudeLog](https://claudelog.com/multi-file-system/)
# SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
# SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
#
# SPDX-License-Identifier: MIT OR Apache-2.0

# shellcheck disable=SC2317,SC2034
# llm-edit.sh - Script to parse JSON files payload and write files in parallel
# Usage: ./llm-edit.sh <json_input_file> [--verbose] [--log-to-file <log_file>]

set -e  # Exit on error

# Default settings
VERBOSE=false
LOG_TO_FILE=false
LOG_FILE=""
CLAUDE_OUTPUT=true  # Set to true by default to use the new styling
OUTPUT_FILE=""
SILENT=false
COMPACT=true  # Enable compact output by default

# ANSI color codes
WHITE='\033[1;37m'    # Bright white
GRAY='\033[0;37m'     # Gray
GREEN='\033[0;32m'    # Green
RED='\033[0;31m'      # Red
NC='\033[0m'          # No Color (reset)

# Icon settings
SUCCESS_ICON="🟢"
NEUTRAL_ICON="⚪"
ERROR_ICON="🔴"
INFO_ICON="ℹ️"
CLOCK_ICON="⏱️"
DATE_ICON="📅"
SIMPLE_CHECK="✓"

# Process command line arguments
process_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --verbose)
                VERBOSE=true
                COMPACT=false  # Disable compact output in verbose mode
                shift
                ;;
            --log-to-file)
                LOG_TO_FILE=true
                LOG_FILE="$2"
                shift 2
                ;;
            --claude-output)
                CLAUDE_OUTPUT=true
                shift
                ;;
            --output-file)
                OUTPUT_FILE="$2"
                SILENT=true  # When output file is specified, run silently by default
                shift 2
                ;;
            --silent)
                SILENT=true
                shift
                ;;
            --no-compact)
                COMPACT=false
                shift
                ;;
            --no-color)
                COLOR=false
                shift
                ;;
            -h|--help)
                print_usage
                exit 0
                ;;
            *)
                if [[ -z "$JSON_FILE" ]]; then
                    JSON_FILE="$1"
                    shift
                else
                    echo "Error: Unknown argument: $1"
                    print_usage
                    exit 1
                fi
                ;;
        esac
    done

    # Validate required arguments
    if [[ -z "$JSON_FILE" ]]; then
        echo "Error: JSON input file is required."
        print_usage
        exit 1
    fi

    # Check if the file exists
    if [[ ! -f "$JSON_FILE" ]]; then
        echo "Error: File $JSON_FILE does not exist."
        exit 1
    fi

    # Set up logging
    if [[ "$LOG_TO_FILE" = true && -n "$LOG_FILE" ]]; then
        # Create log directory if it doesn't exist
        LOG_DIR=$(dirname "$LOG_FILE")
        mkdir -p "$LOG_DIR"
        # Initialize log file
        echo "--- Multi-File Extraction Log $(date) ---" > "$LOG_FILE"
    fi

    # Set up output file if requested
    if [[ -n "$OUTPUT_FILE" ]]; then
        # Create output directory if it doesn't exist
        OUTPUT_DIR=$(dirname "$OUTPUT_FILE")
        mkdir -p "$OUTPUT_DIR"
        # Initialize output file with the new format (no colors)
        echo "Multi-File Extraction Results ${DATE_ICON} $(date)" > "$OUTPUT_FILE"
    fi
}

# Print usage information
print_usage() {
    cat << EOF
Usage: $0 <json_input_file> [--verbose] [--log-to-file <log_file>] [--claude-output] [--output-file <output_file>] [--silent] [--no-compact]

Arguments:
  <json_input_file>        Path to the JSON file containing file data
  --verbose                Show detailed output during extraction (disables compact mode)
  --log-to-file <log_file> Write detailed log to specified file
  --claude-output          Format output for Claude to render (styled output)
  --output-file <file>     Write formatted output to file (for Claude to read later, implies --silent)
  --silent                 Suppress all console output except errors
  --no-compact             Disable compact output format
  --help, -h               Show this help message

Examples:
  $0 tool_data.json                            # Extract files with minimal output
  $0 tool_data.json --verbose                  # Extract with detailed progress
  $0 tool_data.json --log-to-file logs/extraction.log  # Log details to file
  $0 tool_data.json --claude-output            # Format output for Claude rendering
  $0 tool_data.json --output-file results.md   # Write results to file for Claude (silent mode)
  $0 tool_data.json --silent                   # Run without any console output

JSON Format:
  { "files": [ { "file_name": "path/to/file", "file_type": "text", "file_content": "content" } ] }
EOF
}

# Log messages to file if enabled
log_to_file() {
    if [[ "$LOG_TO_FILE" = true && -n "$LOG_FILE" ]]; then
        echo "$(date +"%Y-%m-%d %H:%M:%S") - $1" >> "$LOG_FILE"
    fi
}

# Write to output file if enabled
write_output() {
    if [[ -n "$OUTPUT_FILE" ]]; then
        echo "$1" >> "$OUTPUT_FILE"
    fi

    # Only print to stdout if not in silent mode
    if [[ "$SILENT" = false ]]; then
        echo "$1"
    fi
}

# Write colored output to terminal (no files)
write_colored_output() {
    if [[ "$SILENT" = false ]]; then
        echo -e "$1"  # -e flag enables interpretation of backslash escapes for colors
    fi
}

# Global variables for output
declare -a NUMBER_OUTPUTS
declare -a SIMPLE_OUTPUTS
number_items=0
simple_items=0

# Output functions with the new formatting style
print_section() {
    local text="$1"
    log_to_file "[SECTION] $text"

    # For non-compact mode
    if [[ "$COMPACT" = false ]]; then
        write_output "**$text**"
    fi
    # For compact mode, we don't need section titles
}

# Functions for different types of output
print_number_item() {
    local text="$1"
    local icon="$2"
    log_to_file "[NUMBER_ITEM] $text"

    if [[ "$COMPACT" = false ]]; then
        write_colored_output "${GRAY}${icon} $text${NC}"
    else
        # Store items with numbers for later output on a single line
        number_items=$((number_items + 1))
        NUMBER_OUTPUTS[number_items]="${icon} ${text}"
    fi
}

print_simple_item() {
    local text="$1"
    log_to_file "[SIMPLE_ITEM] $text"

    if [[ "$COMPACT" = false ]]; then
        write_colored_output "${GRAY}${SIMPLE_CHECK} $text${NC}"
    else
        # Store simple items for later output on a single line
        simple_items=$((simple_items + 1))
        SIMPLE_OUTPUTS[simple_items]="${SIMPLE_CHECK} $text"
    fi
}

print_success() {
    local text="$1"
    log_to_file "[SUCCESS] $text"

    # Check if text contains numbers
    if [[ "$text" =~ [0-9] ]]; then
        print_number_item "$text" "$SUCCESS_ICON"
    else
        print_simple_item "$text"
    fi
}

print_warning() {
    local text="$1"
    log_to_file "[WARNING] $text"

    # Check if text contains numbers
    if [[ "$text" =~ [0-9] ]]; then
        print_number_item "$text" "$NEUTRAL_ICON"
    else
        print_simple_item "$text"
    fi
}

print_error() {
    local text="$1"
    log_to_file "[ERROR] $text"

    # Check if text contains numbers
    if [[ "$text" =~ [0-9] ]]; then
        print_number_item "$text" "$ERROR_ICON"
    else
        print_simple_item "$text"
    fi
}

print_info() {
    local text="$1"
    log_to_file "[INFO] $text"

    # Check if text contains numbers
    if [[ "$text" =~ [0-9] ]]; then
        print_number_item "$text" "$INFO_ICON"
    else
        print_simple_item "$text"
    fi
}

# Logical color-coding for file numbers
print_file_count() {
    local count="$1"
    local description="$2"
    log_to_file "[COUNT] $description: $count"

    # Format with colored numbers and add to number items
    local formatted_text="${description}: ${count}"

    # Use green circle for count items
    print_number_item "$formatted_text" "$SUCCESS_ICON"
}

print_file_warning() {
    local count="$1"
    local description="$2"
    log_to_file "[COUNT_WARNING] $description: $count"

    # Format with colored numbers and add to number items
    local formatted_text="${description}: ${count}"

    # Use white circle for warning items
    print_number_item "$formatted_text" "$NEUTRAL_ICON"
}

print_file_error() {
    local count="$1"
    local description="$2"
    log_to_file "[COUNT_ERROR] $description: $count"

    # Format with colored numbers and add to number items
    local formatted_text="${description}: ${count}"

    # Use red circle for error items, but only if count > 0
    if [[ "$count" -gt 0 ]]; then
        print_number_item "$formatted_text" "$ERROR_ICON"
    else
        print_number_item "$formatted_text" "$NEUTRAL_ICON"
    fi
}

print_verbose() {
    local text="$1"
    log_to_file "[VERBOSE] $text"

    if [[ "$VERBOSE" = true ]]; then
        write_output "⚪ $text"
    fi
}

# Function to decode base64 content safely
decode_base64() {
    local content="$1"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS version
        echo "$content" | base64 -D
    else
        # Linux version
        echo "$content" | base64 -d
    fi
}

# Function to format elapsed time
format_elapsed_time() {
    local elapsed=$1

    # Format elapsed time
    if [[ $elapsed -lt 60 ]]; then
        echo "${elapsed}s"
    else
        mins=$((elapsed / 60))
        secs=$((elapsed % 60))
        echo "${mins}m ${secs}s"
    fi
}

# Print all compact outputs
print_compact_output() {
    if [[ "$COMPACT" = true && "$SILENT" = false ]]; then
        # Calculate elapsed time
        end_time=$(date +%s)
        elapsed=$((end_time - start_time))
        elapsed_str=$(format_elapsed_time $elapsed)

        # Print the timestamp header with white title, colored date, and elapsed time
        write_colored_output "${WHITE}Multi-File Extraction Results ${DATE_ICON} $(date) ${CLOCK_ICON} ${elapsed_str}${NC}"

        # Combine all number items on a single line if there are any
        if [[ $number_items -gt 0 ]]; then
            local number_line=""
            for i in $(seq 1 $number_items); do
                if [[ -n "$number_line" ]]; then
                    number_line="${number_line} ${NUMBER_OUTPUTS[i]}"
                else
                    number_line="${NUMBER_OUTPUTS[i]}"
                fi
            done
            write_colored_output "${GRAY}${number_line}${NC}"
        fi

        # Combine all simple items on a single line if there are any
        if [[ $simple_items -gt 0 ]]; then
            local simple_line=""
            for i in $(seq 1 $simple_items); do
                if [[ -n "$simple_line" ]]; then
                    simple_line="${simple_line} ${SIMPLE_OUTPUTS[i]}"
                else
                    simple_line="${SIMPLE_OUTPUTS[i]}"
                fi
            done
            write_colored_output "${GRAY}${simple_line}${NC}"
        fi

        # Final success line - always in green
        if [[ "$CREATED_COUNT" -eq "$FILE_COUNT" ]]; then
            write_colored_output "${GREEN}${SUCCESS_ICON} Extraction completed successfully!${NC}"
        else
            write_colored_output "${RED}${ERROR_ICON} Extraction completed with issues${NC}"
        fi
    fi
}

# Check if jq is installed
check_dependencies() {
    print_section "Checking Dependencies"
    if ! command -v jq &> /dev/null; then
        print_error "This script requires 'jq' to be installed."
        echo "Please install it with: sudo apt-get install jq"
        exit 1
    fi
    print_success "All dependencies satisfied"
}

# Parse JSON file and prepare extraction
prepare_extraction() {
    print_section "Preparing Extraction"

    # Get the number of files to create
    FILE_COUNT=$(jq '.files | length' "$JSON_FILE")
    if [[ "$FILE_COUNT" -eq 0 ]]; then
        print_warning "No files found in the JSON payload."
        exit 0
    fi

    print_file_count "$FILE_COUNT" "Found files to extract"

    # Display file summary in verbose mode
    if [[ "$VERBOSE" = true ]]; then
        for i in $(seq 0 $((FILE_COUNT - 1))); do
            FILE_NAME=$(jq -r ".files[$i].file_name" "$JSON_FILE")
            FILE_TYPE=$(jq -r ".files[$i].file_type // \"text\"" "$JSON_FILE")
            print_verbose "File $((i + 1))/$FILE_COUNT: $FILE_NAME ($FILE_TYPE)"
        done
    fi

    # Create temporary directory for extraction scripts
    TEMP_DIR=$(mktemp -d)
    trap 'rm -rf "$TEMP_DIR"' EXIT

    print_success "Extraction prepared successfully"
}

# Create individual extraction scripts
create_extraction_scripts() {
    print_section "Creating Extraction Scripts"

    for i in $(seq 0 $((FILE_COUNT - 1))); do
        FILE_INFO=$(jq -c ".files[$i]" "$JSON_FILE")

        FILE_NAME=$(echo "$FILE_INFO" | jq -r '.file_name')
        FILE_TYPE=$(echo "$FILE_INFO" | jq -r '.file_type // "text"')

        # Create a separate file to store the content to avoid shell interpretation issues
        CONTENT_FILE="$TEMP_DIR/content_$i.txt"
        jq -r '.file_content' <<< "$FILE_INFO" > "$CONTENT_FILE"

        # Create directory if it doesn't exist
        DIR_NAME=$(dirname "$FILE_NAME")

        # Create extraction script that uses the content file
        cat > "$TEMP_DIR/extract_$i.sh" << EOF
#!/bin/bash
# Create directory structure
mkdir -p "$DIR_NAME"

# Check if file content is base64 encoded
if [[ "$FILE_TYPE" == "binary" ]]; then
    # Handle binary file
    cat "$CONTENT_FILE" | base64 -d > "$FILE_NAME"
    echo "EXTRACTED|binary|$FILE_NAME"
else
    # Handle text file - direct copy without shell interpretation
    cat "$CONTENT_FILE" > "$FILE_NAME"
    echo "EXTRACTED|text|$FILE_NAME"
fi
EOF

        chmod +x "$TEMP_DIR/extract_$i.sh"

        # Log verbose progress
        print_verbose "Created extraction script for: $FILE_NAME"
    done

    print_success "All extraction scripts created successfully"
}

# Execute all extraction scripts in parallel and capture output
execute_extraction() {
    print_section "Extracting Files in Parallel"

    # Create a place to store extraction results
    RESULTS_FILE="$TEMP_DIR/extraction_results.txt"
    touch "$RESULTS_FILE"

    # Execute all extraction scripts in parallel and capture their output
    find "$TEMP_DIR" -name "extract_*.sh" -print0 |
        xargs -0 -P 8 -I {} bash -c "{} >> $RESULTS_FILE 2>/dev/null"

    # Process results
    extract_count=0

    # Display extraction results based on verbosity
    if [[ "$VERBOSE" = true ]]; then
        while IFS= read -r line; do
            if [[ "$line" == EXTRACTED* ]]; then
                IFS='|' read -r _ type file_path <<< "$line"
                extract_count=$((extract_count + 1))
                print_success "Extracted $type file: $file_path"
            fi
        done < "$RESULTS_FILE"
    else
        # Just count the extracted files for non-verbose mode
        extract_count=$(grep -c "EXTRACTED" "$RESULTS_FILE")
    fi
}

# Verify all files were created correctly
verify_extraction() {
    print_section "Verifying Extraction"

    CREATED_COUNT=0
    FAILED_FILES=()

    for i in $(seq 0 $((FILE_COUNT - 1))); do
        FILE_NAME=$(jq -r ".files[$i].file_name" "$JSON_FILE")
        if [[ -f "$FILE_NAME" ]]; then
            CREATED_COUNT=$((CREATED_COUNT + 1))
            print_verbose "Verified: $FILE_NAME"
        else
            FAILED_FILES+=("$FILE_NAME")
            print_verbose "Missing: $FILE_NAME"
        fi
    done

    # Log results to file regardless of verbosity
    log_to_file "Files processed: $FILE_COUNT"
    log_to_file "Files created: $CREATED_COUNT"
    log_to_file "Files failed: $((FILE_COUNT - CREATED_COUNT))"

    if [[ ${#FAILED_FILES[@]} -gt 0 ]]; then
        log_to_file "Failed files:"
        for file in "${FAILED_FILES[@]}"; do
            log_to_file "  $file"
        done
    fi
}

# Print summary statistics at the end
print_summary() {
    print_section "Extraction Summary"
    print_file_count "$FILE_COUNT" "Files processed"
    print_file_count "$CREATED_COUNT" "Files created"
    print_file_error "$((FILE_COUNT - CREATED_COUNT))" "Files failed"

    if [[ "$COMPACT" = false ]]; then
        # Calculate and display elapsed time for non-compact mode
        end_time=$(date +%s)
        elapsed=$((end_time - start_time))
        elapsed_str=$(format_elapsed_time $elapsed)
        write_colored_output "${GRAY}${CLOCK_ICON} Completed in: ${WHITE}${elapsed_str}${NC}"

        if [[ "$CREATED_COUNT" -eq "$FILE_COUNT" ]]; then
            write_colored_output "${GREEN}${SUCCESS_ICON} Extraction completed successfully!${NC}"
        else
            write_colored_output "${RED}${ERROR_ICON} Extraction completed with issues${NC}"
            if [[ "$VERBOSE" = false && ${#FAILED_FILES[@]} -gt 0 ]]; then
                print_info "Run with --verbose flag to see details of failed files"
            fi
        fi

        if [[ "$LOG_TO_FILE" = true && -n "$LOG_FILE" ]]; then
            print_info "Full extraction log available at: $LOG_FILE"
        fi

        # If output file was used but not in silent mode, print its location
        if [[ -n "$OUTPUT_FILE" && "$SILENT" = false ]]; then
            echo "Results saved to: $OUTPUT_FILE"
        fi
    fi
}

# Main function
main() {
    # Record start time
    start_time=$(date +%s)

    # Process and validate arguments
    process_args "$@"

    # Only add timestamp header if not in compact mode
    if [[ "$COMPACT" = false && "$SILENT" = false ]]; then
        # Calculate and display elapsed time for non-compact mode at the end
        end_time=$(date +%s)
        elapsed=$((end_time - start_time))
        elapsed_str=$(format_elapsed_time $elapsed)
        write_colored_output "${WHITE}Multi-File Extraction Results ${DATE_ICON} $(date) ${CLOCK_ICON} ${elapsed_str}${NC}"
    fi

    check_dependencies
    prepare_extraction
    create_extraction_scripts
    execute_extraction
    verify_extraction
    print_summary

    # Print compact output if enabled
    if [[ "$COMPACT" = true ]]; then
        print_compact_output
    fi

    # If we're in silent mode but have an output file, return the path as the only output
    if [[ "$SILENT" = true && -n "$OUTPUT_FILE" ]]; then
        echo "$OUTPUT_FILE"
    fi

    exit 0
}

# Execute the main function with all arguments
main "$@"
