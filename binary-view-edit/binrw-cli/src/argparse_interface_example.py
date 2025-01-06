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

# python src/argparse_interface_example.py --file hello --read --position 0 0
# python src/argparse_interface_example.py --file hello write --splice --position 0 0
main_command_group.add_argument('--file')
main_command_group.add_argument('--metadata')
main_command_group.add_argument('--diff')

file_option_subparser = parser.add_subparsers(help="File options")
read_parser = file_option_subparser.add_parser('--read')
read_offset_group = read_parser.add_mutually_exclusive_group(required=True)
read_offset_group.add_argument('--position', nargs=2, metavar='offset')
read_offset_group.add_argument('--amount', nargs=2)

write_parser = file_option_subparser.add_parser('write')

write_mode_group = write_parser.add_mutually_exclusive_group(required=True)
# TODO: Ideally store in one var with a String enum for flags.
write_mode_group.add_argument('--splice', action='store_true')
write_mode_group.add_argument('--overwrite', action='store_true')

write_offset_group = write_parser.add_mutually_exclusive_group(required=True)
write_offset_group.add_argument('--position', nargs=2, metavar='offset')
write_offset_group.add_argument('--amount', nargs=2)

# write_option_subparser = write_parser.add_subparsers(help="Write options")
# write_splice = write_option_subparser.add_parser("--splice")
# write_overwrite = write_option_subparser.add_parser("--overwrite")

args = parser.parse_args()
print(args)