# File Organizer

A Rust-based command-line tool that automatically organizes files into folders based on their extension, modification date, and custom recipes. Perfect for organizing photos, documents, downloads, and other files into a structured folder hierarchy.

Yes, the name is very uninspired...

## Features

- 📁 **Smart File Organization**: Automatically sorts files based on file extensions and modification dates
- 📅 **Date-based Folder Structure**: Create year/month/day folder hierarchies using customizable date formats
- 🔄 **Flexible File Operations**: Move or copy files based on your preferences
- 📋 **Recipe-based Configuration**: Define multiple organization rules in JSON recipe files
- 🧪 **Dry Run Mode**: Test your organization rules before actually moving files

## Usage

### Basic Command Structure

```bash
file_organizer <RECIPES> [OPTIONS]
```

### Arguments

- `RECIPES` - **Required**: Path to the JSON file containing organization recipes

### Options

- `--dry_run` - Performs a dry run test (no files will be moved or copied)

### Examples

#### Basic Usage

```bash
# Organize files using a recipe file
file_organizer recipes/photos.json

# Test your recipe without actually moving files
file_organizer recipes/photos.json --dry_run
```

## Recipe File Structure

The recipe file is a JSON array containing one or more organization recipes. Each recipe defines how files from a source folder should be organized into a destination folder.

### Recipe Fields

| Field                | Type          | Required | Description                                                                                                                                                               |
| -------------------- | ------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `name`               | String        | ✅       | Unique identifier for the recipe.                                                                                                                                         |
| `source_folder`      | String        | ✅       | Path to the folder containing files to organize.                                                                                                                          |
| `destination_folder` | String        | ✅       | Path to the folder where organized files will be placed.                                                                                                                  |
| `subfolders`         | Array[String] | ❌       | Date format for each level of subfolders (e.g., "%Y" for year). If not set no folder will be created.                                                                     |
| `allowed_extensions` | Array[String] | ❌       | List of file extensions to process (empty array = all extensions). If not set no folder will be created.                                                                  |
| `move_files`         | Boolean       | ❌       | If `true`, files are moved; if `false`, files are copied (default: `false`, files are copied).                                                                                  |
| `last_run`           | String        | ❌       | Date of last execution (automatically managed) that allows resuming the organization from the last execution/the date set manually. If not set, all files are considered. |
| `date_comparator`    | String        | ❌       | Which date to use for file comparison: `CreationDate` or `ModificationDate` (default: `ModificationDate`).                                                                |

### Date Comparison and Format Patterns

The tool can use either the file's creation date or modification date for organization. This is controlled by the `date_comparator` field in your recipe:

- `CreationDate` - Uses the file's creation timestamp
- `ModificationDate` - Uses the file's last modification timestamp (default)

We ecommand using `ModificationDate` when the target files are backup file that may have been created way after the initial file. Use a dry run before any real copy/move to make sure the behaviour is the one expected.

### Example Recipe File

```json
[
	{
		"name": "Photo_Organization",
		"source_folder": "/Users/user/Downloads/Photos",
		"destination_folder": "/Users/user/Pictures/Organized",
		"subfolders": ["%Y"],
		"allowed_extensions": ["jpg", "jpeg", "png", "heic", "heif", "dng", "gif"],
		"move_files": false,
		"date_comparator": "CreationDate",
		"last_run": "2024-01-15"
	},
	{
		"name": "Document_Backup",
		"source_folder": "/Users/user/Documents/Inbox",
		"destination_folder": "/Users/user/Documents/Archive",
		"subfolders": ["%Y", "%m", "%d"],
		"allowed_extensions": ["pdf", "doc", "docx", "txt", "rtf"],
		"move_files": true,
		"last_run": "2024-01-15"
	},
	{
		"name": "Video_Collection",
		"source_folder": "/Volumes/External/Videos",
		"destination_folder": "/Users/user/Videos/Organized",
		"subfolders": ["%Y", "%Y-%m-%d - Videos"],
		"allowed_extensions": ["mp4", "avi", "mov", "mkv", "wmv"],
		"move_files": false,
		"last_run": "2024-01-15"
	}
]
```

## Use Cases

- **Photo Organization**: Sort photos by year/month/day
- **Document Management**: Archive documents by creation date
- **Download Cleanup**: Organize downloads into structured folders
- **Media Library**: Sort videos, music, and other media files
- **Backup Organization**: Structure backup files chronologically

## Dependencies

- `anyhow` - Error handling
- `clap` - Command-line argument parsing
- `serde_json` - JSON serialization/deserialization
- `serde` - Serialization framework
- `log` - Logging facade
- `simple_logger` - Simple logging implementation
- `chrono` - Date and time handling
- `colored` - Terminal color support
- `rayon` - Parallel iteration

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
