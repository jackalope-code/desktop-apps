import argparse

parser = argparse.ArgumentParser(
  prog='binrw-cli',
  description='Mock Binary read/write CLI. Read, write, diff, and edit text or binary fles.',
  epilog='Made with no external library usage. Written in Python.'
)

# read tag: help: "Reads and writes the ID3v1 or EXIF data tags of provided mp3 and jpg files.".to_string()

main_command_group = parser.add_mutually_exclusive_group(required=True)
# TODO: Subparser on file
# Examples:
# binrw file read filename
# binrw file read 0 4 filename
# binrw file read -4 eof filename

# Data and file input flags
# binrw file write overwrite|splice (--file-input | --data-input)

# --file-output as default and consider outputting directly to the console/stdio for a GUI wrapper for a text editor.

# binrw file write splice (input_position) (input_size) (input_data) (output_position) (output_size) (output_file) 
# binrw file write splice (input_position) (input_size) (input_file) (output_position) (output_size) (output_file) 
# binrw file write splice (input_position) (input_end_position) (input_data) (output_end_position) (output_size) (output_file) 
# binrw file write splice (input_position) (input_end_position) (input_file) (output_end_position) (output_size) (output_file) 

# binrw file write overwrite (input_position) (input_size) (input_data) (output_position) (output_size) (output_file) 
# binrw file write overwrite (input_position) (input_size) (input_file) (output_position) (output_size) (output_file) 
# binrw file write overwrite (input_position) (input_end_position) (input_data) (output_end_position) (output_size) (output_file) 
# binrw file write overwrite (input_position) (input_end_position) (input_file) (output_end_position) (output_size) (output_file) 

# binrw file write overwrite (--pad-zeroes start/end | --write_exact (default)) (input_position) (input_end_position) (input_file) (output_end_position) (output_size) (output_file) 

main_command_group.add_argument('--file')
main_command_group.add_argument('--metadata')
main_command_group.add_argument('--diff')

args = parser.parse_args()
print(args)